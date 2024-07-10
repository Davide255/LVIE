#![allow(dead_code)]

use image::{ImageBuffer, Luma, LumaA, Pixel, Primitive, Rgb, Rgba};
use std::ops::{Deref, DerefMut};

use crate::traits::AsFloat;

use crate::linear_srgb::{LinSrgb, LinSrgba};
use crate::matrix::Matrix;

#[derive(PartialEq, Clone, Debug, Copy, Default)]
#[repr(C)]
#[allow(missing_docs)]
pub struct Oklab {
    channels: [f32; 3],
}

impl Oklab {
    pub fn l(&self) -> &f32 {
        &self.channels[0]
    }

    pub fn a(&self) -> &f32 {
        &self.channels[1]
    }

    pub fn b(&self) -> &f32 {
        &self.channels[2]
    }

    pub fn l_mut(&mut self) -> &mut f32 {
        &mut self.channels[0]
    }

    pub fn a_mut(&mut self) -> &mut f32 {
        &mut self.channels[1]
    }

    pub fn b_mut(&mut self) -> &mut f32 {
        &mut self.channels[2]
    }

    pub fn new(l: f32, a: f32, b: f32) -> Oklab {
        Oklab {
            channels: [l, a, b],
        }
    }

    pub fn from_components(oklab: [f32; 3]) -> Oklab {
        Oklab { channels: oklab }
    }
}

#[allow(useless_deprecated)]
impl Pixel for Oklab {
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

    const COLOR_MODEL: &'static str = "OKLAB";

    fn channels4(&self) -> (f32, f32, f32, f32) {
        const CHANNELS: usize = 3;
        let mut channels = [f32::MAX; 4];
        channels[0..CHANNELS].copy_from_slice(&self.channels);
        (channels[0], channels[1], channels[2], channels[3])
    }

    fn from_channels(a: f32, b: f32, c: f32, d: f32) -> Oklab {
        const CHANNELS: usize = 3;
        *<Oklab as Pixel>::from_slice(&[a, b, c, d][..CHANNELS])
    }

    #[deprecated(note = "This function is currently broken because it corrupts some memory!")]
    #[allow(unreachable_code, unused_variables)]
    fn from_slice(slice: &[f32]) -> &Oklab {
        //panic!("This function is currently broken because it corrupts some memory!");
        assert_eq!(slice.len(), 3);
        /*unsafe {
            &std::mem::replace(
                &mut Oklab::new(0.0, 0.0, 0.0),
                Oklab::from_components(*(slice.as_ptr() as *const [f32; 3])),
            )
        }*/
        unsafe { &*(slice.as_ptr() as *const Oklab) }
    }

    #[deprecated(note = "This function is currently broken because it corrupts some memory!")]
    #[allow(unreachable_code, unused_variables)]
    fn from_slice_mut(slice: &mut [f32]) -> &mut Oklab {
        //panic!("This function is currently broken because it corrupts some memory!");
        assert_eq!(slice.len(), 3);
        unsafe { &mut *(slice.as_mut_ptr() as *mut Oklab) }
    }

    fn to_rgb(&self) -> Rgb<f32> {
        <Self as Into<Rgb<f32>>>::into(*self)
    }

    fn to_rgba(&self) -> Rgba<f32> {
        self.to_rgb().to_rgba()
    }

    fn to_luma(&self) -> Luma<f32> {
        Luma([*self.l()])
    }

    fn to_luma_alpha(&self) -> LumaA<f32> {
        LumaA([*self.l(), 1.0])
    }

    fn map<F>(&self, f: F) -> Oklab
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

    fn map_with_alpha<F, G>(&self, f: F, g: G) -> Oklab
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

    fn map2<F>(&self, other: &Self, f: F) -> Oklab
    where
        F: FnMut(f32, f32) -> f32,
    {
        let mut this = (*self).clone();
        this.apply2(other, f);
        this
    }

    fn apply2<F>(&mut self, other: &Oklab, mut f: F)
    where
        F: FnMut(f32, f32) -> f32,
    {
        for (a, &b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = f(*a, b)
        }
    }

    fn invert(&mut self) {}

    fn blend(&mut self, _other: &Oklab) {}
}

impl Deref for Oklab {
    type Target = [f32; 3];
    fn deref(&self) -> &Self::Target {
        &self.channels
    }
}

impl DerefMut for Oklab {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channels
    }
}

