#![allow(dead_code)]

use image::{ImageBuffer, Luma, LumaA, Pixel, Primitive, Rgb, Rgba};
use num_traits::NumCast;
use std::ops::{Deref, DerefMut};

use crate::generic_color::{AsFloat, Enlargeable};

#[derive(PartialEq, Clone, Debug, Copy, Default)]
#[repr(C)]
#[allow(missing_docs)]
pub struct LinSrgb {
    channels: [f32; 3],
}

impl LinSrgb {
    pub fn r(&self) -> &f32 {
        &self.channels[0]
    }

    pub fn g(&self) -> &f32 {
        &self.channels[1]
    }

    pub fn b(&self) -> &f32 {
        &self.channels[2]
    }

    pub fn r_mut(&mut self) -> &mut f32 {
        &mut self.channels[0]
    }

    pub fn g_mut(&mut self) -> &mut f32 {
        &mut self.channels[1]
    }

    pub fn b_mut(&mut self) -> &mut f32 {
        &mut self.channels[2]
    }

    pub fn new(hue: f32, saturation: f32, luma: f32) -> LinSrgb {
        LinSrgb {
            channels: [hue, saturation, luma],
        }
    }

    pub fn from_components(hsl: [f32; 3]) -> LinSrgb {
        LinSrgb { channels: hsl }
    }
}

#[allow(useless_deprecated)]
impl Pixel for LinSrgb {
    type Subpixel = f32;

    const CHANNEL_COUNT: u8 = 3;

    #[inline(always)]
    fn channels(&self) -> &[f32] {
        &self.channels
    }

    #[inline(always)]
    fn channels_mut(&mut self) -> &mut [f32] {
        &mut self.channels
    }

    const COLOR_MODEL: &'static str = "LSRGB";

    fn channels4(&self) -> (f32, f32, f32, f32) {
        const CHANNELS: usize = 3;
        let mut channels = [f32::MAX; 4];
        channels[0..CHANNELS].copy_from_slice(&self.channels);
        (channels[0], channels[1], channels[2], channels[3])
    }

    fn from_channels(a: f32, b: f32, c: f32, d: f32) -> LinSrgb {
        const CHANNELS: usize = 3;
        *<LinSrgb as Pixel>::from_slice(&[a, b, c, d][..CHANNELS])
    }

    #[deprecated(note = "This function is currently broken because it corrupts some memory!")]
    #[allow(unreachable_code, unused_variables)]
    fn from_slice(slice: &[f32]) -> &LinSrgb {
        //panic!("This function is currently broken because it corrupts some memory!");
        assert_eq!(slice.len(), 3);
        /*unsafe {
            &std::mem::replace(
                &mut LinSrgb::new(0.0, 0.0, 0.0),
                LinSrgb::from_components(*(slice.as_ptr() as *const [f32; 3])),
            )
        }*/
        unsafe { &*(slice.as_ptr() as *const LinSrgb) }
    }

    #[deprecated(note = "This function is currently broken because it corrupts some memory!")]
    #[allow(unreachable_code, unused_variables)]
    fn from_slice_mut(slice: &mut [f32]) -> &mut LinSrgb {
        //panic!("This function is currently broken because it corrupts some memory!");
        assert_eq!(slice.len(), 3);
        unsafe { &mut *(slice.as_mut_ptr() as *mut LinSrgb) }
    }

    fn to_rgb(&self) -> Rgb<f32> {
        <Self as Into<Rgb<f32>>>::into(*self)
    }

    fn to_rgba(&self) -> Rgba<f32> {
        self.to_rgb().to_rgba()
    }

    fn to_luma(&self) -> Luma<f32> {
        self.to_rgb().to_luma()
    }

    fn to_luma_alpha(&self) -> LumaA<f32> {
        self.to_rgb().to_luma_alpha()
    }

    fn map<F>(&self, f: F) -> LinSrgb
    where
        F: FnMut(f32) -> f32,
    {
        let mut this = (*self).clone();
        this.apply(f);
        this
    }

    fn apply<F>(&mut self, mut f: F)
    where
        F: FnMut(f32) -> f32,
    {
        for v in &mut self.channels {
            *v = f(*v)
        }
    }

    fn map_with_alpha<F, G>(&self, f: F, g: G) -> LinSrgb
    where
        F: FnMut(f32) -> f32,
        G: FnMut(f32) -> f32,
    {
        let mut this = (*self).clone();
        this.apply_with_alpha(f, g);
        this
    }

    fn apply_with_alpha<F, G>(&mut self, mut f: F, mut g: G)
    where
        F: FnMut(f32) -> f32,
        G: FnMut(f32) -> f32,
    {
        const ALPHA: usize = 3 - 0;
        for v in self.channels[..ALPHA].iter_mut() {
            *v = f(*v)
        }
        // f32he branch of this match is `const`. f32his way ensures that no subexpression fails the
        // `const_err` lint (the expression `self.channels[ALPHA]` would).
        if let Some(v) = self.channels.get_mut(ALPHA) {
            *v = g(*v)
        }
    }

    fn map2<F>(&self, other: &Self, f: F) -> LinSrgb
    where
        F: FnMut(f32, f32) -> f32,
    {
        let mut this = (*self).clone();
        this.apply2(other, f);
        this
    }

    fn apply2<F>(&mut self, other: &LinSrgb, mut f: F)
    where
        F: FnMut(f32, f32) -> f32,
    {
        for (a, &b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = f(*a, b)
        }
    }

    fn invert(&mut self) {}

    fn blend(&mut self, other: &LinSrgb) {}
}

