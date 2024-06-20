use crate::{
    hsl::{Hsl, HslImage, Hsla, HslaImage},
    linear_srgb::LinSrgb,
    matrix::{convolution::split3, Matrix},
    oklab::{Oklab, OklabImage, OklabaImage}, traits::{Scale, ToHsl, ToOklab},
};
use std::ops::RangeInclusive;
use half::f16;
use num_traits::NumCast;
use rayon::prelude::*;
use image::{ImageBuffer, Pixel, Rgb, RgbImage, RgbaImage};
use std::sync::{Arc, Mutex};

pub fn norm_range<T: Primitive + PartialOrd>(r: RangeInclusive<T>, value: T) -> T {
    if r.start() <= &value && &value <= r.end() {
        return value;
    } else if &value < r.start() {
        return *r.start();
    } else {
        return *r.end();
    }
}

pub fn merge_channel<T: Copy>(colors: &mut Vec<T>, channel: usize, content: Vec<T>) {
    for i in 0..colors.len() {
        if i % 3 == channel {
            colors[i] = content[(i - channel) / 3]
        }
    }
}

pub fn l_channel_matrix(img: Matrix<u8>) -> Matrix<f32> {
    let (x, y) = (img.width(), img.height());
    let (rm, gm, bm) = split3(img);
    let (r, g, b) = (
        rm.get_content().to_owned(),
        gm.get_content().to_owned(),
        bm.get_content().to_owned(),
    );

    let mut l = Vec::<f32>::new();
    for i in 0..r.len() {
        let pix = Oklab::from(LinSrgb::from(Rgb([
            r[i] as f32 / 255.0,
            g[i] as f32 / 255.0,
            b[i] as f32 / 255.0,
        ])));
        l.push(*pix.l());
    }

    Matrix::new(l, y, x / 3)
}

pub fn show_l_channel(img: Matrix<f32>) -> Matrix<u8> {
    let black = Oklab::from(LinSrgb::from(Rgb([0.0, 0.0, 0.0])));

    let mut content: Vec<u8> = Vec::new();
    for L in img.get_content() {
        let srgb = Rgb::<f32>::from(LinSrgb::from(Oklab::from_components([
            *L,
            *black.a(),
            *black.b(),
        ])));
        let v = (srgb.0[0] * 255.0).round() as i32;
        match v {
            256..=i32::MAX => content.append(&mut vec![(v - 255) as u8, 100, 0]),
            0..=255 => content.append(&mut vec![v as u8; 3]),
            i32::MIN..=-1 => content.append(&mut vec![0, 0, (255 + v) as u8]),
        }
    }

    Matrix::new(content, img.height(), img.width() * 3)
}

#[allow(unused_must_use)]
pub fn convert_rgb_to_hsl(img: &RgbImage) -> HslImage {
    let out_v = Arc::new(Mutex::new(HslImage::new(img.width(), img.height())));
    
    let out_w = out_v.clone();
    (0..img.height()).into_par_iter().for_each(|y| {
        let mut buff: Vec<Hsl> = Vec::new();
        for x in 0..img.width() {
            buff.push((*img.get_pixel(x, y)).into());
        }
    
        let mut out = out_w.lock().unwrap();
        for x in 0..img.width() { 
            out.put_pixel(x, y, buff[x as usize]);
        }
    });

    drop(out_w);

    return Arc::try_unwrap(out_v).unwrap().into_inner().unwrap();
}

pub fn convert_rgba_to_hsla(img: &RgbaImage) -> HslaImage {
    let out_v = Arc::new(Mutex::new(HslaImage::new(img.width(), img.height())));
    
    let out_w = out_v.clone();
    
    (0..img.height()).into_par_iter().for_each(|y| {
        let mut buff: Vec<Hsla> = Vec::new();
        for x in 0..img.width() {
            buff.push((*img.get_pixel(x, y)).into());
        }
    
        let mut out = out_w.lock().unwrap();
        for x in 0..img.width() { 
            out.put_pixel(x, y, buff[x as usize]);
        }
    });

    drop(out_w);

    return Arc::try_unwrap(out_v).unwrap().into_inner().unwrap();
}

pub fn convert_hsl_to_rgb(img: &HslImage) -> RgbImage {
    let out_v = Arc::new(Mutex::new(RgbImage::new(img.width(), img.height())));
    
    let out_w = out_v.clone();
    
    (0..img.height()).into_par_iter().for_each(|y| {
        let mut buff: Vec<Rgb<u8>> = Vec::new();
        for x in 0..img.width() {
            buff.push((*img.get_pixel(x, y)).into());
        }
    
        let mut out = out_w.lock().unwrap();
        for x in 0..img.width() { 
            out.put_pixel(x, y, buff[x as usize]);
        }
    });

    drop(out_w);

    return Arc::try_unwrap(out_v).unwrap().into_inner().unwrap();
}