fn rgb_to_oklab_faster<T: Primitive + AsFloat>(rgb: &Rgb<T>) -> Oklab {
    let (r, g, b) = (
        rgb.0[0].as_float().powf(2.2),
        rgb.0[1].as_float().powf(2.2),
        rgb.0[2].as_float().powf(2.2),
    );

    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    // Conversione da RGB a linear RGB
    let (lr, lg, lb) = (
        srgb_to_linear(r / 255.0),
        srgb_to_linear(g / 255.0),
        srgb_to_linear(b / 255.0),
    );

    // Conversione da linear RGB a spazio colore XYZ
    let (x, y, z) = (
        0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb,
        0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb,
        0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb,
    );

    // Conversione da XYZ a OKLAB
    let l = 0.2104542553 * x + 0.7936177850 * y - 0.0040720468 * z;
    let m = 1.9779984951 * x - 2.4285922050 * y + 0.4505937099 * z;
    let s = 0.0259040371 * x + 0.7827717662 * y - 0.8086757660 * z;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    Oklab::from_components([l, a, b])
}

fn rgb_to_oklab<T: Primitive + AsFloat>(rgb: &Rgb<T>) -> Oklab {
    let (r, g, b) = (
        rgb.0[0].as_float().powf(2.2),
        rgb.0[1].as_float().powf(2.2),
        rgb.0[2].as_float().powf(2.2),
    );

    let m1 = Matrix::new(
        vec![
            0.4122214708f32,
            0.5363325363f32,
            0.0514459929f32,
            0.2119034982f32,
            0.6806995451f32,
            0.1073969566f32,
            0.0883024619f32,
            0.2817188376f32,
            0.6299787005f32,
        ],
        3,
        3,
    );
    let m2 = Matrix::new(
        vec![
            0.2104542553f32,
            0.7936177850f32,
            -0.0040720468f32,
            1.9779984951f32,
            -2.4285922050f32,
            0.4505937099f32,
            0.0259040371f32,
            0.7827717662f32,
            -0.8086757660f32,
        ],
        3,
        3,
    );

    let v = (m1 * Matrix::new(vec![r, g, b], 3, 1)).unwrap();
    let vc = v.get_content();

    let (l, m, s) = (vc[0].cbrt(), vc[1].cbrt(), vc[2].cbrt());

    let lab = (m2 * Matrix::new(vec![l, m, s], 3, 1))
        .unwrap()
        .get_content()
        .clone();

    Oklab::from_components([lab[0], lab[1], lab[2]])
}

fn linsrgb_to_oklab(srgb: &LinSrgb) -> Oklab {
    let (r, g, b) = (srgb.r(), srgb.g(), srgb.b());

    let m1 = Matrix::new(
        vec![
            0.4122214708f32,
            0.5363325363f32,
            0.0514459929f32,
            0.2119034982f32,
            0.6806995451f32,
            0.1073969566f32,
            0.0883024619f32,
            0.2817188376f32,
            0.6299787005f32,
        ],
        3,
        3,
    );
    let m2 = Matrix::new(
        vec![
            0.2104542553f32,
            0.7936177850f32,
            -0.0040720468f32,
            1.9779984951f32,
            -2.4285922050f32,
            0.4505937099f32,
            0.0259040371f32,
            0.7827717662f32,
            -0.8086757660f32,
        ],
        3,
        3,
    );

    let v = (m1 * Matrix::new(vec![r, g, b], 3, 1)).unwrap();
    let vc = v.get_content();

    let (l, m, s) = (vc[0].cbrt(), vc[1].cbrt(), vc[2].cbrt());

    let lab = (m2 * Matrix::new(vec![l, m, s], 3, 1))
        .unwrap()
        .get_content()
        .clone();

    Oklab::from_components([lab[0], lab[1], lab[2]])
}

fn oklab_to_linsrgb(l: f32, a: f32, b: f32) -> LinSrgb {
    let m1 = Matrix::new(
        vec![
            1.0f32,
            0.3963377774f32,
            0.2158037573f32,
            1.0f32,
            -0.1055613458f32,
            -0.0638541728f32,
            1.0f32,
            -0.0894841775f32,
            -1.2914855480f32,
        ],
        3,
        3,
    );
    let m2 = Matrix::new(
        vec![
            4.0767416621f32,
            -3.3077115913f32,
            0.2309699292f32,
            -1.2684380046f32,
            2.6097574011f32,
            -0.3413193965f32,
            -0.0041960863f32,
            -0.7034186147f32,
            1.7076147010f32,
        ],
        3,
        3,
    );

    let v = (m1 * Matrix::new(vec![l, a, b], 3, 1))
        .unwrap()
        .get_content()
        .clone();

    let (l, m, s) = (v[0] * v[0] * v[0], v[1] * v[1] * v[1], v[2] * v[2] * v[2]);

    let rgb = (m2 * Matrix::new(vec![l, m, s], 3, 1))
        .unwrap()
        .get_content()
        .clone();

    LinSrgb::from_components([rgb[0], rgb[1], rgb[2]])
}

impl From<Oklab> for LinSrgb {
    fn from(value: Oklab) -> Self {
        let channels = value.channels();
        oklab_to_linsrgb(channels[0], channels[1], channels[2])
    }
}

