use std::collections::HashMap;

use image::{Rgb, RgbImage, ImageBuffer, Pixel};
use LVIElib::matrix::{
    convolution::laplacian_of_gaussian, convolution::multithreadded::apply_convolution, Matrix,
};
use LVIElib::utils::{convert_hsl_to_rgb, convert_rgb_to_hsl};

use LVIElib::oklab::{Oklab, OklabImage};

use LVIElib::matrix::convolution::convolve;

pub struct Filters {}

impl Filters {
    #[allow(dead_code)]
    pub fn GaussianBlur(sigma: u32, size: (u32, u32)) -> Matrix<f32> {
        laplacian_of_gaussian(sigma as f32, size.0 as usize, size.1 as usize)
    }

    pub fn BoxBlur(sigma: u32) -> Matrix<f32> {
        let mut kernel: Vec<f32> = Vec::new();
        let avg: f32 = 1f32 / (sigma.pow(2) as f32);
        for _ in 0..sigma {
            for _ in 0..sigma {
                kernel.push(avg);
            }
        }
        let size = sigma as usize;
        return Matrix::new(kernel, size, size);
    }
}

pub fn apply_filter(
    img: &RgbImage,
    kernel: &mut Matrix<f32>,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    kernel.pad(width as usize, height as usize, 0.0);

    let matrix = Matrix::new(img.to_vec(), height as usize, 3 * width as usize);

    let convolved = apply_convolution(matrix, &kernel);

    image::RgbImage::from_raw(width, height, convolved.get_content().clone()).unwrap()
}

pub fn build_low_res_preview(img: &RgbImage) -> RgbImage {
    let resized: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::imageops::resize(
        img,
        img.width() / 3,
        img.height() / 3,
        image::imageops::Nearest,
    );

    resized
}

pub fn collect_histogram_data(img: &RgbImage) -> [HashMap<u8, u32>; 3] {
    let mut r: HashMap<u8, u32> = HashMap::new();

    for n in 0u8..=u8::MAX {
        r.insert(n, 0u32);
    }

    let mut g = r.clone();
    let mut b = r.clone();

    for pixel in img.pixels() {
        *r.get_mut(&pixel.0[0]).unwrap() += 1;
        *g.get_mut(&pixel.0[1]).unwrap() += 1;
        *b.get_mut(&pixel.0[2]).unwrap() += 1;
    }

    [r, g, b]
}

use LVIElib::utils::norm_range_f32;

pub fn saturate(img: &RgbImage, value: f32) -> RgbImage {
    let mut hsl_image = convert_rgb_to_hsl(img);
    for (_, _, pixel) in hsl_image.enumerate_pixels_mut() {
        *pixel.saturation_mut() = norm_range_f32(0.0..=1.0, *pixel.saturation() + value / 2f32);
    }
    convert_hsl_to_rgb(&hsl_image)
}

pub fn sharpen(img: &RgbImage, value: f32, size: usize) -> RgbImage {
    let (mut vl, mut va, mut vb) = (Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new());
    let mut oklab_image = OklabImage::new(img.width(), img.height());
    for (x, y, pixel) in img.enumerate_pixels() {
        let ok_pixel = Oklab::from(*pixel);
        let channels = ok_pixel.channels();

        vl.push(channels[0]);
        va.push(channels[1]);
        vb.push(channels[2]);

        oklab_image.put_pixel(x, y, ok_pixel);
    }

    let l_matrix = Matrix::new(vl, img.height() as usize, img.width() as usize);
    let kernel = laplacian_of_gaussian(value, size, size);

    let gradient = convolve(&l_matrix, &kernel);

    let out_l = (l_matrix - gradient).unwrap();

    vl = out_l.get_content().to_owned();

    let mut out: image::ImageBuffer<Rgb<u8>, Vec<u8>> = RgbImage::new(img.width(), img.height());

    let width = img.width();

    for i in 0..vl.len() {
        out.put_pixel(
            i as u32 % width,
            i as u32 / width,
            Rgb::<u8>::from(Oklab::from_components([vl[i], va[i], vb[i]])),
        );
    }

    out
}

pub fn crop<P: Pixel>(img: &ImageBuffer<P, Vec<P::Subpixel>>, x: u32, y:u32, new_width: u32, new_height:u32) -> ImageBuffer<P, Vec<P::Subpixel>>{
    let mut out: ImageBuffer<P, Vec<P::Subpixel>> = ImageBuffer::new(new_width, new_height);

    for ny in 0..new_height {
        for nx in 0..new_width {
            out.put_pixel(nx, ny, img.get_pixel(nx + x, ny + y).clone());
        }
    }

    out
}