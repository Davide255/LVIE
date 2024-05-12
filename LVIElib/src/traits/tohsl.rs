use crate::hsl::{Hsl, Hsla, HslaImage};
use image::{Rgb, Rgba, Pixel};

pub trait ToHsl {
    fn to_hsl(&self) -> Hsl;
    fn to_hsla(&self) -> Hsla;
}

macro_rules! impl_ToHsl_for_Rgb {
    ($t: ty) => {
        impl ToHsl for Rgb<$t> {
            fn to_hsl(&self) -> Hsl { Hsl::from(*self) }
            fn to_hsla(&self) -> Hsla { Hsla::from(self.to_rgba()) }
        }
    };
}

impl_ToHsl_for_Rgb!(u8);
impl_ToHsl_for_Rgb!(u16);
impl_ToHsl_for_Rgb!(f32);

macro_rules! impl_ToHsl_for_Rgba {
    ($t: ty) => {
        impl ToHsl for Rgba<$t> {
            fn to_hsl(&self) -> Hsl { Hsl::from(self.to_rgb()) }
            fn to_hsla(&self) -> Hsla { Hsla::from(*self) }
        }
    };
}

impl_ToHsl_for_Rgba!(u8);
impl_ToHsl_for_Rgba!(u16);
impl_ToHsl_for_Rgba!(f32);

impl ToHsl for Hsl {
    fn to_hsl(&self) -> Hsl {
        return self.clone();
    }
    fn to_hsla(&self) -> Hsla {
        return Hsla::new(*self.hue(), *self.saturation(), *self.luma(), 1.0);
    }
}

impl ToHsl for Hsla {
    fn to_hsl(&self) -> Hsl {
        return Hsl::new(*self.hue(), *self.saturation(), *self.luma());
    }
    fn to_hsla(&self) -> Hsla {
        return self.clone();
    }
}

use image::GenericImageView;
use num_traits::ToPrimitive;

pub trait ImageToHsla 
where
    Self: GenericImageView + Sized,
    <Self as GenericImageView>::Pixel: ToHsl,
    <<Self as GenericImageView>::Pixel as image::Pixel>::Subpixel: ToPrimitive
{
    fn to_hsla(&self) -> HslaImage {
        return HslaImage::from_vec(
            self.width(), self.height(), 
            {
            let mut out = Vec::<f32>::new();
            for (_,_,p) in self.pixels() {
                for v in p.to_hsla().channels(){ out.push(*v); }
            }
            out
        }
        ).unwrap();
    }
}