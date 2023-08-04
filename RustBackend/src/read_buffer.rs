use std::vec::Vec;
use palette::rgb::Rgb;
use pyo3::prelude::*;
use pyo3::types::PyList;

pub struct Pixel {
    color: Rgb,
    x: usize,
    y: usize,
}

#[pyfunction]
pub fn read(buf: Vec<u8>) -> &'static PyList {
    todo!()
}

fn write(buf: Vec<Pixel>) -> PyList {
    todo!()
}