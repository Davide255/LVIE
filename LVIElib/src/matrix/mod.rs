use std::{
    fmt::Display,
    ops::{Add, Mul, Sub},
};

use rustfft::num_complex::Complex;

pub mod convolution;
pub mod fft2;

#[derive(Debug)]
pub enum MatrixError {
    IncompatibleShapes((usize, usize), (usize, usize)),
    DifferentContentSize(usize, usize),
}

/// Matrix oriented this way:
/// M = [1 2 3
///      4 5 6]
#[derive(Clone)]
pub struct Matrix<T> {
    height: usize,
    width: usize,
    content: Vec<T>,
}

impl<T: Clone> Matrix<T> {
    pub fn new(content: Vec<T>, height: usize, width: usize) -> Self {
        Matrix {
            height,
            width,
            content,
        }
    }

    pub fn update_content(self: &mut Self, content: Vec<T>) -> Result<(), MatrixError> {
        if self.width*self.height == content.len() {
            self.content = content;
            Ok(())
        } else {
            Err(MatrixError::DifferentContentSize(
                self.width*self.height,
                content.len(),
            ))
        }
    }

    pub fn get_content(self: &Self) -> &Vec<T> {
        &self.content
    }

    pub fn height(self: &Self) -> usize {self.height}
    pub fn width(self: &Self) -> usize {self.width}

    pub fn check_size(self: &Self) -> bool {
        self.width * self.height == self.content.len()
    }

    pub fn pad(self: &mut Self, width: usize, height: usize, element: T) {
        if width < self.width || height < self.height {
            panic!("Matrix is too large");
        }
        let mut content: Vec<T> = vec![element.clone(); width*height];
        for x in 0..width {
            for y in 0..height {
                if x < self.width && y < self.height {
                    content[width*y+x] = self.content[self.width*y+x].clone()
                }
            }
        }

        (self.width, self.height) = (width, height);
        self.update_content(content).unwrap();
    }
}

impl From<Matrix<f32>> for Matrix<Complex<f32>> {
    fn from(value: Matrix<f32>) -> Self {
        let content: Vec<Complex<f32>> = value.content.iter().map(|x| x.into()).collect();
        Matrix::<Complex<f32>>::new(content, value.height, value.width)
    }
}

impl From<Matrix<u8>> for Matrix<Complex<f32>> {
    fn from(value: Matrix<u8>) -> Self {
        let content: Vec<Complex<f32>> = value
            .content
            .iter()
            .map(|x| Into::<f32>::into(*x).into())
            .collect();
        Matrix::<Complex<f32>>::new(content, value.height, value.width)
    }
}

// Todo, requires T != Q, would be cool
/* impl <T, Q: From<T>> From<Matrix<T>> for Matrix<Q> {
    fn from(value: Matrix<T>) -> Self {
        let content: Vec<Q> = value.content.iter().map(|x| x.into()).collect();
        Matrix::<Q>::new(content, value.height, value.width)
    }
} */

impl<T: Display> Display for Matrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("\n");
        for row in 0..self.height {
            for col in 0..self.width {
                output.push_str(&format!(" {}", self.content[row * self.width + col]));
            }
            output.push_str("\n");
        }
        write!(f, "{}", output)
    }
}

impl<Q: Copy, T: Add<Output = T> + Mul<Q, Output = T> + Sub<Output = T> + Copy> Mul<Matrix<Q>>
    for Matrix<T>
{
    type Output = Result<Self, MatrixError>;
    fn mul(self, rhs: Matrix<Q>) -> Self::Output {
        let (height, width, content) = (self.height, self.width, self.content);
        if width != rhs.height {
            return Err(MatrixError::IncompatibleShapes((self.height, self.width), (rhs.height, rhs.width)));
        }

        let mut result: Vec<T> = Vec::new();
        for i in 0..(height * rhs.width) {
            //println!("term {}", i);
            let row = i / rhs.width;
            let column = i % rhs.width;
            let mut val = content[0] - content[0];

            for j in 0..width {
                val = val + content[width * row + j] * rhs.content[rhs.width * j + column]
            }

            result.push(val);
        }

        Ok(Matrix::new(result, height, rhs.width))
    }
}