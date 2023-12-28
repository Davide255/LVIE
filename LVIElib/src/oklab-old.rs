use crate::{linear_rgb::LinearRgb, matrix::Matrix};
use image::Rgb;

enum LabChannel {
    L,
    a,
    b,
}

pub struct OkLab {
    pub L: f32,
    pub a: f32,
    pub b: f32,
}

impl OkLab {
    pub fn from_components(L: f32, a: f32, b: f32) -> Self {
        OkLab { L, a, b }
    }

    pub fn components(self: Self) -> (f32, f32, f32) {
        (self.L, self.a, self.b)
    }
}

pub fn linear_srgbf32_to_oklabf32(rgb: LinearRgb) -> OkLab {
    let (r, g, b) = (rgb.r, rgb.g, rgb.b);
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

    OkLab {
        L: lab[0],
        a: lab[1],
        b: lab[2],
    }
}

pub fn oklabf32_to_linear_srgbf32(lab: OkLab) -> LinearRgb {
    let (l, a, b) = (lab.L, lab.a, lab.b);
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

    LinearRgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
    }
}

/*pub fn merge_oklab_channel(colors: &mut Vec<OkLab>, channel: LabChannel, content: Vec<f32>, c: f32) {
    let (mut l, mut a, mut b) = (Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new());
    for i in 0..colors.len() {
        let color = colors[i];
        let (cl, ca, cb) = color.components();
        l.push(cl);
        a.push(ca);
        b.push(cb);
    }
}*/
