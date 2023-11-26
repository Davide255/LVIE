use image::Rgb;

pub struct LinearRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub fn srgb_to_linear(rgb: Rgb<f32>) -> LinearRgb {
    LinearRgb {
        r: rgb.0[0].powf(2.2),
        g: rgb.0[1].powf(2.2),
        b: rgb.0[2].powf(2.2),
    }
}

pub fn linear_to_srgb(rgb: LinearRgb) -> Rgb<f32> {
    Rgb([
        rgb.r.powf(1.0 / 2.2),
        rgb.g.powf(1.0 / 2.2),
        rgb.b.powf(1.0 / 2.2),
    ])
}
