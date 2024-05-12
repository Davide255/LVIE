#![allow(non_snake_case)]
use std::{cmp::{max, min}, path::Path};
use num_traits::NumCast;
use image::Primitive;
use rawloader::decode_file;

fn mean<T: Primitive>(a: T, b:T) -> T { (a + b) / NumCast::from(2).unwrap() }
fn amean<T: Primitive>(a: [T; 2]) -> T { (a[0] + a[1]) / NumCast::from(2).unwrap() }

fn median<T: Primitive + Ord>(a: T, b: T, c: T, d: T) -> [T; 2] {
    [min(max(a, b), max(c, d)), max(min(a,b), min(c, d))]
}

fn bayer_to_rgb(img: &Vec<u16>, width: usize, height: usize, cfa: rawloader::CFA) -> Vec<u16> {
    let mut out = Vec::new();

    for y in 1..height-1 {
        for x in 1..width-1 {
            match cfa.color_at(x, y) {
                0 => {
                    out.push(img[x+y*width]);
                    out.push(amean(
                        median(
                            img[x + (y - 1)*width], 
                            img[x + (y + 1)*width],
                            img[x + 1 + y*width],
                            img[x - 1 + y*width]
                        )));
                    out.push(amean(
                        median(
                            img[x-1 + (y-1)*width], 
                            img[x-1 + (y+1)*width], 
                            img[x+1 + (y-1)*width], 
                            img[x+1 + (y+1)*width]
                        )
                    ));
                },
                1 => {
                    match cfa.color_at(x+1, y) {
                        0 => {
                            out.push(mean(
                                img[x-1 + y*width], 
                                img[x+1 + y*width]
                            ));
                            out.push(img[x + y*width]);
                            out.push(mean(
                                img[x + (y-1)*width], 
                                img[x + (y+1)*width]
                            ));
                        },
                        2 => {
                            out.push(mean(
                                img[x + (y-1)*width], 
                                img[x + (y+1)*width]
                            ));
                            out.push(img[x + y*width]);
                            out.push(mean(
                                img[x-1 + y*width], 
                                img[x+1 + y*width]
                            ));
                        },
                        _ => unreachable!()
                    }
                },
                2 => {
                    out.push(amean(
                        median(
                            img[x-1 + (y-1)*width], 
                            img[x-1 + (y+1)*width], 
                            img[x+1 + (y-1)*width], 
                            img[x+1 + (y+1)*width]
                        )
                    ));
                    out.push(amean(
                        median(
                            img[x + (y - 1)*width], 
                            img[x + (y + 1)*width],
                            img[x + 1 + y*width],
                            img[x - 1 + y*width]
                        )));
                    out.push(img[x+y*width]);
                },
                _ => unreachable!()
            }
        }
    }

    out
}

pub fn decode<P: AsRef<Path>>(path: P) -> Option<image::ImageBuffer<image::Rgba<u16>, Vec<u16>>> {
    
    let f = decode_file(path).unwrap();

    let wb = f.neutralwb();

    if let rawloader::RawImageData::Integer(data) = f.data {

        let data = bayer_to_rgb(&data, f.width, f.height, f.cfa);
        let mut nb: Vec<u16> = Vec::new();

        let factor = 255.0 / (4.0 * ((*data.iter().max().unwrap() as usize * 255) / u16::MAX as usize) as f32);

        for p in 0..(f.width-2)*(f.height-2) {
            nb.push(((data[3*p] as f32 / f.whitelevels[0] as f32) * wb[0] * factor * u16::MAX as f32).round() as u16);
            nb.push(((data[3*p +1] as f32 / f.whitelevels[1] as f32) * wb[1] * factor * u16::MAX as f32).round() as u16);
            nb.push(((data[3*p +2] as f32 / f.whitelevels[2] as f32) * wb[2] * factor * u16::MAX as f32).round() as u16);
            nb.push(u16::MAX);
        }

        let img: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> = image::ImageBuffer::from_vec(f.width as u32 -2, f.height as u32 -2, nb).unwrap();
        return Some(img);
    }
    None
}

pub fn supported_formats() -> Vec<&'static str> {
    return vec![
        "MRW","ARW","SRF","SR2","MEF","ORF","ARI","MOS",
        "SRW","ERF","KDC","DCS","RW2","RAF","DCR","CR2",
        "DNG","PEF","CRW","IIQ","3FR","NRW","NEF"
    ];
}