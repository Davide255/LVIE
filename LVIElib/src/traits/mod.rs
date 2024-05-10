use image::Primitive;
//use rayon::prelude::*;

use image::Pixel;
use num_traits::Bounded;
use num_traits::FromPrimitive;
use num_traits::cast;
use num_traits::NumCast;

use std::any::{TypeId, Any};
use std::ops::Deref;

pub trait Scale
where 
    Self: Bounded + FromPrimitive + NumCast + Copy + Any
{
    fn scale<To: FromPrimitive + Bounded + NumCast + ?Sized + Any>(&self) -> To {
        if TypeId::of::<Self>() == TypeId::of::<To>() { 
            cast(*self).unwrap()
        } else if TypeId::of::<To>() == TypeId::of::<f32>() || TypeId::of::<To>() == TypeId::of::<f64>() {
            cast::<f64, To>(
                <f64 as NumCast>::from(*self).unwrap() * 1.0 / <f64 as NumCast>::from(Self::max_value()).unwrap()
            ).unwrap()
        } else {
            cast::<f64, To>(
                <f64 as NumCast>::from(*self).unwrap() * <f64 as NumCast>::from(To::max_value()).unwrap() / <f64 as NumCast>::from(Self::max_value()).unwrap()
            ).unwrap()
        }
    }
}

impl Scale for u8 {}
impl Scale for u16 {}
impl Scale for f32 {
    fn scale<To: FromPrimitive + Bounded + NumCast + ?Sized + Any>(&self) -> To {
        if TypeId::of::<Self>() == TypeId::of::<To>() { 
            cast(*self).unwrap()
        } else {
            cast::<f64, To>(
                <f64 as NumCast>::from(*self).unwrap() * <f64 as NumCast>::from(To::max_value()).unwrap() / 1.0
            ).unwrap()
        }
    }
}

use image::ImageBuffer;

pub trait ScaleImage
where 
    Self: image::GenericImageView
{
    fn scale_image<P, To>(&self) -> ImageBuffer<To, Vec<To::Subpixel>> 
    where 
        P: Pixel + Send + Sync + 'static, P::Subpixel: Scale + Primitive,
        To: Pixel + Send + Sync + 'static, To::Subpixel: Scale + Clone,
        Vec<To::Subpixel>: Deref<Target = [To::Subpixel]>,
        Self: image::GenericImageView + Deref<Target = [<P as Pixel>::Subpixel]> + Sized
    {
        if TypeId::of::<P>() == TypeId::of::<To>() {
            return ImageBuffer::<To, Vec<To::Subpixel>>::from_vec(
                self.width(), self.height(), self.deref().to_vec().iter().map(|x| cast(*x).unwrap()).collect()
            ).unwrap();
        }
        ImageBuffer::<To, Vec<To::Subpixel>>::from_vec(
            self.width(), self.height(), 
            self.deref().to_vec()
            .into_iter()
            .map(|f| {
                f.scale()
            })
            .collect::<Vec<To::Subpixel>>(),
        ).unwrap()
    }
}

impl<P> ScaleImage for ImageBuffer<P, Vec<P::Subpixel>>
where P: Pixel {}