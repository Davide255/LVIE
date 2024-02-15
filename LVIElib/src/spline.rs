use nalgebra::{DMatrix, DVector};

pub fn spline_coefficients(data: &Vec<f32>, xs: &Vec<f32>) -> Vec<[f32; 4]> {
    let n = data.len();
    let mut output = Vec::<[f32; 4]>::new();

    let A = DMatrix::from_fn(n, n, |r, c| {
        if (r == 0 && c == 0) || (r == n - 1 && c == n - 1) {
            2.0
        } else {
            match r as isize - c as isize {
                -1 | 1 => 1.0,
                0 => 4.0,
                _ => 0.0,
            }
        }
    });

    let b = DVector::from_fn(n, |i, _| match i {
        0 => 3.0 * (data[1] - data[0]),
        x if x == n - 1 => 3.0 * (data[n - 1] - data[n - 2]),
        _ => 3.0 * (data[i + 1] - data[i - 1]),
    });

    let x = A.try_inverse().unwrap() * b;

    for i in 0..n - 1 {
        let h = xs[i+1]-xs[i];
        output.push([
            data[i],
            x[i]*h,
            3.0 * (data[i + 1] - data[i]) - 2.0 * x[i]*h - x[i + 1]*h,
            2.0 * (data[i] - data[i + 1]) + x[i]*h + x[i + 1]*h,
        ]);
    }

    output
}

// assumes the x values are in ascending order
pub fn apply_curve(val: f32, spline: &Vec<[f32; 4]>, x: &Vec<f32>) -> f32 {
    for i in 0..x.len() - 1 {
        if x[i] <= val && val < x[i + 1] {
            let z = (val - x[i]) / (x[i + 1] - x[i]);
            return spline[i][0]
                + spline[i][1] * z
                + spline[i][2] * z * z
                + spline[i][3] * z * z * z;
        }
    }

    val
}

// source: https://math.stackexchange.com/questions/3770662
pub fn bezier_points(spline: &Vec<[f32; 4]>, x: &Vec<f32>) -> Vec<[(f32, f32); 4]> {
    let mut out = Vec::<[(f32, f32); 4]>::new();

    for i in 0..x.len() - 2 {
        let p0 = (x[i], spline[i][0]);
        let p1 = ((2.0 * x[i] + x[i+1]) / 3.0, spline[i][0] + spline[i][1] / 3.0);
        let p2 = (
            (x[i] + 2.0*x[i+1]) / 3.0,
            spline[i][0] + (spline[i][2] + 2.0 * spline[i][1]) / 3.0,
        );
        let p3 = (x[i + 1], spline[i + 1][0]);
        out.push([p0, p1, p2, p3]);
    }

    out
}