pub(crate) use palette::FromColor;
use palette::{Hsl, IntoColor, Saturate, Srgb};
use std::collections::HashMap;
use std::vec::Vec;

use crate::buffer_struct::Buffer;
use crate::helpers::{norm_range, CollectDataType};
use crate::log_mask;

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

pub fn collect_data(buffer: &Buffer<Srgb>, data_type: CollectDataType) -> HashMap<i32, i32> {
    let mut outhash: HashMap<i32, i32> = HashMap::new();

    for pixel in buffer.iter() {
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

    outhash
}

pub fn adjust_saturation(buffer: &Buffer, added_value: f32) -> Buffer {
    let added_value: f32 = norm_range(-0.5..=0.5, added_value as f64) as f32;
    let mut out_buffer: Buffer<Srgb> = buffer.clone();

    for x in 0..buffer.len() {
        let color: Hsl = Hsl::from_color(buffer[x]);
        let out_color: Srgb = color.saturate(added_value).into_color();
        out_buffer.update(x, out_color);
    }

    return out_buffer;
}

pub fn adjust_exposure(buffer: &Buffer, added_value: f32) -> Buffer {
    let added_value: f32 = norm_range(-2.0..=2.0, added_value as f64) as f32;

    let mut out_buffer: Buffer<Srgb> = buffer.new();

    let _added_value: f64;

    if !(-2.0 <= added_value && added_value <= 2.0) {
        _added_value = ((added_value / added_value.abs()) * 2.0).into();
    } else {
        _added_value = added_value.into();
    }

    for pixel in out_buffer.iter() {
        let new_pixel: Srgb = Srgb::new(
            norm_range(0.0..=255.0, ((pixel.red as f64) * 2_f64.powf(_added_value.into())).into()) as f32,
            norm_range(0.0..=255.0, ((pixel.green as f64) * 2_f64.powf(_added_value.into())).into()) as f32,
            norm_range(0.0..=255.0, ((pixel.blue as f64) * 2_f64.powf(_added_value.into())).into()) as f32,
        );

        out_buffer.append(new_pixel)
    }

    return out_buffer;
}


pub fn convert_to_grayscale(buffer: &Buffer) -> Vec<f32> {
    buffer.convert_to::<Hsl>().collect_luma()
}

pub fn combine_grayscale_with_colored(
    gray_scale_buffer: &Vec<f32>,
    buffer: &Buffer,
) -> Buffer {
    let mut out_buffer: Buffer<Hsl> = buffer.convert_to::<Hsl>().clone();

    let mut _index: usize = 0;

    for i in gray_scale_buffer {
        let _rgb: Srgb = buffer[_index];
        let hsl_color: Hsl = Hsl::new(
            out_buffer[_index].hue,
            out_buffer[_index].saturation,
            (i/ 255.0) as f32
        );
        out_buffer.update(_index, hsl_color)
    }

    out_buffer.convert_to::<Srgb>()
}

pub fn adjust_contrast(buffer: &Buffer, added_value: f32) -> Buffer {
    let added_value: f64 = norm_range(-0.5..=0.5, added_value as f64) + 1.0;

    let mut gray_buffer: Vec<f32> = convert_to_grayscale(&buffer);

    let avg_pixel: f64 = ((gray_buffer.iter().sum::<f32>()) / (gray_buffer.len() as f32)) as f64;

    for i in 0..gray_buffer.len() {
        gray_buffer[i] = norm_range(
            0.0..=255.0,
            ((gray_buffer[i] as f64 - avg_pixel) * added_value) + avg_pixel,
        ) as f32;
    }

    return combine_grayscale_with_colored(&gray_buffer, &buffer);
}

pub fn crop_image(
    buffer: &Buffer,
    image_size: (i32, i32),
    crop: (i32, i32, i32, i32),
) -> Buffer {
    let mut out_buffer: Vec<Srgb> = Vec::new();

    for x in crop.1..(image_size.1 - crop.3) {
        let mut _s = buffer[(x * image_size.1 + crop.0) as usize
            ..(x * image_size.1 + image_size.0 - crop.2) as usize]
            .to_vec();
        out_buffer.append(&mut _s);
    }

    Buffer::load(out_buffer)
}