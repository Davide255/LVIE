#![allow(non_snake_case)]
mod ui;
use crate::ui::*;

use i_slint_backend_winit::WinitWindowAccessor;

use crate::raw_decoder::{decode, supported_formats};
use image::{ImageBuffer, Pixel, Primitive};
use img_processing::{collect_histogram_data, Max, _collect_histogram_data_old};
use num_traits::NumCast;
use slint::ComponentHandle;
use slint::{Image, Model, Rgba8Pixel, SharedPixelBuffer, SharedString, Weak};
use LVIElib::traits::ScaleImage;

use std::sync::{Arc, Mutex};
use std::{thread, time};

use itertools::{max, Itertools};

mod img_processing;
mod img_processing_generic;
mod raw_decoder;

mod core;
use crate::core::{CRgbaImage, Data, FilterType, Rendering};

mod settings;
use crate::settings::{keyboard_shortcuts, load_settings};

mod history;
use history::*;

fn maximize_ui(ui: LVIE) {
    ui.window()
        .with_winit_window(
            |winit_window: &i_slint_backend_winit::winit::window::Window| {
                winit_window.set_maximized(true);
                winit_window.set_title("LVIE");
            },
        )
        .expect("Failed to use winit!");
}

build_shortcuts!(
"editor"
>"file"
--"open":(|Window: LVIE, _args: &[&str]| Window.global::<ToolbarCallbacks>().invoke_open_file())
--"close":(|Window: LVIE, _args: &[&str]| Window.global::<ToolbarCallbacks>().invoke_close_window())
>"image"
--"rotate-90-deg":(|Window: LVIE, _args: &[&str]| Window.global::<ToolbarCallbacks>().invoke_rotate_90_deg())
>"preview"
--"undo":(|Window: LVIE, _args: &[&str]| Window.global::<ScreenCallbacks>().invoke_undo() )
--"redo":(|Window: LVIE, _args: &[&str]| Window.global::<ScreenCallbacks>().invoke_redo() )
);

use LVIElib::utils::{graph, GraphColor};

fn _create_hist<P>(buff: &ImageBuffer<P, Vec<P::Subpixel>>) -> [slint::Image; 4]
where
    P: Pixel,
    P::Subpixel: Primitive + Max + std::cmp::Eq + std::hash::Hash,
    std::ops::RangeInclusive<P::Subpixel>: IntoIterator,
    <std::ops::RangeInclusive<<P as Pixel>::Subpixel> as IntoIterator>::Item:
        num_traits::ToPrimitive,
{
    let hist = collect_histogram_data(&buff);

    let r: (Vec<u32>, Vec<u32>) = ((0..hist[0].len() as u32).collect_vec(), hist[0].clone());
    let g: (Vec<u32>, Vec<u32>) = ((0..hist[1].len() as u32).collect_vec(), hist[1].clone());
    let b: (Vec<u32>, Vec<u32>) = ((0..hist[2].len() as u32).collect_vec(), hist[2].clone());

    let size: (u32, u32) = (300, 300);

    let max = max([
        r.1.iter()
            .max_by(|x, y| x.partial_cmp(&y).unwrap())
            .unwrap(),
        g.1.iter()
            .max_by(|x, y| x.partial_cmp(&y).unwrap())
            .unwrap(),
        b.1.iter()
            .max_by(|x, y| x.partial_cmp(&y).unwrap())
            .unwrap(),
    ])
    .unwrap();

    let mut r_b = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(size.0, size.1);
    graph(
        r_b.make_mut_bytes(),
        size,
        &vec![&r.0],
        &vec![&r.1],
        &255,
        max,
        &vec![GraphColor::RED],
        (0, 0, 0, 0),
    )
    .expect("Failed to build the graph (R)");

    let mut g_b = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(size.0, size.1);
    graph(
        g_b.make_mut_bytes(),
        size,
        &vec![&g.0],
        &vec![&g.1],
        &255,
        max,
        &vec![GraphColor::GREEN],
        (0, 0, 0, 0),
    )
    .expect("Failed to build the graph (G)");

    let mut b_b = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(size.0, size.1);
    graph(
        b_b.make_mut_bytes(),
        size,
        &vec![&b.0],
        &vec![&b.1],
        &255,
        max,
        &vec![GraphColor::BLUE],
        (0, 0, 0, 0),
    )
    .expect("Failed to build the graph (B)");

    let mut all3 = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(size.0, size.1);
    graph(
        all3.make_mut_bytes(),
        size,
        &vec![&r.0, &g.0, &b.0],
        &vec![&r.1, &g.1, &b.1],
        &255,
        max,
        &vec![GraphColor::RED, GraphColor::GREEN, GraphColor::BLUE],
        (0, 0, 0, 0),
    )
    .expect("Failed to build the graph (RGB)");

    [
        slint::Image::from_rgb8(r_b),
        slint::Image::from_rgb8(g_b),
        slint::Image::from_rgb8(b_b),
        slint::Image::from_rgb8(all3),
    ]
}

