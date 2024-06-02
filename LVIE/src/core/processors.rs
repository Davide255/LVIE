#![allow(dead_code)]
use rayon::prelude::*;
use LVIElib::traits::{cast_color_to_rgba, AsFloat, ToOklab};
use LVIElib::hsl::HslaImage;
use LVIElib::linear_srgb::LinSrgba;
use LVIElib::white_balance::{xyz_wb_matrix, LINSRGB_TO_XYZ, XYZ_TO_LINSRGB};
use std::sync::{Arc, Mutex};
use image::{Pixel, Primitive, Rgba, RgbaImage};
use LVIElib::matrix::{
    convolution::laplacian_of_gaussian, Matrix,
};

use LVIElib::oklab::{Oklaba, OklabaImage};

use LVIElib::matrix::convolution::convolve;

use LVIE_GPU::{CRgbaImage, Pod};
use LVIElib::traits::{Scale, ToHsl};
use std::fmt::Debug;

pub unsafe fn convert_hsla_to_rgba<P>(img: &HslaImage) -> Option<CRgbaImage<P>>
where 
    P: Pixel + Send + Sync + 'static + Debug + ToHsl,
    P::Subpixel: Scale + Send + Sync + Primitive + std::fmt::Debug + Pod + Debug
{
    // if target is not rgb we cannot cast it
    if P::COLOR_MODEL != "RGBA" {
        return None;
    }

    let out = Arc::new(Mutex::new(vec![P::Subpixel::DEFAULT_MIN_VALUE; img.len()]));

    img.enumerate_rows().par_bridge().for_each(|r| {
        let mut row: Vec<P::Subpixel> = Vec::new();
        for (_, _, p) in r.1 {
            let rgb = p.to_rgba();
            let cmp = rgb.channels();
            // We know what type is it but we have to return a generic, so we transmute it
            row.append(&mut vec![cmp[0].scale(), cmp[1].scale(), cmp[2].scale(), cmp[3].scale()]);
        }
        out.lock().unwrap()[(r.0*img.width()*4) as usize .. (r.0*img.width()*4 + img.width()*4) as usize].clone_from_slice(&row);
    });

    CRgbaImage::<P>::from_vec(img.width(), img.height(), Arc::try_unwrap(out).unwrap().into_inner().unwrap())
}

pub unsafe fn convert_oklaba_to_rgba<P>(img: &OklabaImage) -> Option<CRgbaImage<P>>
where 
    P: Pixel + Send + Sync + 'static + Debug + ToOklab,
    P::Subpixel: Scale + Send + Sync + Primitive + std::fmt::Debug + Pod + Debug
{
    // if target is not rgb we cannot cast it
    if P::COLOR_MODEL != "RGBA" {
        return None;
    }

    let out = Arc::new(Mutex::new(vec![P::Subpixel::DEFAULT_MIN_VALUE; img.len()]));

    img.enumerate_rows().par_bridge().for_each(|r| {
        let mut row: Vec<P::Subpixel> = Vec::new();
        for (_, _, p) in r.1 {
            let rgb = p.to_rgba();
            let cmp = rgb.channels();
            // We know what type is it but we have to return a generic, so we transmute it
            row.append(&mut vec![cmp[0].scale(), cmp[1].scale(), cmp[2].scale(), cmp[3].scale()]);
        }
        out.lock().unwrap()[(r.0*img.width()*4) as usize .. (r.0*img.width()*4 + img.width()*4) as usize].clone_from_slice(&row);
    });

    CRgbaImage::<P>::from_vec(img.width(), img.height(), Arc::try_unwrap(out).unwrap().into_inner().unwrap())
}

pub fn saturate(img: &mut HslaImage, value: f32)
{
    img.enumerate_pixels_mut().par_bridge().for_each(|(_, _, pixel)| {
        *pixel.saturation_mut() += value / 2.0;
    });
}

pub fn sharpen(img: &mut OklabaImage, value: f32, size: usize)
{
    let (mut vl, mut va, mut vb, mut valpha) = (
        Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new()
    );
    for (_, _, pixel) in img.enumerate_pixels() {
        let channels = pixel.channels();

        vl.push(channels[0]);
        va.push(channels[1]);
        vb.push(channels[2]);
        valpha.push(channels[3]);
    }

    let l_matrix = Matrix::new(vl, img.height() as usize, img.width() as usize);
    let kernel = laplacian_of_gaussian(value, size, size);

    let gradient = convolve(&l_matrix, &kernel);

    let out_l = (l_matrix - gradient).unwrap();

    vl = out_l.get_content().to_owned();

    let width = img.width();

    img.enumerate_pixels_mut().par_bridge().for_each(|(x, y, p)| {
        let i = (y*width + x) as usize;
        *p = Oklaba::from_components([vl[i], va[i], vb[i], valpha[i]]);
    });
}


pub fn exposition(img: &mut HslaImage, value: f32)
{
    img.enumerate_pixels_mut().par_bridge().for_each(|(_, _, pixel)| {
        *pixel.luma_mut() *= 2f32.powf(value);
    });
}

pub fn whitebalance<P>(img: &mut CRgbaImage<P>, fromtemp: f32, fromtint: f32, totemp: f32, totint: f32)
where 
    P: Pixel + Send + Sync + 'static + Debug + ToHsl,
    P::Subpixel: Scale + Primitive + std::fmt::Debug + Pod + Debug + Sync + Send + AsFloat
{
    let xyz_wb = xyz_wb_matrix(fromtemp, fromtint, totemp, totint);

    img.enumerate_pixels_mut().par_bridge().for_each(move |(_, _, p)| {
        let linsrgb = LinSrgba::from(p.to_rgba());
        let xyz = (Matrix::new(LINSRGB_TO_XYZ.to_vec(), 3, 3)* linsrgb.to_vec()[0..3].to_vec().into()).unwrap().get_content().to_owned();
        let scale = xyz[1];

        let downscaled = vec![xyz[0] / scale, 1.0, xyz[2] / scale];
        let mut new_v = (xyz_wb.clone() * downscaled.into()).unwrap().get_content().to_owned();

        new_v[0] *= scale;
        new_v[1] *= scale;
        new_v[2] *= scale;

        let rgb = (Matrix::new(XYZ_TO_LINSRGB.to_vec(), 3, 3) * new_v.into()).unwrap().get_content().to_owned();
        *p = cast_color_to_rgba(&LinSrgba::new(rgb[0], rgb[1], rgb[2], *linsrgb.alpha()));
    });

}

pub fn apply_curve(img: &RgbaImage, curve: crate::core::Curve) -> RgbaImage {
    let mut nb = RgbaImage::new(img.width(), img.height());

    for (x, y, pixel) in img.enumerate_pixels() {
        let cmp = pixel.channels();
        let np: Rgba<u8> = Rgba([
            (curve.apply_curve(cmp[0] as f32 / 255.0) * 255.0) as u8, 
            (curve.apply_curve(cmp[1] as f32 / 255.0) * 255.0) as u8, 
            (curve.apply_curve(cmp[2] as f32 / 255.0) * 255.0) as u8, 
            cmp[3]]
        );
        nb.put_pixel(x, y, np);
    }

    nb
}