use crate::{math::normalize_2d, matrix::Matrix};

// White Point uv coorinates equation coefficients
const WP_U: [f32; 6] = [
    0.860117757,
    1.54118254 * 0.0001,
    1.28641212 * 0.0000001,
    1.0,
    8.42420235 * 0.0001,
    7.08145163 * 0.0000001,
];
const WP_V: [f32; 6] = [
    0.317398726,
    4.22806245 * 0.00001,
    4.20481691 * 0.00000001,
    1.0,
    -2.89741816 * 0.00001,
    1.61456053 * 0.0000001,
];

// source: www.image-engineering.de
pub const LINSRGB_TO_XYZ: [f32; 9] = [
    0.4124564, 0.3575761, 0.1804375,
    0.2126729, 0.7151522, 0.0721750,
    0.0193339, 0.1191920, 0.9503041,
];

pub const XYZ_TO_LINSRGB: [f32; 9] =  [
    3.2404542, -1.5371385, -0.4985314,
    -0.9692660, 1.8760108, 0.0415560,
    0.0556434, -0.2040259, 1.0572252,
];

pub fn uv_white_point(temp: f32, tint: f32) -> (f32, f32) {
    // Planck's locus in uv chromacity coordinates
    let u = (WP_U[0] + WP_U[1] * temp + WP_U[2] * temp * temp)
        / (WP_U[3] + WP_U[4] * temp + WP_U[5] * temp * temp);
    let v = (WP_V[0] + WP_V[1] * temp + WP_V[2] * temp * temp)
        / (WP_V[3] + WP_V[4] * temp + WP_V[5] * temp * temp);

    // derivatives of the parametric equations, for calculating the normal vector and moving on the isothermal line
    let (a, b, c, d, f, g, t) = (WP_U[0], WP_U[1], WP_U[2], WP_U[3], WP_U[4], WP_U[5], temp);
    let mut du = (-a * (f + 2.0 * g * t) + b * (d - g * t * t) + c * t * (2.0 * d + f * t))
        / (d + t * (f + g * t)).powi(2);

    let (a, b, c, d, f, g, t) = (WP_V[0], WP_V[1], WP_V[2], WP_V[3], WP_V[4], WP_V[5], temp);
    let mut dv = (-a * (f + 2.0 * g * t) + b * (d - g * t * t) + c * t * (2.0 * d + f * t))
        / (d + t * (f + g * t)).powi(2);

    (du, dv) = normalize_2d(du, dv);

    (u + tint * dv / 1000.0, v - tint * du / 1000.0)
}

pub fn uv_to_xy(u: f32, v: f32) -> (f32, f32) {
    (
        3.0 * u / (2.0 * u - 8.0 * v + 4.0),
        2.0 * v / (2.0 * u - 8.0 * v + 4.0),
    )
}

pub fn xy_white_point(temp: f32) -> (f32, f32) {
    let x: f32;
    if temp < 0.0 {
        x = (-4607000000.0 / (temp * temp * temp))
            + (2967800.0 / (temp * temp))
            + 99.11 / temp
            + 0.244063;
    } else {
        x = (-2006400000.0 / (temp * temp * temp))
            + (1901800.0 / (temp * temp))
            + 247.48 / temp
            + 0.237040;
    };

    let y = -3.0 * x * x + 2.87 * x - 0.275;

    (x, y)
}

pub fn xyz_wb_matrix(fromtemp: f32, fromtint: f32, totemp: f32, totint: f32) -> Matrix<f32> {

    // source: Wikipedia
    let xyz_to_lms = Matrix::from_rows(vec![
        vec![0.8951, 0.2664, -0.1614],
        vec![-0.7502, 1.7135, 0.0367],
        vec![0.0389, -0.0685, 1.0296],
    ]);

    // source: WolframAlpha
    let lms_to_xyz = Matrix::new(
        vec![
            0.986993,
            -0.147054,
            0.159963,
            0.432305,
            0.51836,
            0.0492912,
            -0.00852866,
            0.0400428,
            0.968487,
        ],
        3,
        3,
    );

    let (u, v) = uv_white_point(fromtemp, fromtint);
    let (x, y) = uv_to_xy(u, v);
    //let (x, y) = (0.31271, 0.32902);
    let mut fromwp = vec![x / y, 1.0, (1.0 - x - y) / y];
    fromwp = (xyz_to_lms.clone() * fromwp.into())
        .unwrap()
        .consume_content();

    let (u, v) = uv_white_point(totemp, totint);
    let (x, y) = uv_to_xy(u, v);
    //let (x, y) = (0.28315, 0.29711);
    let mut towp = vec![x / y, 1.0, (1.0 - x - y) / y];
    towp = (xyz_to_lms.clone() * towp.into())
        .unwrap()
        .consume_content();

    let diag = Matrix::from_diagonal(
        vec![
            fromwp[0] / towp[0],
            fromwp[1] / towp[1],
            fromwp[2] / towp[2],
        ],
        0.0,
    );

    ((lms_to_xyz * diag).unwrap() * xyz_to_lms).unwrap()
}