fn _create_svg_path<P: Pixel>(buff: &ImageBuffer<P, Vec<P::Subpixel>>) -> [SharedString; 3]
where
    P: Pixel,
    P::Subpixel: Primitive + Max + std::cmp::Eq + std::hash::Hash + std::cmp::Ord,
    std::ops::RangeInclusive<P::Subpixel>: IntoIterator,
    <std::ops::RangeInclusive<<P as Pixel>::Subpixel> as IntoIterator>::Item:
        num_traits::ToPrimitive,
{
    let hist = _collect_histogram_data_old(&buff);
    let mut _v: Vec<SharedString> = Vec::new();
    for cmp in hist {
        let scale_factor: u32 = 1;
        let max_value: &u32 = &(cmp.values().max().unwrap() / scale_factor);

        let mut s_out: String = String::from(format!("M 0 {}", max_value));

        for k in cmp.keys().sorted() {
            s_out.push_str(&format!(
                " L {} {}",
                {
                    if k == &NumCast::from(0).unwrap() {
                        0u32
                    } else {
                        (<f32 as NumCast>::from(*k).unwrap() * (*max_value as f32 / 255f32)).round()
                            as u32
                    }
                },
                max_value - (cmp.get(k).unwrap() / scale_factor)
            ));
        }

        s_out.push_str(&format!(" L {max_value} {max_value} Z"));
        _v.push(s_out.into());
    }

    [
        _v.get(0).unwrap().clone(),
        _v.get(1).unwrap().clone(),
        _v.get(2).unwrap().clone(),
    ]
}

