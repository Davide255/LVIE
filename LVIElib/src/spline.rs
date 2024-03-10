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

pub fn monotone_spline_coefficients(data: &Vec<f32>, xs: &Vec<f32>) -> Vec<[f32; 4]> {
    let n = data.len();
    let mut output = Vec::<[f32; 4]>::new();

    let mut secants = Vec::<f32>::new();
    for i in 0..n-2 {
        secants.push((data[i+1]-data[i])/(xs[i+1]-xs[i]));
    }

    let mut m = vec![0.0; n];
    for i in 1..n-2 {
        if secants[i-1]*secants[i]<0.0 {
            m[i] = 0.0;
        } else {
            m[i] = (secants[i-1] + secants[i])/2.0;
        }
    }
    m[0] = secants[0];
    m[n-1] = secants[n-2];

    for k in 1..n-1 {
        if secants[k-1]/secants[k-1].abs() != secants[k]/secants[k].abs(){
            m[k] = 0.0;
        }
    }
 
    for k in 0..n-1 {
        if secants[k] == 0.0 {
            m[k] = 0.0;
            m[k+1] = 0.0;
        }
    }

    let mut a: Vec<f32> = m.clone().iter().zip(0..m.len()).map(|(v, i)| { 
        if secants[0] != 0.0 { v / secants[i] } else { std::f32::NAN }
    }).collect();
    let mut b: Vec<f32> = secants.clone().iter().zip(0..secants.len()).map(|(t, i)| m[i+1]/t).collect();

    for i in 0..n - 1 {
        let h = xs[i+1]-xs[i];
        output.push([
            data[i],
            m[i]*h,
            3.0 * (data[i + 1] - data[i]) - 2.0 * m[i]*h - m[i + 1]*h,
            2.0 * (data[i] - data[i + 1]) + m[i]*h + m[i + 1]*h,
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