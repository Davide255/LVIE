use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Weak};

use crate::history::{History, MaskOperationType};

use super::super::{
    super::ui::{MaskCallbacks, LVIE},
    Data,
};

pub fn init_mask_callbacks<P>(
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
        + Default
        + LVIElib::traits::AsFloat,
{
    let Window = Window.unwrap();

    let d_w = DATA.clone();
    Window.global::<MaskCallbacks>().on_apply_mask(move || {
        let d = d_w.lock().unwrap();
        d.masks[0]
            .apply_to_image(&d.full_res_preview)
            .expect("Mask not closed");
    });

    let dw = DATA.clone();
    let ww = Window.as_weak();
    let hw = HISTORY.clone();
    Window.global::<MaskCallbacks>().on_add_mask_point(
        move |x: f32, y: f32, width: f32, height: f32| {
            let mut d = dw.lock().unwrap();
            let i = d.masks[0].add_point([x, y]);

            hw.lock().unwrap().register_Mask_Operation_without_saving(
                &((0, MaskOperationType::MainPointAdded(i))),
            );

            let Window = ww.unwrap();
            Window.set_mask_points(d.masks[0].into_rc_model());
            Window.set_connection_line_points(
                d.masks[0].generate_line_for_slint(Some(width), Some(height)),
            );
            Window.set_bezier_control_points(d.masks[0].get_control_points_model_rc());
            Window.set_control_point_connection_line(
                d.masks[0].generate_control_point_connection_lines_for_slint(),
            );
            return i.try_into().unwrap();
        },
    );

    let d_w = DATA.clone();
    Window.global::<MaskCallbacks>().on_there_is_a_mask_point(
        move |x: f32, y: f32, width: f32, height: f32, size: f32| {
            let mut p_number = -1;

            let data = d_w.try_lock().unwrap();

            let cps = data.masks[0].get_points();

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
    Window
        .global::<MaskCallbacks>()
        .on_there_is_a_control_point(move |x: f32, y: f32, width: f32, height: f32, size: f32| {
            let mut p_number = -1;

            let data = d_w.try_lock().unwrap();

            let cps = data.masks[0].get_control_points();

            for (k, coords) in cps.iter().enumerate() {
                for (i, coord) in coords.iter().enumerate() {
                    let xr = width * coord[0] / 100.0 - size / 2.0;
                    let yr = height * (100.0 - coord[1]) / 100.0 - size / 2.0;

                    if xr <= x && x <= xr + size && yr <= y && y <= yr + size {
                        p_number = (k * 10 + i) as i32;
                        break;
                    }
                }
            }

            return p_number;
        });

    let ww = Window.as_weak();
    let dw = DATA.clone();
    Window
        .global::<MaskCallbacks>()
        .on_update_mask(move |width: f32, height: f32| {
            let mut data = dw.lock().unwrap();

            if data.masks[0].is_empty() {
                return;
            }

            let W = ww.unwrap();
            W.set_mask_points(data.masks[0].into_rc_model());
            W.set_bezier_control_points(data.masks[0].get_control_points_model_rc());
            W.set_connection_line_points(
                data.masks[0].generate_line_for_slint(Some(width), Some(height)),
            );
            W.set_control_point_connection_line(
                data.masks[0].generate_control_point_connection_lines_for_slint(),
            );
        });

    let ww = Window.as_weak();
    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window.global::<MaskCallbacks>().on_update_mask_point(
        move |index: i32, x: f32, y: f32, width: f32, height: f32| {
            let mut data = dw.lock().unwrap();
            match data.masks[0].update_point(index as usize, [x, y]) {
                Ok((x, y)) => {
                    hw.lock().unwrap().register_Mask_Operation_without_saving(&(
                        0,
                        MaskOperationType::MainPointMoved(index as usize, x, y),
                    ));

                    let W = ww.unwrap();
                    W.set_mask_points(data.masks[0].into_rc_model());
                    W.set_bezier_control_points(data.masks[0].get_control_points_model_rc());
                    W.set_connection_line_points(
                        data.masks[0].generate_line_for_slint(Some(width), Some(height)),
                    );
                    W.set_control_point_connection_line(
                        data.masks[0].generate_control_point_connection_lines_for_slint(),
                    );
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        },
    );

    let ww = Window.as_weak();
    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window.global::<MaskCallbacks>().on_update_control_point(
        move |index: i32, x: f32, y: f32, width: f32, height: f32| {
            let mut data = dw.lock().unwrap();
            match data.masks[0]
                .update_control_point([index as usize / 10, index as usize % 10], [x, y])
            {
                Ok((x, y)) => {
                    hw.lock().unwrap().register_Mask_Operation_without_saving(&(
                        0,
                        MaskOperationType::ControlPointMoved(
                            index as usize / 10,
                            index as usize % 10,
                            x,
                            y,
                        ),
                    ));

                    let W = ww.unwrap();
                    W.set_bezier_control_points(data.masks[0].get_control_points_model_rc());
                    W.set_connection_line_points(
                        data.masks[0].generate_line_for_slint(Some(width), Some(height)),
                    );
                    W.set_control_point_connection_line(
                        data.masks[0].generate_control_point_connection_lines_for_slint(),
                    );
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        },
    );

    let d_w = DATA.clone();
    let ww = Window.as_weak();
    let hw = HISTORY.clone();
    Window.global::<MaskCallbacks>().on_remove_mask_point(
        move |index: i32, width: f32, height: f32| {
            let mut d = d_w.lock().unwrap();
            match d.masks[0].remove_point(index as usize) {
                Ok((x, y)) => {
                    hw.lock().unwrap().register_Mask_Operation_without_saving(&(
                        0,
                        MaskOperationType::MainPointRemoved(index as usize, x, y),
                    ));

                    let Window = ww.unwrap();
                    Window.set_mask_points(d.masks[0].into_rc_model());
                    Window.set_bezier_control_points(d.masks[0].get_control_points_model_rc());
                    Window.set_connection_line_points(
                        d.masks[0].generate_line_for_slint(Some(width), Some(height)),
                    );
                    Window.set_control_point_connection_line(
                        d.masks[0].generate_control_point_connection_lines_for_slint(),
                    );
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        },
    );

    let d_w = DATA.clone();
    let ww = Window.as_weak();
    let hw = HISTORY.clone();
    Window
        .global::<MaskCallbacks>()
        .on_close_mask_path(move |width: f32, height: f32| {
            let mut d = d_w.lock().unwrap();
            if !d.masks[0].is_closed() {
                d.masks[0].close();

                hw.lock()
                    .unwrap()
                    .register_Mask_Operation_without_saving(&(0, MaskOperationType::MaskClosed()));

                let Window = ww.unwrap();
                Window.set_mask_points(d.masks[0].into_rc_model());
                Window.set_bezier_control_points(d.masks[0].get_control_points_model_rc());
                Window.set_connection_line_points(
                    d.masks[0].generate_line_for_slint(Some(width), Some(height)),
                );
                Window.set_control_point_connection_line(
                    d.masks[0].generate_control_point_connection_lines_for_slint(),
                );
            }
        });
}
