pub(crate) use palette::convert::FromColorUnclamped;
use palette::FromColor;
use palette::IntoColor;
use palette::{Hsl, Srgb};
//use palette::{Hsv, Oklab, Oklch, Xyz};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::vec::Vec;
use std::slice::SliceIndex;

use crate::helpers::{CollectDataType, norm_range_f32};

#[derive(Clone)]
pub struct Buffer<T = Srgb> {
    _buffer: Vec<Vec<T>>,
    image_size: (u32, u32),
}

impl<T> Buffer<T>
where
    T: FromColorUnclamped<T> + IntoColor<T> + Copy + Clone + std::fmt::Debug,
{
    #[allow(dead_code)]
    pub fn convert_to<TO>(&self) -> Buffer<TO>
    where
        T: FromColorUnclamped<T> + IntoColor<TO> + Copy + std::fmt::Debug,
        TO: FromColorUnclamped<TO> + FromColor<T> + FromColor<TO> + Copy + std::fmt::Debug,
    {
        let mut out_buffer: Vec<Vec<TO>> = Vec::new();
        for y in &self._buffer {
            let mut row: Vec<TO> = Vec::new();
            for c in y {
                let color: TO = (*c).into_color();
                row.push(color);
            }
            out_buffer.push(row);
        }
        Buffer::<TO>::load(out_buffer, self.image_size)
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::vec::IntoIter<Vec<T>> {
        self._buffer.clone().into_iter()
    }

    #[allow(dead_code)]
    pub fn iter_pixels(&self) -> std::vec::IntoIter<T> {
        let mut pixels: Vec<T> = Vec::new();
        for y in self.iter() {
            for x in y { pixels.push(x); }
        }

        pixels.into_iter()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let mut _len: usize = 0;
        for y in &self._buffer {
            _len += y.len();
        }
        _len
    }

    #[allow(dead_code)]
    pub fn max_len(&self) -> usize {
        (self.image_size.0*self.image_size.1) as usize
    }

    #[allow(dead_code)]
    pub fn new(image_size: (u32, u32)) -> Buffer<T> {
        Buffer::<T> {
            _buffer: Vec::<Vec<T>>::new(),
            image_size
        }
    }

    pub fn get_size_in_byte(&self) -> usize {
        let mut used_memory: usize = 0;
        for i in self.iter() {
            used_memory += std::mem::size_of_val(&*i);
        }
        if self.len() > 0 { println!("Single struct used bytes: {}", std::mem::size_of_val(&self[0][0]));}
        used_memory + std::mem::size_of_val(self)
    }

    #[allow(dead_code)]
    pub fn get_pixel(&self, x: u32, y: u32) -> &T {
        &self._buffer[y as usize][x as usize]
    }

    #[allow(dead_code)]
    pub fn update(&mut self, x: u32, y: u32, pixel: T) {
        self._buffer[y as usize].remove(x as usize);
        self._buffer[y as usize].insert(x as usize, pixel);
    }

    #[allow(dead_code)]
    pub fn add_row(&mut self, row: Vec<T>){
        if self._buffer.len() < self.image_size.1 as usize {
            self._buffer.push(row);
        } else {
            panic!("Buffer is full! Consider increasing the image size")
        }
    }

    #[allow(dead_code)]
    pub fn add_item_to_row(&mut self, row_number: usize, item: T) {
        if self._buffer[row_number].len() < self.image_size.0 as usize {
            self._buffer[row_number].push(item);
        } else {
            panic!("Buffer is full! Consider increasing the image size")
        }
    }

    pub fn add_item(&mut self, item: T) {
        if self._buffer.len() == 0 ||  self._buffer[self.last_row_number()].len() == self.image_size.0 as usize {
            self.add_row(Vec::new());
            self.add_item_to_row(self.last_row_number(), item);
        } else { self.add_item_to_row(self.last_row_number(), item); }
    }

    #[allow(dead_code)]
    pub fn load(buffer: Vec<Vec<T>>, image_size: (u32, u32)) -> Buffer::<T> {
        Buffer::<T> { _buffer: buffer, image_size }
    }

    #[allow(dead_code)]
    pub fn load_from_pixels(buffer: Vec<T>, image_size: (u32, u32)) -> Buffer::<T> {
        let mut new_buffer: Vec<Vec<T>> = Vec::new();

        if buffer.len() != (image_size.0*image_size.1) as usize {
            panic!("Sizes are not matching")
        }

        for y in 0..image_size.1 {
            new_buffer.push(buffer[(y*image_size.1)as usize..(y*image_size.1+image_size.0) as usize].to_vec());
        }

        Buffer::<T> { _buffer: new_buffer, image_size }
    }

    #[allow(dead_code)]
    pub fn as_vec(&self) -> &Vec<Vec<T>> {
        &self._buffer
    }

    #[allow(dead_code)]
    pub fn as_vec_mut(&mut self) -> &mut Vec<Vec<T>> {
        &mut self._buffer
    }

    #[allow(dead_code)]
    pub fn get_image_size(&self) -> (u32, u32) {
        self.image_size
    }

    pub fn get_area(&self, pos: (u32, u32), size: (u32, u32)) -> Buffer<T> {
        let mut out_buf: Buffer<T> = Buffer::<T>::new(size);
        for y in pos.1..pos.1+size.1{
            let row: Vec<T> = self[y as usize][(pos.0) as usize..(pos.0+size.0) as usize].to_vec();
            out_buf.add_row(row);
        }
        out_buf
    }

    pub fn last_row_number(&self) -> usize {
        self._buffer.len()-1
    }

    #[allow(dead_code)]
    pub fn save_as_gray_buffer(&self, buffer: &Vec<Vec<f32>>, path: &str) -> Result<(), image::ImageError> {
        let mut out_buffer: Vec<u8> = Vec::new();

        for y in buffer{
            for x in y {
                for _ in 0..3 { out_buffer.push((x * 255.0) as u8); }
            }
        }

        let (width, height) = self.image_size;

        image::save_buffer_with_format(path, &out_buffer.as_slice(), width, height, image::ColorType::Rgb8, image::ImageFormat::Jpeg)
    }

}

impl Buffer<Srgb> {

    pub fn get_all(&self, data_type: CollectDataType) -> Vec<Vec<f32>>{
        let index: usize;
        match data_type {
            CollectDataType::Red => index = 0,
            CollectDataType::Green => index = 1,
            CollectDataType::Blue => index = 2,
            CollectDataType::Luminance => return self.convert_to::<Hsl>().collect_luma(),
        }

        let mut out_buffer: Vec<Vec<f32>> = Vec::new();

        for y in self.iter() {
            let mut row: Vec<f32> = Vec::new();
            for x in y {
                let c: f32 = match index {
                    0 => x.into_components().0,
                    1 => x.into_components().1,
                    2 => x.into_components().2,
                    _ => panic!()
                };
                row.push(c);
            }
            out_buffer.push(row);
        }

        out_buffer
    }

    #[allow(dead_code)]
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for y in &self._buffer{
            for pixel in y {
                let (r, g, b) = pixel.into_components();
                out_buffer.push(vec![
                    (r * 255.0).into(),
                    (g * 255.0).into(),
                    (b * 255.0).into(),
                ])
            }
        }
        out_buffer
    }

    #[allow(dead_code)]
    pub fn from_float_buffer(buffer: &Vec<Vec<f32>>, image_size: (u32, u32)) -> Buffer {
        let mut out_buffer: Buffer<Srgb> = Buffer::<Srgb>::new(image_size);
        
        for rgb in buffer{
            out_buffer.add_item(Srgb::new(
                rgb[0] as f32 / 255f32, 
                rgb[1] as f32 / 255f32, 
                rgb[2] as f32 / 255f32
            ));
        }

        out_buffer
    }

    pub fn combine_grayscale_with_colored(
        &self, 
        gray_scale_buffer: &Vec<Vec<f32>>
    ) -> Buffer {
        let _buffer: Buffer<Hsl> = self.convert_to::<Hsl>();
        let mut out_buffer: Buffer<Hsl> = Buffer::<Hsl>::new(self.get_image_size());

        for y in 0.._buffer.get_image_size().1 as usize {
            for x in 0.._buffer.get_image_size().0 as usize {
                let hsl_color: Hsl = Hsl::new(
                    _buffer[y][x].hue,
                    _buffer[y][x].saturation,
                    gray_scale_buffer[y][x]
                );
                out_buffer.add_item(hsl_color);
            }
        }
    
        out_buffer.convert_to::<Srgb>()
    }

    pub fn save_jpeg_image(&self, path: &str) -> Result<(), image::ImageError> {
        let mut out_buf: Vec<u8> = Vec::new();

        for y in self.iter() {
            for i in  y {
                let comp = i.into_components();
                out_buf.push((comp.0 * 255.0) as u8);
                out_buf.push((comp.1 * 255.0) as u8);
                out_buf.push((comp.2 * 255.0) as u8);
            }
        }

        let (width, height) = self.image_size;

        image::save_buffer_with_format(path, &out_buf.as_slice(), width, height, image::ColorType::Rgb8, image::ImageFormat::Jpeg)
    }

//    pub fn apply_3x3_convolution_mask(&self, mask: [[f32; 3]; 3]) -> Buffer {
//        let mut new_buffer: Vec<f32> = Vec::new();
//        
//        for y in -1..(self.image_size.1-1) as i64 {
//            for x in -1..(self.image_size.0-1) as i6//4 {
//
//                let mut _conv_out: Vec<f32> = Vec::new();
//                if x < 0 && y < 0 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, 0), (2,2)).convert_to::<Hsl>().collect_luma();
//                    _conv_out.push(mask[1][1]*buf_matrix[0]);
//                    _conv_out.push(mask[1][2]*buf_matrix[1]);
//                    _conv_out.push(mask[2][1]*buf_matrix[2]);
//                    _conv_out.push(mask[2][2]*buf_matrix[3]);
//                } 
//                else if y < 0 && 
//                    x >= 0 && x < (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, 0), (3,2)).convert_to::<Hsl>().collect_luma();
//                    _conv_out.push(mask[1][0]*buf_matrix[0]);
//                    _conv_out.push(mask[1][1]*buf_matrix[1]);
//                    _conv_out.push(mask[1][2]*buf_matrix[2]);
//                    _conv_out.push(mask[2][0]*buf_matrix[3]);
//                    _conv_out.push(mask[2][1]*buf_matrix[4]);
//                    _conv_out.push(mask[2][2]*buf_matrix[5]);
//                } 
//                else if y < 0 && x == (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, 0), (2,2)).convert_to::<Hsl>().collect_luma();
//                    _conv_out.push(mask[1][0]*buf_matrix[0]);
//                    _conv_out.push(mask[1][1]*buf_matrix[1]);
//                    _conv_out.push(mask[2][0]*buf_matrix[2]);
//                    _conv_out.push(mask[2][1]*buf_matrix[3]);
//                } 
//                else if y >= 0 && 
//                    y != (self.image_size.1-1) as i64 && 
//                    x >= 0 && x != (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, y as u32), (9,9)).convert_to::<Hsl>().collect_luma();
//                    for y in 0..3 {
//                        _conv_out.push(mask[y][0]*buf_matrix[0+y*3]);
//                        _conv_out.push(mask[y][1]*buf_matrix[1+y*3]);
//                        _conv_out.push(mask[y][2]*buf_matrix[2+y*3]);
//                    }
//                } 
//                else if y >= 0 && y != (self.image_size.1-1) as i64 && x < 0 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, y as u32), (2,3)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][1]*buf_matrix[0]);
//                    _conv_out.push(mask[0][2]*buf_matrix[1]);
//                    _conv_out.push(mask[1][1]*buf_matrix[2]);
//                    _conv_out.push(mask[1][2]*buf_matrix[3]);
//                    _conv_out.push(mask[2][1]*buf_matrix[4]);
//                    _conv_out.push(mask[2][2]*buf_matrix[5]);
//                } 
//                else if y >= 0 && 
//                    y != (self.image_size.1-1) as i64 && 
//                    x == (self.image_size.0-1) as i64
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, y as u32), (2,3)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][0]*buf_matrix[0]);
//                    _conv_out.push(mask[0][1]*buf_matrix[1]);
//                    _conv_out.push(mask[1][0]*buf_matrix[2]);
//                    _conv_out.push(mask[1][1]*buf_matrix[3]);
//                    _conv_out.push(mask[2][0]*buf_matrix[4]);
//                    _conv_out.push(mask[2][1]*buf_matrix[5]);
//                }
//                else if y == (self.image_size.1-1) as i64 && 
//                    x < 0
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, y as u32), (2,2)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][1]*buf_matrix[0]);
//                    _conv_out.push(mask[0][2]*buf_matrix[1]);
//                    _conv_out.push(mask[1][1]*buf_matrix[2]);
//                    _conv_out.push(mask[1][2]*buf_matrix[3]);
//                } 
//                else if y == (self.image_size.1-1) as i64 && 
//                    x >= 0 && x != (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, y as u32), (3,2)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][0]*buf_matrix[0]);
//                    _conv_out.push(mask[0][1]*buf_matrix[1]);
//                    _conv_out.push(mask[0][2]*buf_matrix[2]);
//                    _conv_out.push(mask[1][0]*buf_matrix[3]);
//                    _conv_out.push(mask[1][1]*buf_matrix[4]);
//                    _conv_out.push(mask[1][2]*buf_matrix[5]);
//                } 
//                else if y < 0 && x == (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, 0), (2,2)).convert_to::<Hsl>().collect_luma();
//
//                    _conv_out.push(mask[1][0]*buf_matrix[0]);
//                    _conv_out.push(mask[1][1]*buf_matrix[1]);
//                    _conv_out.push(mask[2][0]*buf_matrix[2]);
//                    _conv_out.push(mask[2][1]*buf_matrix[3]);
//                }
//                new_buffer.push(_conv_out.iter().sum::<f32>());
//            }
//        }
//
//        self.combine_grayscale_with_colored(&new_buffer)
//
//    }

    pub fn apply_convolution_mask(&self, mask: [[f32; 3]; 3]) -> Buffer {

        let buffer = self.add_padding((1,1,1,1));

        //let rgb_map: HashMap<u32, Vec<Vec<f32>>> = HashMap::new();
        //rgb_map.insert(0, buffer.get_all(CollectDataType::Red));
        //rgb_map.insert(1, buffer.get_all(CollectDataType::Green));
        //rgb_map.insert(2, buffer.get_all(CollectDataType::Blue));

        let mut c_val_map: HashMap<u32, CollectDataType> = HashMap::new();
        c_val_map.insert(0, CollectDataType::Red);
        c_val_map.insert(1, CollectDataType::Green);
        c_val_map.insert(2, CollectDataType::Blue);


        let mut out_b: Buffer = Buffer::new(self.image_size);

        for y in 0..self.image_size.1 {
            for x in 0..self.image_size.0 {
                print!("\rpixel: ({}, {}) ", x, y);
                let mut c_buf: Vec<f32> = Vec::new();
                for c in 0..3 {
                    let mut _conv_out: f32 = 0.0;

                    let buf_matrix: Vec<Vec<f32>> = buffer.get_area(
                        (x as u32, y as u32), 
                        (3,3)
                    )
                    .get_all(c_val_map[&c]);

                    for _y in 0..3usize {
                        _conv_out += mask[_y][0]*buf_matrix[_y][0] + mask[_y][1]*buf_matrix[_y][1] + mask[_y][2]*buf_matrix[_y][2];
                    }
                    c_buf.push(_conv_out);
                }
                out_b.add_item(
                    Srgb::new(
                        norm_range_f32(0f32..=1f32, c_buf[0]), 
                        norm_range_f32(0f32..=1f32, c_buf[1]),
                        norm_range_f32(0f32..=1f32, c_buf[2])
                    )
                );
            }
        }

        println!("\nEnd loop");

        out_b
    }

    pub fn add_padding(&self, padding: (u32, u32, u32, u32)) -> Buffer {
        let (sx, top,dx, bottom) = padding;

        let mut _buf_v: Vec<Vec<Srgb>> = Vec::new();

        let black = || Srgb::new(0.0, 0.0, 0.0);

        if top > 0 {
            for _ in 0..top { 
                let mut row: Vec<Srgb> = Vec::new();
                for _ in 0..(self.image_size.0 + sx+dx) {
                    row.push(black());
                }
                _buf_v.push(row);
            }
        }

        for y in &self._buffer {
            let mut row: Vec<Srgb> = Vec::new();
            if sx > 0 { for _ in 0..sx { row.push(black()); } }

            for x in y { row.push(*x); }

            if dx > 0 { for _ in 0..dx { row.push(black()); } }

            _buf_v.push(row);
        }

        if bottom > 0 {
            for _ in 0..bottom { 
                let mut row: Vec<Srgb> = Vec::new();
                for _ in 0..(self.image_size.0 + sx+dx) {
                    row.push(black());
                }
                _buf_v.push(row);
            }
        }

        Buffer::load(_buf_v, (self.image_size.0 + sx + dx, self.image_size.1 + top + bottom))
    }

}

