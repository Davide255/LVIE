#![allow(non_snake_case)]
//pub trait VectorMapping<T, O> {
//    fn mapv<F>(&self, mut f: F) -> Vec<O>
//    where
//        F: FnMut(T) -> O,
//        T: Primitive,
//        O: Primitive,
//        Self: IntoIterator<Item = T> + Clone,
//    {
//        let mut out: Vec<O> = Vec::new();
//
//        for x in (*self).clone().into_iter(){
//            out.push(f(x));
//        }
//
//        out
//    }
//}
//
//impl<T: Primitive, O: Primitive> VectorMapping<T, O> for Vec<T> {}

use std::cmp::{min, max};

use image::Primitive;
use rawloader::decode_file;


fn mean<T: Primitive>(a: T, b:T) -> T { (a + b) / num::cast(2).unwrap() }
fn amean<T: Primitive>(a: [T; 2]) -> T { (a[0] + a[1]) / num::cast(2).unwrap() }

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
                    let r = amean(
                        median(
                            img[x-1 + (y-1)*width], 
                            img[x-1 + (y+1)*width], 
                            img[x+1 + (y-1)*width], 
                            img[x+1 + (y+1)*width]
                        )
                    );
                    out.push(r);
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

fn main() {

    let f = decode_file("C:\\Users\\david\\Documents\\workspaces\\DSC09907.ARW").unwrap();

    println!("{} - {}, {}, {:?}", f.width, f.height, f.cpp, f.is_monochrome());

    if let rawloader::RawImageData::Integer(data) = f.data {
        let data = bayer_to_rgb(&data, f.width, f.height, f.cfa);

        let mut nb: Vec<u8> = Vec::new();

        for p in 0..(f.width-2)*(f.height-2) {
            nb.push(((data[3*p] as f32 / f.whitelevels[0] as f32) * 255.0).round() as u8);
            nb.push(((data[3*p +1] as f32 / f.whitelevels[1] as f32) * 255.0).round() as u8);
            nb.push(((data[3*p +2] as f32 / f.whitelevels[2] as f32) * 255.0).round() as u8);
        }

        for x in 0..10 {
            println!("[{}, {}, {}]", nb[3*x], nb[3*x +1], nb[3*x + 2]);
        }

        let img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_vec(f.width as u32 -2, f.height as u32 -2, nb).unwrap();

        img.save_with_format("prova.jpg", image::ImageFormat::Jpeg).expect("Failed to write image");
    }

}