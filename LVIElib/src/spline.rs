use plotters::prelude::*;

pub enum SplineConstrains {
    FirstDerivatives(f32, f32),
    SecondDerivatives(f32, f32),
}

fn solve_tridiagonal_system(up: Vec<f32>, mid: Vec<f32>, down: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let n = mid.len();
    let mut out = vec![0.0; n];

    let mut gamma = vec![0.0; n-1];
    let mut delta = vec![0.0; n];

    gamma[0] = up[0]/mid[0];
    delta[0] = b[0]/mid[0];

    for i in 1..n-1 {
        gamma[i] = up[i]/(mid[i]-gamma[i-1]*down[i-1]);
        delta[i] = (b[i]-down[i-1]*delta[i-1])/(mid[i]-gamma[i-1]*down[i-1]);
    }

    delta[n-1] = (b[n-1]-down[n-2]*delta[n-2])/(mid[n-1]-gamma[n-2]*down[n-2]);

    out[n-1] = delta[n-1];
    for i in (0..n-1).rev() {
        out[i] = delta[i] - gamma[i]*out[i+1];
    }

    out
}

pub fn spline_coefficients(data: &Vec<f32>, xs: &Vec<f32>, cstr: SplineConstrains) -> Vec<[f32; 4]> {
    let n = data.len();
    let mut output = Vec::<[f32; 4]>::new();

    let mut b= vec![0.0; n];

    // $$ h1 D0 + 2(h1 + h0)D1 + h0 D2 = 3(-h1/h0 y0 + (h1^2 - h0^2)/h1h0 y1 + h0/h1 y2) $$
    let mut up = vec![0.0; n-1];
    let mut mid = vec![1.0; n];
    let mut down = vec![0.0; n-1];

    for i in 1..n-1 {
        let (h0, h1) = (xs[i]-xs[i-1] ,xs[i+1]-xs[i]);

        (down[i-1], mid[i], up[i])=(h1, 2.0*(h1+h0), h0);
        b[i] = 3.0*((data[i]-data[i-1])*h1/h0 + (data[i+1]-data[i])*h0/h1);
    }

    match cstr {
        SplineConstrains::FirstDerivatives(start, end) => (b[0], b[n-1]) = (start, end),
        SplineConstrains::SecondDerivatives(start, end) => {
            // 6.0 * (data[i + 1] - data[i])/h - 4.0 * x[i] - 2.0 * x[i + 1] = k*h
            // 6.0 * (data[i + 1] - data[i])/h -k*h = 4.0 * x[i] + 2.0 * x[i + 1]
            (mid[0], up[0]) = (4.0, 2.0);
            b[0] = 6.0 * (data[1]-data[0])/(xs[1]-xs[0]) - start*(xs[1]-xs[0]);

            // (6(yn-1 - yn)/h + 2Dn-1 +4Dn) / h = k
            // kh + 6(yn - yn-1)/h = 2Dn-1 + 4Dn
            (down[n-2], mid[n-1]) = (2.0, 4.0);
            b[n-1] = 6.0 * (data[n-1] - data[n-2])/(xs[n-1] - xs[n-2]) + end*(xs[n-1] - xs[n-2]);
        },
    }

    let x = solve_tridiagonal_system(up, mid, down, b);

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
    for i in 0..n-1 {
        secants.push((data[i+1]-data[i])/(xs[i+1]-xs[i]));
    }

    let mut m = vec![0.0; n];
    for i in 1..n-1 {
        if secants[i-1]*secants[i]<0.0 {
            m[i] = 0.0;
        } else {
            m[i] = (secants[i-1] + secants[i])/2.0;
        }
    }
    m[0] = secants[0];
    m[n-1] = secants[n-2];
 
    for k in 0..n-1 {
        if secants[k] == 0.0 {
            m[k] = 0.0;
            m[k+1] = 0.0;
        }
    }

    let a: Vec<f32> = (0..secants.len()).map(|i| m[i]/secants[i]).collect();
    let b: Vec<f32> = (0..secants.len()).map(|i| m[i+1]/secants[i]).collect();

    for i in 0..n-1 {
        if a[i]*a[i] + b[i]*b[i] > 9.0 {
            let t = 3.0/((a[i]*a[i] + b[i]*b[i]).sqrt());
            m[i] = t*a[i]*secants[i];
            m[i+1] = t*b[i]*secants[i];
        }
    }

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
#[allow(unused_parens)]
pub fn apply_curve(val: f32, spline: &Vec<[f32; 4]>, x: &Vec<f32>) -> f32 {
    for i in 0..x.len() - 1 {
        if x[i] <= val && val < x[i + 1] {
            let z = (val - x[i]) / (x[i + 1] - x[i]);
            return (
                spline[i][0]
                + spline[i][1] * z
                + spline[i][2] * z * z
                + spline[i][3] * z * z * z
            );
        }
    }

    val
}
pub fn apply_1st_derivative(val: f32, spline: &Vec<[f32; 4]>, x: &Vec<f32>) -> f32 {
    for i in 0..x.len() - 1 {
        if x[i] <= val && val < x[i + 1] {
            let z = (val - x[i]) / (x[i + 1] - x[i]);
            return (
                spline[i][1]
                + 2.0*spline[i][2] * z
                + 3.0*spline[i][3] * z * z
            )/(x[i+1]-x[i]);
        }
    }

    val
}
pub fn apply_2nd_derivative(val: f32, spline: &Vec<[f32; 4]>, x: &Vec<f32>) -> f32 {
    for i in 0..x.len() - 1 {
        if x[i] <= val && val < x[i + 1] {
            let z = (val - x[i]) / (x[i + 1] - x[i]);
            return (2.0*spline[i][2] + 6.0*spline[i][3] * z)/(x[i+1]-x[i]).powi(2);
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


pub fn create_plot_view<S: plotters::style::SizeDesc + num_traits::NumCast>(
    buf: &mut [u8], 
    size: (u32, u32), 
    xs: &Vec<f32>, ys: &Vec<f32>,
    margins: Option<(S, S, S, S)>,
    coefficients: Option<&Vec<[f32; 4]>>
) -> Result<(), Box<dyn std::error::Error>> {

    let root = BitMapBackend::with_buffer(buf, size).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut builder = ChartBuilder::on(&root);
    if margins.is_some(){
        let m = margins.unwrap();
        builder
            .margin_top(m.0)
            .margin_right(m.1)
            .margin_bottom(m.2)
            .margin_left(m.3);
    };

    let mut chart = builder.build_cartesian_2d(0f32..*xs.last().unwrap() / 100.0, 0f32..*ys.last().unwrap() / 100.0)?;

    chart.configure_mesh().draw()?;

    let sc = spline_coefficients(ys, xs, SplineConstrains::FirstDerivatives(0.0, 0.0));

    chart
        .draw_series(LineSeries::new(
            (0..=5000).map(|x| x as f32/50.0).map(|x| (x / 100.0, {
                apply_curve(x, {
                    if coefficients.is_none() {
                        &sc
                    } else {
                        coefficients.unwrap()
                    }
                }, &xs) / 100.0
            })),
            &RED,
        ))?;

    //chart.draw_series((0..xs.len()).map(|i| Circle::new((xs[i] / 100.0, ys[i] / 100.0), 5, &RED)))?;

    return Ok(());
}

#[cfg(test)]
mod tests {
    use nalgebra::{DMatrix, DVector};
    use crate::spline::solve_tridiagonal_system;

    #[test]
    fn tridiagonal_solve_test() {
        let n = 23;
        let data: Vec<f32> = vec![
            0.0, 12.3, 32.3, 134.4, 13.4,
            52.7, 43.6, 38.9, 27.0, 15.3,
            22.0, 13.0, 23.3, 44.4, 64.6,
            178.4, 2.2, 5.5233, 55.76, 3.534,
            123.321, 123.42, 72.0].iter().map(|x| x/178.4).collect();

        // Solve via matrix inversion
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
        let x_inv = A.try_inverse().unwrap() * b.clone();

        // Solve via forward elimination and back substitution
        let up = vec![1.0; n-1];
        let mut mid = vec![4.0; n];
        let down = vec![1.0; n-1];
        mid[0]=2.0;
        mid[n-1]=2.0;
        let x_fwe = solve_tridiagonal_system(up, mid, down, b.data.into());

        for i in 0..n {
            assert!((x_fwe[i]-x_inv[i]).abs() < 0.000001);
        }
    }

    #[test]
    fn harder_tridiagonal_test() {
        let n = 15;

        let data: Vec<f32> = vec![
            0.0, 12.3, 32.3, 134.4, 13.4,
            52.7, 43.6, 38.9, 27.0, 15.3,
            22.0, 13.0, 23.3, 44.4, 64.6,
        ].iter().map(|x| x/178.4).collect();

        let up = vec![
            1.0, 23.0, 42.5, 123.9, 53.3,
            0.005, 23.555, 34.04, 49.24, 294.94,
            24.68, 0.5959, 34.68, 04.85,
        ];
        let mid = vec![
            4.0, 178.4, 2.2, 5.5233, 55.76,
            3.534, 123.321, 123.42, 72.0, 29.58,
            07.38, 348.85, 0.004, 12.45, 49.34
        ];
        let down = vec![
            1.0, 28.58, 23.52, 49.96 ,48.92,
            237.482, 12.23, 0.594, 0.0043, 23.53,
            58.853, 29.95, 57.37, 95.85,
        ];

        // Solve via matrix inversion
        let A = DMatrix::from_fn(n, n, |r, c| {
                match r as isize - c as isize {
                    -1 => up[r],
                    0 => mid[r],
                    1 => down[c],
                    _ => 0.0,
                }
        });
        let b = DVector::from(data);
        let x_inv = A.try_inverse().unwrap() * b.clone();

        // Solve via forward elimination and back substitution
        let x_fwe = solve_tridiagonal_system(up, mid, down, b.data.into());

        for i in 0..n {
            assert!((x_fwe[i]-x_inv[i]).abs() < 0.000001);
        }
    }
}