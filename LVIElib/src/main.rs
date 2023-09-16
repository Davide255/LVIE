#![allow(non_snake_case)]
mod matrix;

#[cfg(test)]
mod tests {
    /*use std::fmt::Error;

    use crate::matrix::{convolution::standard::apply_convolution, Matrix};
    use image;

    #[test]
    fn test_convolution() -> Result<(), Error> {
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
        Ok(())
    }*/

    use LVIElib::hsl::*;

    #[test]
    fn test_hsl_converion() {
        let c = rgbf32_to_hslf32(127.0, 177.0, 77.0).0;
        assert_eq!([127, 177, 77], hslf32_to_rgb8(c[0], c[1], c[2]).0);
    }
}

use image::Rgb;
use LVIElib::hsl::*;

fn main() {
    let rgb: Rgb<u8> = Rgb([100, 100, 100]);
    let hsl: Hsl = rgb.into();
    println!("{:?}", hsl)
}
