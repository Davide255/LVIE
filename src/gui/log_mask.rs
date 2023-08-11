pub(crate) use std::f64::consts;

use crate::helpers;

pub fn l_o_g_mask(x: i32, y: i32, sigma: f64) -> f64 {
    let x = x as f64;
    let y = y as f64;

    let nom: f64 = (y.powf(2.0) + x.powf(2.0) - 2_f64 * sigma.powf(2.0)).into();
    let denom: f64 = 2_f64 * consts::PI * (sigma as f64).powf(6.0);
    let expo: f64 = (-(x.powf(2.0) + y.powf(2.0)) / (2_f64 * sigma.powf(2.0))).into();

    return nom * expo.exp() / denom;
}

pub fn create_log(sigma: f64, size: i32) -> Vec<Vec<f64>> {
    let w: i32 = ((size as f64) * sigma).ceil() as i32;

    if w % 2 == 0 {
        let w: i32 = w + 1;
    }

    let mut log_mask: Vec<f64> = Vec::new();

    let w_range: i32 = (w as f64 / 2.0).floor() as i32;

    for i in -w_range..w_range {
        for j in -w_range..w_range {
            log_mask.push(l_o_g_mask(i, j, sigma));
        }
    }

    return helpers::vec_reshape(log_mask, (w, w));
}

pub fn convolve(buffer: Vec<f64>, image_size: (i32, i32), mask: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let width: i32 = image_size.0;
    let height: i32 = image_size.1;

    let w_range: i32 = (mask.len() as f64 / 2.0).floor() as i32;

    let mut res_image: Vec<Vec<f64>> = helpers::create_zeros_buffer((height, width));

    for i in w_range..width - w_range {
        for j in w_range..height - w_range {
            for k in -w_range..=w_range {
                for h in -w_range..=w_range {
                    res_image[j as usize][i as usize] += mask[(w_range + h) as usize]
                        [(w_range + k) as usize]
                        * (buffer[(((j + h) * width) + (i + k)) as usize] as f64)
                }
            }
        }
    }

    return res_image;
}

pub fn z_c_test(l_o_g_image: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut zc_image: Vec<Vec<f64>> =
        helpers::create_zeros_buffer((l_o_g_image.len() as i32, l_o_g_image[0].len() as i32));

    for i in 1..(l_o_g_image.len() as i32) - 1 {
        for j in 1..(l_o_g_image[0].len() as i32) - 1 {
            let mut neg_count: i32 = 0;
            let mut pos_count: i32 = 0;

            for a in -1..=1 {
                for b in -1..=1 {
                    if a != 0 && b != 0 {
                        if l_o_g_image[(i + a) as usize][(j + b) as usize] < 0.0 {
                            neg_count += 1;
                        } else if l_o_g_image[(i + a) as usize][(j + b) as usize] > 0.0 {
                            pos_count += 1;
                        }
                    }
                }
            }

            if neg_count > 0 && pos_count > 0 {
                zc_image[i as usize][j as usize] = 1.0;
            }
        }
    }

    return zc_image;
}
