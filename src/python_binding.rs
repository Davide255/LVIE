pub(crate) use pyo3::prelude::*;

mod helpers;
mod lib;
mod log_mask;

use lib::*;

#[pyfunction]
#[pyo3(name = "adjust_saturation")]
fn py_adjust_saturation(buffer: Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
    adjust_saturation(&buffer, added_value)
}

#[pyfunction]
#[pyo3(name = "adjust_exposure")]
fn py_adjust_exposure(buffer: Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
    adjust_exposure(&buffer, added_value)
}

#[pyfunction]
#[pyo3(name = "convert_to_grayscale")]
fn py_convert_to_grayscale(buffer: Vec<Vec<f64>>) -> Vec<f64> {
    convert_to_grayscale(&buffer)
}

#[pyfunction]
#[pyo3(name = "combine_grayscale_with_colored")]
fn py_combine_grayscale_with_colored(
    gray_scale_buffer: Vec<f64>,
    buffer: Vec<Vec<f64>>,
) -> Vec<Vec<f64>> {
    combine_grayscale_with_colored(&gray_scale_buffer, &buffer)
}

#[pyfunction]
#[pyo3(name = "adjust_contrast")]
fn py_adjust_contrast(buffer: Vec<Vec<f64>>, added_value: f32) -> Vec<Vec<f64>> {
    adjust_contrast(&buffer, added_value)
}

#[pyfunction]
#[pyo3(name = "find_edges_mask")]
fn py_find_edges_mask(buffer: Vec<f64>, image_size: (i32, i32), sigma: f64, size: i32) -> Vec<f64> {
    find_edges_mask(&buffer, image_size, sigma, size)
}

#[pyfunction]
#[pyo3(name = "crop_image")]
fn py_crop_image(
    buffer: Vec<Vec<f64>>,
    image_size: (i32, i32),
    crop: (i32, i32, i32, i32),
) -> Vec<Vec<f64>> {
    crop_image(&buffer, image_size, crop)
}

#[pymodule]
fn rustlib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_adjust_saturation, m)?)?;
    m.add_function(wrap_pyfunction!(py_adjust_exposure, m)?)?;
    m.add_function(wrap_pyfunction!(py_adjust_contrast, m)?)?;
    m.add_function(wrap_pyfunction!(py_convert_to_grayscale, m)?)?;
    m.add_function(wrap_pyfunction!(py_find_edges_mask, m)?)?;
    m.add_function(wrap_pyfunction!(py_combine_grayscale_with_colored, m)?)?;
    m.add_function(wrap_pyfunction!(py_crop_image, m)?)?;
    Ok(())
}