/*impl Buffer<Srgb<u8>> {

    pub fn get_all(&self, data_type: CollectDataType) -> Vec<Vec<f32>>{
        let index: usize;
        match data_type {
            CollectDataType::Red => index = 0,
            CollectDataType::Green => index = 1,
            CollectDataType::Blue => index = 2,
            CollectDataType::Luminance => return self.convert_to::<Hsl>().collect_luma(),
        }

        let mut out_buffer: Vec<Vec<f32>> = Vec::new();

        for y in self.iter() {
            let mut row: Vec<f32> = Vec::new();
            for x in y {
                let c: f32 = match index {
                    0 => x.into_components().0,
                    1 => x.into_components().1,
                    2 => x.into_components().2,
                    _ => panic!()
                };
                row.push(c);
            }
            out_buffer.push(row);
        }

        out_buffer
    }

    #[allow(dead_code)]
    pub fn convert_to_f64(&self) -> Vec<Vec<u8>> {
        let mut out_buffer: Vec<Vec<u8>> = Vec::new();
        for y in &self._buffer{
            for pixel in y {
                let (r, g, b) = pixel.into_components();
                out_buffer.push(vec![
                    r, g, b,
                ])
            }
        }
        out_buffer
    }

    #[allow(dead_code)]
    pub fn from_float_buffer(buffer: &Vec<Vec<f32>>, image_size: (u32, u32)) -> Buffer {
        let mut out_buffer: Buffer<Srgb> = Buffer::<Srgb>::new(image_size);
        
        for rgb in buffer{
            out_buffer.add_item(Srgb::new(
                rgb[0] as f32 / 255f32, 
                rgb[1] as f32 / 255f32, 
                rgb[2] as f32 / 255f32
            ));
        }

        out_buffer
    }

    pub fn combine_grayscale_with_colored(
        &self, 
        gray_scale_buffer: &Vec<Vec<f32>>
    ) -> Buffer {
        let _buffer: Buffer<Hsl> = self.convert_to::<Hsl>();
        let mut out_buffer: Buffer<Hsl> = Buffer::<Hsl>::new(self.get_image_size());

        for y in 0.._buffer.get_image_size().1 as usize {
            for x in 0.._buffer.get_image_size().0 as usize {
                let hsl_color: Hsl = Hsl::new(
                    _buffer[y][x].hue,
                    _buffer[y][x].saturation,
                    gray_scale_buffer[y][x]
                );
                out_buffer.add_item(hsl_color);
            }
        }
    
        out_buffer.convert_to::<Srgb>()
    }

    pub fn save_jpeg_image(&self, path: &str) -> Result<(), image::ImageError> {
        let mut out_buf: Vec<u8> = Vec::new();

        for y in self.iter() {
            for i in  y {
                let comp = i.into_components();
                out_buf.push((comp.0 * 255.0) as u8);
                out_buf.push((comp.1 * 255.0) as u8);
                out_buf.push((comp.2 * 255.0) as u8);
            }
        }

        let (width, height) = self.image_size;

        image::save_buffer_with_format(path, &out_buf.as_slice(), width, height, image::ColorType::Rgb8, image::ImageFormat::Jpeg)
    }

//    pub fn apply_3x3_convolution_mask(&self, mask: [[f32; 3]; 3]) -> Buffer {
//        let mut new_buffer: Vec<f32> = Vec::new();
//        
//        for y in -1..(self.image_size.1-1) as i64 {
//            for x in -1..(self.image_size.0-1) as i6//4 {
//
//                let mut _conv_out: Vec<f32> = Vec::new();
//                if x < 0 && y < 0 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, 0), (2,2)).convert_to::<Hsl>().collect_luma();
//                    _conv_out.push(mask[1][1]*buf_matrix[0]);
//                    _conv_out.push(mask[1][2]*buf_matrix[1]);
//                    _conv_out.push(mask[2][1]*buf_matrix[2]);
//                    _conv_out.push(mask[2][2]*buf_matrix[3]);
//                } 
//                else if y < 0 && 
//                    x >= 0 && x < (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, 0), (3,2)).convert_to::<Hsl>().collect_luma();
//                    _conv_out.push(mask[1][0]*buf_matrix[0]);
//                    _conv_out.push(mask[1][1]*buf_matrix[1]);
//                    _conv_out.push(mask[1][2]*buf_matrix[2]);
//                    _conv_out.push(mask[2][0]*buf_matrix[3]);
//                    _conv_out.push(mask[2][1]*buf_matrix[4]);
//                    _conv_out.push(mask[2][2]*buf_matrix[5]);
//                } 
//                else if y < 0 && x == (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, 0), (2,2)).convert_to::<Hsl>().collect_luma();
//                    _conv_out.push(mask[1][0]*buf_matrix[0]);
//                    _conv_out.push(mask[1][1]*buf_matrix[1]);
//                    _conv_out.push(mask[2][0]*buf_matrix[2]);
//                    _conv_out.push(mask[2][1]*buf_matrix[3]);
//                } 
//                else if y >= 0 && 
//                    y != (self.image_size.1-1) as i64 && 
//                    x >= 0 && x != (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, y as u32), (9,9)).convert_to::<Hsl>().collect_luma();
//                    for y in 0..3 {
//                        _conv_out.push(mask[y][0]*buf_matrix[0+y*3]);
//                        _conv_out.push(mask[y][1]*buf_matrix[1+y*3]);
//                        _conv_out.push(mask[y][2]*buf_matrix[2+y*3]);
//                    }
//                } 
//                else if y >= 0 && y != (self.image_size.1-1) as i64 && x < 0 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, y as u32), (2,3)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][1]*buf_matrix[0]);
//                    _conv_out.push(mask[0][2]*buf_matrix[1]);
//                    _conv_out.push(mask[1][1]*buf_matrix[2]);
//                    _conv_out.push(mask[1][2]*buf_matrix[3]);
//                    _conv_out.push(mask[2][1]*buf_matrix[4]);
//                    _conv_out.push(mask[2][2]*buf_matrix[5]);
//                } 
//                else if y >= 0 && 
//                    y != (self.image_size.1-1) as i64 && 
//                    x == (self.image_size.0-1) as i64
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, y as u32), (2,3)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][0]*buf_matrix[0]);
//                    _conv_out.push(mask[0][1]*buf_matrix[1]);
//                    _conv_out.push(mask[1][0]*buf_matrix[2]);
//                    _conv_out.push(mask[1][1]*buf_matrix[3]);
//                    _conv_out.push(mask[2][0]*buf_matrix[4]);
//                    _conv_out.push(mask[2][1]*buf_matrix[5]);
//                }
//                else if y == (self.image_size.1-1) as i64 && 
//                    x < 0
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, y as u32), (2,2)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][1]*buf_matrix[0]);
//                    _conv_out.push(mask[0][2]*buf_matrix[1]);
//                    _conv_out.push(mask[1][1]*buf_matrix[2]);
//                    _conv_out.push(mask[1][2]*buf_matrix[3]);
//                } 
//                else if y == (self.image_size.1-1) as i64 && 
//                    x >= 0 && x != (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((x as u32, y as u32), (3,2)).convert_to::<Hsl>().collect_luma//();
//
//                    _conv_out.push(mask[0][0]*buf_matrix[0]);
//                    _conv_out.push(mask[0][1]*buf_matrix[1]);
//                    _conv_out.push(mask[0][2]*buf_matrix[2]);
//                    _conv_out.push(mask[1][0]*buf_matrix[3]);
//                    _conv_out.push(mask[1][1]*buf_matrix[4]);
//                    _conv_out.push(mask[1][2]*buf_matrix[5]);
//                } 
//                else if y < 0 && x == (self.image_size.0-1) as i64 
//                {
//                    let buf_matrix: Vec<f32> = self.get_area((0, 0), (2,2)).convert_to::<Hsl>().collect_luma();
//
//                    _conv_out.push(mask[1][0]*buf_matrix[0]);
//                    _conv_out.push(mask[1][1]*buf_matrix[1]);
//                    _conv_out.push(mask[2][0]*buf_matrix[2]);
//                    _conv_out.push(mask[2][1]*buf_matrix[3]);
//                }
//                new_buffer.push(_conv_out.iter().sum::<f32>());
//            }
//        }
//
//        self.combine_grayscale_with_colored(&new_buffer)
//
//    }

    pub fn apply_convolution_mask(&self, mask: [[f32; 3]; 3]) -> Buffer {

        let buffer = self.add_padding((1,1,1,1));

        //let rgb_map: HashMap<u32, Vec<Vec<f32>>> = HashMap::new();
        //rgb_map.insert(0, buffer.get_all(CollectDataType::Red));
        //rgb_map.insert(1, buffer.get_all(CollectDataType::Green));
        //rgb_map.insert(2, buffer.get_all(CollectDataType::Blue));

        let mut c_val_map: HashMap<u32, CollectDataType> = HashMap::new();
        c_val_map.insert(0, CollectDataType::Red);
        c_val_map.insert(1, CollectDataType::Green);
        c_val_map.insert(2, CollectDataType::Blue);


        let mut out_b: Buffer = Buffer::new(self.image_size);

        for y in 0..self.image_size.1 {
            for x in 0..self.image_size.0 {
                print!("\rpixel: ({}, {}) ", x, y);
                let mut c_buf: Vec<f32> = Vec::new();
                for c in 0..3 {
                    let mut _conv_out: f32 = 0.0;

                    let buf_matrix: Vec<Vec<f32>> = buffer.get_area(
                        (x as u32, y as u32), 
                        (3,3)
                    )
                    .get_all(c_val_map[&c]);

                    for _y in 0..3usize {
                        _conv_out += mask[_y][0]*buf_matrix[_y][0] + mask[_y][1]*buf_matrix[_y][1] + mask[_y][2]*buf_matrix[_y][2];
                    }
                    c_buf.push(_conv_out);
                }
                out_b.add_item(
                    Srgb::new(
                        norm_range_f32(0f32..=1f32, c_buf[0]), 
                        norm_range_f32(0f32..=1f32, c_buf[1]),
                        norm_range_f32(0f32..=1f32, c_buf[2])
                    )
                );
            }
        }

        println!("\nEnd loop");

        out_b
    }

    pub fn add_padding(&self, padding: (u32, u32, u32, u32)) -> Buffer {
        let (sx, top,dx, bottom) = padding;

        let mut _buf_v: Vec<Vec<Srgb>> = Vec::new();

        let black = || Srgb::new(0.0, 0.0, 0.0);

        if top > 0 {
            for _ in 0..top { 
                let mut row: Vec<Srgb> = Vec::new();
                for _ in 0..(self.image_size.0 + sx+dx) {
                    row.push(black());
                }
                _buf_v.push(row);
            }
        }

        for y in &self._buffer {
            let mut row: Vec<Srgb> = Vec::new();
            if sx > 0 { for _ in 0..sx { row.push(black()); } }

            for x in y { row.push(*x); }

            if dx > 0 { for _ in 0..dx { row.push(black()); } }

            _buf_v.push(row);
        }

        if bottom > 0 {
            for _ in 0..bottom { 
                let mut row: Vec<Srgb> = Vec::new();
                for _ in 0..(self.image_size.0 + sx+dx) {
                    row.push(black());
                }
                _buf_v.push(row);
            }
        }

        Buffer::load(_buf_v, (self.image_size.0 + sx + dx, self.image_size.1 + top + bottom))
    }

}*/

