use palette::{Srgb, Hsl};
use std::vec::Vec;

mod buffer_struct;
mod buflib;
mod helpers;

use buflib::*;

use buffer_struct::Buffer;

use std::time::Instant;

use image::{self, GenericImageView, Pixel, Rgb};

fn main() {

    println!("Starting...");

    let now = Instant::now();

    let img = image::open("src\\prova.jpg").unwrap();

    println!("Image loaded!\n - Image size: {:?}", img.dimensions());

    let mut buff: Vec<Vec<f64>> = Vec::new();

    for i in img.pixels(){
        let rgb: Vec<u8> = i.2.channels().to_vec();
        buff.push(vec![rgb[0] as f64 / 255f64, rgb[1] as f64 / 255f64, rgb[2] as f64 / 255f64]);
    }

    println!("Buffer created (len: {})", buff.len());

    let buffer: Buffer = Buffer::<Srgb>::from_f64_buffer(&buff, img.dimensions());

    let elapsed = now.elapsed();

    println!("Loaded buffer\n - Elapsed: {}", elapsed.as_secs());

    let now = Instant::now();

    let w_contrast = adjust_contrast(&buffer, 0.1);
    let w_saturation = adjust_saturation(&w_contrast, 0.3);
    let w_expo = adjust_exposure(&w_saturation, -0.3);

    let box_blur_mask: [[f32;3];3] = [
        [1f32 / 9f32, 1f32 / 9f32, 1f32 / 9f32], 
        [1f32 / 9f32, 1f32 / 9f32, 1f32 / 9f32], 
        [1f32 / 9f32, 1f32 / 9f32, 1f32 / 9f32]
    ];

    let res = w_expo.apply_convolution_mask(box_blur_mask);

    println!("Saturated buffer\n - Elapsed: {}", now.elapsed().as_secs());

    res.save_jpeg_image("src\\out.jpeg", img.dimensions()).expect("Failed to save image")

}