pub fn convert_rgb_to_oklab(img: &RgbImage) -> OklabImage {
    let out_v = Arc::new(Mutex::new(OklabImage::new(img.width(), img.height())));
    
    let out_w = out_v.clone();
    
    (0..img.height()).into_par_iter().for_each(|y| {
        let mut buff: Vec<Oklab> = Vec::new();
        for x in 0..img.width() {
            buff.push((*img.get_pixel(x, y)).into());
        }
    
        let mut out = out_w.lock().unwrap();
        for x in 0..img.width() { 
            out.put_pixel(x, y, buff[x as usize]);
        }
    });

    drop(out_w);

    return Arc::try_unwrap(out_v).unwrap().into_inner().unwrap();
}

pub fn convert_oklab_to_rgb(img: &OklabImage) -> RgbImage {
    let out_v = Arc::new(Mutex::new(RgbImage::new(img.width(), img.height())));
    
    let out_w = out_v.clone();
    
    (0..img.height()).into_par_iter().for_each(|y| {
        let mut buff: Vec<Rgb<u8>> = Vec::new();
        for x in 0..img.width() {
            buff.push((*img.get_pixel(x, y)).into());
        }
    
        let mut out = out_w.lock().unwrap();
        for x in 0..img.width() { 
            out.put_pixel(x, y, buff[x as usize]);
        }
    });

    drop(out_w);

    return Arc::try_unwrap(out_v).unwrap().into_inner().unwrap();
}
use image::Primitive;

pub fn _max<T: Primitive>(c: [T; 3]) -> (T, u8) {
    if c[0] > c[1] && c[0] > c[2] {
        (c[0], 0)
    } else if c[1] > c[0] && c[1] > c[2] {
        (c[1], 1)
    } else if c[2] > c[0] && c[2] > c[1] {
        (c[2], 2)
    } else {
        if c[0] == c[1] && c[1] != c[2] {
            (c[0], 0)
        } else if c[0] == c[2] && c[1] != c[2] {
            (c[0], 0)
        } else if c[1] == c[2] && c[1] != c[0] {
            (c[1], 1)
        } else if c[0] == c[1] && c[1] == c[2] {
            (c[0], 0)
        } else {
            panic!("Something went wrong");
        }
    }
}

pub fn _min<T: Primitive>(c: [T; 3]) -> (T, u8) {
    if c[0] < c[1] && c[0] < c[2] {
        (c[0], 0)
    } else if c[1] < c[0] && c[1] < c[2] {
        (c[1], 1)
    } else if c[2] < c[0] && c[2] < c[1] {
        (c[2], 2)
    } else {
        if c[0] == c[1] && c[1] != c[2] {
            (c[0], 0)
        } else if c[0] == c[2] && c[1] != c[2] {
            (c[0], 0)
        } else if c[1] == c[2] && c[1] != c[0] {
            (c[1], 1)
        } else if c[0] == c[1] && c[1] == c[2] {
            (c[0], 0)
        } else {
            panic!("Something went wrong");
        }
    }
}

#[allow(unused_must_use)]
pub fn convert_rgb_to_rgba(img: &image::RgbImage) -> image::RgbaImage {
    let out = Arc::new(Mutex::new(image::RgbaImage::new(img.width(), img.height())));

    let out_w = out.clone();
    img.enumerate_pixels().par_bridge().for_each(move |(x, y, pixel)| {
        out_w.lock().unwrap().put_pixel(x, y, image::Rgba([pixel.0[0], pixel.0[1], pixel.0[2], 255]));
    });
    
    Arc::try_unwrap(out).unwrap().into_inner().unwrap()
}

#[allow(unused_must_use)]
pub fn convert_rgba_to_rgb(img: &image::RgbaImage) -> image::RgbImage {
    let out = Arc::new(Mutex::new(image::RgbImage::new(img.width(), img.height())));

    let out_w = out.clone();
    img.enumerate_pixels().par_bridge().for_each(move |(x, y, pixel)| {
        out_w.lock().unwrap().put_pixel(x, y, image::Rgb([pixel.0[0], pixel.0[1], pixel.0[2]]));
    });

    Arc::try_unwrap(out).unwrap().into_inner().unwrap()
}

use plotters::prelude::*;

pub enum GraphColor {
    RED,
    GREEN, 
    BLUE
}