impl Buffer<Hsl> {
    #[allow(dead_code)]
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for y in &self._buffer {
            for pixel in y {
                let (h, s, v) = pixel.into_components();
                out_buffer.push(vec![h.into_degrees() as f64, s as f64, v as f64])
            }
        }
        out_buffer
    }

    #[allow(dead_code)]
    pub fn from_f64_buffer(buffer: Vec<Vec<f64>>, image_size: (u32, u32)) -> Buffer<Hsl> {
        let mut out_buffer: Vec<Vec<Hsl>> = Vec::new();
        for y in 0..image_size.1 {
            let mut row: Vec<Hsl> = Vec::new();
            for x in 0..image_size.0 {
                let i: &Vec<f64> = &buffer[(y*image_size.1+x) as usize];
                row.push(Hsl::from_components((
                    i[0] as f32,
                    i[1] as f32,
                    i[2] as f32,
                )));
            }
            out_buffer.push(row);
        }
        Buffer::<Hsl> { _buffer: out_buffer, image_size }
    }

    pub fn collect_luma(&self) -> Vec<Vec<f32>> {
        let mut out_buffer: Vec<Vec<f32>> = Vec::new();
        for y in &self._buffer{
            let mut row: Vec<f32> = Vec::new();
            for x in y{
                row.push(x.lightness);
            }
            out_buffer.push(row);
        }
        out_buffer
    }
}

