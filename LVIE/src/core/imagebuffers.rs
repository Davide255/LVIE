use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use image::{Pixel, Primitive};
use LVIElib::{hsl::HslaImage, oklab::OklabaImage};
use LVIE_GPU::Pod;

use LVIElib::traits::*;

use rayon::prelude::*;

use LVIE_GPU::CRgbaImage;

#[derive(Debug, Clone)]
pub struct ImageBuffers<P>
where
    P: Pixel + Send + Sync + Debug + ToHsl + 'static,
    P::Subpixel: Scale + Primitive + Debug + Pod + Send + Sync + AsFloat,
{
    rgb: CRgbaImage<P>,
    hsl: HslaImage,
    oklab: OklabaImage,
    enbled: [bool; 3],
    updated: [bool; 3],
}

#[allow(dead_code)]
impl<P> ImageBuffers<P>
where
    P: Pixel + Send + Sync + Debug + ToHsl + 'static + ToOklab,
    P::Subpixel: Scale + Primitive + Debug + Pod + Send + Sync + AsFloat,
{
    pub fn new() -> ImageBuffers<P> {
        ImageBuffers {
            rgb: CRgbaImage::<P>::default(),
            hsl: HslaImage::default(),
            oklab: OklabaImage::default(),
            enbled: [false; 3],
            updated: [true; 3],
        }
    }

    pub fn from_rgb(img: CRgbaImage<P>) -> ImageBuffers<P> {
        ImageBuffers {
            rgb: img,
            hsl: HslaImage::default(),
            oklab: OklabaImage::default(),
            enbled: [true, false, false],
            updated: [true; 3],
        }
    }

    pub fn from_hsl(img: HslaImage) -> ImageBuffers<P> {
        ImageBuffers {
            rgb: CRgbaImage::<P>::default(),
            hsl: img,
            oklab: OklabaImage::default(),
            enbled: [false, true, false],
            updated: [true; 3],
        }
    }

    pub fn from_oklab(img: OklabaImage) -> ImageBuffers<P> {
        ImageBuffers {
            rgb: CRgbaImage::<P>::default(),
            hsl: HslaImage::default(),
            oklab: img,
            enbled: [false, false, true],
            updated: [true; 3],
        }
    }

    pub fn set_updates(&mut self, hsl: bool, oklab: bool) {
        self.enbled = [true, hsl, oklab];
    }

    pub fn set_updated(&mut self, rgb: bool, hsl: bool, oklab: bool) {
        self.updated = [rgb, hsl, oklab];
    }

    pub fn get_rgb(&self) -> &CRgbaImage<P> {
        &self.rgb
    }
    pub fn get_hsl(&self) -> &HslaImage {
        &self.hsl
    }
    pub fn get_oklab(&self) -> &OklabaImage {
        &self.oklab
    }

    pub fn get_rgb_updated(&mut self) -> &CRgbaImage<P> {
        self.update_rgb();
        &self.rgb
    }
    pub fn get_hsl_updated(&mut self) -> &HslaImage {
        self.update_hsl();
        &self.hsl
    }
    pub fn get_oklab_updated(&mut self) -> &OklabaImage {
        self.update_oklab();
        &self.oklab
    }

    pub fn get_rgb_mut_updated(&mut self) -> &mut CRgbaImage<P> {
        self.update_rgb();
        &mut self.rgb
    }

    pub fn get_hsl_mut_updated(&mut self) -> &mut HslaImage {
        self.update_hsl();
        &mut self.hsl
    }

    pub fn get_oklab_mut_updated(&mut self) -> &mut OklabaImage {
        self.update_oklab();
        &mut self.oklab
    }

    pub fn replace_rgb(&mut self, new_rgb: CRgbaImage<P>) {
        self.rgb = new_rgb;
        self.updated = [true, false, false];
    }

    pub fn replace_hsl(&mut self, new_hsl: HslaImage) {
        self.hsl = new_hsl;
        self.updated = [false, true, false];
    }

    pub fn replace_oklab(&mut self, new_oklab: OklabaImage) {
        self.oklab = new_oklab;
        self.updated = [false, false, true];
    }

    pub fn update(&mut self) {
        self.update_rgb();
        self.update_hsl();
        self.update_oklab();
    }

    pub fn update_rgb(&mut self) {
        if self.updated[0] {
            return;
        }

        if self.updated[1] {
            let s = std::time::Instant::now();
            self.rgb = unsafe { crate::core::processors::convert_hsla_to_rgba(&self.hsl).unwrap() };
            println!(
                "Conversion hsl -> rgb done in {}ms",
                s.elapsed().as_millis()
            );
            self.updated[0] = true;
        } else if self.updated[2] {
            let s = std::time::Instant::now();
            self.rgb =
                unsafe { crate::core::processors::convert_oklaba_to_rgba(&self.oklab).unwrap() };
            println!(
                "Conversion oklab -> rgb done in {}ms",
                s.elapsed().as_millis()
            );
            self.updated[0] = true;
        }
    }

    pub fn update_hsl(&mut self) {
        if self.updated[1] {
            return;
        }

        if self.updated[2] {
            self.update_rgb();
        }

        let s = std::time::Instant::now();
        self.hsl = HslaImage::from_vec(self.rgb.width(), self.rgb.height(), {
            let v = vec![0f32; (self.rgb.width() * self.rgb.height() * 4) as usize];
            let out = Arc::new(Mutex::new(v));

            let width = self.rgb.width();
            let out_w = out.clone();
            self.rgb.enumerate_rows().par_bridge().for_each(|(y, row)| {
                let mut r = Vec::<f32>::new();
                for (_, _, p) in row {
                    r.append(&mut p.to_hsla().channels().to_vec());
                }

                out_w.lock().unwrap()[(y * width * 4) as usize..((y + 1) * width * 4) as usize]
                    .copy_from_slice(&r);
            });
            drop(out_w);

            Arc::try_unwrap(out).unwrap().into_inner().unwrap()
        })
        .unwrap();
        println!(
            "Conversion rgb -> hsl done in {}ms",
            s.elapsed().as_millis()
        );
        self.updated[1] = true;
    }

    pub fn update_oklab(&mut self) {
        if self.updated[2] {
            return;
        }

        if self.updated[1] {
            self.update_rgb();
        }

        let s = std::time::Instant::now();
        self.oklab = OklabaImage::from_vec(self.rgb.width(), self.rgb.height(), {
            let out = Arc::new(Mutex::new(vec![
                0f32;
                (self.rgb.width() * self.rgb.height() * 4)
                    as usize
            ]));

            let width = self.rgb.width();
            let out_w = out.clone();
            self.rgb.enumerate_rows().par_bridge().for_each(|(y, row)| {
                let mut r = Vec::<f32>::new();
                for (_, _, p) in row {
                    r.append(&mut p.to_oklaba().channels().to_vec());
                }

                out_w.lock().unwrap()[(y * width * 4) as usize..((y + 1) * width * 4) as usize]
                    .copy_from_slice(&r);
            });
            drop(out_w);

            Arc::try_unwrap(out).unwrap().into_inner().unwrap()
        })
        .unwrap();
        println!(
            "Conversion rgb -> oklab done in {}ms",
            s.elapsed().as_millis()
        );
        self.updated[2] = true;
    }

    pub fn reset(&mut self) {
        self.rgb = CRgbaImage::<P>::default();
        self.hsl = HslaImage::default();
        self.oklab = OklabaImage::default();
        self.updated = [true; 3];
    }
}
