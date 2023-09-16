#![allow(dead_code)]

use image::{ImageBuffer, Luma, LumaA, Pixel, Rgb, Rgba};
use num_traits::{Bounded, Num, NumCast, Zero};
use std::ops::AddAssign;

pub trait Enlargeable: Sized + Bounded + NumCast {
    type Larger: Copy + NumCast + Num + PartialOrd<Self::Larger> + Clone + Bounded + AddAssign;

    fn clamp_from(n: Self::Larger) -> Self {
        if n > Self::max_value().to_larger() {
            Self::max_value()
        } else if n < Self::min_value().to_larger() {
            Self::min_value()
        } else {
            NumCast::from(n).unwrap()
        }
    }

    fn to_larger(self) -> Self::Larger {
        NumCast::from(self).unwrap()
    }
}

impl Enlargeable for u8 {
    type Larger = u32;
}
impl Enlargeable for u16 {
    type Larger = u32;
}
impl Enlargeable for u32 {
    type Larger = u64;
}
impl Enlargeable for u64 {
    type Larger = u128;
}
impl Enlargeable for usize {
    // Note: On 32-bit architectures, u64 should be enough here.
    type Larger = u128;
}
impl Enlargeable for i8 {
    type Larger = i32;
}
impl Enlargeable for i16 {
    type Larger = i32;
}
impl Enlargeable for i32 {
    type Larger = i64;
}
impl Enlargeable for i64 {
    type Larger = i128;
}
impl Enlargeable for isize {
    // Note: On 32-bit architectures, i64 should be enough here.
    type Larger = i128;
}
impl Enlargeable for f32 {
    type Larger = f64;
}
impl Enlargeable for f64 {
    type Larger = f64;
}

#[derive(PartialEq, Clone, Debug, Copy)]
#[repr(C)]
#[allow(missing_docs)]
pub struct Hsl(pub [f32; 3]);

impl Pixel for Hsl {
    type Subpixel = f32;

    const CHANNEL_COUNT: u8 = 3;

    #[inline(always)]
    fn channels(&self) -> &[f32] {
        &self.0
    }

    #[inline(always)]
    fn channels_mut(&mut self) -> &mut [f32] {
        &mut self.0
    }

    const COLOR_MODEL: &'static str = "HSL";

    fn channels4(&self) -> (f32, f32, f32, f32) {
        const CHANNELS: usize = 3;
        let mut channels = [f32::MAX; 4];
        channels[0..CHANNELS].copy_from_slice(&self.0);
        (channels[0], channels[1], channels[2], channels[3])
    }

    fn from_channels(a: f32, b: f32, c: f32, d: f32) -> Hsl {
        const CHANNELS: usize = 3;
        *<Hsl as Pixel>::from_slice(&[a, b, c, d][..CHANNELS])
    }

    fn from_slice(slice: &[f32]) -> &Hsl {
        assert_eq!(slice.len(), 3);
        unsafe { &*(slice.as_ptr() as *const Hsl) }
    }
    fn from_slice_mut(slice: &mut [f32]) -> &mut Hsl {
        assert_eq!(slice.len(), 3);
        unsafe { &mut *(slice.as_mut_ptr() as *mut Hsl) }
    }

    fn to_rgb(&self) -> Rgb<f32> {
        <Self as Into<Rgb<f32>>>::into(*self)
    }

    fn to_rgba(&self) -> Rgba<f32> {
        self.to_rgb().to_rgba()
    }

    fn to_luma(&self) -> Luma<f32> {
        Luma([self.0[3]])
    }

    fn to_luma_alpha(&self) -> LumaA<f32> {
        LumaA([self.0[2], 1.0])
    }

    fn map<F>(&self, f: F) -> Hsl
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
        for v in &mut self.0 {
            *v = f(*v)
        }
    }

    fn map_with_alpha<F, G>(&self, f: F, g: G) -> Hsl
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
        for v in self.0[..ALPHA].iter_mut() {
            *v = f(*v)
        }
        // f32he branch of this match is `const`. f32his way ensures that no subexpression fails the
        // `const_err` lint (the expression `self.0[ALPHA]` would).
        if let Some(v) = self.0.get_mut(ALPHA) {
            *v = g(*v)
        }
    }

    fn map2<F>(&self, other: &Self, f: F) -> Hsl
    where
        F: FnMut(f32, f32) -> f32,
    {
        let mut this = (*self).clone();
        this.apply2(other, f);
        this
    }

    fn apply2<F>(&mut self, other: &Hsl, mut f: F)
    where
        F: FnMut(f32, f32) -> f32,
    {
        for (a, &b) in self.0.iter_mut().zip(other.0.iter()) {
            *a = f(*a, b)
        }
    }

    fn invert(&mut self) {
        self.0[2] = (self.0[2] + 180.0) % 360.0;
    }

    fn blend(&mut self, other: &Hsl) {
        //Blend::blend(self, other)
    }
}

fn rgb8_to_hslf32(r: u8, g: u8, b: u8) -> Hsl {
    rgbf32_to_hslf32(r as f32, g as f32, b as f32)
}
fn rgb16_to_hslf32(r: u16, g: u16, b: u16) -> Hsl {
    rgbf32_to_hslf32(r as f32, g as f32, b as f32)
}

