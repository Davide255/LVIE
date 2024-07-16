use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer, Weak};
use LVIElib::traits::ScaleImage;

use crate::{
    core::Mask,
    history::{History, *},
};

use super::super::{
    ui::{ScreenCallbacks, LVIE},
    Data,
};

pub fn init_history_callbacks<P>(
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
        + num_traits::ToBytes
        + LVIElib::traits::AsFloat,
{
    let Window = Window.unwrap();

    let ww = Window.as_weak();
    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window.global::<ScreenCallbacks>().on_undo(move || {
        let mut history = hw.lock().unwrap();
        let mut data = dw.lock().unwrap();

        if history.can_undo() {
            let op = history.undo().unwrap();
            let img = match op.get_type() {
                &OperationType::Filter => {
                    let nop = op.as_ref().downcast_ref::<FilterOperation>().unwrap();
                    data.update_filters(nop.get_content().clone());

                    if history.preview_aviable() {
                        data.norm_filters();
                        data.full_res_preview = history.get_precomputed_preview().unwrap().unwrap();
                        data.manual_reset_rendering();
                        data.full_res_preview.clone()
                    } else {
                        data.update_image()
                    }
                }
                &OperationType::Geometric => {
                    let nop = op.as_ref().downcast_ref::<GeometricOperation>().unwrap();
                    match nop.get_content() {
                        GeometricOperationType::Rotation(x) => {
                            data.rotation -= x;
                            if history.preview_aviable() {
                                let out = history.get_precomputed_preview().unwrap().unwrap();
                                data.load_image(out.clone(), false);
                                out
                            } else {
                                let new_image = image::imageops::rotate270(&data.full_res_preview);
                                data.load_image(new_image.clone(), false);
                                new_image
                            }
                        }
                        GeometricOperationType::Traslation(_ox, _oy) => {
                            todo!()
                        }
                    }
                }
                &OperationType::Logic => {
                    let nop = op.as_ref().downcast_ref::<LogicOperation>().unwrap();
                    match nop.get_content() {
                        LogicOperationType::Reset(filters) => {
                            data.update_filters(filters.clone());
                            data.update_image()
                        }
                        LogicOperationType::FileLoaded() => {
                            return;
                        }
                    }
                }
                &OperationType::Mask => {
                    let nop = op.as_ref().downcast_ref::<MaskOperation>().unwrap();
                    let (mask_number, content) = nop.get_content();

                    match content {
                        &MaskOperationType::MainPointMoved(index, ox, oy, _, _) => {
                            data.masks[*mask_number]
                                .update_point(index, [ox, oy])
                                .expect("Failed to update mask point");
                        }
                        &MaskOperationType::ControlPointMoved(index, subindex, ox, oy, _, _) => {
                            data.masks[*mask_number]
                                .update_control_point([index, subindex], [ox, oy])
                                .expect("Failed to update control point");
                        }
                        &MaskOperationType::MainPointAdded(index, _, _) => {
                            data.masks[*mask_number]
                                .remove_point(index)
                                .expect("Failed to remove point");
                        }
                        &MaskOperationType::MainPointRemoved(index, ox, oy) => {
                            data.masks[*mask_number].add_point_at_index([ox, oy], index);
                        }
                        &MaskOperationType::MaskOpened(_, _) => {
                            data.masks.remove(*mask_number);
                        }
                        MaskOperationType::MaskClosed() => {
                            data.masks[*mask_number].undo_close();
                        }
                    }

                    let W = ww.unwrap();
                    W.set_mask_points(data.masks[0].into_rc_model());
                    W.set_bezier_control_points(data.masks[0].get_control_points_model_rc());
                    #[cfg(not(debug_assertions))]
                    {
                        W.set_connection_line_points(
                            data.masks[0].generate_line_for_slint(None, None),
                        );
                        W.set_control_point_connection_line(
                            data.masks[0].generate_control_point_connection_lines_for_slint(),
                        );
                    }

                    return;
                }
                &OperationType::Curve => {
                    let nop = op.as_ref().downcast_ref::<CurveOperation>().unwrap();
                    match nop.get_content() {
                        &CurveOperationType::CurvePointAdded(index, _, _) => {
                            data.curve
                                .remove_point(index)
                                .expect("Failed to remove point");
                        }
                        &CurveOperationType::CurvePointMoved(index, ox, oy, _, _) => {
                            data.curve
                                .update_curve_point(index, [ox, oy])
                                .expect("Failed to set point value");
                        }
                        &CurveOperationType::CurvePointRemoved(_, ox, oy) => {
                            data.curve.add_point([ox, oy]).expect("Failed to add point");
                        }
                        &CurveOperationType::CurveTypeChanged(c_type) => {
                            data.curve.set_curve_type(c_type);
                        }
                    }

                    let W = ww.unwrap();
                    W.set_curve(data.curve.to_image((300, 300)));
                    W.set_curve_points(data.curve.into_rc_model());

                    return;
                }
            };

            ww.upgrade_in_event_loop(move |Window| {
                // loading the image into the UI
                let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    &img.scale_image::<P, image::Rgba<u8>>(),
                    img.width(),
                    img.height(),
                );

                Window.set_image(slint::Image::from_rgba8(pix_buf));
            })
            .expect("Failed to call from event loop");
        }
    });

    let ww = Window.as_weak();
    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window.global::<ScreenCallbacks>().on_redo(move || {
        let mut history = hw.lock().unwrap();
        let mut data = dw.lock().unwrap();

        if history.can_redo() {
            let op = history.redo().unwrap();
            let img = match op.get_type() {
                &OperationType::Filter => {
                    let nop = op.as_ref().downcast_ref::<FilterOperation>().unwrap();
                    data.update_filters(nop.get_content().clone());

                    if history.preview_aviable() {
                        data.norm_filters();
                        data.full_res_preview = history.get_precomputed_preview().unwrap().unwrap();
                        data.manual_reset_rendering();
                        data.full_res_preview.clone()
                    } else {
                        data.update_image()
                    }
                }
                &OperationType::Geometric => {
                    let nop = op.as_ref().downcast_ref::<GeometricOperation>().unwrap();
                    match nop.get_content() {
                        GeometricOperationType::Rotation(x) => {
                            data.rotation += x;
                            if history.preview_aviable() {
                                let out = history.get_precomputed_preview().unwrap().unwrap();
                                data.load_image(out.clone(), false);
                                out
                            } else {
                                let new_image = image::imageops::rotate90(&data.full_res_preview);
                                data.load_image(new_image.clone(), false);
                                new_image
                            }
                        }
                        GeometricOperationType::Traslation(_ox, _oy) => {
                            todo!()
                        }
                    }
                }
                &OperationType::Logic => {
                    let nop = op.as_ref().downcast_ref::<LogicOperation>().unwrap();
                    match nop.get_content() {
                        LogicOperationType::Reset(_) => {
                            data.reset();
                            data.full_res_preview.clone()
                        }
                        LogicOperationType::FileLoaded() => {
                            return;
                        }
                    }
                }
                &OperationType::Mask => {
                    let nop = op.as_ref().downcast_ref::<MaskOperation>().unwrap();
                    let (mask_number, content) = nop.get_content();

                    match content {
                        &MaskOperationType::MainPointMoved(index, _, _, x, y) => {
                            data.masks[*mask_number]
                                .update_point(index, [x, y])
                                .expect("Failed to update mask point");
                        }
                        &MaskOperationType::ControlPointMoved(index, subindex, _, _, x, y) => {
                            data.masks[*mask_number]
                                .update_control_point([index, subindex], [x, y])
                                .expect("Failed to update control point");
                        }
                        &MaskOperationType::MainPointAdded(index, x, y) => {
                            data.masks[*mask_number].add_point_at_index([x, y], index);
                        }
                        &MaskOperationType::MainPointRemoved(index, _, _) => {
                            data.masks[*mask_number]
                                .remove_point(index)
                                .expect("Failed to remove point");
                        }
                        &MaskOperationType::MaskOpened(x, y) => {
                            let mut m = Mask::new();
                            m.add_point([x, y]);
                            data.masks.insert(*mask_number, m);
                        }
                        MaskOperationType::MaskClosed() => {
                            data.masks[*mask_number].undo_close();
                        }
                    }

                    let W = ww.unwrap();
                    W.set_mask_points(data.masks[0].into_rc_model());
                    W.set_bezier_control_points(data.masks[0].get_control_points_model_rc());
                    #[cfg(not(debug_assertions))]
                    {
                        W.set_connection_line_points(
                            data.masks[0].generate_line_for_slint(None, None),
                        );
                        W.set_control_point_connection_line(
                            data.masks[0].generate_control_point_connection_lines_for_slint(),
                        );
                    }

                    return;
                }
                &OperationType::Curve => {
                    let nop = op.as_ref().downcast_ref::<CurveOperation>().unwrap();
                    match nop.get_content() {
                        &CurveOperationType::CurvePointAdded(_, x, y) => {
                            data.curve.add_point([x, y]).expect("Failed to add point");
                        }
                        &CurveOperationType::CurvePointMoved(index, _, _, x, y) => {
                            data.curve
                                .update_curve_point(index, [x, y])
                                .expect("Failed to set point value");
                        }
                        &CurveOperationType::CurvePointRemoved(index, _, _) => {
                            data.curve
                                .remove_point(index)
                                .expect("Failed to remove point");
                        }
                        &CurveOperationType::CurveTypeChanged(c_type) => {
                            data.curve.set_curve_type(c_type);
                        }
                    }

                    let W = ww.unwrap();
                    W.set_curve(data.curve.to_image((300, 300)));
                    W.set_curve_points(data.curve.into_rc_model());

                    return;
                }
            };

            ww.upgrade_in_event_loop(move |Window| {
                // loading the image into the UI
                let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    &img.scale_image::<P, image::Rgba<u8>>(),
                    img.width(),
                    img.height(),
                );

                Window.set_image(slint::Image::from_rgba8(pix_buf));
            })
            .expect("Failed to call from event loop");
        }
    });
}