impl From<Oklab> for Rgb<u8> {
    fn from(value: Oklab) -> Rgb<u8> {
        Rgb::from(LinSrgb::from(value))
    }
}

impl From<Oklab> for Rgb<u16> {
    fn from(value: Oklab) -> Rgb<u16> {
        Rgb::from(LinSrgb::from(value))
    }
}

impl From<Oklab> for Rgb<f32> {
    fn from(value: Oklab) -> Rgb<f32> {
        Rgb::from(LinSrgb::from(value))
    }
}

impl<T: Primitive + AsFloat> From<Rgb<T>> for Oklab {
    fn from(rgb: Rgb<T>) -> Self {
        rgb_to_oklab_faster(&rgb)
    }
}

impl From<LinSrgb> for Oklab {
    fn from(rgb: LinSrgb) -> Self {
        linsrgb_to_oklab(&rgb)
    }
}

pub type OklabImage = ImageBuffer<Oklab, Vec<f32>>;

#[derive(PartialEq, Clone, Debug, Copy, Default)]
#[repr(C)]
#[allow(missing_docs)]
pub struct Oklaba {
    channels: [f32; 4],
}

impl Oklaba {
    pub fn l(&self) -> &f32 {
        &self.channels[0]
    }

    pub fn a(&self) -> &f32 {
        &self.channels[1]
    }

    pub fn b(&self) -> &f32 {
        &self.channels[2]
    }

    pub fn alpha(&self) -> &f32 {
        &self.channels[3]
    }

    pub fn l_mut(&mut self) -> &mut f32 {
        &mut self.channels[0]
    }

    pub fn a_mut(&mut self) -> &mut f32 {
        &mut self.channels[1]
    }

    pub fn b_mut(&mut self) -> &mut f32 {
        &mut self.channels[2]
    }

    pub fn alpha_mut(&mut self) -> &mut f32 {
        &mut self.channels[3]
    }

    pub fn new(l: f32, a: f32, b: f32, alpha: f32) -> Oklaba {
        Oklaba {
            channels: [l, a, b, alpha],
        }
    }

    pub fn from_components(oklab: [f32; 4]) -> Oklaba {
        Oklaba { channels: oklab }
    }
}

#[allow(useless_deprecated)]
impl Pixel for Oklaba {
    type Subpixel = f32;

    const CHANNEL_COUNT: u8 = 4;

    #[inline(always)]
    fn channels(&self) -> &[f32] {
        &self.channels
    }

    #[inline(always)]
    fn channels_mut(&mut self) -> &mut [f32] {
        &mut self.channels
    }

    const COLOR_MODEL: &'static str = "OKLABA";

    fn channels4(&self) -> (f32, f32, f32, f32) {
        const CHANNELS: usize = 3;
        let mut channels = [f32::MAX; 4];
        channels[0..CHANNELS].copy_from_slice(&self.channels);
        (channels[0], channels[1], channels[2], channels[3])
    }

    fn from_channels(l: f32, a: f32, b: f32, alpha: f32) -> Oklaba {
        const CHANNELS: usize = 4;
        *<Oklaba as Pixel>::from_slice(&[l, a, b, alpha][..CHANNELS])
    }

    #[deprecated(note = "This function is currently broken because it corrupts some memory!")]
    #[allow(unreachable_code, unused_variables)]
    fn from_slice(slice: &[f32]) -> &Oklaba {
        //panic!("This function is currently broken because it corrupts some memory!");
        assert_eq!(slice.len(), 4);
        /*unsafe {
            &std::mem::replace(
                &mut Oklab::new(0.0, 0.0, 0.0),
                Oklab::from_components(*(slice.as_ptr() as *const [f32; 3])),
            )
        }*/
        unsafe { &*(slice.as_ptr() as *const Oklaba) }
    }

    #[deprecated(note = "This function is currently broken because it corrupts some memory!")]
    #[allow(unreachable_code, unused_variables)]
    fn from_slice_mut(slice: &mut [f32]) -> &mut Oklaba {
        //panic!("This function is currently broken because it corrupts some memory!");
        assert_eq!(slice.len(), 4);
        unsafe { &mut *(slice.as_mut_ptr() as *mut Oklaba) }
    }

    fn to_rgb(&self) -> Rgb<f32> {
        self.to_rgba().to_rgb()
    }

    fn to_rgba(&self) -> Rgba<f32> {
        <Self as Into<Rgba<f32>>>::into(*self)
    }

    fn to_luma(&self) -> Luma<f32> {
        Luma([*self.l()])
    }

    fn to_luma_alpha(&self) -> LumaA<f32> {
        LumaA([*self.l(), 1.0])
    }

    fn map<F>(&self, f: F) -> Oklaba
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

    fn map_with_alpha<F, G>(&self, f: F, g: G) -> Oklaba
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

