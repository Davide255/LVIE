use image::{ImageBuffer, Pixel, Rgb, Rgba, RgbaImage};
use LVIElib::{hsl::{Hsl, Hsla, HslaImage}, oklab::{Oklab, Oklaba, OklabaImage}, traits::Scale, utils::{convert_hsla_to_rgba, convert_oklaba_to_rgba}};

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

            let angle = angle % 360.0;

            let flip = {
                if 0.0 < angle && angle <= 90.0 { (false, false) }
                else if 90.0 < angle && angle <= 180.0 { (true, false) }
                else if 180.0 < angle && angle <= 270.0 { (true, true) }
                else { (false, true) }
            };

            let angle = angle % 90.0;
            
            if angle == 0.0 {
                let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, 0.0];

                let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                    if i%4 == 3 { 1f32 } else {f[i % 4] - steps[i % 4]*((i / 4) as f32).floor() } 
                }).collect::<Vec<f32>>();

                let mut img: Vec<f32> = Vec::new();
                for _ in 0..height { img.append(&mut row.clone()) };
                let mut image_buff = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width, height, img.into_iter().map(|v| {v.scale()}).collect::<Vec<u8>>()).unwrap();

                if flip.0 { image_buff = image::imageops::flip_horizontal(&image_buff); }
                image_buff
            
            } else {
                let mut image_buff = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
                let r = [-angle.to_radians().tan(), 1.0];

                let a = -1.0/r[0];
                let b = 1f32;
                let c = -a*width as f32 - height as f32;

                let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();
                let steps = [(f[0]-t[0]), (f[1]-t[1]), (f[2]-t[2]), 0.0];

                for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
                    let a = -1.0/r[0];
                    let b = 1f32;
                    let c = -a*x as f32 - height as f32 + y as f32;

                    let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

                    let channels = [
                        (f[0] - steps[0] * s/w).scale(), 
                        (f[1] - steps[1] * s/w).scale(), 
                        (f[2] - steps[2] * s/w).scale(), 
                        255
                    ];
                    
                    *pixel = *Rgba::from_slice(channels.as_slice());
                }

                if flip.0 {
                    image_buff = image::imageops::flip_horizontal(&image_buff);
                }
                if flip.1 {
                    image_buff = image::imageops::flip_vertical(&image_buff);
                }
                image_buff
            }
        },
        ColorSpace::RGBA => {
            let f = from_color.parse().into_iter().map(|v| { v.scale() }).collect::<Vec<f32>>();
            let t = to_color.parse().into_iter().map(|v| { v.scale() }).collect::<Vec<f32>>();

            let angle = angle % 360.0;

            let flip = {
                if 0.0 < angle && angle <= 90.0 { (false, false) }
                else if 90.0 < angle && angle <= 180.0 { (true, false) }
                else if 180.0 < angle && angle <= 270.0 { (true, true) }
                else { (false, true) }
            };

            let angle = angle % 90.0;
            
            if angle == 0.0 {
                let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, (f[3]-t[3]) / width as f32];
            
                let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                    f[i % 4] - steps[i % 4]*((i / 4) as f32).floor()
                }).collect::<Vec<f32>>();

                let mut img: Vec<f32> = Vec::new();
                for _ in 0..height { img.append(&mut row.clone()) };
                let mut image_buff = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width, height, img.into_iter().map(|v| {v.scale()}).collect::<Vec<u8>>()).unwrap();

                if flip.0 { image_buff = image::imageops::flip_horizontal(&image_buff); }
                image_buff
            
            } else {
                let mut image_buff = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
                let r = [-angle.to_radians().tan(), 1.0];

                let a = -1.0/r[0];
                let b = 1f32;
                let c = -a*width as f32 - height as f32;

                let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();
                let steps = [(f[0]-t[0]), (f[1]-t[1]), (f[2]-t[2]), (f[3]-t[3])];

                for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
                    let a = -1.0/r[0];
                    let b = 1f32;
                    let c = -a*x as f32 - height as f32 + y as f32;

                    let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

                    let channels = [
                        (f[0] - steps[0] * s/w).scale(), 
                        (f[1] - steps[1] * s/w).scale(), 
                        (f[2] - steps[2] * s/w).scale(), 
                        (f[3] - steps[3] * s/w).scale()
                    ];
                    
                    *pixel = *Rgba::from_slice(channels.as_slice());
                }

                if flip.0 {
                    image_buff = image::imageops::flip_horizontal(&image_buff);
                }
                if flip.1 {
                    image_buff = image::imageops::flip_vertical(&image_buff);
                }
                image_buff
            }
        },
        ColorSpace::OKLAB => {
            let a = Oklab::from(*Rgb::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Oklab::from(*Rgb::from_slice(&to_color.parse()));
            let t = b.channels();

            let angle = angle % 360.0;

            let flip = {
                if 0.0 < angle && angle <= 90.0 { (false, false) }
                else if 90.0 < angle && angle <= 180.0 { (true, false) }
                else if 180.0 < angle && angle <= 270.0 { (true, true) }
                else { (false, true) }
            };

            let angle = angle % 90.0;
            
            if angle == 0.0 {
                let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, 0.0];

                let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                    if i%4 == 3 { 1f32 } else {f[i % 4] - steps[i % 4]*((i / 4) as f32).floor() } 
                }).collect::<Vec<f32>>();

                let mut img: Vec<f32> = Vec::new();
                for _ in 0..height { img.append(&mut row.clone()) };
                let mut image_buff = OklabaImage::from_vec(width, height, img).unwrap();

                if flip.0 { image_buff = image::imageops::flip_horizontal(&image_buff); }
                unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
            } else {
                let mut image_buff = OklabaImage::new(width, height);
                let r = [-angle.to_radians().tan(), 1.0];

                let a = -1.0/r[0];
                let b = 1f32;
                let c = -a*width as f32 - height as f32;

                let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();
                let steps = [(f[0]-t[0]), (f[1]-t[1]), (f[2]-t[2]), 0.0];

                for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
                    let a = -1.0/r[0];
                    let b = 1f32;
                    let c = -a*x as f32 - height as f32 + y as f32;

                    let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

                    let channels = [
                        f[0] - steps[0] * s/w, 
                        f[1] - steps[1] * s/w, 
                        f[2] - steps[2] * s/w,
                        1.0
                    ];
                    
                    *pixel = *Oklaba::from_slice(channels.as_slice());
                }

                if flip.0 {
                    image_buff = image::imageops::flip_horizontal(&image_buff);
                }
                if flip.1 {
                    image_buff = image::imageops::flip_vertical(&image_buff);
                }
                unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
            }
        },
        ColorSpace::OKLABA => {
            let a = Oklab::from(*Rgb::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Oklab::from(*Rgb::from_slice(&to_color.parse()));
            let t = b.channels();

            let angle = angle % 360.0;

            let flip = {
                if 0.0 < angle && angle <= 90.0 { (false, false) }
                else if 90.0 < angle && angle <= 180.0 { (true, false) }
                else if 180.0 < angle && angle <= 270.0 { (true, true) }
                else { (false, true) }
            };

            let angle = angle % 90.0;
            
            if angle == 0.0 {
                let steps = [(f[0]-t[0]) / width as f32, (f[1]-t[1]) / width as f32, (f[2]-t[2]) / width as f32, (f[3]-t[3]) / width as f32];
            
                let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                    f[i % 4] - steps[i % 4]*((i / 4) as f32).floor()
                }).collect::<Vec<f32>>();

                let mut img: Vec<f32> = Vec::new();
                for _ in 0..height { img.append(&mut row.clone()) };
                let mut image_buff = OklabaImage::from_vec(width, height, img).unwrap();

                if flip.0 { image_buff = image::imageops::flip_horizontal(&image_buff); }
                unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
            
            } else {
                let mut image_buff = OklabaImage::new(width, height);
                let r = [-angle.to_radians().tan(), 1.0];

                let a = -1.0/r[0];
                let b = 1f32;
                let c = -a*width as f32 - height as f32;

                let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();
                let steps = [(f[0]-t[0]), (f[1]-t[1]), (f[2]-t[2]), (f[3]-t[3])];

                for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
                    let a = -1.0/r[0];
                    let b = 1f32;
                    let c = -a*x as f32 - height as f32 + y as f32;

                    let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

                    let channels = [
                        f[0] - steps[0] * s/w, 
                        f[1] - steps[1] * s/w, 
                        f[2] - steps[2] * s/w, 
                        f[3] - steps[3] * s/w
                    ];
                    
                    *pixel = *Oklaba::from_slice(channels.as_slice());
                }

                if flip.0 {
                    image_buff = image::imageops::flip_horizontal(&image_buff);
                }
                if flip.1 {
                    image_buff = image::imageops::flip_vertical(&image_buff);
                }
                unsafe { convert_oklaba_to_rgba(&image_buff).unwrap() }
            }
        },
        ColorSpace::HSL => {
            let a = Hsl::from(*Rgb::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Hsl::from(*Rgb::from_slice(&to_color.parse()));
            let t = b.channels();

            let angle = angle % 360.0;

            let flip = {
                if 0.0 < angle && angle <= 90.0 { (false, false) }
                else if 90.0 < angle && angle <= 180.0 { (true, false) }
                else if 180.0 < angle && angle <= 270.0 { (true, true) }
                else { (false, true) }
            };

            let angle = angle % 90.0;
            
            if angle == 0.0 {
                let steps = [
                    {
                        if (f[0]-t[0]).abs() < (f[0] - t[0] + 360.0).abs() {
                            f[0] - t[0]
                        } else {
                            f[0] - t[0] + 360.0
                        }
                    } / width as f32, 
                    (f[1]-t[1]) / width as f32, 
                    (f[2]-t[2]) / width as f32, 0.0];

                let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                    if i%4 == 3 { 1f32 } else { f[i % 4] - steps[i % 4]*((i / 4) as f32).floor() % 360.0 } 
                }).collect::<Vec<f32>>();

                let mut img: Vec<f32> = Vec::new();
                for _ in 0..height { img.append(&mut row.clone()) };
                let mut image_buff = HslaImage::from_vec(width, height, img).unwrap();

                if flip.0 { image_buff = image::imageops::flip_horizontal(&image_buff); }
                unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
            
            } else {
                let mut image_buff = HslaImage::new(width, height);
                let r = [-angle.to_radians().tan(), 1.0];

                let a = -1.0/r[0];
                let b = 1f32;
                let c = -a*width as f32 - height as f32;

                let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();
                let steps = [
                    {
                        if (f[0]-t[0]).abs() < (f[0] - t[0] + 360.0).abs() {
                            f[0] - t[0]
                        } else {
                            println!("going backwords is faster, {}", f[0] - t[0] + 360.0);
                            f[0] - t[0] + 360.0
                        }
                    }, 
                    f[1]-t[1], 
                    f[2]-t[2], 
                    0.0];

                for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
                    let a = -1.0/r[0];
                    let b = 1f32;
                    let c = -a*x as f32 - height as f32 + y as f32;

                    let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

                    let channels = [
                        (f[0] - steps[0] * s/w + 360.0) % 360.0, 
                        f[1] - steps[1] * s/w, 
                        f[2] - steps[2] * s/w, 
                        1.0
                    ];
                    
                    *pixel = *Hsla::from_slice(channels.as_slice());
                }

                if flip.0 {
                    image_buff = image::imageops::flip_horizontal(&image_buff);
                }
                if flip.1 {
                    image_buff = image::imageops::flip_vertical(&image_buff);
                }
                unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
            }
        },
        ColorSpace::HSLA => {
            let a = Hsla::from(*Rgba::from_slice(&from_color.parse()));
            let f = a.channels();
            let b = Hsla::from(*Rgba::from_slice(&to_color.parse()));
            let t = b.channels();

            let angle = angle % 360.0;

            let flip = {
                if 0.0 < angle && angle <= 90.0 { (false, false) }
                else if 90.0 < angle && angle <= 180.0 { (true, false) }
                else if 180.0 < angle && angle <= 270.0 { (true, true) }
                else { (false, true) }
            };

            let angle = angle % 90.0;
            
            if angle == 0.0 {
                let steps = [
                    {
                        if (f[0]-t[0]).abs() < (f[0] - t[0] + 360.0).abs() {
                            f[0] - t[0]
                        } else {
                            f[0] - t[0] + 360.0
                        }
                    } / width as f32, 
                    (f[1]-t[1]) / width as f32, 
                    (f[2]-t[2]) / width as f32, 
                    (f[3] - t[3]) / width as f32];

                let row = vec![0f32; (width as usize)*4].into_iter().enumerate().map(|(i, _)| {
                    f[i % 4] - steps[i % 4]*((i / 4) as f32).floor() % 360.0
                }).collect::<Vec<f32>>();

                let mut img: Vec<f32> = Vec::new();
                for _ in 0..height { img.append(&mut row.clone()) };
                let mut image_buff = HslaImage::from_vec(width, height, img).unwrap();

                if flip.0 { image_buff = image::imageops::flip_horizontal(&image_buff); }
                unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
            
            } else {
                let mut image_buff = HslaImage::new(width, height);
                let r = [-angle.to_radians().tan(), 1.0];

                let a = -1.0/r[0];
                let b = 1f32;
                let c = -a*width as f32 - height as f32;

                let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();
                let steps = [
                    {
                        if (f[0]-t[0]).abs() < (f[0] - t[0] + 360.0).abs() {
                            f[0] - t[0]
                        } else {
                            println!("going backwords is faster, {}", f[0] - t[0] + 360.0);
                            f[0] - t[0] + 360.0
                        }
                    }, 
                    f[1]-t[1], 
                    f[2]-t[2], 
                    f[3]-t[3]];

                for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
                    let a = -1.0/r[0];
                    let b = 1f32;
                    let c = -a*x as f32 - height as f32 + y as f32;

                    let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

                    let channels = [
                        (f[0] - steps[0] * s/w + 360.0) % 360.0, 
                        f[1] - steps[1] * s/w, 
                        f[2] - steps[2] * s/w, 
                        f[3] - steps[3] * s/w
                    ];
                    
                    *pixel = *Hsla::from_slice(channels.as_slice());
                }

                if flip.0 {
                    image_buff = image::imageops::flip_horizontal(&image_buff);
                }
                if flip.1 {
                    image_buff = image::imageops::flip_vertical(&image_buff);
                }
                unsafe { convert_hsla_to_rgba(&image_buff).unwrap() }
            }
        }
    }
}