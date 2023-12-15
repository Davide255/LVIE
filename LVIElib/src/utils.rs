use crate::{
    hsl::{Hsl, HslImage},
    linear_srgb::LinSrgb,
    matrix::{convolution::split3, Matrix},
    oklab::{Oklab, OklabImage},
};
use std::ops::RangeInclusive;

use image::{Rgb, RgbImage};

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

pub fn convert_rgb_to_hsl(img: &RgbImage) -> HslImage {
    let mut hsl_img = HslImage::new(img.width(), img.height());

    for (x, y, pixel) in img.enumerate_pixels() {
        if *Hsl::from(*pixel).saturation() > 1.0 {
            panic!();
        }
        hsl_img.put_pixel(x, y, (*pixel).into());
    }

    hsl_img
}

pub fn convert_hsl_to_rgb(img: &HslImage) -> RgbImage {
    let mut hsl_img = RgbImage::new(img.width(), img.height());

    for (x, y, pixel) in img.enumerate_pixels() {
        hsl_img.put_pixel(x, y, (*pixel).into());
    }

    hsl_img
}

pub fn convert_rgb_to_oklab(img: &RgbImage) -> OklabImage {
    let mut oklab_image = OklabImage::new(img.width(), img.height());
    for (x, y, pixel) in img.enumerate_pixels() {
        oklab_image.put_pixel(x, y, (*pixel).into());
    }
    oklab_image
}

pub fn convert_oklab_to_rgb(img: &OklabImage) -> RgbImage {
    let mut rgb_image = RgbImage::new(img.width(), img.height());
    for (x, y, pixel) in img.enumerate_pixels() {
        rgb_image.put_pixel(x, y, (*pixel).into());
    }
    rgb_image
}
