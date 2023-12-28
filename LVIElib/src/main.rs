#![allow(non_snake_case)]
use image::{self, Rgb};
use LVIElib::image_geometry::homography;
use LVIElib::linear_srgb::LinSrgb;
use LVIElib::spline::{apply_curve, spline_coefficients};

use LVIElib::contrast::set_contrast;
use LVIElib::matrix::convolution::{convolve, laplacian_of_gaussian, split3};
use LVIElib::matrix::Matrix;
use LVIElib::white_balance::wb_matrix;

fn main() {
    let img = image::open("IMG_4230.JPG").unwrap().to_rgb8();
    let ((width, height), img_buf) = (img.dimensions(), img.into_raw());

    let mat = wb_matrix(6500.0, 0.0, 6500.0, 1.0);

    let mut out = Vec::<u8>::new();

    for i in 0..img_buf.len() / 3 {
        let pix: LinSrgb = Rgb([
            img_buf[3 * i] as f32 / 255.0,
            img_buf[3 * i + 1] as f32 / 255.0,
            img_buf[3 * i + 2] as f32 / 255.0,
        ])
        .into();

        let v = (mat.clone() * pix.to_vec().into())
            .unwrap()
            .consume_content();

        let rgb: Rgb<f32> = LinSrgb::new(v[0], v[1], v[2]).into();

        out.push((rgb[0] * 255.0) as u8);
        out.push((rgb[1] * 255.0) as u8);
        out.push((rgb[2] * 255.0) as u8);
    }

    image::save_buffer(
        "white_balance_rust.png",
        &out,
        width,
        height,
        image::ColorType::Rgb8,
    )
    .unwrap();

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
