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

    use image::{Pixel, Rgb};
    use LVIElib::hsl::*;

    #[test]
    fn test_hsl_conversion() {
        let rgb: Rgb<u8> = Rgb([255, 157, 44]);
        assert_eq!(rgb, Rgb::from(Hsl::from(rgb)));
    }

    #[test]
    fn test_functions() {
        let mut hsl = Hsl::new(0f32, 0f32, 0f32);

        println!("{:?}", hsl.channels());
        hsl.map(|x: f32| x);

        println!("{:?}", Hsl::from_slice(hsl.as_slice()));
        println!("{:?}", hsl.invert());
    }
}

use image::Rgb;
use LVIElib::hsl::Hsl;

#[allow(unreachable_code)]
fn main() {
    println!("{:?}", Hsl::from(Rgb([255u8, 202u8, 166u8])))
}
