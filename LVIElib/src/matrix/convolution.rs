use std::f32::consts::PI;

use rustfft::{num_complex::Complex, FftDirection};

use super::Matrix;

pub fn split3<T: Copy>(buf: Matrix<T>) -> (Matrix<T>, Matrix<T>, Matrix<T>) {
    if !buf.check_size() || buf.content.len() % 3 != 0 {
        panic!("Wrong size")
    };
    let content = buf.content;
    let mut r_buf = Matrix::<T>::new(
        vec![content[0].clone(); content.len() / 3],
        buf.height,
        buf.width / 3,
    );
    let mut g_buf = Matrix::<T>::new(
        vec![content[0].clone(); content.len() / 3],
        buf.height,
        buf.width / 3,
    );
    let mut b_buf = Matrix::<T>::new(
        vec![content[0].clone(); content.len() / 3],
        buf.height,
        buf.width / 3,
    );
    let (mut r_vec, mut g_vec, mut b_vec): (Vec<T>, Vec<T>, Vec<T>) =
        (Vec::new(), Vec::new(), Vec::new());
    for i in 0..(content.len() / 3) {
        let (r, g, b) = (content[i * 3], content[i * 3 + 1], content[i * 3 + 2]);
        r_vec.push(r);
        g_vec.push(g);
        b_vec.push(b);
    }

    r_buf.update_content(r_vec).unwrap();
    g_buf.update_content(g_vec).unwrap();
    b_buf.update_content(b_vec).unwrap();

    (r_buf, g_buf, b_buf)
}

pub fn convolve(buf: &Matrix<f32>, kernel: &Matrix<f32>) -> Matrix<f32> {
    let mut f_buf: Matrix<Complex<f32>> = buf.clone().into();
    f_buf = f_buf.fft2d(FftDirection::Forward);

    let mut pad_kernel = kernel.clone();
    pad_kernel.pad(buf.width(), buf.height(), 0.0);
    let mut f_kernel: Matrix<Complex<f32>> = pad_kernel.into();
    f_kernel = f_kernel.fft2d(FftDirection::Forward);

    f_buf
        .update_content(
            (0..f_buf.content.len())
                .map(|x| f_buf.content[x] * f_kernel.content[x])
                .collect(),
        )
        .unwrap();
    let result = f_buf.fft2d(FftDirection::Inverse);
    let content: Vec<f32> = result.content.iter().map(|x| x.re).collect();

    Matrix::new(content, buf.height, buf.width)
}

fn convolve_u8(buf: &Matrix<u8>, kernel: &Matrix<f32>) -> Matrix<u8> {
    let mut f_buf: Matrix<Complex<f32>> = buf.clone().into();
    f_buf = f_buf.fft2d(FftDirection::Forward);

    let mut pad_kernel = kernel.clone();
    pad_kernel.pad(buf.width(), buf.height(), 0.0);
    let mut f_kernel: Matrix<Complex<f32>> = pad_kernel.into();
    f_kernel = f_kernel.fft2d(FftDirection::Forward);

    f_buf
        .update_content(
            (0..f_buf.content.len())
                .map(|x| f_buf.content[x] * f_kernel.content[x])
                .collect(),
        )
        .unwrap();
    let result = f_buf.fft2d(FftDirection::Inverse);
    let content: Vec<u8> = result.content.iter().map(|x| x.re.round() as u8).collect();

    Matrix::new(content, buf.height, buf.width)
}

pub mod standard {
    use super::{convolve_u8, split3, Matrix};

    #[allow(dead_code)]
    pub fn apply_convolution(buf: Matrix<u8>, kernel: &Matrix<f32>) -> Matrix<u8> {
        let (mut r, mut g, mut b) = split3(buf);

        (r, g, b) = (
            convolve_u8(&r, kernel),
            convolve_u8(&g, kernel),
            convolve_u8(&b, kernel),
        );

        let mut output: Vec<u8> = Vec::new();
        for i in 0..r.content.len() {
            output.push(r.content[i]);
            output.push(g.content[i]);
            output.push(b.content[i]);
        }

        Matrix::new(output, r.height, 3 * r.width)
    }
}

pub mod multithreadded {
    use super::{convolve_u8, split3, Matrix};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[allow(dead_code)]
    pub fn apply_convolution(buf: Matrix<u8>, kernel: &Matrix<f32>) -> Matrix<u8> {
        let (mut _r, mut _g, mut _b) = split3(buf);

        let r: Arc<Mutex<Matrix<u8>>> = Arc::new(Mutex::new(_r));
        let g: Arc<Mutex<Matrix<u8>>> = Arc::new(Mutex::new(_g));
        let b: Arc<Mutex<Matrix<u8>>> = Arc::new(Mutex::new(_b));

        let r_kernel = kernel.clone();
        let r_weak = Arc::clone(&r);
        let r_thread = thread::Builder::new()
            .name("red_channel".into())
            .spawn(move || {
                let mut channel = r_weak.lock().unwrap();
                *channel = convolve_u8(&channel, &r_kernel);
            });

        let g_kernel = kernel.clone();
        let g_weak = Arc::clone(&g);
        let g_thread = thread::Builder::new()
            .name("green_channel".into())
            .spawn(move || {
                let mut channel = g_weak.lock().unwrap();
                *channel = convolve_u8(&channel, &g_kernel);
            });

        let b_kernel = kernel.clone();
        let b_weak = Arc::clone(&b);
        let b_thread = thread::Builder::new()
            .name("blue_channel".into())
            .spawn(move || {
                let mut channel = b_weak.lock().unwrap();
                *channel = convolve_u8(&channel, &b_kernel);
            });

        r_thread.unwrap().join().expect("Failed to join thread");
        g_thread.unwrap().join().expect("Failed to join thread");
        b_thread.unwrap().join().expect("Failed to join thread");

        let r = r.lock().unwrap();
        let g = g.lock().unwrap();
        let b = b.lock().unwrap();

        let mut output: Vec<u8> = Vec::new();
        for i in 0..r.content.len() {
            output.push(r.content[i]);
            output.push(g.content[i]);
            output.push(b.content[i]);
        }

        Matrix::new(output, r.height, 3 * r.width)
    }
}

pub fn laplacian_of_gaussian(sigma: f32, width: usize, height: usize) -> Matrix<f32> {
    let mut content: Vec<f32> = Vec::new();
    let mut sum = 0.0;
    let zeros = -1f32 / (PI * sigma * sigma * sigma * sigma);

    for iy in 0..height {
        let y: f32 = iy as f32 - (height / 2) as f32;
        for ix in 0..width {
            let x: f32 = ix as f32 - (width / 2) as f32;

            let value = (-((width as f32 - 1f32) * (height as f32 - 1f32)) / zeros)
                * ((x * x + y * y) / (2f32 * sigma * sigma) - 1f32)
                * (-(x * x + y * y) / (2f32 * sigma * sigma)).exp()
                / (PI * sigma * sigma * sigma * sigma);

            sum += value;
            content.push(value);
        }
    }

    let size = width * height;
    content = content.iter().map(|x| x - (sum / size as f32)).collect();

    Matrix::new(content, height, width)
}
