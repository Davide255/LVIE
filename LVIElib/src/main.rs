#![allow(non_snake_case)]
use image;

use crate::matrix::{convolution::standard::apply_convolution, Matrix};

mod matrix;

fn main() {
    let img = image::open("original.jpg").unwrap().to_rgb8();
    let ((width, height), img_buf) = (img.dimensions(), img.into_raw());
    println!("Dimensions: {} x {}", width, height);
    let matrix = Matrix::new(img_buf, height as usize, 3 * width as usize);

    let mut kernel = Matrix::new(vec![0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0], 3, 3);
    kernel.pad(width as usize, height as usize, 0.0);

    let convolved = apply_convolution(matrix, &kernel);

    println!("Correct size: {}", convolved.check_size());
    println!("{}, {}", convolved.width(), convolved.height());

    image::save_buffer(
        "roustput.png",
        convolved.get_content(),
        (convolved.width() / 3) as u32,
        convolved.height() as u32,
        image::ColorType::Rgb8,
    )
    .unwrap();
}
