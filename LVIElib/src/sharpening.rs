use crate::{
    l_channel_matrix,
    linear_srgb::LinSrgb,
    matrix::convolution::{convolve, laplacian_of_gaussian},
    oklab::Oklab,
    show_l_channel, Matrix,
};

use image::{Pixel, Rgb};

pub fn sharpening(image: Matrix<u8>, size: usize, sigma: f32) -> Matrix<u8> {
    let (mut vl, mut va, mut vb) = (Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new());
    let content = image.get_content().to_owned();
    for i in 0..content.len() / 3 {
        let o_color = Oklab::from(LinSrgb::from(Rgb([
            content[3 * i] as f32 / 255f32,
            content[3 * i + 1] as f32 / 255f32,
            content[3 * i + 2] as f32 / 255f32,
        ])));

        let channels = o_color.channels();

        let (cl, ca, cb) = (channels[0], channels[1], channels[2]);

        vl.push(cl);
        va.push(ca);
        vb.push(cb);
    }

    let l_matrix = Matrix::new(vl, image.height(), image.width() / 3);
    let kernel = laplacian_of_gaussian(sigma, size, size);

    let gradient = convolve(&l_matrix, &kernel);

    let out_l = (l_matrix - gradient).unwrap();

    vl = out_l.get_content().to_owned();

    let mut out: Vec<f32> = Vec::new();

    for i in 0..vl.len() {
        let rgb = Rgb::from(LinSrgb::from(Oklab::from_components([vl[i], va[i], vb[i]]))).0;

        out.append(&mut rgb.to_vec());
    }

    Matrix::new(
        out.iter().map(|x| (*x * 255.0) as u8).collect(),
        image.height(),
        image.width(),
    )
}
