use palette::{Srgb, Hsl};
use std::vec::Vec;

mod buffer_struct;

use buffer_struct::Buffer;

use std::time::Instant;

use image::{self, GenericImageView, Pixel};

fn main() {

    println!("Starting...");

    let now = Instant::now();

    let img = image::open("src\\prova.jpg").unwrap();

    println!("Image loaded!\n - Image size: {:?}", img.dimensions());

    let mut buff: Vec<Vec<f64>> = Vec::new();

    for i in img.pixels(){
        let rgb: Vec<u8> = i.2.channels().to_vec();
        buff.push(vec![(rgb[0] / 255) as f64, (rgb[1] / 255) as f64, (rgb[2] / 255) as f64]);
    }

    println!("Buffer created (len: {})", buff.len());

    let mut buffer: Buffer = Buffer::<Srgb>::from_f64_buffer(&buff);

    let elapsed = now.elapsed();

    println!("Loaded buffer\n - Elapsed: {}", elapsed.as_secs());

    let now = Instant::now();

    let new_buffer: Buffer<Hsl> = buffer.convert_to::<Hsl>();

    println!("Converted buffer to Hsl\n - Elapsed: {}", now.elapsed().as_secs())

}
