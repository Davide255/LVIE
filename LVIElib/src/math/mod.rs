use std::{collections::HashMap, fmt::Debug};

use image::{ImageBuffer, Pixel};
use num_traits::NumCast;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::traits::Scale;

pub fn normalize_2d(x: f32, y: f32) -> (f32, f32) {
    let norm = (x * x + y * y).sqrt();

    (x / norm, y / norm)
}

pub fn cumulative_distribution(data: &HashMap<u8, u32>) -> HashMap<u8, u32> {
    let mut output: HashMap<u8, u32> = HashMap::new();
    output.insert(0, *data.get(&0).unwrap());
    for i in 0..255 {
        output.insert(
            i + 1,
            *data.get(&(i + 1)).unwrap() + output.get(&i).unwrap(),
        );
    }

    output
}

fn get_color_at<T: Scale + Debug>(
    v: &Vec<T>,
    p: &Vec<f32>,
    size: f32,
    pos: f32,
    hsl: bool,
) -> Option<T> {
    let mut i = p.len() + 1;
    for k in 0..p.len() {
        if size * p[k] / 100.0 <= pos && pos < size * p[k + 1] / 100.0 {
            i = k;
            break;
        }
    }

    if i > p.len() {
        return None;
    }
    let d = size * p[i + 1] / 100.0 - size * p[i] / 100.0;
    let np = pos - size * p[i] / 100.0;

    let step = {
        if hsl {
            if (v[i].scale::<f32>() - v[i + 1].scale::<f32>()).abs()
                < (v[i].scale::<f32>() - v[i + 1].scale::<f32>() + 360.0).abs()
            {
                v[i].scale::<f32>() - v[i + 1].scale::<f32>()
            } else {
                // going backwards on the color wheel is smoother
                v[i].scale::<f32>() - v[i + 1].scale::<f32>() + 360.0
            }
        } else {
            v[i].scale::<f32>() - v[i + 1].scale::<f32>()
        }
    };

    Some((v[i].scale::<f32>() - step * np / d).scale())
}

