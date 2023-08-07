use palette::{oklch::Oklch, rgb::Rgb, OklabHue};
use palette::{FromColor, Srgb};
use pyo3::prelude::*;
use std::collections::HashMap;

mod read_buffer;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn suba(a: usize, b: usize) -> PyResult<usize> {
    Ok(a - b)
}

/// A Python module implemented in Rust.
#[pymodule]
fn rust_backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(suba, m)?)?;
    m.add_function(wrap_pyfunction!(shift_hue, m)?)?;
    m.add_function(wrap_pyfunction!(circle, m)?)?;
    m.add_function(wrap_pyfunction!(read_write, m)?)?;
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
fn read_write(buf: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let mut out: Vec<Vec<f32>> = Vec::new();
    for p in buf {
        out.push(p);
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
