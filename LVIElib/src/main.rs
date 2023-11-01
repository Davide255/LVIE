#![allow(non_snake_case)]

use image::{Pixel, Rgb};
use LVIElib::{linear_srgb::LinSrgb, oklab::Oklab};

fn main() {
    let color1: Rgb<f32> = Rgb::<f32>([0.0, 0.50588, 0.98039]);

    let dlinsrgb = LinSrgb::from(color1);

    println!("{:?}", dlinsrgb.channels());

    let doklab = Oklab::from(dlinsrgb);

    println!("{:?}", doklab.channels());
}
