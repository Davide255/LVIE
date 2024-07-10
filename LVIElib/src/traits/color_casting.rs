use image::{Pixel, Primitive, Rgb, Rgba};

use super::Scale;

pub fn cast_color_to_rgb<F, T>(hsl: &F) -> T
where
    F: Pixel + Send + Sync + 'static + std::fmt::Debug,
    <F as image::Pixel>::Subpixel: Scale,
    T: Pixel + Send + Sync + 'static + std::fmt::Debug,
    T::Subpixel: Scale + Primitive + std::fmt::Debug + std::fmt::Debug,
{
    let cmp = hsl.to_rgb().0;
    unsafe {
        std::mem::transmute_copy::<Rgb<T::Subpixel>, T>(&Rgb([
            cmp[0].scale(),
            cmp[1].scale(),
            cmp[2].scale(),
        ]))
    }
}

pub fn cast_color_to_rgba<F, T>(hsl: &F) -> T
where
    F: Pixel + Send + Sync + 'static + std::fmt::Debug,
    <F as image::Pixel>::Subpixel: Scale,
    T: Pixel + Send + Sync + 'static + std::fmt::Debug,
    T::Subpixel: Scale + Primitive + std::fmt::Debug + std::fmt::Debug,
{
    let cmp = hsl.to_rgba().0;
    unsafe {
        std::mem::transmute_copy::<Rgba<T::Subpixel>, T>(&Rgba([
            cmp[0].scale(),
            cmp[1].scale(),
            cmp[2].scale(),
            cmp[3].scale(),
        ]))
    }
}
