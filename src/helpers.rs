pub(crate) use std::ops::RangeInclusive;

pub enum CollectDataType {
    Red,
    Green,
    Blue,
    Luminance,
}

pub enum BufferFormat {
    Rgb,
    Hsl,
    Hsv,
    Oklab,
    Oklch,
}

pub enum ColorLuminanceType {
    HighLight,
    Light,
    Midtone,
    Shadow,
}

/* Format a buffer of value in a buffer of pixel with this rule:
Vec<f64>: [r, g, b, r, g, b, ...] -> Vec<Vec<f64>>: [[r, g, b],[r, g, b],...]*/
pub fn normalize_buffer(buffer: &Vec<i32>) -> Vec<Vec<i32>> {
    let mut out_buffer: Vec<Vec<i32>> = Vec::new();

    for x in 0..(buffer.len() / 4) {
        out_buffer.append(&mut vec![buffer[x * 4..(x * 4 + 3)].to_vec()]);
    }

    return out_buffer;
}

/* Check if a value is inside a range,
if not, it clamps the value to the range limits */
pub fn norm_range(r: RangeInclusive<f64>, value: f64) -> f64 {
    if r.start() <= &value && &value <= r.end() {
        return value;
    } else if &value < r.start() {
        return *r.start();
    } else {
        return *r.end();
    }
}

/* Buld an empty buffer all filled with zeros
- taken from python numpy.zeros() */
pub fn create_zeros_buffer(format: (i32, i32)) -> Vec<Vec<f64>> {
    let mut out_v: Vec<Vec<f64>> = Vec::new();

    for x in 0..format.0 {
        let mut add_v = Vec::new();
        for y in 0..format.1 {
            add_v.push(0.0);
        }
        out_v.push(add_v);
    }

    return out_v;
}

/* Reshape a vector to a different format
- taken from python numpy.shape() */
pub fn vec_reshape(v: Vec<f64>, shape: (i32, i32)) -> Vec<Vec<f64>> {
    let mut out_v: Vec<Vec<f64>> = Vec::new();

    if shape.0 * shape.1 != v.len() as i32 {
        panic!("Not enough or too much elemnts for this format");
    }

    for x in 0..shape.0 {
        let s: &[f64] = &v[(x * shape.1) as usize..(x * shape.1 + shape.1) as usize];
        out_v.push(s.to_vec());
    }

    return out_v;
}
