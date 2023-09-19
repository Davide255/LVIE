#![allow(non_snake_case)]

pub mod matrix;

pub type Matrix<T> = matrix::Matrix<T>;
pub type Complex<T> = rustfft::num_complex::Complex<T>;

use rustfft::FftDirection;

pub type FFTDirection = FftDirection;

pub mod hsl;
mod math;
pub mod contrast;
