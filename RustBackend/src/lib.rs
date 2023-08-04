use palette::{oklch::Oklch, rgb::Rgb};
use palette::{FromColor, Srgb};
use pyo3::prelude::*;

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
    m.add_function(wrap_pyfunction!(shift_chroma, m)?)?;
    Ok(())
}

#[pyfunction]
fn shift_chroma(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let mut pix: Oklch = Oklch::from_color(Srgb::<f32>::from_components((
        (r as f32) / 255.0,
        (g as f32) / 255.0,
        (b as f32) / 255.0,
    )));
    let (l, c, h) = pix.into_components();
    pix = Oklch::new(l, c + 50.0, h);
    let pix_rgb: Rgb = Rgb::from_color(pix);
    let (r, g, b) = pix_rgb.into_components();
    return ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
}