pub fn linear_gradient<P>(
    size: (u32, u32),
    colors: Vec<(P, f32)>,
    angle: f32,
) -> ImageBuffer<P, Vec<P::Subpixel>>
where
    P: Pixel + 'static,
    P::Subpixel: Scale + Debug,
{
    let (width, height) = size;

    let f = colors[0].0.channels();
    let t = colors[1].0.channels();

    let angle = angle % 360.0;

    let flip = {
        if 0.0 < angle && angle <= 90.0 {
            (false, false)
        } else if 90.0 < angle && angle <= 180.0 {
            (true, false)
        } else if 180.0 < angle && angle <= 270.0 {
            (true, true)
        } else {
            (false, true)
        }
    };

    let angle = angle % 90.0;

    if angle == 0.0 {
        let steps: [f32; 4] = [
            {
                if P::COLOR_MODEL == "HSL" || P::COLOR_MODEL == "HSLA" {
                    if <f32 as NumCast>::from(f[0] - t[0]).unwrap().abs()
                        < <f32 as NumCast>::from(f[0] - t[0] + NumCast::from(360.0).unwrap())
                            .unwrap()
                            .abs()
                    {
                        <f32 as NumCast>::from(f[0] - t[0]).unwrap()
                    } else {
                        // going backwards on the color wheel is smoother
                        <f32 as NumCast>::from(f[0] - t[0]).unwrap() + 360.0
                    }
                } else {
                    NumCast::from(f[0] - t[0]).unwrap()
                }
            } / <f32 as NumCast>::from(width).unwrap(),
            (<f32 as NumCast>::from(f[1]).unwrap() - <f32 as NumCast>::from(t[1]).unwrap())
                / <f32 as NumCast>::from(width).unwrap(),
            (<f32 as NumCast>::from(f[2]).unwrap() - <f32 as NumCast>::from(t[2]).unwrap())
                / <f32 as NumCast>::from(width).unwrap(),
            (<f32 as NumCast>::from(f[3]).unwrap() - <f32 as NumCast>::from(t[3]).unwrap())
                / <f32 as NumCast>::from(width).unwrap(),
        ];

        let row = vec![0f32; (width as usize) * 4]
            .into_iter()
            .enumerate()
            .map(|(i, _)| {
                if P::COLOR_MODEL == "HSL" || P::COLOR_MODEL == "HSLA" {
                    NumCast::from(
                        <f32 as NumCast>::from(f[i % 4]).unwrap()
                            - steps[i % 4] * ((i / 4) as f32).floor() % 360.0,
                    )
                    .unwrap()
                } else {
                    NumCast::from(
                        <f32 as NumCast>::from(f[i % 4]).unwrap()
                            - steps[i % 4] * ((i / 4) as f32).floor(),
                    )
                    .unwrap()
                }
            })
            .collect::<Vec<P::Subpixel>>();

        let mut img: Vec<P::Subpixel> = Vec::new();
        for _ in 0..height {
            img.append(&mut row.clone())
        }
        let mut image_buff =
            ImageBuffer::<P, Vec<P::Subpixel>>::from_vec(width, height, img).unwrap();

        if flip.0 {
            image_buff = image::imageops::flip_horizontal(&image_buff);
        }
        image_buff
    } else {
        let mut image_buff = ImageBuffer::<P, Vec<P::Subpixel>>::new(width, height);
        let r = [-angle.to_radians().tan(), 1.0];

        let a = -1.0 / r[0];
        let b = 1f32;
        let c = -a * width as f32 - height as f32;

        let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();
        let steps: [f32; 4] = [
            {
                if P::COLOR_MODEL == "HSL" || P::COLOR_MODEL == "HSLA" {
                    if <f32 as NumCast>::from(f[0] - t[0]).unwrap().abs()
                        < <f32 as NumCast>::from(f[0] - t[0] + NumCast::from(360.0).unwrap())
                            .unwrap()
                            .abs()
                    {
                        <f32 as NumCast>::from(f[0] - t[0]).unwrap()
                    } else {
                        // going backwards on the color wheel is smoother
                        <f32 as NumCast>::from(f[0] - t[0]).unwrap() + 360.0
                    }
                } else {
                    NumCast::from(f[0] - t[0]).unwrap()
                }
            },
            <f32 as NumCast>::from(f[1]).unwrap() - <f32 as NumCast>::from(t[1]).unwrap(),
            <f32 as NumCast>::from(f[2]).unwrap() - <f32 as NumCast>::from(t[2]).unwrap(),
            <f32 as NumCast>::from(f[3]).unwrap() - <f32 as NumCast>::from(t[3]).unwrap(),
        ];

        for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
            let a = -1.0 / r[0];
            let b = 1f32;
            let c = -a * x as f32 - height as f32 + y as f32;

            let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

            let channels = [
                {
                    if P::COLOR_MODEL == "HSL" || P::COLOR_MODEL == "HSLA" {
                        NumCast::from(
                            (<f32 as NumCast>::from(f[0]).unwrap() - steps[0] * s / w + 360.0)
                                % 360.0,
                        )
                        .unwrap()
                    } else {
                        NumCast::from(<f32 as NumCast>::from(f[0]).unwrap() - steps[0] * s / w)
                            .unwrap()
                    }
                },
                NumCast::from(<f32 as NumCast>::from(f[1]).unwrap() - steps[1] * s / w).unwrap(),
                NumCast::from(<f32 as NumCast>::from(f[2]).unwrap() - steps[2] * s / w).unwrap(),
                NumCast::from(<f32 as NumCast>::from(f[3]).unwrap() - steps[3] * s / w).unwrap(),
            ];

            *pixel = *P::from_slice(channels.as_slice());
        }

        if flip.0 {
            image_buff = image::imageops::flip_horizontal(&image_buff);
        }
        if flip.1 {
            image_buff = image::imageops::flip_vertical(&image_buff);
        }
        image_buff
    }
}

