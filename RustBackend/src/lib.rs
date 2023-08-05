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
    m.add_function(wrap_pyfunction!(read_buffer::read, m)?)?;
    m.add_function(wrap_pyfunction!(shift_hue, m)?)?;
    Ok(())
}

#[pyfunction]
fn shift_hue(buf: Vec<Vec<f32>>) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = Vec::new();
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
        out.push(vec![(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]);
    }
    out
}

fn circle(buf: Vec<Vec<u8>>, width: usize, height: usize) -> Vec<Vec<u8>> {
    if buf.len() != width*height {panic!("The buffer has not the expected length")};
    let mut out: Vec<Vec<u8>> = Vec::new();
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

    for ((x, y), color) in image {
        let center = (width / 2 as usize, height / 2 as usize);
    }

    out 
}