/*impl Buffer<Hsv> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for y in &self._buffer {
            for pixel in y {
                let (h, s, v) = pixel.into_components();
                out_buffer.push(vec![h.into_degrees() as f64, s as f64, v as f64])
            }
        }
        out_buffer
    }

    pub fn from_f64_buffer(buffer: Vec<Vec<f64>>, image_size: (u32, u32)) -> Buffer<Hsv> {
        let mut out_buffer: Vec<Vec<Hsv>> = Vec::new();
        for y in 0..image_size.1 {
            let mut row: Vec<Hsv> = Vec::new();
            for x in 0..image_size.0 {
                let i: &Vec<f64>  = &buffer[(y*image_size.1+x) as usize];
                row.push(Hsv::from_components((
                    i[0] as f32,
                    i[1] as f32,
                    i[2] as f32,
                )));
            }
            out_buffer.push(row);
        }
        Buffer::<Hsv> { _buffer: out_buffer, image_size }
    }

}

impl Buffer<Hsl> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for y in &self._buffer {
            for pixel in y {
                let (h, s, v) = pixel.into_components();
                out_buffer.push(vec![h.into_degrees() as f64, s as f64, v as f64])
            }
        }
        out_buffer
    }

    pub fn from_f64_buffer(buffer: Vec<Vec<f64>>, image_size: (u32, u32)) -> Buffer<Hsl> {
        let mut out_buffer: Vec<Vec<Hsl>> = Vec::new();
        for y in 0..image_size.1 {
            let mut row: Vec<Hsl> = Vec::new();
            for x in 0..image_size.0 {
                let i: &Vec<f64> = &buffer[(y*image_size.1+x) as usize];
                row.push(Hsl::from_components((
                    i[0] as f32,
                    i[1] as f32,
                    i[2] as f32,
                )));
            }
            out_buffer.push(row);
        }
        Buffer::<Hsl> { _buffer: out_buffer, image_size }
    }

    pub fn collect_luma(&self) -> Vec<Vec<f32>> {
        let mut out_buffer: Vec<Vec<f32>> = Vec::new();
        for y in &self._buffer{
            let mut row: Vec<f32> = Vec::new();
            for x in y{
                row.push(x.lightness);
            }
            out_buffer.push(row);
        }
        out_buffer
    }
}

impl Buffer<Oklab> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for y in &self._buffer {
            for pixel in y {
                let (l, a, b) = pixel.into_components();
                out_buffer.push(vec![l as f64, a as f64, b as f64])
            }
        }
        out_buffer
    }

    pub fn from_f64_buffer(buffer: &Vec<Vec<f64>>, image_size: (u32, u32)) -> Buffer<Oklab> {
        let mut out_buffer: Vec<Vec<Oklab>> = Vec::new();
        for y in 0..image_size.1 {
            let mut row: Vec<Oklab> = Vec::new();
            for x in 0..image_size.0 {
                let i: &Vec<f64> = &buffer[(y*image_size.1+x) as usize];
                row.push(Oklab::from_components((
                    i[0] as f32,
                    i[1] as f32,
                    i[2] as f32,
                )));
            }
            out_buffer.push(row);
        }

        Buffer { _buffer: out_buffer, image_size }
    }
}

impl Buffer<Oklch> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for y in &self._buffer {
            for pixel in y {
                let (l, c, h) = pixel.into_components();
                out_buffer.push(vec![l as f64, c as f64, h.into_degrees() as f64])
            }
        }
        out_buffer
    }

    pub fn from_f64_buffer(buffer: &Vec<Vec<f64>>, image_size: (u32, u32)) -> Buffer<Oklch> {
        let mut out_buffer: Vec<Vec<Oklch>> = Vec::new();

        for y in 0..image_size.1 {
            let mut row: Vec<Oklch> = Vec::new();
            for x in 0..image_size.0 {
                let i: &Vec<f64> = &buffer[(y*image_size.1+x) as usize];
                row.push(Oklch::from_components((
                    i[0] as f32,
                    i[1] as f32,
                    i[2] as f32,
                )));
            }
            out_buffer.push(row);
        }

        Buffer { _buffer: out_buffer, image_size }
    }
}

impl Buffer<Xyz> {
    pub fn convert_to_f64(&self) -> Vec<Vec<f64>> {
        let mut out_buffer: Vec<Vec<f64>> = Vec::new();
        for y in &self._buffer {
            for pixel in y {
                let (x, y, z) = pixel.into_components();
                out_buffer.push(vec![x as f64, y as f64, z as f64])
            }
        }
        out_buffer
    }

    pub fn from_f64_buffer(buffer: &Vec<Vec<f64>>, image_size: (u32, u32)) -> Buffer<Xyz> {
        let mut out_buffer: Vec<Vec<Xyz>> = Vec::new();

        for y in 0..image_size.1 {
            let mut row: Vec<Xyz> = Vec::new();
            for x in 0..image_size.0 {
                let i: &Vec<f64> = &buffer[(y*image_size.1+x) as usize];
                row.push(Xyz::from_components((
                    i[0] as f32,
                    i[1] as f32,
                    i[2] as f32,
                )));
            }
            out_buffer.push(row);
        }

        Buffer { _buffer: out_buffer, image_size }
    }
}
*/

impl<T> IntoIterator for Buffer<T>
where
    T: FromColorUnclamped<T> + IntoColor<T> + Copy,
{
    type Item = Vec<T>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self._buffer.into_iter()
    }
}

impl<T, I: SliceIndex<[Vec<T>]>> Index<I> for Buffer<T> {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self._buffer, index)
    }
}

impl<T, I: SliceIndex<[Vec<T>]>> IndexMut<I> for Buffer<T> {
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        IndexMut::index_mut(&mut self._buffer, index)
    }
}