use crate::{
    math::cumulative_distribution,
    matrix::{convolution::split3, Matrix},
    oklab::Oklab,
};
use image::Rgb;
use std::collections::HashMap;

pub enum ContrastAlgorithm {
    Linear,
    HistogramEqualize,
}

pub fn set_contrast(img: Matrix<u8>, c: f32) -> Matrix<u8> {
    let (x, y) = (img.width(), img.height());
    let (rm, gm, bm) = split3(img);
    let (r, g, b) = (
        rm.get_content().to_owned(),
        gm.get_content().to_owned(),
        bm.get_content().to_owned(),
    );

    let (mut l_, mut a_, mut b_) = (Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new());
    for i in 0..r.len() {
        let pix = Oklab::from(Rgb([
            r[i] as f32 / 255.0,
            g[i] as f32 / 255.0,
            b[i] as f32 / 255.0,
        ]));
        l_.push(*pix.l());
        a_.push(*pix.a());
        b_.push(*pix.b());
    }

    l_ = histogram_equalize(
        l_.into_iter().map(|x| (x * 255.0).round() as u8).collect(),
        c,
    )
    .into_iter()
    .map(|x| (x as f32) / 255.0)
    .collect();

    /*a_ = histogram_equalize(
        a_.into_iter().map(|x| (x * 255.0).round() as u8).collect(),
        c,
    )
    .into_iter()
    .map(|x| (x as f32) / 255.0)
    .collect();

    b_ = histogram_equalize(
        b_.into_iter().map(|x| (x * 255.0).round() as u8).collect(),
        c,
    )
    .into_iter()
    .map(|x| (x as f32) / 255.0)
    .collect();*/

    let mut output: Vec<u8> = Vec::new();
    for i in 0..l_.len() {
        let pix = Rgb::<f32>::from(Oklab::from_components([l_[i], a_[i], b_[i]])).0;
        output.push((pix[0] * 255.0) as u8);
        output.push((pix[1] * 255.0) as u8);
        output.push((pix[2] * 255.0) as u8);
    }

    Matrix::new(output, y, x)
}

// linearly increases the contrast of a channel
pub fn adjust_contrast(buf: Vec<u8>, c: f32, avg: f32) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    for pix in buf {
        let mut p = c * (pix as f32 - avg) + avg;
        p = if p < 0.0 {
            0.0
        } else if p > 255.0 {
            255.0
        } else {
            p.round()
        };
        output.push(p as u8);
    }

    output
}

pub fn histogram_equalize(buf: Vec<u8>, c: f32) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();

    let size = buf.len();

    let mut histogram: HashMap<u8, u32> = HashMap::new();
    for i in 0u8..=255u8 {
        histogram.insert(i, 0);
    }

    for value in buf.clone() {
        *histogram
            .get_mut(&value)
            .expect("Unexpected error regarding the histogram Hashmap") += 1;
    }

    let cdf = cumulative_distribution(&histogram);
    let cdf_min = cdf.values().min().unwrap().clone();

    for value in buf {
        let mut o = (cdf.get(&value).unwrap().clone() as f32 - cdf_min as f32)
            / (size as f32 - cdf_min as f32)
            * 255.0;
        o = (1.0 - c) * value as f32 + c * o;
        o = if o < 0.0 {
            0.0
        } else if o > 255.0 {
            255.0
        } else {
            o.round()
        };
        output.push(o as u8)
    }

    output
}