pub fn linear_gradient_more_points<P>(
    size: (u32, u32),
    colors: Vec<(P, f32)>,
    angle: f32,
) -> ImageBuffer<P, Vec<P::Subpixel>>
where
    P: Pixel + 'static,
    P::Subpixel: Scale + Debug,
{
    let (width, height) = size;

    let mut p = vec![];
    let mut v = vec![vec![], vec![], vec![], vec![]];

    for (pixel, percent) in colors {
        let cmps = pixel.channels();
        v[0].push(cmps[0]);
        v[1].push(cmps[1]);
        v[2].push(cmps[2]);
        v[3].push(cmps[3]);
        p.push(percent);
    }

    let angle = angle % 360.0;

    let flip = {
        if 0.0 < angle && angle <= 90.0 {
            (false, false)
        } else if 90.0 < angle && angle <= 180.0 {
            (true, false)
        } else if 180.0 < angle && angle <= 270.0 {
            (true, true)
        } else {
            (false, true)
        }
    };

    let angle = angle % 90.0;

    if angle == 0.0 {
        let row = vec![0f32; (width as usize) * 4]
            .into_iter()
            .enumerate()
            .map(|(i, _)| {
                get_color_at(
                    &v[i % 4],
                    &p,
                    width as f32,
                    i as f32 / 4.0,
                    if i % 4 == 0 {
                        P::COLOR_MODEL == "HSL" || P::COLOR_MODEL == "HSLA"
                    } else {
                        false
                    },
                )
                .unwrap()
            })
            .collect::<Vec<P::Subpixel>>();

        let mut img: Vec<P::Subpixel> = Vec::new();
        for _ in 0..height {
            img.append(&mut row.clone())
        }
        let mut image_buff =
            ImageBuffer::<P, Vec<P::Subpixel>>::from_vec(width, height, img).unwrap();

        if flip.0 {
            image_buff = image::imageops::flip_horizontal(&image_buff);
        }
        image_buff
    } else {
        let mut image_buff = ImageBuffer::<P, Vec<P::Subpixel>>::new(width, height);
        let r = [-angle.to_radians().tan(), 1.0];

        let a = -1.0 / r[0];
        let b = 1f32;
        let c = -a * width as f32 - height as f32;

        let w = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

        for (x, y, pixel) in image_buff.enumerate_pixels_mut() {
            let a = -1.0 / r[0];
            let b = 1f32;
            let c = -a * x as f32 - height as f32 + y as f32;

            let s = ((r[1] * c) / (r[0] * b - a * r[1])) / angle.to_radians().cos();

            let channels = [
                get_color_at(
                    &v[0],
                    &p,
                    w,
                    s,
                    P::COLOR_MODEL == "HSL" || P::COLOR_MODEL == "HSLA",
                )
                .unwrap(),
                get_color_at(&v[1], &p, w, s, false).unwrap(),
                get_color_at(&v[2], &p, w, s, false).unwrap(),
                get_color_at(&v[3], &p, w, s, false).unwrap(),
            ];

            *pixel = *P::from_slice(channels.as_slice());
        }

        if flip.0 {
            image_buff = image::imageops::flip_horizontal(&image_buff);
        }
        if flip.1 {
            image_buff = image::imageops::flip_vertical(&image_buff);
        }
        image_buff
    }
}

pub fn bezier_cubic_curve(points: [[f32; 2]; 4], steps: Option<usize>) -> Vec<[f32; 2]> {
    fn decasteljau(t: f32, c: &(f32, f32, f32, f32)) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        return c.0 * mt3 + 3.0 * c.1 * mt2 * t + 3.0 * c.2 * mt * t2 + c.3 * t3;
    }

    let x = (points[0][0], points[1][0], points[2][0], points[3][0]);
    let y = (points[0][1], points[1][1], points[2][1], points[3][1]);

    let mut out = Vec::new();

    let steps =
        steps.unwrap_or_else(|| (curve_lenght_approximation(&points) / 1.5).ceil() as usize);

    for k in 0..steps {
        out.push([
            decasteljau(k as f32 / steps as f32, &x),
            decasteljau(k as f32 / steps as f32, &y),
        ]);
    }

    out
}

