pub(crate) use palette::FromColor;
use palette::{Hsl, IntoColor, Saturate, Srgb};
use std::collections::HashMap;
use std::vec::Vec;


use crate::buffer_struct::Buffer;
use crate::helpers::{norm_range, norm_range_f32, CollectDataType};

//impl<X, U> From<Buffer<X>> for Buffer<U>
//where
//    X: FromColorUnclamped<X> + IntoColor<X> + Copy,
//    U: FromColorUnclamped<U> + IntoColor<U> + Copy,
//{
//    fn from(buffer: Buffer<X>) -> Self {
//        let mut out_buffer: Vec<U> = Vec::new();
//        for pixel in buffer.buffer {
//            let color: U = pixel.into_color();
//            out_buffer.push(color);
//        }
//        Buffer { buffer: out_buffer }
//    }
//}

#[allow(dead_code)]
pub fn collect_data(buffer: &Buffer<Srgb>, data_type: CollectDataType) -> HashMap<i32, i32> {
    let mut outhash: HashMap<i32, i32> = HashMap::new();
    
    for y in buffer.iter(){
        for pixel in y {
            let v: i32;

            match data_type {
                CollectDataType::Red => {
                    v = pixel.red as i32;
                }
                CollectDataType::Green => {
                    v = pixel.green as i32;
                }
                CollectDataType::Blue => {
                    v = pixel.blue as i32;
                }
                CollectDataType::Luminance => {
                    v = (Hsl::from_color(pixel).lightness * 255.0) as i32;
                }
            }

            if outhash.get(&v) == None {
                outhash.insert(v, 1);
            } else {
                let counter: i32 = *outhash.get(&v).unwrap() + 1;
                outhash.remove(&v);
                outhash.insert(v, counter);
            }
        }
    }

    outhash
}

pub fn box_blur(buffer: &Buffer, kernel_size: u32) -> Buffer {
    let p = kernel_size / 2;
    let new_buffer = buffer.add_padding((p, p, p, p));

    let (width, height) = buffer.get_image_size();

    let mut c_val_map: HashMap<u32, CollectDataType> = HashMap::new();
    c_val_map.insert(0, CollectDataType::Red);
    c_val_map.insert(1, CollectDataType::Green);
    c_val_map.insert(2, CollectDataType::Blue);

    let mut out_buffer: Buffer = Buffer::new(buffer.get_image_size());

    for y in 0..height {
        for x in 0..width {
            print!("\rpixel: ({}, {}) ", x, y);
            let mut c_buf: Vec<f32> = Vec::new();
            for c in 0..3 {
                let mut _conv_out: f32 = 0.0;
                let buf_matrix: Vec<Vec<f32>> = new_buffer.get_area(
                    (x as u32, y as u32), 
                    (kernel_size, kernel_size)
                )
                .get_all(c_val_map[&c]);

                for _y in 0..3usize {
                    _conv_out += buf_matrix[_y].iter().sum::<f32>();
                }
                
                c_buf.push(_conv_out / (kernel_size.pow(2)) as f32);
            }

            out_buffer.add_item(
                Srgb::new(
                    norm_range_f32(0f32..=1f32, c_buf[0]), 
                    norm_range_f32(0f32..=1f32, c_buf[1]),
                    norm_range_f32(0f32..=1f32, c_buf[2])
                )
            );
        }
    }

    println!();

    out_buffer
}

pub fn adjust_saturation(buffer: &Buffer, added_value: f32) -> Buffer {
    let added_value: f32 = norm_range(-0.5..=0.5, added_value as f64) as f32;
    let mut out_buffer: Buffer<Srgb> = Buffer::new(buffer.get_image_size());

    for y in buffer.iter() {
        for x in y {
            let out_color: Srgb = Hsl::from_color(x).saturate(added_value).into_color();
            out_buffer.add_item(out_color);
        }
    }

    return out_buffer;
}

pub fn adjust_exposure(buffer: &Buffer, added_value: f32) -> Buffer {
    let added_value: f32 = norm_range(-2.0..=2.0, added_value as f64) as f32;

    let mut out_buffer: Buffer<Srgb> = Buffer::new(buffer.get_image_size());

    let _added_value: f64;

    if !(-2.0 <= added_value && added_value <= 2.0) {
        _added_value = ((added_value / added_value.abs()) * 2.0).into();
    } else {
        _added_value = added_value.into();
    }

    for y in buffer.iter() {
        for pixel in y {
            let new_pixel: Srgb = Srgb::new(
                norm_range(0.0..=255.0, ((pixel.red as f64) * 2_f64.powf(_added_value.into())).into()) as f32,
                norm_range(0.0..=255.0, ((pixel.green as f64) * 2_f64.powf(_added_value.into())).into()) as f32,
                norm_range(0.0..=255.0, ((pixel.blue as f64) * 2_f64.powf(_added_value.into())).into()) as f32,
            );

            out_buffer.add_item(new_pixel)
        }
    }

    return out_buffer;
}


pub fn convert_to_grayscale(buffer: &Buffer) -> Vec<Vec<f32>> {
    buffer.convert_to::<Hsl>().collect_luma()
}

#[allow(dead_code)]
pub fn combine_grayscale_with_colored(
    gray_scale_buffer: &Vec<Vec<f32>>,
    buffer: &Buffer,
) -> Buffer {
    buffer.combine_grayscale_with_colored(gray_scale_buffer)
}

pub fn adjust_contrast(buffer: &Buffer, added_value: f32) -> Buffer {
    let added_value: f64 = norm_range(-0.5..=0.5, added_value as f64) + 1.0;

    let mut gray_buffer: Vec<Vec<f32>> = convert_to_grayscale(&buffer);

    let gray_buffer_sum = |gray_buffer: &Vec<Vec<f32>>| -> f32 {
        let mut sum: f32 = 0f32;
        for x in gray_buffer {
            sum += x.iter().sum::<f32>();
        }
        sum
    };

    let tot_len = |gray_buffer: &Vec<Vec<f32>>| -> usize {
        let mut len: usize = 0usize;
        for x in gray_buffer { len += x.len();}
        len
    };

    let avg_pixel: f64 = ((gray_buffer_sum(&gray_buffer)) / (tot_len(&gray_buffer) as f32)) as f64;

    for i in 0..gray_buffer.len() {
        for x in 0..gray_buffer[i].len() {
            gray_buffer[i][x] = norm_range(
                0.0..=255.0,
                ((gray_buffer[i][x] as f64 - avg_pixel) * added_value) + avg_pixel,
            ) as f32; 
        }
    }

    return buffer.combine_grayscale_with_colored(&gray_buffer);
}

//pub fn crop_image(
//    buffer: &Buffer,
//    image_size: (i32, i32),
//    crop: (i32, i32, i32, i32),
//) -> Buffer {
//    let mut out_buffer: Vec<Srgb> = Vec::new();
//
//    for x in crop.1..(image_size.1 - crop.3) {
//        let mut _s = buffer[(x * image_size.1 + crop.0) as usize
//            ..(x * image_size.1 + image_size.0 - crop.2) as usize]
//            .to_vec();
//        out_buffer.push(&mut _s);
//    }
//
//    Buffer::load(out_buffer, buffer.get_image_size())
//}

