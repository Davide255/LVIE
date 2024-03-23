use plotters::prelude::*;
use plotters::prelude::full_palette::ORANGE_400;
use plotters::style::full_palette::{ORANGE_800};
use LVIElib::spline::{apply_1st_derivative, apply_2nd_derivative, apply_curve, monotone_spline_coefficients, spline_coefficients, SplineConstrains};

#[test]
fn plot() -> Result<(), Box<dyn std::error::Error>> {

    let xs = vec![0.0, 020.0, 050.0, 090.0, 100.0];
    let ys = vec![0.0, 070.0, 080.0, 090.0, 100.0];

    //let xs = vec![0.0, 20.0, 30.0, 100.0];
    //let ys = vec![80.0, 20.0, 10.0, 100.0];

    let root = BitMapBackend::new("spline-plot.png", (3000, 4500)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .build_cartesian_2d(0f32..1f32, -0.5f32..1f32)?;

    chart.configure_mesh().draw()?;

    let coeffs = spline_coefficients(&ys, &xs, SplineConstrains::FirstDerivatives(8.0, 2.0));
    chart
        .draw_series(LineSeries::new(
            (0..=5000).map(|x| x as f32/50.0).map(|x| (x/100.0, {
                apply_curve(x, &coeffs, &xs) / 100.0
            })),
            &RED,
        ))?;
    let coeffs = spline_coefficients(&ys, &xs, SplineConstrains::FirstDerivatives(0.0, 0.0));
    chart
        .draw_series(LineSeries::new(
            (0..=5000).map(|x| x as f32/50.0).map(|x| (x/100.0, {
                apply_curve(x, &coeffs, &xs) / 100.0
            })),
            &ORANGE_800,
        ))?;
    /*chart
        .draw_series(LineSeries::new(
            (0..=5000).map(|x| x as f32/50.0).map(|x| (x/100.0, {
                apply_1st_derivative(x, &coeffs, &xs) / 10.0
            })),
            &ORANGE_800,
        ))?;
    chart
        .draw_series(LineSeries::new(
            (0..=5000).map(|x| x as f32/50.0).map(|x| (x/100.0, {
                apply_2nd_derivative(x, &coeffs, &xs) / 10.0
            })),
            &ORANGE_400,
        ))?;*/

    chart
        .draw_series(LineSeries::new(
            (0..=500).map(|x| x as f32/5.0).map(|x| (x/100.0, apply_curve(x, &monotone_spline_coefficients(&ys, &xs), &xs)/100.0)),
            &BLUE,
        ))?;

    root.present()?;

    Ok(())
}