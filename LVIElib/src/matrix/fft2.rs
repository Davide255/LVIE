use rustfft::{num_complex::Complex, FftDirection, FftPlanner};
use transpose::transpose;

use super::{Matrix, MatrixError};

impl Matrix<Complex<f32>> {
    pub fn fft2d(self: &Self, direction: FftDirection) -> Self {
        let mut matrix = self.rows_fft(direction);

        let mut content: Vec<Complex<f32>> = vec![Complex::default(); matrix.content.len()];
        transpose(&matrix.content, &mut content, self.width, self.height);
        matrix.update_content(content).unwrap();
        (matrix.width, matrix.height) = (matrix.height(), matrix.width());
        matrix = matrix.rows_fft(direction);

        let mut content: Vec<Complex<f32>> = vec![Complex::default(); matrix.content.len()];
        transpose(&matrix.content, &mut content, self.height, self.width);
        (matrix.width, matrix.height) = (matrix.height(), matrix.width());
        content = match direction {
            FftDirection::Forward => content,
            FftDirection::Inverse => content
                .iter()
                .map(|x| x / Into::<Complex<f32>>::into((self.height * self.width) as f32))
                .collect(),
        };
        match matrix.update_content(content) {
            Err(MatrixError::DifferentContentSize(lhs, rhs)) => panic!("{} != {}", lhs, rhs),
            _ => {}
        };

        matrix
    }

    pub fn rows_fft(self: &Self, direction: FftDirection) -> Self {
        let mut content: Vec<Complex<f32>> = Vec::new();
        let mut planner = FftPlanner::new();
        let fft = match direction {
            FftDirection::Forward => planner.plan_fft_forward(self.width),
            FftDirection::Inverse => planner.plan_fft_inverse(self.width),
        };

        for r in 0..self.height {
            let mut row: Vec<Complex<f32>> =
                self.content.as_slice()[r * self.width..(r + 1) * self.width].to_vec();
            fft.process(&mut row);
            content.append(&mut row);
        }

        Matrix::new(content, self.height, self.width)
    }

    /* pub fn columns_realfft(self: &mut Self) {
        let (height, width) = (self.height, self.width);
        let mut content = vec![self.content[0]; height * width];

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(height);

        for c in 0..width {
            let mut column: Vec<Complex<f32>> = Vec::new();
            for r in 0..height {
                column.push(self.content[r * width + c].into())
            }
            fft.process(&mut column);
            println!("{:?}", column);
            for r in 0..height {
                content[r * width + c] = column[r].re.into();
            }
        }

        self.update_content(content).unwrap();
    } */
}
