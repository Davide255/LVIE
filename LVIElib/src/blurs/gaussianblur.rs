use image::{RgbImage, RgbaImage};
//use image::{Rgb, GenericImageView, Pixel};

use crate::traits::Scale;

use super::boxblur::{CRgbaImage, FastBoxBlur, FastBoxBlur_rgb, FastBoxBlur_rgba};

fn boxesForGauss(sigma: f32, n: f32) -> Vec<u16> // standard deviation, number of boxes
{
    let wIdeal = ((12f32 * sigma * sigma / n) + 1f32).sqrt(); // Ideal averaging filter width
    let mut wl = wIdeal.floor();
    if wl % 2f32 == 0f32 {
        wl -= 1f32;
    };
    let wu = wl + 2f32;

    let mIdeal =
        (12f32 * sigma * sigma - n * wl * wl - 4f32 * n * wl - 3f32 * n) / (-4f32 * wl - 4f32);
    let m = mIdeal.round();
    // var sigmaActual = Math.sqrt( (m*wl*wl + (n-m)*wu*wu - n)/12 );

    let mut sizes: Vec<u16> = Vec::new();
    for i in 0..n as u8 {
        if (i as f32) < m {
            sizes.push(wl as u16);
        } else {
            sizes.push(wu as u16);
        }
    }
    return sizes;
}

pub fn FastGaussianBlur_rgb(img: &RgbImage, sigma: f32, n: u8) -> RgbImage {
    let bxs = boxesForGauss(sigma, n as f32);

    let mut out = FastBoxBlur_rgb(img, ((bxs[0] - 1u16) / 2u16) as u32);

    for pass in 0..n as usize {
        out = FastBoxBlur_rgb(img, ((bxs[pass] - 1u16) / 2u16) as u32);
    }

    out
}

pub fn FastGaussianBlur_rgba(img: &RgbaImage, sigma: f32, n: u8) -> RgbaImage {
    let bxs = boxesForGauss(sigma, n as f32);

    let mut out = FastBoxBlur_rgba(img, ((bxs[0] - 1u16) / 2u16) as u32);

    for pass in 0..n as usize {
        out = FastBoxBlur_rgba(img, ((bxs[pass] - 1u16) / 2u16) as u32);
    }

    out
}

use image::{Pixel, Primitive};

pub fn FastGaussianBlur<P>(img: &CRgbaImage<P>, sigma: f32, n: u8) -> CRgbaImage<P>
where
    P: Pixel + Send + Sync + 'static + std::fmt::Debug,
    P::Subpixel: Scale + Primitive + std::fmt::Debug + Send + Sync,
{
    let bxs = boxesForGauss(sigma, n as f32);

    let mut out = FastBoxBlur(img, ((bxs[0] - 1u16) / 2u16) as u32);

    for pass in 0..n as usize {
        out = FastBoxBlur(img, ((bxs[pass] - 1u16) / 2u16) as u32);
    }

    out
}

/*
fn gaussBlur_4 (img: &RgbImage, r: u32) {
    let bxs = boxesForGauss(r, 3);
    boxBlur_4 (scl, tcl, (bxs[0]-1)/2);
    boxBlur_4 (tcl, scl, (bxs[1]-1)/2);
    boxBlur_4 (scl, tcl, (bxs[2]-1)/2);
}

fn boxBlur_4 (img: &RgbImage, out: &mut RgbImage, r: u32) {
    boxBlurH_4(img, out, r);
    boxBlurT_4(scl, tcl, w, h, r);
}

fn boxBlurH_4 (img: &RgbImage, out: &mut RgbImage, r: u32) {
    let iarr = 1f32 / ((r as f32).powi(2) +1f32);
    let (w, h) = img.dimensions();
    for i in 0..h {
        let fv = img.get_pixel(0, i);
        let lv = img.get_pixel(w-1, i);
        for c in 0usize..3usize {
            let mut li = ti;
            let mut ri = ti + r;
            let mut val: u32 = (r+1)*fv.0[c] as u32;
            for j in 0..r {
                val += img.get_pixel(j, i).0[c] as u32;
            }
            for j in 0..r {
                ri += 1;
                val += img.get_pixel(r+1, i).0[c] as u32 - fv.0[c] as u32;
                out.get_pixel_mut(1, i).0[c] = (val as f32 * iarr).round() as u8;
            }
            for j in r+1..w-r {
                val += img.get_pixel(r+1, i).0[c] as u32 - img.get_pixel(r+1, i).0[c] as u32;
                out.get_pixel_mut(1, i).0[c] = (val as f32 * iarr).round() as u8;
            }
            for(var j=w-r; j<w  ; j++) { val += lv        - scl[li++];   tcl[ti++] = Math.round(val*iarr); }
        }
    }
}
function boxBlurT_4 (scl, tcl, w, h, r) {
    var iarr = 1 / (r+r+1);
    for(var i=0; i<w; i++) {
        var ti = i, li = ti, ri = ti+r*w;
        var fv = scl[ti], lv = scl[ti+w*(h-1)], val = (r+1)*fv;
        for(var j=0; j<r; j++) val += scl[ti+j*w];
        for(var j=0  ; j<=r ; j++) { val += scl[ri] - fv     ;  tcl[ti] = Math.round(val*iarr);  ri+=w; ti+=w; }
        for(var j=r+1; j<h-r; j++) { val += scl[ri] - scl[li];  tcl[ti] = Math.round(val*iarr);  li+=w; ri+=w; ti+=w; }
        for(var j=h-r; j<h  ; j++) { val += lv      - scl[li];  tcl[ti] = Math.round(val*iarr);  li+=w; ti+=w; }
    }
}
*/
