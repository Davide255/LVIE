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

pub fn convolve(buf: &Matrix<u8>, kernel: &Matrix<f32>) -> Matrix<u8> {
    let mut f_buf: Matrix<Complex<f32>> = buf.clone().into();
    f_buf = f_buf.fft2d(FftDirection::Forward);

    let mut f_kernel: Matrix<Complex<f32>> = kernel.clone().into();
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
    use super::{convolve, split3, Matrix};

    #[allow(dead_code)]
    pub fn apply_convolution(buf: Matrix<u8>, kernel: &Matrix<f32>) -> Matrix<u8> {
        let (mut r, mut g, mut b) = split3(buf);

        (r, g, b) = (
            convolve(&r, kernel),
            convolve(&g, kernel),
            convolve(&b, kernel),
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
    use super::{convolve, split3, Matrix};
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
                *channel = convolve(&channel, &r_kernel);
            });

        let g_kernel = kernel.clone();
        let g_weak = Arc::clone(&g);
        let g_thread = thread::Builder::new()
            .name("green_channel".into())
            .spawn(move || {
                let mut channel = g_weak.lock().unwrap();
                *channel = convolve(&channel, &g_kernel);
            });

        let b_kernel = kernel.clone();
        let b_weak = Arc::clone(&b);
        let b_thread = thread::Builder::new()
            .name("blue_channel".into())
            .spawn(move || {
                let mut channel = b_weak.lock().unwrap();
                *channel = convolve(&channel, &b_kernel);
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
