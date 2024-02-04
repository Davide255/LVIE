#![allow(non_snake_case)]
#![allow(unused_imports)]
use LVIElib::image_geometry::homography;
use LVIElib::spline::{apply_curve, spline_coefficients};

use LVIElib::contrast::set_contrast;
use LVIElib::matrix::convolution::{convolve, laplacian_of_gaussian, split3};
use LVIElib::matrix::Matrix;

use image::{Rgb, RgbImage, Pixel};

use LVIElib::hsl::{HslImage, Hsl};
use LVIElib::generic_color::PixelMapping;

fn main() {
    let coef = spline_coefficients(&vec![0.0, 20.7, 80.8, 165.5, 224.6, 255.0]);
    let x = vec![0.0, 59.1, 117.2, 152.7, 195.0, 255.0];

    let img = image::open("IMG_4230.JPG").unwrap().to_rgb8();
    let ((width, height), mut img_buf) = (img.dimensions(), img.into_raw());

    for i in 0..img_buf.len() {
        img_buf[i] = apply_curve(img_buf[i] as f32, &coef, &x) as u8;
    }

    image::save_buffer("curve.png", &img_buf, width, height, image::ColorType::Rgb8).unwrap();

    /* SHARPENING
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
    */

    /* HOMOGRAPHIES
    let (x, y) = img.dimensions();
    let img_matrix = Matrix::new(img.into_raw(), y as usize, (x * 3) as usize);

    #[rustfmt::skip]
    let homography_matrix = Matrix::new(
        vec![
             1.0,  0.02,  0.01,
             0.02, 1.0,  -0.02,
             0.0,  0.0,   1.0,
        ], 3, 3
    );

    let (mut mr, mut mg, mut mb) = split3(img_matrix);

    homography(homography_matrix.clone(), &mut mr, 128);
    homography(homography_matrix.clone(), &mut mg, 128);
    homography(homography_matrix, &mut mb, 128);

    let (r, g, b) = (
        /*histogram_equalize(mr.get_content().clone()),
        histogram_equalize(mg.get_content().clone()),
        histogram_equalize(mb.get_content().clone()),*/
        mr.consume_content(),
        mg.consume_content(),
        mb.consume_content(),
    );

    let mut output: Vec<u8> = Vec::new();
    for i in 0..(x * y) as usize {
        output.push(r[i]);
        output.push(g[i]);
        output.push(b[i]);
    }

    image::save_buffer(
        "roustput_homography.png",
        &output,
        x,
        y,
        image::ColorType::Rgb8,
    )
    .unwrap();
    */

    /*let col = linear_srgbf32_to_oklabf32(16.0 / 255.0, 180.0 / 255.0, 33.0 / 255.0);
    let rgb = oklabf32_to_linear_srgbf32(col.L, col.a, col.b);
    println!(
        "RGB: (16, 180, 33), OkLab: ({}, {}, {}), RGB: ({}, {}, {})",
        col.L, col.a, col.b, rgb[0]*255.0, rgb[1]*255.0, rgb[2]*255.0
    );*/
}