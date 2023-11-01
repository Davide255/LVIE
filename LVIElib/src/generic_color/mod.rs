use num_traits::{Bounded, Num, NumCast};
use std::ops::AddAssign;

// cloned from image crate because it's in a private module
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

pub trait AsFloat {
    fn as_float(&self) -> f32;
}

impl AsFloat for f32 {
    fn as_float(&self) -> f32 {
        return *self;
    }
}

impl AsFloat for u8 {
    fn as_float(&self) -> f32 {
        return <f32 as NumCast>::from(*self).unwrap() / u8::MAX as f32;
    }
}

impl AsFloat for u16 {
    fn as_float(&self) -> f32 {
        return <f32 as NumCast>::from(*self).unwrap() / u16::MAX as f32;
    }
}