fn _max(c: [f32; 3]) -> (f32, u8) {
    if c[0] > c[1] && c[0] > c[2] {
        (c[0], 0)
    } else if c[1] > c[0] && c[1] > c[2] {
        (c[1], 1)
    } else if c[2] > c[0] && c[2] > c[1] {
        (c[2], 2)
    } else {
        (c[0], 0)
    }
}

fn _min(c: [f32; 3]) -> (f32, u8) {
    if c[0] < c[1] && c[0] < c[2] {
        (c[0], 0)
    } else if c[1] < c[0] && c[1] < c[2] {
        (c[1], 1)
    } else if c[2] < c[0] && c[2] < c[1] {
        (c[2], 2)
    } else {
        (c[0], 0)
    }
}

pub fn rgbf32_to_hslf32(r: f32, g: f32, b: f32) -> Hsl {
    let mut cmp: [f32; 3] = [Zero::zero(), Zero::zero(), Zero::zero()];

    let c: [f32; 3] = [r, g, b];
    let (cmax, cmaxindex) = _max(c);
    let (cmin, _) = _min(c);

    cmp[2] = (cmax + cmin) / 2f32;

    let delta = cmax - cmin;

    if delta != 0f32 {
        cmp[1] = -delta / (1f32 - ((2f32 * cmp[2]) - 1f32).abs()) * 100f32;

        if cmaxindex == 0 {
            cmp[0] = ((g - b) / delta) % 6f32;
        } else if cmaxindex == 1 {
            cmp[0] = ((b - r) / delta) + 2f32;
        } else if cmaxindex == 2 {
            cmp[0] = ((r - g) / delta) + 4f32;
        }

        cmp[0] = (cmp[0] * 60f32).round();
    }

    if cmp[0] < 0.0 {
        let m = cmp[0] % 360f32;
        if m != 0.0 {
            cmp[0] = m + 360f32
        }
    }

    cmp[2] = cmp[2] * (100f32 / 255f32);

    Hsl(cmp)
}

pub fn hslf32_to_rgb8(h: f32, s: f32, l: f32) -> Rgb<u8> {
    let c = hslf32_to_rgbf32(h, s, l).0;
    Rgb::<u8>([
        (c[0] * 255f32).round() as u8,
        (c[1] * 255f32).round() as u8,
        (c[2] * 255f32).round() as u8,
    ])
}
pub fn hslf32_to_rgb16(h: f32, s: f32, l: f32) -> Rgb<u16> {
    let c = hslf32_to_rgbf32(h, s, l).0;
    Rgb::<u16>([
        (c[0] * u16::MAX as f32).round() as u16,
        (c[1] * u16::MAX as f32).round() as u16,
        (c[2] * u16::MAX as f32).round() as u16,
    ])
}

pub fn hslf32_to_rgbf32(h: f32, s: f32, l: f32) -> Rgb<f32> {
    let c = (s / 100f32) * (1f32 - (2f32 * l / 100f32 - 1f32).abs());
    let x = c * (1f32 - ((h / 60f32) % 2f32 - 1f32).abs());
    let m = (l / 100f32) - c / 2f32;

    let mut rgb: [f32; 3] = [0f32; 3];

    if 0.0 <= h && h < 60f32 {
        rgb = [c, x, Zero::zero()];
    } else if 60.0 <= h && h < 120.0 {
        rgb = [x, c, Zero::zero()];
    } else if 120.0 <= h && h < 180.0 {
        rgb = [Zero::zero(), c, x];
    } else if 180.0 <= h && h < 240.0 {
        rgb = [Zero::zero(), x, c];
    } else if 240.0 <= h && h < 300.0 {
        rgb = [x, Zero::zero(), c];
    } else if 300.0 <= h && h < 360.0 {
        rgb = [c, Zero::zero(), x];
    } else {
        panic!("Hue is out of range!")
    }
    Rgb::<f32>([(rgb[0] + m), (rgb[1] + m), (rgb[2] + m)])
}

impl From<Hsl> for Rgb<u8> {
    fn from(value: Hsl) -> Rgb<u8> {
        let channels = value.channels();
        hslf32_to_rgb8(channels[0], channels[1], channels[2])
    }
}

impl From<Hsl> for Rgb<u16> {
    fn from(value: Hsl) -> Rgb<u16> {
        let channels = value.channels();
        hslf32_to_rgb16(channels[0], channels[1], channels[2])
    }
}

impl From<Hsl> for Rgb<f32> {
    fn from(value: Hsl) -> Rgb<f32> {
        let channels = value.channels();
        hslf32_to_rgbf32(channels[0], channels[1], channels[2])
    }
}

impl From<Rgb<u8>> for Hsl {
    fn from(rgb: Rgb<u8>) -> Hsl {
        let channels = rgb.channels();
        rgb8_to_hslf32(channels[0], channels[1], channels[2])
    }
}

impl From<Rgb<u16>> for Hsl {
    fn from(rgb: Rgb<u16>) -> Hsl {
        let channels = rgb.channels();
        rgb16_to_hslf32(channels[0], channels[1], channels[2])
    }
}

impl From<Rgb<f32>> for Hsl {
    fn from(rgb: Rgb<f32>) -> Hsl {
        let channels = rgb.channels();
        rgbf32_to_hslf32(channels[0], channels[1], channels[2])
    }
}

pub type HslImage = ImageBuffer<Hsl, Vec<f32>>;
