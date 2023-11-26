#![allow(non_snake_case)]

pub mod contrast;
pub mod math;
pub mod matrix;

pub type Matrix<T> = matrix::Matrix<T>;
pub type Complex<T> = rustfft::num_complex::Complex<T>;

use image::Rgb;
use linear_rgb::{linear_to_srgb, srgb_to_linear};
use matrix::convolution::split3;
use oklab::{linear_srgbf32_to_oklabf32, oklabf32_to_linear_srgbf32, OkLab};
use rustfft::FftDirection;

pub type FFTDirection = FftDirection;

pub mod hsl;
pub mod image_geometry;
pub mod linear_rgb;
pub mod oklab;
pub mod sharpening;
pub mod spline;

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
        let pix = linear_srgbf32_to_oklabf32(srgb_to_linear(Rgb([
            r[i] as f32 / 255.0,
            g[i] as f32 / 255.0,
            b[i] as f32 / 255.0,
        ])));
        l.push(pix.L);
    }

    Matrix::new(l, y, x / 3)
}

pub fn show_l_channel(img: Matrix<f32>) -> Matrix<u8> {
    let black = linear_srgbf32_to_oklabf32(srgb_to_linear(Rgb([0.0, 0.0, 0.0])));

    let mut content: Vec<u8> = Vec::new();
    for L in img.get_content() {
        let srgb = linear_to_srgb(oklabf32_to_linear_srgbf32(OkLab::from_components(
            *L, black.a, black.b,
        )));
        let v = (srgb.0[0] * 255.0).round() as i32;
        match v {
            256..=i32::MAX => content.append(&mut vec![(v - 255) as u8, 100, 0]),
            0..=255 => content.append(&mut vec![v as u8; 3]),
            i32::MIN..=-1 => content.append(&mut vec![0, 0, (255 + v) as u8]),
        }
    }

    Matrix::new(content, img.height(), img.width() * 3)
}
