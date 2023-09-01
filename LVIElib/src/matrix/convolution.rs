use rustfft::{num_complex::Complex, FftDirection};

use super::Matrix;

pub fn split3<T: Copy>(buf: Matrix<T>) -> (Matrix<T>, Matrix<T>, Matrix<T>) {
    if !buf.check_size() || buf.content.len() % 3 != 0 {panic!("Wrong size")};
    let content = buf.content;
    let mut r_buf = Matrix::<T>::new(vec![content[0].clone(); content.len()/3], buf.height, buf.width/3);
    let mut g_buf = Matrix::<T>::new(vec![content[0].clone(); content.len()/3], buf.height, buf.width/3);
    let mut b_buf = Matrix::<T>::new(vec![content[0].clone(); content.len()/3], buf.height, buf.width/3);
    let (mut r_vec, mut g_vec, mut b_vec): (Vec<T>, Vec<T>, Vec<T>) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..(content.len()/3) {
        let (r, g, b) = (content[i*3], content[i*3+1], content[i*3+2]);
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

    println!("Array multiplication started");
    f_buf.update_content((0..f_buf.content.len()).map(|x| f_buf.content[x]*f_kernel.content[x]).collect()).unwrap();
    println!("Matrix multiplication ended");
    let result = f_buf.fft2d(FftDirection::Inverse);
    let content: Vec<u8> = result.content.iter().map(|x| x.re.round() as u8).collect();

    println!("Convolved one channel");

    Matrix::new(content, buf.height, buf.width)
}

pub fn apply_convolution(buf: Matrix<u8>, kernel: &Matrix<f32>) -> Matrix<u8> {
    let (mut r, mut g, mut b) = split3(buf);
    println!("Buffer splitted!");

    println!("{}, {}", r.content.len(), kernel.content.len());

    (r, g, b) = (convolve(&r, kernel), convolve(&g, kernel), convolve(&b, kernel));
    println!("All channels convolved!");

    let mut output: Vec<u8> = Vec::new();
    println!("r: {}", r.content.len());
    for i in 0..r.content.len() {
        output.push(r.content[i]);
        output.push(g.content[i]);
        output.push(b.content[i]);
    };

    println!("{}", output.len());

    Matrix::new(output, r.height, 3*r.width)
}