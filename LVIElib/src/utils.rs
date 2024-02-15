use crate::{
    hsl::{Hsl, Hsla, HslImage, HslaImage},
    linear_srgb::LinSrgb,
    matrix::{convolution::split3, Matrix},
    oklab::{Oklab, OklabImage},
};
use std::ops::RangeInclusive;
use rayon::prelude::*;
use image::{Rgb, Rgba, RgbImage, RgbaImage};
use std::sync::{Arc, Mutex};

pub fn norm_range_f32(r: RangeInclusive<f32>, value: f32) -> f32 {
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

    return out_v.lock().unwrap().clone();
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

    return out_v.lock().unwrap().clone();
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

    return out_v.lock().unwrap().clone();
}

pub fn convert_hsla_to_rgba(img: &HslaImage) -> RgbaImage {
    let out_v = Arc::new(Mutex::new(RgbaImage::new(img.width(), img.height())));
    
    let out_w = out_v.clone();
    
    (0..img.height()).into_par_iter().for_each(|y| {
        let mut buff: Vec<Rgba<u8>> = Vec::new();
        for x in 0..img.width() {
            buff.push((*img.get_pixel(x, y)).into());
        }
    
        let mut out = out_w.lock().unwrap();
        for x in 0..img.width() { 
            out.put_pixel(x, y, buff[x as usize]);
        }
    });

    return out_v.lock().unwrap().clone();
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

    return out_v.lock().unwrap().clone();
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

    return out_v.lock().unwrap().clone();
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