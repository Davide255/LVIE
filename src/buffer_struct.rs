use palette::convert::FromColorUnclamped;
use palette::FromColor;
use palette::IntoColor;
use palette::{Hsl, Hsv, Oklab, Oklch, Srgb, Xyz};
use std::ops::{Index, IndexMut};
use std::vec::Vec;
use std::slice::SliceIndex;

#[derive(Clone)]
pub struct Buffer<T = Srgb> {
    _buffer: Vec<T>,
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
        for c in &self._buffer {
            let color: TO = (*c).into_color();
            out_buffer.push(color);
        }
        Buffer { _buffer: out_buffer }
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> <Buffer<T> as IntoIterator>::IntoIter {
        self._buffer.clone().into_iter()
    }

    #[allow(dead_code)]
    pub fn iter(&self)  -> <Buffer<T> as IntoIterator>::IntoIter {
        let mut sself = self.clone();
        sself.iter_mut()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self._buffer.len()
    }

    #[allow(dead_code)]
    pub fn new(&self) -> Buffer<T> {
        Buffer::<T> {
            _buffer: Vec::<T>::new(),
        }
    }
    
    #[allow(dead_code)]
    pub fn as_mut(&self) -> Self{
        self.clone()
    }

    #[allow(dead_code)]
    pub fn get_pixel(&self, index: usize) -> &T {
        self._buffer.index(index)
    }

    #[allow(dead_code)]
    pub fn update(&mut self, index: usize, pixel: T) {
        self._buffer.remove(index);
        self._buffer.insert(index, pixel);
    }

    #[allow(dead_code)]
    pub fn append(&mut self, value: T){
        self._buffer.push(value);
    }

    #[allow(dead_code)]
    pub fn from_rgb_f64_buffer(buffer: Vec<Vec<f64>>) -> Buffer {
        let mut out_buffer: Vec<Srgb> = Vec::new();
        for i in buffer {
            out_buffer.push(Srgb::from_components((
                i[0] as f32,
                i[1] as f32,
                i[2] as f32,
            )));
        }

        Buffer { _buffer: out_buffer }
    }

    #[allow(dead_code)]
    pub fn load(buffer: Vec<T>) -> Buffer::<T> {
        Buffer::<T> { _buffer: buffer }
    }
}

impl Buffer<Srgb> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self._buffer {
            let (r, g, b) = pixel.into_components();
            out_buffer.push(vec![
                (r * 255.0).into(),
                (g * 255.0).into(),
                (b * 255.0).into(),
            ])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Srgb> {
        let mut out_buffer: Vec<Srgb> = Vec::new();
        for i in buffer {
            out_buffer.push(Srgb::from_components((
                i[0] as f32,
                i[1] as f32,
                i[2] as f32,
            )));
        }

        Buffer { _buffer: out_buffer }
    }
}

impl Buffer<Hsv> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self._buffer {
            let (h, s, v) = pixel.into_components();
            out_buffer.push(vec![h.into_degrees() as f64, s as f64, v as f64])
        }
        out_buffer
    }
    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Hsv> {
        let mut out_buffer: Vec<Hsv> = Vec::new();
        for i in buffer {
            out_buffer.push(Hsv::from_components((
                i[0] as f32,
                i[1] as f32,
                i[2] as f32,
            )));
        }

        Buffer { _buffer: out_buffer }
    }
}

impl Buffer<Hsl> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self._buffer {
            let (h, s, l) = pixel.into_components();
            out_buffer.push(vec![h.into_degrees() as f64, s as f64, l as f64])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Hsl> {
        let mut out_buffer: Vec<Hsl> = Vec::new();
        for i in buffer {
            out_buffer.push(Hsl::from_components((
                i[0] as f32,
                i[1] as f32,
                i[2] as f32,
            )));
        }

        Buffer { _buffer: out_buffer }
    }

    pub fn collect_luma(&self) -> Vec<f32> {
        let mut out_buffer: Vec<f32> = Vec::new();
        for i in self._buffer.iter(){
            out_buffer.push(i.lightness);
        }
        out_buffer
    }
}

impl Buffer<Oklab> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self._buffer {
            let (l, a, b) = pixel.into_components();
            out_buffer.push(vec![l as f64, a as f64, b as f64])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Oklab> {
        let mut out_buffer: Vec<Oklab> = Vec::new();
        for i in buffer {
            out_buffer.push(Oklab::from_components((
                i[0] as f32,
                i[1] as f32,
                i[2] as f32,
            )));
        }

        Buffer { _buffer: out_buffer }
    }
}

impl Buffer<Oklch> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self._buffer {
            let (l, c, h) = pixel.into_components();
            out_buffer.push(vec![l as f64, c as f64, h.into_degrees() as f64])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Oklch> {
        let mut out_buffer: Vec<Oklch> = Vec::new();
        for i in buffer {
            out_buffer.push(Oklch::from_components((
                i[0] as f32,
                i[1] as f32,
                i[2] as f32,
            )));
        }

        Buffer { _buffer: out_buffer }
    }
}

impl Buffer<Xyz> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for pixel in &self._buffer {
            let (x, y, z) = pixel.into_components();
            out_buffer.push(vec![x as f64, y as f64, z as f64])
        }
        out_buffer
    }

    pub fn from_components(buffer: &Vec<Vec<f64>>) -> Buffer<Xyz> {
        let mut out_buffer: Vec<Xyz> = Vec::new();
        for i in buffer {
            out_buffer.push(Xyz::from_components((
                i[0] as f32,
                i[1] as f32,
                i[2] as f32,
            )));
        }

        Buffer { _buffer: out_buffer }
    }
}

impl<T> IntoIterator for Buffer<T>
where
    T: FromColorUnclamped<T> + IntoColor<T> + Copy,
{
    type Item = T;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self._buffer.into_iter()
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for Buffer<T> {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        //let _vec: Vec<T> = self._buffer;
        //_vec.index(index)
        Index::index(&self._buffer, index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for Buffer<T> {
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        IndexMut::index_mut(&mut self._buffer, index)
    }
}