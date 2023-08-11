use palette::{hsl::Hsl, luma::SrgbLuma, IntoColor, Saturate, Srgb};
use palette::{hsv::Hsv, FromColor};
use pyo3::prelude::*;
use std::f64::consts;
use std::vec::Vec;

enum ColorLuminanceType {
    HighLight,
    Light,
    Midtone,
    Shadow,
}

fn normalize_buffer(buffer: Vec<i32>) -> Vec<Vec<i32>> {
    let mut out_buffer: Vec<Vec<i32>> = Vec::new();

    for x in 0..(buffer.len() / 3) {
        out_buffer.append(&mut vec![buffer[x..(x + 3)].to_vec()]);
    }

    return out_buffer;
}

#[pyfunction]
fn adjust_saturation(buffer: Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
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

#[pyfunction]
fn adjust_exposure(buffer: Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
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
                ((component as f64) * 2_f64.powf(_added_value.into())).into();
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

#[pyfunction]
fn convert_to_grayscale(buffer: Vec<Vec<f64>>) -> Vec<f64> {
    let mut out_buffer: Vec<f64> = Vec::new();

    for i in buffer {
        let _c: (f64, f64, f64) = (i[0] / 255.0, i[1] / 255.0, i[2] / 255.0);
        let _l: Hsl = Hsl::from_color(Srgb::from_components(_c));
        out_buffer.push(_l.luma * 255.0);
    }

    return out_buffer;
}

fn adjust_contrast(buffer: Vec<Vec<f64>>, added_value: f32) {}

fn l_o_g_mask(x: i32, y: i32, sigma: f64) -> f64 {
    let x = x as f64;
    let y = y as f64;

    let nom: f64 = (y.powf(2.0) + x.powf(2.0) - 2_f64 * sigma.powf(2.0)).into();
    let denom: f64 = 2_f64 * consts::PI * (sigma as f64).powf(6.0);
    let expo: f64 = (-(x.powf(2.0) + y.powf(2.0)) / (2_f64 * sigma.powf(2.0))).into();

    return nom * expo.exp() / denom;
}

fn vec_reshape(v: Vec<f64>, shape: (i32, i32)) -> Vec<Vec<f64>> {
    let mut out_v: Vec<Vec<f64>> = Vec::new();

    if shape.0 * shape.1 != v.len() as i32 {
        panic!("Not enough or too much elemnts for this format");
    }

    for x in 0..shape.0 {
        let s: &[f64] = &v[(x * shape.1) as usize..(x * shape.1 + shape.1) as usize];
        out_v.push(s.to_vec());
    }

    return out_v;
}

fn create_log(sigma: f64, size: i32) -> Vec<Vec<f64>> {
    let w: i32 = ((size as f64) * sigma).ceil() as i32;

    if w % 2 == 0 {
        let w: i32 = w + 1;
    }

    let mut log_mask: Vec<f64> = Vec::new();

    let w_range: i32 = (w as f64 / 2.0).floor() as i32;

    for i in -w_range..w_range {
        for j in -w_range..w_range {
            log_mask.push(l_o_g_mask(i, j, sigma));
        }
    }

    return vec_reshape(log_mask, (w, w));
}

fn create_zeros_buffer(format: (i32, i32)) -> Vec<Vec<f64>> {
    let mut out_v: Vec<Vec<f64>> = Vec::new();

    for x in 0..format.0 {
        let mut add_v = Vec::new();
        for y in 0..format.1 {
            add_v.push(0.0);
        }
        out_v.push(add_v);
    }

    return out_v;
}

fn convolve(buffer: Vec<f64>, image_size: (i32, i32), mask: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let width: i32 = image_size.0;
    let height: i32 = image_size.1;

    let w_range: i32 = (mask.len() as f64 / 2.0).floor() as i32;

    let mut res_image: Vec<Vec<f64>> = create_zeros_buffer((height, width));

    for i in w_range..width - w_range {
        for j in w_range..height - w_range {
            for k in -w_range..=w_range {
                for h in -w_range..=w_range {
                    res_image[j as usize][i as usize] += mask[(w_range + h) as usize]
                        [(w_range + k) as usize]
                        * (buffer[(((j + h) * width) + (i + k)) as usize] as f64)
                }
            }
        }
    }

    return res_image;
}

fn z_c_test(l_o_g_image: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut zc_image: Vec<Vec<f64>> =
        create_zeros_buffer((l_o_g_image.len() as i32, l_o_g_image[0].len() as i32));

    for i in 1..(l_o_g_image.len() as i32) - 1 {
        for j in 1..(l_o_g_image[0].len() as i32) - 1 {
            let mut neg_count: i32 = 0;
            let mut pos_count: i32 = 0;

            for a in -1..=1 {
                for b in -1..=1 {
                    if a != 0 && b != 0 {
                        if l_o_g_image[(i + a) as usize][(j + b) as usize] < 0.0 {
                            neg_count += 1;
                        } else if l_o_g_image[(i + a) as usize][(j + b) as usize] > 0.0 {
                            pos_count += 1;
                        }
                    }
                }
            }

            if neg_count > 0 && pos_count > 0 {
                zc_image[i as usize][j as usize] = 1.0;
            }
        }
    }

    return zc_image;
}

fn find_edges_mask(buffer: Vec<f64>, image_size: (i32, i32), sigma: f64, size: i32) -> Vec<f64> {
    let log_mask: Vec<Vec<f64>> = create_log(sigma, size);

    println!("Smoothing");

    let log_image: Vec<Vec<f64>> = convolve(buffer, image_size, log_mask);

    let zc_image: Vec<Vec<f64>> = z_c_test(log_image);

    let mut out_v: Vec<f64> = Vec::new();

    for i in zc_image {
        for k in i {
            out_v.push(k);
        }
    }

    out_v
}

#[pymodule]
fn rustlib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(adjust_saturation, m)?)?;
    m.add_function(wrap_pyfunction!(adjust_exposure, m)?)?;
    m.add_function(wrap_pyfunction!(convert_to_grayscale, m)?)?;
    Ok(())
}


