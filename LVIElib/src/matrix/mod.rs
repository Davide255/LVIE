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
#[derive(Clone, Debug, PartialEq)]
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

    /// Creates a ```Matrix<T>``` from a vector of rows of type ```Vec<T>```
    /// Example:
    /// ```rust
    /// use LVIElib::matrix::Matrix;
    ///
    /// let rows = vec![vec![1, 2, 3], vec![4, 5, 6]];
    /// let mat = Matrix::from_rows(rows);
    ///
    /// assert_eq!(mat, Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3));
    /// ```
    pub fn from_rows(rows: Vec<Vec<T>>) -> Self {
        let mut content = Vec::<T>::new();
        let height = rows.len();
        let width = rows[0].len();
        for row in rows {
            if row.len() != width {
                panic!("Unable to convert rows vector into a Matrix")
            } else {
                for elem in row {
                    content.push(elem);
                }
            }
        }

        Matrix::new(content, height, width)
    }

    /// Creates a square diagonal ```Matrix<T>``` from the diagonal elements
    /// Example:
    /// ```rust
    /// use LVIElib::matrix::Matrix;
    ///
    /// let mat = Matrix::from_diagonal(vec![2, 4, 6, 8], 1);
    ///
    /// assert_eq!(mat, Matrix::from_rows(vec![
    ///    vec![2, 1, 1, 1],
    ///    vec![1, 4, 1, 1],
    ///    vec![1, 1, 6, 1],
    ///    vec![1, 1, 1, 8],
    /// ]));
    /// ```
    pub fn from_diagonal(diag: Vec<T>, fill: T) -> Matrix<T> {
        let size = diag.len();
        let mut content = Vec::<T>::new();

        content.push(diag[0].clone());
        for elem in 1..size {
            content.append(&mut vec![fill.clone(); size]);
            content.push(diag[elem].clone());
        }

        Matrix::new(content, size, size)
    }

    pub fn update_content(self: &mut Self, content: Vec<T>) -> Result<(), MatrixError> {
        if self.width * self.height == content.len() {
            self.content = content;
            Ok(())
        } else {
            Err(MatrixError::DifferentContentSize(
                self.width * self.height,
                content.len(),
            ))
        }
    }

    pub fn get_content(self: &Self) -> &Vec<T> {
        &self.content
    }

    pub fn consume_content(self: Self) -> Vec<T> {
        self.content
    }

    pub fn height(self: &Self) -> usize {
        self.height
    }
    pub fn width(self: &Self) -> usize {
        self.width
    }

    pub fn check_size(self: &Self) -> bool {
        self.width * self.height == self.content.len()
    }

    pub fn pad(self: &mut Self, width: usize, height: usize, element: T) {
        if width < self.width || height < self.height {
            panic!("Matrix is too large");
        }
        let mut content: Vec<T> = vec![element.clone(); width * height];
        for x in 0..width {
            for y in 0..height {
                if x < self.width && y < self.height {
                    content[width * y + x] = self.content[self.width * y + x].clone()
                }
            }
        }

        (self.width, self.height) = (width, height);
        self.update_content(content).unwrap();
    }

    pub fn get_element(self: &Self, x: usize, y: usize) -> Result<T, MatrixError> {
        let (width, height) = (self.width(), self.height());
        if x >= width || y >= height {
            return Err(MatrixError::IncompatibleShapes(
                (x + 1, y + 1),
                (width, height),
            ));
        } else {
            return Ok(self.get_content()[y * width + x].to_owned());
        }
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

impl<T: Clone> From<Vec<T>> for Matrix<T> {
    fn from(value: Vec<T>) -> Self {
        let len = value.len();
        Matrix::new(value, len, 1)
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
            return Err(MatrixError::IncompatibleShapes(
                (self.height, self.width),
                (rhs.height, rhs.width),
            ));
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

impl<T: Mul<f32, Output = T> + Clone + Copy> Mul<Matrix<T>> for f32 {
    type Output = Matrix<T>;

    fn mul(self, rhs: Matrix<T>) -> Self::Output {
        Matrix::new(
            rhs.get_content()
                .to_owned()
                .iter()
                .map(|x| *x * self)
                .collect(),
            rhs.height(),
            rhs.width(),
        )
    }
}

impl Add<Matrix<u8>> for Matrix<u8> {
    type Output = Result<Matrix<u8>, MatrixError>;

    fn add(self, rhs: Matrix<u8>) -> Self::Output {
        let mut content: Vec<u8> = Vec::new();
        let (x, y) = (self.width(), self.height);
        if (x, y) != (rhs.width(), rhs.height()) {
            return Err(MatrixError::IncompatibleShapes(
                (x, y),
                (rhs.width(), rhs.height()),
            ));
        }

        for i in 0..self.get_content().len() {
            content.push(self.get_content()[i] + rhs.get_content()[i]);
        }

        Ok(Matrix::new(content, y, x))
    }
}

impl<O: Clone, T: Sub<T, Output = O> + Copy> Sub<Matrix<T>> for Matrix<T> {
    type Output = Result<Matrix<O>, MatrixError>;

    fn sub(self, rhs: Matrix<T>) -> Self::Output {
        let mut content: Vec<O> = Vec::new();
        let (x, y) = (self.width(), self.height);
        if (x, y) != (rhs.width(), rhs.height()) {
            return Err(MatrixError::IncompatibleShapes(
                (x, y),
                (rhs.width(), rhs.height()),
            ));
        }

        for i in 0..self.get_content().len() {
            content.push(self.get_content()[i] - rhs.get_content()[i])
        }

        Ok(Matrix::new(content, y, x))
    }
}

impl Matrix<f64> {
    pub fn round(&mut self, digits: u32) {
        let pow = 10f64.powi(digits as i32);
        self.update_content(
            self.get_content()
                .into_iter()
                .map(|x| (x * pow).round() / pow)
                .collect(),
        )
        .unwrap();
    }
}