fn curve_lenght_approximation(points: &[[f32; 2]; 4]) -> f32 {
    fn CubicN(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        return a
            + (-a * 3.0 + t * (3.0 * a - a * t)) * t
            + (3.0 * b + t * (-6.0 * b + b * 3.0 * t)) * t
            + (c * 3.0 - c * 3.0 * t) * t2
            + d * t3;
    }

    let x: Vec<f32> = points.into_iter().map(|p| p[0]).collect();
    let y: Vec<f32> = points.into_iter().map(|p| p[1]).collect();

    let steps: usize = 1000000;

    let mut polyx = Vec::new();
    let mut polyy = Vec::new();

    for k in 0..=steps {
        polyx.push(CubicN(k as f32 / steps as f32, x[0], x[1], x[2], x[3]));
        polyy.push(CubicN(k as f32 / steps as f32, y[0], y[1], y[2], y[3]));
    }

    (0..steps - 1)
        .into_par_iter()
        .map(|k| {
            let (x, y) = (polyx[k], polyy[k]);
            let (x1, y1) = (polyx[k + 1], polyy[k + 1]);

            ((x1 - x).powi(2) + (y1 - y).powi(2)).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::math::cumulative_distribution;

    #[test]
    fn cdf_test() {
        let buf: Vec<u8> = vec![
            52, 55, 61, 59, 79, 61, 76, 61, 62, 59, 55, 104, 94, 85, 59, 71, 63, 65, 66, 113, 144,
            104, 63, 72, 64, 70, 70, 126, 154, 109, 71, 69, 67, 73, 68, 106, 122, 88, 68, 68, 68,
            79, 60, 70, 77, 66, 58, 75, 69, 85, 64, 58, 55, 61, 65, 83, 70, 87, 69, 68, 65, 73, 78,
            90,
        ];

        let mut histogram: HashMap<u8, u32> = HashMap::new();
        for i in 0u8..=255u8 {
            histogram.insert(i, 0);
        }

        for value in buf {
            *histogram
                .get_mut(&value)
                .expect("Unexpected error regarding the histogram Hashmap") += 1;
        }

        let output = cumulative_distribution(&histogram);
        let mut expected_output: HashMap<u8, u32> = HashMap::new();
        for i in 0u8..=255u8 {
            expected_output.insert(i, 0);
        }

        expected_output.insert(52, 1);
        expected_output.insert(55, 4);
        expected_output.insert(58, 6);
        expected_output.insert(59, 9);
        expected_output.insert(60, 10);
        expected_output.insert(61, 14);
        expected_output.insert(62, 15);
        expected_output.insert(63, 17);
        expected_output.insert(64, 19);
        expected_output.insert(65, 22);
        expected_output.insert(66, 24);
        expected_output.insert(67, 25);
        expected_output.insert(68, 30);
        expected_output.insert(69, 33);
        expected_output.insert(70, 37);
        expected_output.insert(71, 39);
        expected_output.insert(72, 40);
        expected_output.insert(73, 42);
        expected_output.insert(75, 43);
        expected_output.insert(76, 44);
        expected_output.insert(77, 45);
        expected_output.insert(78, 46);
        expected_output.insert(79, 48);
        expected_output.insert(83, 49);
        expected_output.insert(85, 51);
        expected_output.insert(87, 52);
        expected_output.insert(88, 53);
        expected_output.insert(90, 54);
        expected_output.insert(94, 55);
        expected_output.insert(104, 57);
        expected_output.insert(106, 58);
        expected_output.insert(109, 59);
        expected_output.insert(113, 60);
        expected_output.insert(122, 61);
        expected_output.insert(126, 62);
        expected_output.insert(144, 63);
        expected_output.insert(154, 64);

        for i in 0..=255u8 {
            match expected_output.get(&i) {
                Some(0) => continue,
                Some(v) => assert_eq!(Some(v), output.get(&i)),
                None => assert_eq!(None, output.get(&i)),
            }
        }
    }
}
