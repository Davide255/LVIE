use std::sync::{Arc, Mutex};
use std::cmp::{max, min};

use image::{RgbImage, Pixel, RgbaImage};

fn idx(row: &u32, col: &u32, width: &u32, g: u32) -> u32 { (row * width + col) * g }

use rayon::prelude::*;

pub fn FastBoxBlur(img: &RgbImage, value: u32) -> RgbImage{

    // for benching:
    // let start = std::time::Instant::now();

    let (width, height) = img.dimensions();

    let out = Arc::new(Mutex::new(vec![0u8; (width*height*3) as usize]));

    let mut sum_r: Vec<u32> = vec![0; (width*height) as usize];
    let mut sum_g: Vec<u32> = vec![0; (width*height) as usize];
    let mut sum_b: Vec<u32> = vec![0; (width*height) as usize];

    for row in 0..height{
        for col in 1..width{
            let pixel = img.get_pixel(col, row).channels();
            sum_r[idx(&row, &col, &width, 1) as usize] = pixel[0] as u32 + sum_r[idx(&row, &(col - 1), &width, 1) as usize];
            sum_g[idx(&row, &col, &width, 1) as usize] = pixel[1] as u32 + sum_g[idx(&row, &(col - 1), &width, 1) as usize];
            sum_b[idx(&row, &col, &width, 1) as usize] = pixel[2] as u32 + sum_b[idx(&row, &(col - 1), &width, 1) as usize];
        }
    }

    for col in 0..width {
        for row  in 1..height {
            sum_r[idx(&row, &col, &width, 1) as usize] += sum_r[idx(&(row - 1), &col, &width, 1) as usize];
            sum_g[idx(&row, &col, &width, 1) as usize] += sum_g[idx(&(row - 1), &col, &width, 1) as usize];
            sum_b[idx(&row, &col, &width, 1) as usize] += sum_b[idx(&(row - 1), &col, &width, 1) as usize];
        }
    }

    let out_weak = Arc::clone(&out);
    (0..height).into_par_iter().for_each(move |row: u32| {
        let mut r = Vec::<u8>::new();
        for col in 0..width {
            // Coordinated of the corners of the square surrounding the pixel.
            let x_min = max(col - {if value <= col { value } else {0}}, 0);
            let x_max = min(col + value, width - 1);
            let y_min = max(row - {if value <= row { value } else {0}}, 0);
            let y_max = min(row + value, height - 1);

            // Number of pixels in the square.
            let pixels = ((x_max as i32 - (x_min as i32 - 1)) * (y_max as i32 - (y_min as i32 - 1))) as u32;

            // Do for each color channel (red, green, blue).
            // let mut rgb: [u8; 3] = [0u8; 3];
            for color in 0..3 {
                let sums_color: &Vec<u32>;
                if color == 0 {
                    sums_color = &sum_r;
                } else if color == 1 {
                    sums_color = &sum_g;
                } else { sums_color = &sum_b; }

                // The computation occurring below can be visually described,
                //      0      m        n
                //    0 +------+--------+-> rows
                //      |  a   |   b    |
                //    p +------+--------+
                //      |      |        |
                //      |  c   |   d    |
                //      |      |        |
                //    q +------+--------+
                //      |
                //      v
                //     columns
                //
                //  Where,
                //     'a' is a rectangle from (0, 0) to (p, m)
                //     'b' is a rectangle from (0, 0) to (p, n)
                //     'c' is a rectangle from (0, 0) to (q, m)
                //     'd' is a rectangle from (0, 0) to (q, n)
                //
                // The current pixel is in the middle of the box from (p, m) to
                // (q, n). The sum of all the pixels in the box surrounding the
                // pixel is then equal to `d - (c + b - a)`.
                let a: u32 = {if y_min < 1 || x_min < 1
                    { 0 } else { 
                        sums_color[idx(&(y_min - 1), &(x_min - 1), &width, 1) as usize]
                    }
                };

                let b: u32 = {if y_min < 1 { 0 }
                    else { sums_color[idx(&(y_min - 1), &x_max, &width, 1) as usize] }};

                let c: u32 = {if x_min < 1 { 0 }
                    else { sums_color[idx(&y_max, &(x_min - 1), &width, 1) as usize]}};
                let d = sums_color[idx(&y_max, &x_max, &width, 1) as usize];

                // Pixel's blurred value
                // rgb[color] = ((d - (b + c - a)) / pixels) as u8;
                r.push(((d - (b + c - a)) / pixels) as u8);
            }
        }
        let mut v = out_weak.lock().unwrap();
        for i in 0..r.len() {
            v[(row*width*3) as usize + i] = r[i];
        }
    });

    let buf = Arc::try_unwrap(out).unwrap().into_inner().unwrap();

    // for benching:
    // println!("Ended in {} milliseconds", start.elapsed().as_millis());

    RgbImage::from_raw(width, height, buf).unwrap()
}

