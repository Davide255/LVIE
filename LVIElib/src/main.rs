#![allow(non_snake_case)]

use std::fmt::Debug;

use image::{Pixel, Rgb};
use LVIElib::traits::{Scale, ToHsl};

#[derive(Debug)]
struct Prova<P>
where
    P: Pixel + ToHsl + Debug + Sized,
{
    pixel: P,
}

impl<P: Pixel + ToHsl + Debug + Sized> Prova<P> {
    pub fn cast(&self) -> P {
        let hsl = self.pixel.to_hsl().to_rgb();
        unsafe {
            let cmp = hsl.channels();
            std::mem::transmute_copy::<Rgb<u8>, P>(&Rgb([
                cmp[0].scale(),
                cmp[1].scale(),
                cmp[2].scale(),
            ]))
        }
    }
}

fn main() {
    println!(
        "{:?}",
        Prova {
            pixel: Rgb::<u8>([54, 33, 78])
        }
        .cast()
    );
}
