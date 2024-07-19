use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Model, Weak};

use crate::history::{CurveOperationType, History};

use super::super::{
    super::ui::{CurveCallbacks, LVIE},
    Data,
};

pub fn init_curve_callbacks<P>(
    Window: Weak<LVIE>,
    DATA: Arc<Mutex<Data<P>>>,
    HISTORY: Arc<Mutex<History>>,
) where
    P: image::Pixel
        + Send
        + Sync
        + std::fmt::Debug
        + LVIElib::traits::ToHsl
        + LVIElib::traits::ToOklab
        + 'static,
    P::Subpixel: LVIElib::traits::Scale
        + image::Primitive
        + std::fmt::Debug
        + bytemuck::Pod
        + Send
        + Sync
        + LVIElib::traits::AsFloat
        + num_traits::ToBytes,
{
    let Window = Window.unwrap();

    let ww = Window.as_weak();
    let dw = DATA.clone();
    Window.global::<CurveCallbacks>().on_update_curve(
        move |points: slint::ModelRc<slint::ModelRc<f32>>| {
            let mut data = dw.lock().unwrap();

            let (mut xs, mut ys) = data.curve.get_raw_data();

            for (i, point) in points.iter().enumerate() {
                let p: Vec<f32> = point.iter().collect();
                if !(xs[i] == p[0] && ys[i] == p[1]) {
                    xs[i] = p[0];
                    ys[i] = p[1];
                }
            }

            data.curve.update_curve(xs, ys);

            let W = ww.unwrap();

            W.set_curve(data.curve.to_image((300, 300)));
            W.set_curve_points(data.curve.into_rc_model());
        },
    );

    let dw = DATA.clone();
    let ww = Window.as_weak();
    let hw = HISTORY.clone();
    Window
        .global::<CurveCallbacks>()
        .on_add_curve_point(move |x: f32, y: f32| {
            let mut d = dw.lock().unwrap();
            let i = d.curve.add_point([x, y]).expect("Failed to add a point");

            hw.lock().unwrap().register_Curve_Operation_without_saving(
                &CurveOperationType::CurvePointAdded(i, x, y),
            );

            let Window = ww.unwrap();
            Window.set_curve(d.curve.to_image((300, 300)));
            Window.set_curve_points(d.curve.into_rc_model());

            return i.try_into().unwrap();
        });

    let d_w = DATA.clone();
    Window.global::<CurveCallbacks>().on_there_is_a_curve_point(
        move |x: f32, y: f32, width: f32, height: f32, size: f32| {
            let mut p_number = -1;

            let data = d_w.try_lock().unwrap();

            let cps = data.curve.get_points();

            for (i, coords) in cps.iter().enumerate() {
                let xr = width * coords[0] / 100.0 - size / 2.0;
                let yr = height * (100.0 - coords[1]) / 100.0 - size / 2.0;

                if xr <= x && x <= xr + size && yr <= y && y <= yr + size {
                    p_number = i as i32;
                    break;
                }
            }

            return p_number;
        },
    );

    let d_w = DATA.clone();
    let ww = Window.as_weak();
    let hw = HISTORY.clone();
    Window
        .global::<CurveCallbacks>()
        .on_remove_curve_point(move |index: i32| {
            let mut d = d_w.lock().unwrap();
            match d.curve.remove_point(index as usize) {
                Ok((x, y)) => {
                    hw.lock().unwrap().register_Curve_Operation_without_saving(
                        &CurveOperationType::CurvePointRemoved(index as usize, x, y),
                    );

                    let Window = ww.unwrap();
                    Window.set_curve(d.curve.to_image((300, 300)));
                    Window.set_curve_points(d.curve.into_rc_model());
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        });

    let d_w = DATA.clone();
    let ww = Window.as_weak();
    let hw = HISTORY.clone();
    Window
        .global::<CurveCallbacks>()
        .on_set_curve_type(move |curve_type: i32| {
            let mut d = d_w.lock().unwrap();

            let new_c_type = match curve_type {
                0 => crate::core::CurveType::MONOTONE,
                1 => crate::core::CurveType::SMOOTH,
                _ => unimplemented!(),
            };

            if d.curve.get_curve_type() != &new_c_type {
                hw.lock().unwrap().register_Curve_Operation_without_saving(
                    &CurveOperationType::CurveTypeChanged(d.curve.get_curve_type().clone()),
                );
                d.curve.set_curve_type(new_c_type);
                ww.unwrap().set_curve(d.curve.to_image((300, 300)));
            }
        });

    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window
        .global::<CurveCallbacks>()
        .on_update_history(move |index, x, y| {
            let data = dw.lock().unwrap();
            let p = data.curve.get_point(index as usize);

            hw.lock().unwrap().register_Curve_Operation_without_saving(
                &CurveOperationType::CurvePointMoved(index as usize, x, y, p[0], p[1]),
            )
        });
}