pub fn graph<X: Primitive, Y: Primitive, S: plotters::style::SizeDesc + num_traits::NumCast>(
    buf: &mut [u8], 
    size: (u32, u32),
    x: &Vec<&Vec<X>>, y: &Vec<&Vec<Y>>,
    x_max: &X, y_max: &Y,
    color: &Vec<GraphColor>,
    margins: (S, S, S, S),
) -> Result<(), Box<dyn std::error::Error>>{

    let root = BitMapBackend::with_buffer(buf, size).into_drawing_area();
    root.fill(&WHITE)?;

    let mut builder = ChartBuilder::on(&root);
    let m = margins;
    builder
        .margin_top(m.0)
        .margin_right(m.1)
        .margin_bottom(m.2)
        .margin_left(m.3);

    let mut chart = builder.build_cartesian_2d(
        0f32..NumCast::from(*x_max).unwrap(), 
        0f32..NumCast::from(*y_max).unwrap()
    )?;

    chart.configure_mesh().draw()?;
    
    for serie in 0..x.len() {
        let s_color = {
            match color[serie] {
                GraphColor::RED => RED,
                GraphColor::GREEN => GREEN,
                GraphColor::BLUE => BLUE
            }
        };
        
        chart.draw_series(
            AreaSeries::new(
                x[serie].iter().map(|x| <f32 as NumCast>::from(*x).unwrap())
                    .zip(y[serie].iter().map(|y| <f32 as NumCast>::from(*y).unwrap())),
                0f32,
                &s_color.mix(0.6),
            ).border_style(&s_color)
        )?;
    }

    Ok(())
}

pub fn f32_vec_to_f16_vec(src: &Vec<f32>, size: (u32, u32)) -> Vec<f16> {
    let vt = Arc::new(Mutex::new(vec![f16::default(); (size.0*size.1) as usize]));

    // parallel compute many rows at a single time
    (0..size.1).into_par_iter().for_each(|x| {
        let mut r: Vec<f16> = Vec::new();
        for k in &src[(x*size.0) as usize..(x*size.0 + size.0) as usize] {
            r.push(NumCast::from(*k).unwrap());
        }

        vt.lock().unwrap()[(x*size.0) as usize..(x*size.0 + size.0) as usize].copy_from_slice(&r);
    });

    Arc::try_unwrap(vt).unwrap().into_inner().unwrap()

}

pub unsafe fn convert_hsla_to_rgba<P>(img: &HslaImage) -> Option<ImageBuffer<P, Vec<P::Subpixel>>>
where 
    P: Pixel + Send + Sync + 'static + std::fmt::Debug + ToHsl,
    P::Subpixel: Scale + Send + Sync + Primitive + std::fmt::Debug + std::fmt::Debug
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

    ImageBuffer::<P, Vec<P::Subpixel>>::from_vec(img.width(), img.height(), Arc::try_unwrap(out).unwrap().into_inner().unwrap())
}

pub unsafe fn convert_oklaba_to_rgba<P>(img: &OklabaImage) -> Option<ImageBuffer<P, Vec<P::Subpixel>>>
where 
    P: Pixel + Send + Sync + 'static + std::fmt::Debug + ToOklab,
    P::Subpixel: Scale + Send + Sync + Primitive + std::fmt::Debug + std::fmt::Debug
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

    ImageBuffer::<P, Vec<P::Subpixel>>::from_vec(img.width(), img.height(), Arc::try_unwrap(out).unwrap().into_inner().unwrap())
}

use std::collections::VecDeque;

pub fn boundary_fill<P>(
    track: &ImageBuffer<P, Vec<P::Subpixel>>,
    x: Option<isize>,
    y: Option<isize>,
    img: &ImageBuffer<P, Vec<P::Subpixel>>,
    boundary_color: &P,
    opposite: bool,
) -> ImageBuffer<P, Vec<P::Subpixel>> 
where 
    P: image::Pixel + std::fmt::Debug + 'static + Send + Sync,
    P::Subpixel: image::Primitive + std::fmt::Debug + Send + Sync + Default + super::traits::Scale + Send + Sync
{
    let mut out: ImageBuffer<P, Vec<<P as Pixel>::Subpixel>> = {
        if opposite {
            img.clone()
        } else { 
            ImageBuffer::new(track.width(), track.height())
        }
    };

    let black = vec![0u8.scale(); P::CHANNEL_COUNT as usize];

    let mut stack: VecDeque<(isize, isize)> = VecDeque::new();
    stack.push_back((x.unwrap_or_default(), y.unwrap_or_default()));

    while let Some((x, y)) = stack.pop_back() {
        if x >= 0 && x < track.width() as isize && y >= 0 && y < track.height() as isize {
            if track.get_pixel(x as u32, y as u32).channels() != boundary_color.channels()
                && out.get_pixel(x as u32, y as u32).channels() != {
                    if opposite {
                        black.as_slice()
                    } else {
                        img.get_pixel(x as u32, y as u32).channels()
                    }
                }
            {
                out.put_pixel(x as u32, y as u32, {
                    if opposite {
                        *P::from_slice(&vec![0u8.scale(); P::CHANNEL_COUNT as usize])
                    } else {
                        *img.get_pixel(x as u32, y as u32)
                    }});
                stack.push_back((x + 1, y));
                stack.push_back((x - 1, y));
                stack.push_back((x, y + 1));
                stack.push_back((x, y - 1));
                stack.push_back((x + 1, y + 1));
                stack.push_back((x - 1, y - 1));
                stack.push_back((x + 1, y - 1));
                stack.push_back((x - 1, y + 1));
            } 
        }
    }

    out
}