#[allow(unreachable_code)]
fn main() {
    const WINIT_BACKEND: bool = if cfg!(windows) { true } else { false };
    const INTERNAL_CLOCK_TIME: u64 = 2;

    let s: crate::settings::Settings = load_settings(None).unwrap();

    let CORE: Rendering<image::Rgba<u8>> = Rendering::init(s.backend);

    let HISTORY = Arc::new(Mutex::new(history::History::init(
        Some(std::path::PathBuf::from(s.temp_file_directory.clone())),
        s.use_temp_file,
        Some(s.max_mem_size),
    )));

    let SETTINGS = Arc::new(Mutex::new(s));

    if WINIT_BACKEND {
        slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap()))
            .expect("Failed to set winit backend!");
    }

    let Window: LVIE = LVIE::new().unwrap();

    let d = Data::new(CORE, None, None);

    let curves = &d.curve;
    Window.set_curve(curves.to_image((300, 300)));
    Window.set_curve_points(curves.into_rc_model());

    let DATA = Arc::new(Mutex::new(d));

    let CLOCK = Arc::new(Mutex::new(slint::Timer::default()));

    // CALLBACKS:
    // open image:
    let data_weak = DATA.clone();
    //let prev_w = preview.clone();
    let Window_weak = Window.as_weak();
    Window.global::<ToolbarCallbacks>().on_open_file(move || {
        // get the file with native file dialog
        let fd = rfd::FileDialog::new()
            .add_filter(
                "all image formats",
                &[
                    supported_formats().as_slice(),
                    ["jpg", "jpeg", "png"].as_slice(),
                ]
                .concat(),
            )
            .pick_file();
        if fd.is_none() {
            return;
        }
        let binding = fd.unwrap();

        let img: CRgbaImage<image::Rgba<u8>>;

        if supported_formats().contains(
            &binding
                .as_path()
                .extension()
                .unwrap()
                .to_str()
                .unwrap()
                .to_uppercase()
                .as_str(),
        ) {
            let buff = decode(binding.as_path());
            if buff.is_none() {
                println!("Cannot decode file {}", binding.as_path().display());
                return;
            }
            img = buff
                .unwrap()
                .scale_image::<image::Rgba<u16>, image::Rgba<u8>>();
        } else {
            let buff = image::open(binding.as_path().to_str().unwrap());
            if buff.is_err() {
                println!("Cannot decode file {}", binding.as_path().display());
                return;
            }
            img = buff.unwrap().to_rgba8();
        }

        let mut data = data_weak.lock().unwrap();

        // load the image
        data.load_image(img.clone());

        Window_weak
            .upgrade_in_event_loop(move |Window| {
                // loading the image into the UI
                let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    &img,
                    img.width(),
                    img.height(),
                );

                // create the histogram and update the UI
                let ww = Window.as_weak();
                thread::spawn(move || {
                    ww.upgrade_in_event_loop(move |window| {
                        let path = _create_hist(&img);
                        window.set_new_histogram(path.into());
                    })
                    .expect("Failed to run in event loop");
                });
                Window.set_image(Image::from_rgba8(pix_buf));
            })
            .expect("Failed to call from event loop");
    });

    let data_weak = DATA.clone();
    let Window_weak = Window.as_weak();
    let hw = HISTORY.clone();
    Window
        .global::<ToolbarCallbacks>()
        .on_rotate_90_deg(move || {
            let mut data = data_weak.lock().unwrap();
            data.rotation += 90.0;
            let mut history = hw.lock().unwrap();

            let img = image::imageops::rotate90(&data.full_res_preview);
            // load the image
            data.load_image(img.clone());

            history
                .register_Geometric_Operation_and_save(
                    &GeometricOperationType::Rotation(90.0),
                    &img,
                )
                .expect("Failed to load into history");

            //let lpw = prev_w.clone();
            Window_weak
                .upgrade_in_event_loop(move |Window| {
                    // loading the image into the UI
                    let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        &img,
                        img.width(),
                        img.height(),
                    );

                    Window.set_image(Image::from_rgba8(pix_buf));
                })
                .expect("Failed to call from event loop");
        });

    // close window: (quit the slint event loop)
    Window.global::<ToolbarCallbacks>().on_close_window(|| {
        slint::quit_event_loop().expect("Failed to stop the event loop");
    });

    let ww = Window.as_weak();
    let sw = SETTINGS.clone();
    Window.on_handle_shortcut(
        move |kvalue: SharedString, alt: bool, ctrl: bool, shift: bool| {
            let settings = sw.lock().unwrap();

            let kvalue: String = kvalue.to_lowercase();

            let mut modifiers: Vec<keyboard_shortcuts::MODIFIER> = Vec::new();
            if alt {
                modifiers.push(keyboard_shortcuts::MODIFIER::ALT);
            }
            if ctrl {
                modifiers.push(keyboard_shortcuts::MODIFIER::CTRL);
            }
            if shift {
                modifiers.push(keyboard_shortcuts::MODIFIER::SHIFT);
            }

            for key in &settings.keyboard_shortcuts {
                if key.is(&kvalue) {
                    if let Some(b) = key.get_binding_by_modifiers(&modifiers) {
                        // the pattern is 'editor.*.*'
                        let _ts = b.action().clone();
                        let action: Vec<&str> = _ts.split(".").collect_vec();
                        handle_shortcut_action(ww.clone(), action);
                    }
                }
            }
        },
    );

    let sw = SETTINGS.clone();
    Window
        .global::<SettingsCallbacks>()
        .on_load_settings(move || {
            let settings = sw.lock().unwrap();
            ((
                SharedString::from(settings.backend.name()),
                settings.max_mem_size as i32,
                settings.use_temp_file,
                SharedString::from(settings.temp_file_directory.clone()),
                settings.start_maximized,
            ),)
        });

    //reset
    let data_weak = DATA.clone();
    let Window_weak = Window.as_weak();
    let hw = HISTORY.clone();
    Window.global::<ScreenCallbacks>().on_reset(move || {
        let mut data = data_weak.lock().expect("Failed to lock data");

        hw.lock()
            .unwrap()
            .register_Logic_Operation_without_saving(&LogicOperationType::Reset(
                data.get_loaded_filters().clone(),
            ));

        // restore filters
        data.reset();

        // restore all the previews to the original image
        let img = data.full_res_preview.clone();

        Window_weak
            .upgrade_in_event_loop(move |Window: LVIE| {
                Window.set_image(Image::from_rgba8(
                    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        &img,
                        img.width(),
                        img.height(),
                    ),
                ));

                let ww = Window.as_weak();
                thread::spawn(move || {
                    let path = _create_svg_path(&img);
                    ww.upgrade_in_event_loop(move |window| {
                        window.set_svg_path(path.into());
                    })
                    .expect("Failed to run in event loop");
                });
            })
            .expect("Failed to call event loop");
    });

    let ww = Window.as_weak();
    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window.global::<CurveCallbacks>().on_update_curve(
        move |points: slint::ModelRc<slint::ModelRc<f32>>| {
            let mut data = dw.lock().unwrap();

            let (mut xs, mut ys) = data.curve.get_raw_data();

            for (i, point) in points.iter().enumerate() {
                let p: Vec<f32> = point.iter().collect();
                if !(xs[i] == p[0] && ys[i] == p[1]) {
                    hw.lock().unwrap().register_Curve_Operation_without_saving(
                        &CurveOperationType::CurvePointMoved(i, xs.remove(i), ys.remove(i)),
                    );

                    xs.insert(i, p[0]);
                    ys.insert(i, p[1]);
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

            hw.lock()
                .unwrap()
                .register_Curve_Operation_without_saving(&CurveOperationType::CurvePointAdded(i));

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
                0 => core::CurveType::MONOTONE,
                1 => core::CurveType::SMOOTH,
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

    let d_w = DATA.clone();
    Window.global::<MaskCallbacks>().on_apply_mask(move || {
        let d = d_w.lock().unwrap();
        d.masks[0]
            .apply_to_image(&d.full_res_preview)
            .expect("Mask not closed")
            .save("applyed_mask.png")
            .expect("Failed to save");
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

    // apply filters
    let data_weak = DATA.clone();
    let Window_weak = Window.as_weak();
    let clock_w = CLOCK.clone();
    let hw = HISTORY.clone();
    Window.global::<ScreenCallbacks>().on_apply_filters(
        move |exposition: f32,
              box_blur: f32,
              gaussian_blur: f32,
              sharpening: f32,
              temp: f32,
              tint: f32,
              saturation: f32| {
            let mut data = data_weak.lock().expect("Failed to lock");

            if data.image_dimensions() == (0, 0) {
                return;
            }

            data.update_filter(FilterType::Exposition, vec![exposition]);
            data.update_filter(FilterType::Saturation, vec![saturation]);
            data.update_filter(FilterType::Sharpening, vec![sharpening, 5.0]);
            data.update_filter(FilterType::Boxblur, vec![box_blur, 5.0]);
            data.update_filter(FilterType::GaussianBlur, vec![gaussian_blur, 5.0]);
            data.update_filter(
                FilterType::WhiteBalance,
                vec![2000f32 * temp + 6000f32, tint * 50.0],
            );

            let processed = data.update_image();

            let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &data.full_res_preview,
                data.image_dimensions().0,
                data.image_dimensions().1,
            );

            Window_weak
                .upgrade_in_event_loop(move |Window: LVIE| {
                    Window.set_image(Image::from_rgba8(pix_buf));
                })
                .expect("Failed to call event loop");

            let ww = Window_weak.clone();
            thread::spawn(move || {
                let path = _create_svg_path(&processed);
                ww.upgrade_in_event_loop(|Window| Window.set_svg_path(path.into()))
                    .expect("Cannot update the histogram");
            });

            if !cfg!(debug_assertions) {
                let clock = clock_w.lock().unwrap();
                if clock.running() {
                    clock.restart();
                } else {
                    let dw = data_weak.clone();
                    let hw = hw.clone();
                    clock.start(
                        slint::TimerMode::SingleShot,
                        std::time::Duration::from_secs(INTERNAL_CLOCK_TIME),
                        move || {
                            let mut data = dw.lock().unwrap();
                            hw.lock()
                                .unwrap()
                                .register_Filter_Operation_and_save(
                                    data.get_loaded_filters(),
                                    &data.full_res_preview,
                                )
                                .expect("Failed to save the history");
                            data.update_all_color_spaces();
                        },
                    );
                }
            }
        },
    );

    Window.global::<Linespace>().on_create_line(
        move |from_x: f32, from_y: f32, to_x: f32, to_y: f32, steps: i32| {
            let mut out: Vec<slint::ModelRc<f32>> = Vec::new();

            let s = (
                (to_x - from_x) / steps as f32,
                (to_y - from_y) / steps as f32,
            );

            for k in 0..steps {
                out.push(slint::ModelRc::new(slint::VecModel::from(vec![
                    from_x + s.0 * k as f32,
                    from_y + s.1 * k as f32,
                ])));
            }

            slint::ModelRc::new(slint::VecModel::from(out))
        },
    );

    let ww = Window.as_weak();
    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window.global::<ScreenCallbacks>().on_undo(move || {
        let mut history = hw.lock().unwrap();
        let mut data = dw.lock().unwrap();

        if history.can_undo() {
            let op = history.undo().unwrap();
            let img = match op.get_type() {
                &history::OperationType::Filter => {
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
                &history::OperationType::Geometric => {
                    let nop = op.as_ref().downcast_ref::<GeometricOperation>().unwrap();
                    match nop.get_content() {
                        &GeometricOperationType::Rotation(x) => {
                            data.rotation -= x;
                            if history.preview_aviable() {
                                let out = history.get_precomputed_preview().unwrap().unwrap();
                                data.load_image(out.clone());
                                out
                            } else {
                                let new_image = image::imageops::rotate270(&data.full_res_preview);
                                data.load_image(new_image.clone());
                                new_image
                            }
                        }
                        &GeometricOperationType::Traslation(_ox, _oy) => {
                            todo!()
                        }
                    }
                }
                &history::OperationType::Logic => {
                    let nop = op.as_ref().downcast_ref::<LogicOperation>().unwrap();
                    todo!()
                }
                &history::OperationType::Mask => {
                    let nop = op.as_ref().downcast_ref::<MaskOperation>().unwrap();
                    let (mask_number, content) = nop.get_content();

                    match content {
                        &MaskOperationType::MainPointMoved(index, ox, oy) => {
                            data.masks[*mask_number]
                                .update_point(index, [ox, oy])
                                .expect("Failed to update mask point");
                        }
                        &MaskOperationType::ControlPointMoved(index, subindex, ox, oy) => {
                            data.masks[*mask_number]
                                .update_control_point([index, subindex], [ox, oy])
                                .expect("Failed to update control point");
                        }
                        &MaskOperationType::MainPointAdded(index) => {
                            data.masks[*mask_number]
                                .remove_point(index)
                                .expect("Failed to remove point");
                        }
                        &MaskOperationType::MainPointRemoved(index, ox, oy) => {
                            data.masks[*mask_number].add_point_at_index([ox, oy], index);
                        }
                        &MaskOperationType::MaskClosed() => {
                            data.masks[*mask_number].undo_close();
                        }
                    }

                    let W = ww.unwrap();
                    W.set_mask_points(data.masks[0].into_rc_model());
                    W.set_bezier_control_points(data.masks[0].get_control_points_model_rc());
                    W.set_connection_line_points(data.masks[0].generate_line_for_slint(None, None));
                    W.set_control_point_connection_line(
                        data.masks[0].generate_control_point_connection_lines_for_slint(),
                    );

                    return;
                }
                &history::OperationType::Curve => {
                    let nop = op.as_ref().downcast_ref::<CurveOperation>().unwrap();
                    todo!()
                }
            };

            ww.upgrade_in_event_loop(move |Window| {
                // loading the image into the UI
                let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    &img,
                    img.width(),
                    img.height(),
                );

                Window.set_image(Image::from_rgba8(pix_buf));
            })
            .expect("Failed to call from event loop");
        }
    });

    //set_Alert_Message
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_set_Warning_Message(
        move |message: slint::SharedString| {
            let ui = Window_weak.unwrap();
            ui.set_AlertBoxType(AlertType::Warning);
            ui.set_AlertText(message);
        },
    );

    //save
    let data_weak = DATA.clone();
    Window
        .global::<ScreenCallbacks>()
        .on_save_file(move |path: SharedString| {
            data_weak
                .lock()
                .unwrap()
                .export()
                .save(path.as_str())
                .expect("Failed to save file");
        });

    // startup procedure
    let l_weak: Weak<LVIE> = Window.as_weak();

    {
        if WINIT_BACKEND && SETTINGS.lock().unwrap().start_maximized {
            thread::Builder::new()
                .name("waiter".to_string())
                .spawn(move || {
                    thread::sleep(time::Duration::from_millis(100));
                    l_weak
                        .upgrade_in_event_loop(move |handle| {
                            maximize_ui(handle);
                        })
                        .expect("Failed to call from the main thread");
                })
                .expect("Failed to spawn thread");
        }
    }

    let _ = Window.show();
    slint::run_event_loop().expect("Failed to create the event loop");
    let _ = Window.hide();
}
