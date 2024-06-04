use image::{ImageBuffer, Pixel, Rgb, Rgba, RgbaImage};
use LVIElib::{hsl::Hsla, oklab::Oklaba, utils::{convert_hsla_to_rgba, convert_oklaba_to_rgba}};

use crate::{Color, ColorSpace};

pub fn solid_fill(width: u32, height: u32, color: Color, color_space: ColorSpace) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    
    let mut cmps = color.parse();
    
    let c = match color_space {
        ColorSpace::RGB => {
            cmps.push(255);
            Rgba::from_slice(&cmps)
        },
        ColorSpace::RGBA => {
            Rgba::from_slice(&cmps)
        },
        _ => unimplemented!("Cannot parse other color spaces")
    };

    let mut img = RgbaImage::new(width, height);

    img.enumerate_pixels_mut().for_each(|(_, _, p)| {
        *p = c.clone();
    });

    img
}

pub fn linear_gradient(width: u32, height: u32, color_space: ColorSpace, from_color: Color, to_color: Color, angle: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    match color_space {
        ColorSpace::RGB => {
            LVIElib::math::linear_gradient(
                (width, height), 
                vec![
                    (Rgb::from_slice(&from_color.parse()).to_rgba(), 0.0),
                    (Rgb::from_slice(&to_color.parse()).to_rgba(), 100.0),
                    ], 
                angle
            )
        },
        ColorSpace::RGBA => {
            LVIElib::math::linear_gradient(
                (width, height), 
                vec![
                    (*Rgba::from_slice(&from_color.parse()), 0.0),
                    (*Rgba::from_slice(&to_color.parse()), 100.0),
                    ], 
                angle
            )
        },
        ColorSpace::OKLAB => {
            let a = Oklaba::from(Rgb::from_slice(&from_color.parse()).to_rgba());
            let b = Oklaba::from(Rgb::from_slice(&to_color.parse()).to_rgba());

            let image_buff = LVIElib::math::linear_gradient(
                (width, height), 
                vec![(a, 0.0), (b, 100.0)], 
                angle
            );

            unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
        },
        ColorSpace::OKLABA => {
            let a = Oklaba::from(*Rgba::from_slice(&from_color.parse()));
            let b = Oklaba::from(*Rgba::from_slice(&to_color.parse()));

            let image_buff = LVIElib::math::linear_gradient(
                (width, height), 
                vec![(a, 0.0), (b, 100.0)], 
                angle
            );

            unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
        },
        ColorSpace::HSL => {
            let a = Hsla::from(Rgb::from_slice(&from_color.parse()).to_rgba());
            let b = Hsla::from(Rgb::from_slice(&to_color.parse()).to_rgba());

            let image_buff = LVIElib::math::linear_gradient(
                (width, height), 
                vec![(a, 0.0), (b, 100.0)], angle
            );

            unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
        },
        ColorSpace::HSLA => {
            let a = Hsla::from(*Rgba::from_slice(&from_color.parse()));
            let b = Hsla::from(*Rgba::from_slice(&to_color.parse()));

            let image_buff = LVIElib::math::linear_gradient(
                (width, height), 
                vec![(a, 0.0), (b, 100.0)], angle
            );

            unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
        }
    }
}