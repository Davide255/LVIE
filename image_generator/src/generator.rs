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

pub fn linear_gradient(width: u32, height: u32, color_space: ColorSpace, colors: Vec<(Color, f32)>, angle: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    match color_space {
        ColorSpace::RGB => {
            LVIElib::math::linear_gradient_more_points(
                (width, height), 
                colors.into_iter().map(|(c, p)| {
                    (Rgb::from_slice(&c.parse()).to_rgba(), p)
                }).collect(),
                angle
            )
        },
        ColorSpace::RGBA => {
            LVIElib::math::linear_gradient_more_points(
                (width, height), 
                colors.into_iter().map(|(c, p)| {
                    (*Rgba::from_slice(&c.parse()), p)
                }).collect(),
                angle
            )
        },
        ColorSpace::OKLAB => {
            let image_buff = LVIElib::math::linear_gradient_more_points(
                (width, height), 
                colors.into_iter().map(|(c, p)| {
                    (Oklaba::from(Rgb::from_slice(&c.parse()).to_rgba()), p)
                }).collect(),
                angle
            );

            unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
        },
        ColorSpace::OKLABA => {
            let image_buff = LVIElib::math::linear_gradient_more_points(
                (width, height), 
                colors.into_iter().map(|(c, p)| {
                    (Oklaba::from(*Rgba::from_slice(&c.parse())), p)
                }).collect(),
                angle
            );

            unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
        },
        ColorSpace::HSL => {
            let image_buff = LVIElib::math::linear_gradient_more_points(
                (width, height), 
                colors.into_iter().map(|(c, p)| {
                    (Hsla::from(Rgb::from_slice(&c.parse()).to_rgba()), p)
                }).collect(),
                angle
            );

            unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
        },
        ColorSpace::HSLA => {
            let image_buff = LVIElib::math::linear_gradient_more_points(
                (width, height), 
                colors.into_iter().map(|(c, p)| {
                    (Hsla::from(*Rgba::from_slice(&c.parse())), p)
                }).collect(),
                angle
            );

            unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
        }
    }
}