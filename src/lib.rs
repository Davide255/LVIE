pub(crate) use palette::FromColor;
use palette::{hsl::Hsl, IntoColor, Saturate, Srgb};
use std::vec::Vec;

use crate::helpers::norm_range;
use crate::log_mask;

pub fn adjust_saturation(buffer: &Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
    let added_value: f32 = norm_range(-0.5..=0.5, added_value as f64) as f32;
    let mut out_buffer: Vec<Vec<f64>> = Vec::new();

    for x in 0..buffer.len() {
        let cvec: Vec<f64> = buffer[x].to_vec();

        let color: Hsl = Srgb::from_components((
            (cvec[0] / 255.0) as f32,
            (cvec[1] / 255.0) as f32,
            (cvec[2] / 255.0) as f32,
        ))
        .into_color();
        let out_color: Srgb = color.saturate(added_value).into_color();

        let (r, g, b) = out_color.into_components();
        out_buffer.push(vec![
            (r * 255.0) as f64,
            (g * 255.0) as f64,
            (b * 255.0) as f64,
        ]);
    }

    return out_buffer;
}

pub fn adjust_exposure(buffer: &Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
    let added_value: f32 = norm_range(-2.0..=2.0, added_value as f64) as f32;

    let mut out_buffer: Vec<Vec<f64>> = Vec::new();

    let mut _added_value: f64;

    if !(-2.0 <= added_value && added_value <= 2.0) {
        _added_value = ((added_value / added_value.abs()) * 2.0).into();
    } else {
        _added_value = added_value.into();
    }

    for pixel in buffer {
        let mut rgb_vec: Vec<f64> = Vec::new();
        for component in pixel {
            let mut new_component: f64 =
                ((*component as f64) * 2_f64.powf(_added_value.into())).into();
            if new_component < 0.0 {
                new_component = 0.0;
            } else if new_component > 255.0 {
                new_component = 255.0
            } else {
            }
            rgb_vec.push(new_component as f64);
        }
        out_buffer.push(rgb_vec);
    }

    return out_buffer;
}

pub fn convert_to_grayscale(buffer: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut out_buffer: Vec<f64> = Vec::new();

    for i in buffer {
        let (r, g, b) = (i[0] / 255.0, i[1] / 255.0, i[2] / 255.0);
        let _l: Hsl = Hsl::from_color(Srgb::from_components((r as f32, g as f32, b as f32)));
        out_buffer.push((_l.lightness as f64) * 255.0);
    }

    return out_buffer;
}

pub fn combine_grayscale_with_colored(
    gray_scale_buffer: &Vec<f64>,
    buffer: &Vec<Vec<f64>>,
) -> Vec<Vec<f64>> {
    let mut out_buffer: Vec<Vec<f64>> = Vec::new();

    let mut _index: usize = 0;

    for i in gray_scale_buffer {
        let _rgb: &Vec<f64> = &buffer[_index];
        let hsl_color: Hsl = Hsl::from_color(Srgb::from_components((
            (_rgb[0] / 255.0) as f32,
            (_rgb[1] / 255.0) as f32,
            (_rgb[2] / 255.0) as f32,
        )));

        let to_rgb: (f32, f32, f32) = Srgb::from_color(Hsl::new(
            hsl_color.hue,
            hsl_color.saturation,
            (i / 255.0) as f32,
        ))
        .into_components();

        out_buffer.push(vec![
            (to_rgb.0 * 255.0) as f64,
            (to_rgb.1 * 255.0) as f64,
            (to_rgb.2 * 255.0) as f64,
        ]);
    }

    out_buffer
}

pub fn adjust_contrast(buffer: &Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
    let added_value: f64 = norm_range(-0.5..=0.5, added_value as f64) + 1.0;

    let mut gray_buffer: Vec<f64> = convert_to_grayscale(&buffer);

    let avg_pixel: f64 = (gray_buffer.iter().sum::<f64>()) / (gray_buffer.len() as f64);

    for i in 0..gray_buffer.len() {
        gray_buffer[i] = norm_range(
            0.0..=255.0,
            ((gray_buffer[i] - avg_pixel) * added_value) + avg_pixel,
        );
    }

    return combine_grayscale_with_colored(&gray_buffer, &buffer);
}

pub fn crop_image(
    buffer: &Vec<Vec<f64>>,
    image_size: (i32, i32),
    crop: (i32, i32, i32, i32),
) -> Vec<Vec<f64>> {
    let mut out_buffer: Vec<Vec<f64>> = Vec::new();

    for x in crop.1..(image_size.1 - crop.3) {
        let mut _s = buffer[(x * image_size.1 + crop.0) as usize
            ..(x * image_size.1 + image_size.0 - crop.2) as usize]
            .to_vec();
        out_buffer.append(&mut _s);
    }

    out_buffer
}

pub fn find_edges_mask(
    buffer: &Vec<f64>,
    image_size: (i32, i32),
    sigma: f64,
    size: i32,
) -> Vec<f64> {
    let log_mask: Vec<Vec<f64>> = log_mask::create_log(sigma, size);

    println!("Smoothing");

    let log_image: Vec<Vec<f64>> = log_mask::convolve(&buffer, image_size, &log_mask);

    let zc_image: Vec<Vec<f64>> = log_mask::z_c_test(&log_image);

    let mut out_v: Vec<f64> = Vec::new();

    for i in zc_image {
        for k in i {
            out_v.push(k);
        }
    }

    out_v
}