/* pub fn wb_matrix(fromtemp: f32, fromtint: f32, totemp: f32, totint: f32) -> Matrix<f32> {
    // source: www.image-engineering.de
    let linrgb_to_xyz = Matrix::new(
        vec![
            0.4124564, 0.3575761, 0.1804375, 0.2126729, 0.7151522, 0.0721750, 0.0193339, 0.1191920,
            0.9503041,
        ],
        3,
        3,
    );

    // source: Wikipedia
    let xyz_to_lms = Matrix::from_rows(vec![
        vec![0.8951, 0.2664, -0.1614],
        vec![-0.7502, 1.7135, 0.0367],
        vec![0.0389, -0.0685, 1.0296],
    ]);

    // source: WolframAlpha
    let lms_to_xyz = Matrix::new(
        vec![
            0.986993,
            -0.147054,
            0.159963,
            0.432305,
            0.51836,
            0.0492912,
            -0.00852866,
            0.0400428,
            0.968487,
        ],
        3,
        3,
    );

    // source: www.image-engineering.de
    let xyz_to_linrgb = Matrix::new(
        vec![
            3.2404542, -1.5371385, -0.4985314, -0.9692660, 1.8760108, 0.0415560, 0.0556434,
            -0.2040259, 1.0572252,
        ],
        3,
        3,
    );

    let (u, v) = uv_white_point(fromtemp, fromtint);
    let (x, y) = uv_to_xy(u, v);
    //let (x, y) = (0.31271, 0.32902);
    let mut fromwp = vec![x / y, 1.0, (1.0 - x - y) / y];
    fromwp = (xyz_to_lms.clone() * fromwp.into())
        .unwrap()
        .consume_content();

    let (u, v) = uv_white_point(totemp, totint);
    let (x, y) = uv_to_xy(u, v);
    //let (x, y) = (0.28315, 0.29711);
    let mut towp = vec![x / y, 1.0, (1.0 - x - y) / y];
    towp = (xyz_to_lms.clone() * towp.into())
        .unwrap()
        .consume_content();

    let diag = Matrix::from_diagonal(
        vec![
            fromwp[0] / towp[0],
            fromwp[1] / towp[1],
            fromwp[2] / towp[2],
        ],
        0.0,
    );

    ((((xyz_to_linrgb * lms_to_xyz).unwrap() * diag).unwrap() * xyz_to_lms).unwrap()
        * linrgb_to_xyz)
        .unwrap()
} */

#[cfg(test)]
mod test {
    use crate::matrix::Matrix;

    #[test]
    fn inverses() {
        let I = Matrix::from_diagonal(vec![1.0, 1.0, 1.0], 0.0);

        // source: www.image-engineering.de
        let linrgb_to_xyz = Matrix::new(
            vec![
                0.4124564, 0.3575761, 0.1804375, 0.2126729, 0.7151522, 0.0721750, 0.0193339,
                0.1191920, 0.9503041,
            ],
            3,
            3,
        );

        // source: Wikipedia
        let xyz_to_lms = Matrix::from_rows(vec![
            vec![0.8951, 0.2664, -0.1614],
            vec![-0.7502, 1.7135, 0.0367],
            vec![0.0389, -0.0685, 1.0296],
        ]);

        // source: WolframAlpha
        let lms_to_xyz = Matrix::new(
            vec![
                0.986993,
                -0.147054,
                0.159963,
                0.432305,
                0.51836,
                0.0492912,
                -0.00852866,
                0.0400428,
                0.968487,
            ],
            3,
            3,
        );

        // source: www.image-engineering.de
        let xyz_to_linrgb = Matrix::new(
            vec![
                3.2404542, -1.5371385, -0.4985314, -0.9692660, 1.8760108, 0.0415560, 0.0556434,
                -0.2040259, 1.0572252,
            ],
            3,
            3,
        );

        let mut m = (linrgb_to_xyz * xyz_to_linrgb).unwrap();
        m.round(5);
        assert_eq!(m, I);

        let mut m = (xyz_to_lms * lms_to_xyz).unwrap();
        m.round(5);
        assert_eq!(m, I);
    }
}