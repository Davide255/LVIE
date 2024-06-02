use image::{ImageBuffer, Pixel, Rgb, Rgba, RgbaImage};
use LVIElib::{hsl::{Hsl, Hsla}, oklab::{Oklab, Oklaba}, traits::Scale, utils::{convert_hsla_to_rgba, convert_oklaba_to_rgba}};

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
            let f = from_color.parse().into_iter().map(|v| { v.scale() }).collect::<Vec<f32>>();
            let t = to_color.parse().into_iter().map(|v| { v.scale() }).collect::<Vec<f32>>();

            let steps = [(f[0]-t[0]) / (width-1) as f32, (f[1]-t[1]) / (width-1) as f32, (f[2]-t[2]) / (width-1) as f32, 0.0];
            
            let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                if i%4 == 3 { 1f32 } else { f[i % 4] - steps[i % 4]*((i / 4) as f32).floor() } 
            }).collect::<Vec<f32>>();
            
            let mut img: Vec<f32> = Vec::new();
            for _ in 0..height { img.append(&mut row.clone()) };

            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width, height, img.into_iter().map(|v| {v.scale()}).collect::<Vec<u8>>()).unwrap()
        },
        ColorSpace::RGBA => {
            let f = from_color.parse().into_iter().map(|v| { v.scale() }).collect::<Vec<f32>>();
            let t = to_color.parse().into_iter().map(|v| { v.scale() }).collect::<Vec<f32>>();

            let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, (f[3]-t[3]) / width as f32];
            
            let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                f[i % 4] - steps[i % 4]*((i/4) as f32).floor()
            }).collect::<Vec<f32>>();
            
            let mut img: Vec<f32> = Vec::new();
            for _ in 0..height { img.append(&mut row.clone()) };
            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width, height, img.into_iter().map(|v| {v.scale()}).collect::<Vec<u8>>()).unwrap()
        },
        ColorSpace::OKLAB => {
            let a = Oklab::from(*Rgb::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Oklab::from(*Rgb::from_slice(&to_color.parse()));
            let t = b.channels();

            let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, 0.0];
            
            let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                if i%4 == 3 { 1f32 } else {f[i % 4] - steps[i % 4]*((i / 4) as f32).floor() } 
            }).collect::<Vec<f32>>();
            
            let mut img: Vec<f32> = Vec::new();
            for _ in 0..height { img.append(&mut row.clone()) };
            unsafe { convert_oklaba_to_rgba(&ImageBuffer::<Oklaba, Vec<f32>>::from_vec(width, height, img).unwrap()).unwrap() }
        },
        ColorSpace::OKLABA => {
            let a = Oklaba::from(*Rgba::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Oklaba::from(*Rgba::from_slice(&to_color.parse()));
            let t = b.channels();

            let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, (f[3]-t[3]) / width as f32];
            
            let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                f[i % 4] - steps[i % 4]*((i / 4) as f32).floor()
            }).collect::<Vec<f32>>();
            
            let mut img: Vec<f32> = Vec::new();
            for _ in 0..height { img.append(&mut row.clone()) };
            unsafe { convert_oklaba_to_rgba(&ImageBuffer::<Oklaba, Vec<f32>>::from_vec(width, height, img).unwrap()).unwrap() }
        },
        ColorSpace::HSL => {
            let a = Hsl::from(*Rgb::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Hsl::from(*Rgb::from_slice(&to_color.parse()));
            let t = b.channels();

            let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, 0.0];
            
            let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                if i%4 == 3 { 1f32 } else {f[i % 4] - steps[i % 4]*((i / 4) as f32).floor() } 
            }).collect::<Vec<f32>>();
            
            let mut img: Vec<f32> = Vec::new();
            for _ in 0..height { img.append(&mut row.clone()) };
            unsafe { convert_hsla_to_rgba(&ImageBuffer::<Hsla, Vec<f32>>::from_vec(width, height, img).unwrap()).unwrap() }
        },
        ColorSpace::HSLA => {
            let a = Hsla::from(*Rgba::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Hsla::from(*Rgba::from_slice(&to_color.parse()));
            let t = b.channels();

            let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, (f[3]-t[3]) / width as f32];
            
            let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                f[i % 4] - steps[i % 4]*((i / 4) as f32).floor()
            }).collect::<Vec<f32>>();
            
            let mut img: Vec<f32> = Vec::new();
            for _ in 0..height { img.append(&mut row.clone()) };
            unsafe { convert_hsla_to_rgba(&ImageBuffer::<Hsla, Vec<f32>>::from_vec(width, height, img).unwrap()).unwrap() }
        }
    }
}