    fn map2<F>(&self, other: &Self, f: F) -> Oklaba
    where
        F: FnMut(f32, f32) -> f32,
    {
        let mut this = (*self).clone();
        this.apply2(other, f);
        this
    }

    fn apply2<F>(&mut self, other: &Oklaba, mut f: F)
    where
        F: FnMut(f32, f32) -> f32,
    {
        for (a, &b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = f(*a, b)
        }
    }

    fn invert(&mut self) {}

    fn blend(&mut self, _other: &Oklaba) {}
}

impl Deref for Oklaba {
    type Target = [f32; 4];
    fn deref(&self) -> &Self::Target {
        &self.channels
    }
}

impl DerefMut for Oklaba {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channels
    }
}

fn rgba_to_oklaba<T: Primitive + AsFloat>(rgb: &Rgba<T>) -> Oklaba {
    let linsrgb = LinSrgba::from(*rgb);
    linsrgba_to_oklaba(&linsrgb)
}

fn linsrgba_to_oklaba(srgb: &LinSrgba) -> Oklaba {
    let (r, g, b) = (srgb.r(), srgb.g(), srgb.b());

    let m1 = Matrix::new(
        vec![
            0.4122214708f32,
            0.5363325363f32,
            0.0514459929f32,
            0.2119034982f32,
            0.6806995451f32,
            0.1073969566f32,
            0.0883024619f32,
            0.2817188376f32,
            0.6299787005f32,
        ],
        3,
        3,
    );
    let m2 = Matrix::new(
        vec![
            0.2104542553f32,
            0.7936177850f32,
            -0.0040720468f32,
            1.9779984951f32,
            -2.4285922050f32,
            0.4505937099f32,
            0.0259040371f32,
            0.7827717662f32,
            -0.8086757660f32,
        ],
        3,
        3,
    );

    let v = (m1 * Matrix::new(vec![r, g, b], 3, 1)).unwrap();
    let vc = v.get_content();

    let (l, m, s) = (vc[0].cbrt(), vc[1].cbrt(), vc[2].cbrt());

    let lab = (m2 * Matrix::new(vec![l, m, s], 3, 1))
        .unwrap()
        .get_content()
        .clone();

    Oklaba::from_components([lab[0], lab[1], lab[2], *srgb.alpha()])
}

fn oklaba_to_linsrgba(l: f32, a: f32, b: f32, alpha: f32) -> LinSrgba {
    let m1 = Matrix::new(
        vec![
            1.0f32,
            0.3963377774f32,
            0.2158037573f32,
            1.0f32,
            -0.1055613458f32,
            -0.0638541728f32,
            1.0f32,
            -0.0894841775f32,
            -1.2914855480f32,
        ],
        3,
        3,
    );
    let m2 = Matrix::new(
        vec![
            4.0767416621f32,
            -3.3077115913f32,
            0.2309699292f32,
            -1.2684380046f32,
            2.6097574011f32,
            -0.3413193965f32,
            -0.0041960863f32,
            -0.7034186147f32,
            1.7076147010f32,
        ],
        3,
        3,
    );

    let v = (m1 * Matrix::new(vec![l, a, b], 3, 1))
        .unwrap()
        .get_content()
        .clone();

    let (l, m, s) = (v[0] * v[0] * v[0], v[1] * v[1] * v[1], v[2] * v[2] * v[2]);

    let rgb = (m2 * Matrix::new(vec![l, m, s], 3, 1))
        .unwrap()
        .get_content()
        .clone();

    LinSrgba::from_components([rgb[0], rgb[1], rgb[2], alpha])
}

impl From<Oklaba> for LinSrgba {
    fn from(value: Oklaba) -> Self {
        let channels = value.channels();
        oklaba_to_linsrgba(channels[0], channels[1], channels[2], channels[3])
    }
}

impl From<Oklaba> for Rgba<u8> {
    fn from(value: Oklaba) -> Rgba<u8> {
        Rgba::from(LinSrgba::from(value))
    }
}

impl From<Oklaba> for Rgba<u16> {
    fn from(value: Oklaba) -> Rgba<u16> {
        Rgba::from(LinSrgba::from(value))
    }
}

impl From<Oklaba> for Rgba<f32> {
    fn from(value: Oklaba) -> Rgba<f32> {
        Rgba::from(LinSrgba::from(value))
    }
}

impl<T: Primitive + AsFloat> From<Rgba<T>> for Oklaba {
    fn from(rgb: Rgba<T>) -> Self {
        rgba_to_oklaba(&rgb)
    }
}

impl From<LinSrgba> for Oklaba {
    fn from(rgb: LinSrgba) -> Self {
        linsrgba_to_oklaba(&rgb)
    }
}

pub type OklabaImage = ImageBuffer<Oklaba, Vec<f32>>;
