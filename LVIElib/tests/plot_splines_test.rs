use plotters::prelude::*;
use LVIElib::spline::{apply_curve, monotone_spline_coefficients, spline_coefficients};

#[test]
fn plot() -> Result<(), Box<dyn std::error::Error>> {

    let xs = vec![0.0, 020.0, 050.0, 090.0, 100.0];
    let ys = vec![0.0, 070.0, 080.0, 090.0, 100.0];

    let root = BitMapBackend::new("spline-plot.png", (4000, 4000)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .build_cartesian_2d(0f32..1f32, -1f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (0..=5000).map(|x| x as f32/50.0).map(|x| (x/100.0, apply_curve(x, &spline_coefficients(&ys, &xs), &xs)/100.0)),
            &RED,
        ))?;

    chart
        .draw_series(LineSeries::new(
            (0..=500).map(|x| x as f32/5.0).map(|x| (x/100.0, apply_curve(x, &monotone_spline_coefficients(&ys, &xs), &xs)/100.0)),
            &BLUE,
        ))?;

    root.present()?;

    Ok(())
}