use image;
use rustfft::{num_complex::Complex, FftDirection};

use crate::matrix::{Matrix, convolution::apply_convolution};

mod matrix;

fn main() {
    let m: Matrix<Complex<f32>> = Matrix::new(
        vec![
            1.0, 2.0, 3.0, 4.0, 0.0, 1.0, 0.0, -1.0, 0.0, 3.0, -5.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        4,
        4,
    )
    .into();
    
    println!("{m}");
    let m = m.fft2d(FftDirection::Forward);
    println!("{m}");
    let m = m.fft2d(FftDirection::Inverse);
    println!("{m}");

    let img = image::open("/home/bolli/Pictures/fllauncher.png").unwrap().to_rgb8();
    let ((width, height), img_buf) = (img.dimensions(),img.into_raw());
    println!("Dimensions: {} x {}", width, height);
    let matrix = Matrix::new(img_buf, height as usize, 3*width as usize);
    let mut kernel = Matrix::new(vec![0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0], 3, 3);
    kernel.pad(height as usize, width as usize, 0.0);
    image::save_buffer("/home/bolli/Desktop/roustput.png", &apply_convolution(matrix, &kernel), width, height, image::ColorType::Rgb8).unwrap();
}