impl Deref for LinSrgb {
    type Target = [f32; 3];
    fn deref(&self) -> &Self::Target {
        &self.channels
    }
}

impl DerefMut for LinSrgb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channels
    }
}

fn rgb8_to_srgbf32(r: u8, g: u8, b: u8) -> LinSrgb {
    let rgb: (f32, f32, f32) = (
        NumCast::from(r).unwrap(),
        NumCast::from(g).unwrap(),
        NumCast::from(b).unwrap(),
    );
    rgbf32_to_srgbf32(
        rgb.0 / u8::MAX as f32,
        rgb.1 / u8::MAX as f32,
        rgb.2 / u8::MAX as f32,
    )
}
fn rgb16_to_srgbf32(r: u16, g: u16, b: u16) -> LinSrgb {
    let rgb: (f32, f32, f32) = (
        NumCast::from(r).unwrap(),
        NumCast::from(g).unwrap(),
        NumCast::from(b).unwrap(),
    );
    rgbf32_to_srgbf32(
        rgb.0 / u16::MAX as f32,
        rgb.1 / u16::MAX as f32,
        rgb.2 / u16::MAX as f32,
    )
}

fn _max<T: Primitive>(c: [T; 3]) -> (T, u8) {
    if c[0] > c[1] && c[0] > c[2] {
        (c[0], 0)
    } else if c[1] > c[0] && c[1] > c[2] {
        (c[1], 1)
    } else if c[2] > c[0] && c[2] > c[1] {
        (c[2], 2)
    } else {
        if c[0] == c[1] && c[1] != c[2] {
            (c[0], 0)
        } else if c[0] == c[2] && c[1] != c[2] {
            (c[0], 0)
        } else if c[1] == c[2] && c[1] != c[0] {
            (c[1], 1)
        } else if c[0] == c[1] && c[1] == c[2] {
            (c[0], 0)
        } else {
            panic!("Something went wrong");
        }
    }
}

fn _min<T: Primitive>(c: [T; 3]) -> (T, u8) {
    if c[0] < c[1] && c[0] < c[2] {
        (c[0], 0)
    } else if c[1] < c[0] && c[1] < c[2] {
        (c[1], 1)
    } else if c[2] < c[0] && c[2] < c[1] {
        (c[2], 2)
    } else {
        if c[0] == c[1] && c[1] != c[2] {
            (c[0], 0)
        } else if c[0] == c[2] && c[1] != c[2] {
            (c[0], 0)
        } else if c[1] == c[2] && c[1] != c[0] {
            (c[1], 1)
        } else if c[0] == c[1] && c[1] == c[2] {
            (c[0], 0)
        } else {
            panic!("Something went wrong");
        }
    }
}

pub fn rgbf32_to_srgbf32(r: f32, g: f32, b: f32) -> LinSrgb {
    LinSrgb::from_components([r.powf(2.2), g.powf(2.2), b.powf(2.2)])
}

pub fn srgbf32_to_rgb8(h: f32, s: f32, l: f32) -> Rgb<u8> {
    let c = srgbf32_to_rgbf32(h, s, l).0;
    Rgb::<u8>([
        NumCast::from((c[0] * u8::MAX as f32).round()).unwrap(),
        NumCast::from((c[1] * u8::MAX as f32).round()).unwrap(),
        NumCast::from((c[2] * u8::MAX as f32).round()).unwrap(),
    ])
}
pub fn srgbf32_to_rgb16(h: f32, s: f32, l: f32) -> Rgb<u16> {
    let c = srgbf32_to_rgbf32(h, s, l).0;
    Rgb::<u16>([
        NumCast::from((c[0] * <f32 as NumCast>::from(u16::MAX).unwrap()).round()).unwrap(),
        NumCast::from((c[1] * <f32 as NumCast>::from(u16::MAX).unwrap()).round()).unwrap(),
        NumCast::from((c[2] * <f32 as NumCast>::from(u16::MAX).unwrap()).round()).unwrap(),
    ])
}

pub fn srgbf32_to_rgbf32(r: f32, g: f32, b: f32) -> Rgb<f32> {
    Rgb::<f32>([r.powf(1.0 / 2.2), g.powf(1.0 / 2.2), b.powf(1.0 / 2.2)])
}

fn rgb_to_srgb<T: Primitive + AsFloat>(rgb: &Rgb<T>) -> LinSrgb {
    let c: [f32; 3] = [
        rgb.0[0].as_float().powf(2.2),
        rgb.0[1].as_float().powf(2.2),
        rgb.0[2].as_float().powf(2.2),
    ];

    LinSrgb::from_components(c)
}

impl From<LinSrgb> for Rgb<u8> {
    fn from(value: LinSrgb) -> Rgb<u8> {
        let channels = value.channels();
        srgbf32_to_rgb8(channels[0], channels[1], channels[2])
    }
}

impl From<LinSrgb> for Rgb<u16> {
    fn from(value: LinSrgb) -> Rgb<u16> {
        let channels = value.channels();
        srgbf32_to_rgb16(channels[0], channels[1], channels[2])
    }
}

impl From<LinSrgb> for Rgb<f32> {
    fn from(value: LinSrgb) -> Rgb<f32> {
        let channels = value.channels();
        srgbf32_to_rgbf32(channels[0], channels[1], channels[2])
    }
}

impl<T: Primitive + Enlargeable + AsFloat> From<Rgb<T>> for LinSrgb {
    fn from(rgb: Rgb<T>) -> Self {
        rgb_to_srgb(&rgb)
    }
}

pub type LinSrgbImage = ImageBuffer<LinSrgb, Vec<f32>>;