pub fn FastBoxBlur_rgba(img: &RgbaImage, value: u32) -> RgbaImage{

    // for benching:
    // let start = std::time::Instant::now();

    let (width, height) = img.dimensions();

    let out = Arc::new(Mutex::new(vec![0u8; (width*height*4) as usize]));

    let mut sum_r: Vec<u32> = vec![0; (width*height) as usize];
    let mut sum_g: Vec<u32> = vec![0; (width*height) as usize];
    let mut sum_b: Vec<u32> = vec![0; (width*height) as usize];
    let mut alphas: Vec<u8> = vec![0; (width*height) as usize];

    for row in 0..height{
        for col in 1..width{
            let pixel = img.get_pixel(col, row).channels();
            sum_r[idx(&row, &col, &width, 1) as usize] = pixel[0] as u32 + sum_r[idx(&row, &(col - 1), &width, 1) as usize];
            sum_g[idx(&row, &col, &width, 1) as usize] = pixel[1] as u32 + sum_g[idx(&row, &(col - 1), &width, 1) as usize];
            sum_b[idx(&row, &col, &width, 1) as usize] = pixel[2] as u32 + sum_b[idx(&row, &(col - 1), &width, 1) as usize];
            alphas[idx(&row, &col, &width, 1) as usize] = pixel[3];
        }
    }

    for col in 0..width {
        for row  in 1..height {
            if col == 0 {
                // add missing alphas
                alphas[idx(&row, &col, &width, 1) as usize] = img.get_pixel(col, row).0[3];
            }
            sum_r[idx(&row, &col, &width, 1) as usize] += sum_r[idx(&(row - 1), &col, &width, 1) as usize];
            sum_g[idx(&row, &col, &width, 1) as usize] += sum_g[idx(&(row - 1), &col, &width, 1) as usize];
            sum_b[idx(&row, &col, &width, 1) as usize] += sum_b[idx(&(row - 1), &col, &width, 1) as usize];
        }
    }

    let out_weak = Arc::clone(&out);
    (0..height).into_par_iter().for_each(move |row: u32| {
        let mut r = Vec::<u8>::new();
        for col in 0..width {
            // Coordinated of the corners of the square surrounding the pixel.
            let x_min = max(col - {if value <= col { value } else {0}}, 0);
            let x_max = min(col + value, width - 1);
            let y_min = max(row - {if value <= row { value } else {0}}, 0);
            let y_max = min(row + value, height - 1);

            // Number of pixels in the square.
            let pixels = ((x_max as i32 - (x_min as i32 - 1)) * (y_max as i32 - (y_min as i32 - 1))) as u32;

            // Do for each color channel (red, green, blue).
            // let mut rgb: [u8; 4] = [0u8; 4];
            for color in 0..4 {
                if color == 3 { r.push(alphas[idx(&row, &col, &width, 1) as usize]); }
                else {
                    let sums_color: &Vec<u32>;
                    if color == 0 {
                        sums_color = &sum_r;
                    } else if color == 1 {
                        sums_color = &sum_g;
                    } else { sums_color = &sum_b; }

                    // The computation occurring below can be visually described,
                    //      0      m        n
                    //    0 +------+--------+-> rows
                    //      |  a   |   b    |
                    //    p +------+--------+
                    //      |      |        |
                    //      |  c   |   d    |
                    //      |      |        |
                    //    q +------+--------+
                    //      |
                    //      v
                    //     columns
                    //
                    //  Where,
                    //     'a' is a rectangle from (0, 0) to (p, m)
                    //     'b' is a rectangle from (0, 0) to (p, n)
                    //     'c' is a rectangle from (0, 0) to (q, m)
                    //     'd' is a rectangle from (0, 0) to (q, n)
                    //
                    // The current pixel is in the middle of the box from (p, m) to
                    // (q, n). The sum of all the pixels in the box surrounding the
                    // pixel is then equal to `d - (c + b - a)`.
                    let a: u32 = {if y_min < 1 || x_min < 1
                        { 0 } else { 
                            sums_color[idx(&(y_min - 1), &(x_min - 1), &width, 1) as usize]
                        }
                    };

                    let b: u32 = {if y_min < 1 { 0 }
                        else { sums_color[idx(&(y_min - 1), &x_max, &width, 1) as usize] }};

                    let c: u32 = {if x_min < 1 { 0 }
                        else { sums_color[idx(&y_max, &(x_min - 1), &width, 1) as usize]}};
                    let d = sums_color[idx(&y_max, &x_max, &width, 1) as usize];

                    // Pixel's blurred value
                    // rgb[color] = ((d - (b + c - a)) / pixels) as u8;
                    r.push(((d - (b + c - a)) / pixels) as u8);
                }
            }
        }
        let mut v = out_weak.lock().unwrap();
        for i in 0..r.len() {
            v[(row*width*4) as usize + i] = r[i];
        }
    });

    let buf = Arc::try_unwrap(out).unwrap().into_inner().unwrap();

    // for benching:
    // println!("Ended in {} milliseconds", start.elapsed().as_millis());

    RgbaImage::from_raw(width, height, buf).unwrap()
}