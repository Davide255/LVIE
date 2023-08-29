//use palette::{Srgb, Hsl};
use palette::Srgb;
use std::vec::Vec;

mod buflib;
mod helpers;
mod buffer_struct;

use buflib::*;

use buffer_struct::Buffer;

use std::time::Instant;

use image::{GenericImageView, Pixel};

use std::io;
use std::io::prelude::*;

fn pause() {
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {

    let rgb_color: Srgb<u8> = Srgb::new(100u8, 255u8, 30u8);

    println!("rgb_color size: {}", std::mem::size_of_val(&rgb_color));

    return;

    println!("Starting...");

    let img = image::open("src\\original.jpg").unwrap();

    println!("Image loaded!\n - Image size: {:?}", img.dimensions());

    pause();

    let mut buffer: Buffer = Buffer::new(img.dimensions());

    println!("Size of buffer: {}", buffer.get_size_in_byte());

    pause();

    for (_, _, pixel) in img.pixels(){
        let rgb: Vec<u8> = pixel.channels().to_vec();
        buffer.add_item(Srgb::new(rgb[0] as f32 / 255f32, rgb[1] as f32 / 255f32, rgb[2] as f32 / 255f32));
    }

    println!("Buffer created (len: {})", buffer.len());
    println!("Size of buffer: {}", buffer.get_size_in_byte());

    drop(img);

    println!("img dropped");

    pause();

    //let now = Instant::now();
    //
    //img.adjust_contrast(3f32).save_with_format("contrast_img.jpeg", image::ImageFormat::Jpeg);
    //
    //println!("Contrast added in {}ms (img mathod)", now.elapsed().as_millis() as f64 / 1000f64);
    //
    //pause();

    let now = Instant::now();
    
    adjust_contrast(&&buffer, 0.1).save_jpeg_image("contrast_buff.jpeg").expect("Failed to save the image");

    println!("Contrast added in {}ms (buffer mathod)", now.elapsed().as_millis() as f64 / 1000f64);

    pause();

    drop(buffer);

    pause();

    return;

    let new_buffer = buffer.get_area((1500, 1000), (1000, 1000));

    let box_blur_mask: [[f32;3];3] = [
        [1f32 / 9f32, 1f32 / 9f32, 1f32 / 9f32], 
        [1f32 / 9f32, 1f32 / 9f32, 1f32 / 9f32], 
        [1f32 / 9f32, 1f32 / 9f32, 1f32 / 9f32]
    ];

    let sharpening_mask: [[f32; 3]; 3] = [
        [0f32, -1f32, 0f32],
        [-1f32, 5f32, -1f32],
        [0f32, -1f32, 0f32]
    ];

    let gaussian_blur: [[f32; 3]; 3] = [
        [1f32 / 16f32, 1f32 / 8f32, 1f32 / 16f32],
        [1f32 / 8f32, 1f32 / 4f32, 1f32 / 8f32],
        [1f32 / 16f32, 1f32 / 8f32, 1f32 / 16f32]
    ];

    //new_buffer
    //.apply_convolution_mask(gaussian_blur)
    //.save_jpeg_image("src\\out2_300x300.jpeg")
    //.expect("Failed to save image");

    let now = Instant::now();

    box_blur(&new_buffer, 15)
    .save_jpeg_image("src\\out2_1000x1000.jpeg")
    .expect("Failed to save the image");

    println!("Blurred buffer\n - Elapsed: {}", now.elapsed().as_millis() as f64 / 1000f64 );

    return;

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

    res.save_jpeg_image("src\\out.jpeg").expect("Failed to save image")

}
