use std::ops::AddAssign;

use image::Primitive;

pub fn channelimage2window<T: Primitive + std::fmt::Debug>(img: Vec<T>, w: u32, h: u32, wf: u32, hf: u32) -> Vec<T> {
    let mut out: Vec<T> = Vec::new(); //img[0..hf as usize].leto();

    for ch in 0..=(h-hf) {
        for y in 0..hf {
            out.push(img[((ch+y)*w) as usize]);
        }
        for cw in 0..=(w-wf) {
            for px in 1..wf {
                for py in 0..hf { 
                    out.push(img[((ch+py)*w+ cw+px) as usize]); 
                }
            }
        }
    }

    out
}

pub fn conv<T:Primitive + std::fmt::Debug + AddAssign>(img: Vec<T>, ind: (u32, u32, u32), filter: Vec<T>, filter_descriptor: (u32, u32, u32), stride: u32) -> Vec<T>{

    let mut out: Vec<T> = img.clone();

    let img = channelimage2window(img, ind.1, ind.2, stride, stride);
    let filter = channelimage2window(filter, filter_descriptor.1, filter_descriptor.2, stride, stride);

    let input_channel = ind.0;
    let input_height = ind.2;
    let input_width = ind.1;

    let filter_channel = filter_descriptor.0;
    let filter_height = filter_descriptor.2;
    let filter_width = filter_descriptor.1;

    let output_height = input_height;
    let output_width = input_width;    

    let stride_width = stride;

    let window_area = output_height * output_width;
    let filter_area = filter_height * filter_width;
    let filter_c_area = filter_area * filter_channel;
    let gap_width = stride_width * filter_height;
    let window_store = filter_height * input_width;
    let c_window_store = output_height * window_store;
    let sum_window_store = input_channel * c_window_store;
    let sum_filter_area = window_area;


    let bD = sum_window_store;
    let bY = sum_filter_area;
    for c in 0..input_channel {
        let cbD = bD + c * c_window_store;
        let cW = c * filter_area;

        let headW = filter_c_area + cW;
        let nbY = bY + window_area;
        for i in 0..output_height {
            let iD = i * window_store + cbD;
            let inbY = nbY + i * output_width;
            for j in 0..output_width{
                        let indexY = inbY + j;
                        let headD = iD + j * gap_width;
                        for w in 0..filter_width {
                            let wcD = w * filter_height + headD;
                            let wcW = w * filter_height + headW;
                            for h in 0..filter_height {
                                let indexD = wcD + h;
                                let indexW = wcW + h; 
                                out[indexY as usize] += img[indexD as usize] * filter[indexW as usize];
                            }
                        }
                    }
                }
        }

    out
}