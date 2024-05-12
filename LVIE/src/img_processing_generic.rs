#![allow(dead_code)]
use rayon::prelude::*;
use LVIElib::traits::{AsFloat, cast_color_to_rgba};
use LVIElib::hsl::HslaImage;
use LVIElib::linear_srgb::LinSrgba;
use LVIElib::white_balance::{xyz_wb_matrix, LINSRGB_TO_XYZ, XYZ_TO_LINSRGB};
use std::sync::{Arc, Mutex};
use image::{Pixel, Primitive, Rgb, Rgba, RgbaImage};
use LVIElib::matrix::{
    convolution::laplacian_of_gaussian, Matrix,
};

use LVIElib::oklab::{Oklaba, OklabaImage};

use LVIElib::matrix::convolution::convolve;

use LVIE_GPU::{CRgbaImage, Pod};
use LVIElib::traits::{Scale, ToHsl};
use std::fmt::Debug;

unsafe fn convert_hsla_to_rgba<P>(img: &HslaImage) -> Option<CRgbaImage<P>>
where 
    P: Pixel + Send + Sync + 'static + Debug + ToHsl,
    P::Subpixel: Scale + Primitive + std::fmt::Debug + Pod + Debug
{
    let mut out = CRgbaImage::<P>::new(img.width(), img.height());

    // if target is not rgb we cannot cast it
    if P::COLOR_MODEL != "RGB" {
        return None;
    }

    for (x, y, p) in img.enumerate_pixels() {
        let rgb = p.to_rgb();
        let cmp = rgb.channels();
        // We know what type is it but we have to return a generic, so we transmute it
        out.put_pixel(x, y, std::mem::transmute_copy::<Rgb<P::Subpixel>, P>(&Rgb([
            cmp[0].scale(),
            cmp[1].scale(),
            cmp[2].scale()
        ])));
    }

    Some(out)
}

pub fn saturate<P>(img: &CRgbaImage<P>, value: f32) -> CRgbaImage<P>
where 
    P: Pixel + Send + Sync + 'static + Debug + ToHsl,
    P::Subpixel: Scale + Primitive + std::fmt::Debug + Pod + Debug
{
    let mut hsl_image = HslaImage::from_vec(
        img.width(), img.height(), 
        {
        let mut out = Vec::<f32>::new();
        for (_,_, p) in img.enumerate_pixels() {
            for v in p.to_hsla().channels(){ out.push(*v); }
        }
        out
    }
    ).unwrap();

    for (_, _, pixel) in hsl_image.enumerate_pixels_mut() {
        *pixel.saturation_mut() = *pixel.saturation() + value / 10f32;
    }
    unsafe {
        convert_hsla_to_rgba(&hsl_image).unwrap()
    }
}

pub fn sharpen<P>(img: &CRgbaImage<P>, value: f32, size: usize) -> CRgbaImage<P> 
where 
    P: Pixel + Send + Sync + 'static + Debug + ToHsl,
    P::Subpixel: Scale + Primitive + std::fmt::Debug + Pod + Debug + AsFloat
{
    let (mut vl, mut va, mut vb, mut valpha) = (
        Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new()
    );
    let mut oklab_image = OklabaImage::new(img.width(), img.height());
    for (x, y, pixel) in img.enumerate_pixels() {
        let ok_pixel = Oklaba::from(pixel.to_rgba());
        let channels = ok_pixel.channels();

        vl.push(channels[0]);
        va.push(channels[1]);
        vb.push(channels[2]);
        valpha.push(channels[3]);

        oklab_image.put_pixel(x, y, ok_pixel);
    }

    let l_matrix = Matrix::new(vl, img.height() as usize, img.width() as usize);
    let kernel = laplacian_of_gaussian(value, size, size);

    let gradient = convolve(&l_matrix, &kernel);

    let out_l = (l_matrix - gradient).unwrap();

    vl = out_l.get_content().to_owned();

    let mut out = CRgbaImage::<P>::new(img.width(), img.height());

    let width = img.width();

    for i in 0..vl.len() {
        out.put_pixel(
            i as u32 % width,
            i as u32 / width,
            cast_color_to_rgba(&Oklaba::from_components([vl[i], va[i], vb[i], valpha[i]])),
        );
    }

    out
}


pub fn exposition<P>(img: &CRgbaImage<P>, value: f32) -> CRgbaImage<P>
where 
    P: Pixel + Send + Sync + 'static + Debug + ToHsl,
    P::Subpixel: Scale + Primitive + std::fmt::Debug + Pod + Debug + Sync + Send
{
    let out = Arc::new(Mutex::new(CRgbaImage::<P>::new(img.width(), img.height())));

    let out_w = out.clone();
    (0..img.height()).into_par_iter().for_each(|y| {
        let mut row = Vec::<P>::new();
        for x in 0..img.width(){
            let mut hsl = img.get_pixel(x, y).to_hsla();
            *hsl.luma_mut() *= 2f32.powf(value);
            row.push(cast_color_to_rgba(&hsl));
        }

        let mut out = out_w.lock().unwrap();
        for x in 0..img.width() { 
            out.put_pixel(x, y, row[x as usize]);
        }
    });

    return out.lock().unwrap().clone();
}

pub fn whitebalance<P>(img: &CRgbaImage<P>, fromtemp: f32, fromtint: f32, totemp: f32, totint: f32) -> CRgbaImage<P> 
where 
    P: Pixel + Send + Sync + 'static + Debug + ToHsl,
    P::Subpixel: Scale + Primitive + std::fmt::Debug + Pod + Debug + Sync + Send + AsFloat
{
    let out = Arc::new(Mutex::new(CRgbaImage::<P>::new(img.width(), img.height())));

    let xyz_wb = xyz_wb_matrix(fromtemp, fromtint, totemp, totint);

    let out_v = out.clone();
    (0..img.height()).into_par_iter().for_each(move |y| {
        let mut row = Vec::<P>::new();
        for x in 0..img.width() {
            let linsrgb = LinSrgba::from(img.get_pixel(x, y).to_rgba());
            let xyz = (Matrix::new(LINSRGB_TO_XYZ.to_vec(), 3, 3)* linsrgb.to_vec()[0..3].to_vec().into()).unwrap().get_content().to_owned();
            let scale = xyz[1];

            let downscaled = vec![xyz[0] / scale, 1.0, xyz[2] / scale];
            let mut new_v = (xyz_wb.clone() * downscaled.into()).unwrap().get_content().to_owned();

            new_v[0] *= scale;
            new_v[1] *= scale;
            new_v[2] *= scale;

            let rgb = (Matrix::new(XYZ_TO_LINSRGB.to_vec(), 3, 3) * new_v.into()).unwrap().get_content().to_owned();
            row.push(cast_color_to_rgba(&LinSrgba::new(rgb[0], rgb[1], rgb[2], *linsrgb.alpha())));
        }

        let mut out = out_v.lock().unwrap();
        for x in 0..img.width() {
            out.put_pixel(x, y, row[x as usize]);
        }
    });

    return out.lock().unwrap().clone();

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