#[pyfunction]
pub fn shift_hue(buf: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let mut out: Vec<Vec<f32>> = Vec::new();
    for p in buf {
        let (r, g, b) = (p[0], p[1], p[2]);
        let mut pix: Oklch = Oklch::from_color(Srgb::<f32>::from_components((
            r / 255.0,
            g / 255.0,
            b / 255.0,
        )));
        let (l, c, h) = pix.into_components();
        pix = Oklch::new(l, c, OklabHue::from_degrees(h.into_degrees() + 180.0));
        let pix_rgb: Rgb = Rgb::from_color(pix);
        let (r, g, b) = pix_rgb.into_components();
        out.push(vec![r * 255.0, g * 255.0, b * 255.0]);
    }
    out
}

#[pyfunction]
fn circle(buf: Vec<Vec<f32>>, width: usize, height: usize, radius: f32, c_x: f32, c_y: f32) -> Vec<Vec<f32>> {
    if buf.len() != width*height {panic!("The buffer has not the expected length")};
    let mut out: Vec<Vec<f32>> = Vec::new();
    let mut image: HashMap<(usize, usize), Srgb> = HashMap::new();

    let mut y: usize = 0;
    let mut pos: usize = 0;
    for pix in buf {
        let (r, g, b) = (pix[0] as f32, pix[1] as f32, pix[2] as f32);
        pos += 1;
        if pos % width == 1 {y += 1};

        if pos % width == 0 {
            image.insert((width, y), Srgb::from_components((r, g, b)));
        } else {
            image.insert((pos % width, y), Srgb::from_components((r, g, b)));
        }
    }

    for ((x, y), mut color) in image {
        if (((x as f32 - c_x)*(x as f32 - c_x)) as f32) < radius && (((y as f32 - c_y)*(y as f32 - c_y)) as f32) < radius {
            let (l, c, h) = Oklch::from_color(color).into_components();
            color = Srgb::from_color(Oklch::from_components((
                l,
                c,
                OklabHue::from_degrees(h.into_degrees() + 180.0),
            )));
        }

        let (r, g, b) = color.into_components();
        out.push(vec![r * 255.0, g*255.0, b*255.0]);
    }

    out 
}
