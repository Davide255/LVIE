use palette::convert::FromColorUnclamped;
use palette::white_point::D65;
use palette::FromColor;
use palette::{Hsl, Hsv, IntoColor, Oklab, Oklch, Srgb, Xyz};
use std::vec::Vec;

pub struct Buffer<T = Srgb<f64>> {
    pub buffer: Vec<T>,
}

impl<T> Buffer<T>
where
    T: FromColorUnclamped<T> + IntoColor<T> + Copy,
{
    pub fn convert_to<TO>(&self) -> Buffer<TO>
    where
        T: FromColorUnclamped<T> + IntoColor<TO> + Copy,
        TO: FromColorUnclamped<TO> + FromColor<T> + FromColor<TO> + Copy,
    {
        let mut out_buffer: Vec<TO> = Vec::new();
        for c in &self.buffer {
            let color: TO = (*c).into_color();
            out_buffer.push(color);
        }
        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Srgb<f64>> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (r, g, b) = pixel.into_components();
            out_buffer.push(vec![r * 255.0, g * 255.0, b * 255.0])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Srgb<f64>> {
        let mut out_buffer: Vec<Srgb<f64>> = Vec::new();
        for i in buffer {
            out_buffer.push(Srgb::<f64>::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Hsv<palette::rgb::Rgb, f64>> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (h, s, v) = pixel.into_components();
            out_buffer.push(vec![h.into_degrees() as f64, s as f64, v as f64])
        }
        out_buffer
    }
    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Hsv<Srgb, f64>> {
        let mut out_buffer: Vec<Hsv<Srgb, f64>> = Vec::new();
        for i in buffer {
            out_buffer.push(Hsv::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Hsl<palette::rgb::Rgb, f64>> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (h, s, l) = pixel.into_components();
            out_buffer.push(vec![h.into_degrees(), s, l])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Hsl<palette::rgb::Rgb, f64>> {
        let mut out_buffer: Vec<Hsl<palette::rgb::Rgb, f64>> = Vec::new();
        for i in buffer {
            out_buffer.push(Hsl::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Oklab<f64>> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (l, a, b) = pixel.into_components();
            out_buffer.push(vec![l, a, b])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Oklab<f64>> {
        let mut out_buffer: Vec<Oklab<f64>> = Vec::new();
        for i in buffer {
            out_buffer.push(Oklab::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Oklch<f64>> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (l, c, h) = pixel.into_components();
            out_buffer.push(vec![l, c, h.into_degrees()])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Oklch<f64>> {
        let mut out_buffer: Vec<Oklch<f64>> = Vec::new();
        for i in buffer {
            out_buffer.push(Oklch::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Xyz<D65, f64>> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (x, y, z) = pixel.into_components();
            out_buffer.push(vec![x, y, z])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Xyz<D65, f64>> {
        let mut out_buffer: Vec<Xyz<D65, f64>> = Vec::new();
        for i in buffer {
            out_buffer.push(Xyz::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}
