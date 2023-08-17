use palette::convert::FromColorUnclamped;
use palette::white_point::D65;
use palette::FromColor;
use palette::IntoColor;
use std::vec::Vec;

#[allow(dead_code)]
type Hsl = palette::hsl::Hsl<palette::rgb::Rgb, f64>;
#[allow(dead_code)]
type Hsv = palette::hsv::Hsv<palette::rgb::Rgb, f64>;
#[allow(dead_code)]
type Oklab = palette::oklab::Oklab<f64>;
#[allow(dead_code)]
type Oklch = palette::oklch::Oklch<f64>;
#[allow(dead_code)]
type Rgb = palette::rgb::Rgb<palette::encoding::Srgb, f64>;
#[allow(dead_code)]
type Xyz = palette::xyz::Xyz<D65, f64>;

#[derive(Clone)]
pub struct Buffer<T = Rgb> {
    pub buffer: Vec<T>,
}

impl<T> Buffer<T>
where
    T: FromColorUnclamped<T> + IntoColor<T> + Copy + Clone,
{
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn iter(&self) -> <Buffer<T> as IntoIterator>::IntoIter {
        self.buffer.clone().into_iter()
    }
}

impl Buffer<Rgb> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (r, g, b) = pixel.into_components();
            out_buffer.push(vec![r * 255.0, g * 255.0, b * 255.0])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Rgb> {
        let mut out_buffer: Vec<Rgb> = Vec::new();
        for i in buffer {
            out_buffer.push(Rgb::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Hsv> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (h, s, v) = pixel.into_components();
            out_buffer.push(vec![h.into_degrees() as f64, s as f64, v as f64])
        }
        out_buffer
    }
    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Hsv> {
        let mut out_buffer: Vec<Hsv> = Vec::new();
        for i in buffer {
            out_buffer.push(Hsv::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Hsl> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (h, s, l) = pixel.into_components();
            out_buffer.push(vec![h.into_degrees(), s, l])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Hsl> {
        let mut out_buffer: Vec<Hsl> = Vec::new();
        for i in buffer {
            out_buffer.push(Hsl::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Oklab> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (l, a, b) = pixel.into_components();
            out_buffer.push(vec![l, a, b])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Oklab> {
        let mut out_buffer: Vec<Oklab> = Vec::new();
        for i in buffer {
            out_buffer.push(Oklab::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Oklch> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (l, c, h) = pixel.into_components();
            out_buffer.push(vec![l, c, h.into_degrees()])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Oklch> {
        let mut out_buffer: Vec<Oklch> = Vec::new();
        for i in buffer {
            out_buffer.push(Oklch::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl Buffer<Xyz> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self.buffer {
            let (x, y, z) = pixel.into_components();
            out_buffer.push(vec![x, y, z])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Xyz> {
        let mut out_buffer: Vec<Xyz> = Vec::new();
        for i in buffer {
            out_buffer.push(Xyz::from_components((i[0], i[1], i[2])));
        }

        Buffer { buffer: out_buffer }
    }
}

impl<T> IntoIterator for Buffer<T>
where
    T: FromColorUnclamped<T> + IntoColor<T> + Copy,
{
    type Item = T;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}
