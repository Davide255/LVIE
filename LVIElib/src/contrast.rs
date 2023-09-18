use crate::math::cumulative_distribution;
use std::collections::HashMap;

// Works in RGB
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

pub fn histogram_equalize(buf: Vec<u8>) -> Vec<u8> {
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
        /*println!(
            "{}",
            cdf.get(&value).unwrap().clone() as f32 / size as f32
        );*/
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
