#![allow(non_snake_case)]
use image;
use LVIElib::sharpening::sharpening;
use LVIElib::{l_channel_matrix, show_l_channel};

use LVIElib::contrast::set_contrast;
use LVIElib::matrix::convolution::{convolve, laplacian_of_gaussian};
use LVIElib::matrix::Matrix;

fn main() {
    let img = image::open("IMG_4230.JPG").unwrap().to_rgb8();
    /*let ((width, height), img_buf) = (img.dimensions(), img.into_raw());
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
    .unwrap();*/
    let (x, y) = img.dimensions();
    let img_matrix = Matrix::new(img.into_raw(), y as usize, (x * 3) as usize);
    let matrix = sharpening(img_matrix, 5, 1.4);
    /*let (mr, mg, mb) = split3(matrix);
    let (r, g, b) = (
        histogram_equalize(mr.get_content().clone()),
        histogram_equalize(mg.get_content().clone()),
        histogram_equalize(mb.get_content().clone()),
    );
    let mut output: Vec<u8> = Vec::new();
    for i in 0..(x * y) as usize {
        output.push(r[i]);
        output.push(g[i]);
        output.push(b[i]);
    }*/

    image::save_buffer(
        "roustput_sharpen.png",
        matrix.get_content(),
        x,
        y,
        image::ColorType::Rgb8,
    )
    .unwrap();

    /*let col = linear_srgbf32_to_oklabf32(16.0 / 255.0, 180.0 / 255.0, 33.0 / 255.0);
    let rgb = oklabf32_to_linear_srgbf32(col.L, col.a, col.b);
    println!(
        "RGB: (16, 180, 33), OkLab: ({}, {}, {}), RGB: ({}, {}, {})",
        col.L, col.a, col.b, rgb[0]*255.0, rgb[1]*255.0, rgb[2]*255.0
    );*/
}
