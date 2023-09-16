#![allow(non_snake_case)]
slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use image::RgbImage;
pub mod hsl;
use img_processing::{build_low_res_preview, collect_histogram_data};
use slint::{Image, Rgb8Pixel, SharedPixelBuffer, SharedString, Weak};

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use rfd::FileDialog;

use itertools::Itertools;

mod history;
mod img_processing;
mod loading;

use crate::img_processing::Filters;

fn maximize_ui(ui: LVIE) {
    ui.window()
        .with_winit_window(|winit_window: &winit::window::Window| {
            winit_window.set_maximized(true);
            winit_window.set_title("LVIE");
        })
        .expect("Failed to use winit!");
}

fn _create_svg_path(buff: &RgbImage) -> [SharedString; 3] {
    let hist = collect_histogram_data(&buff);
    let mut _v: Vec<SharedString> = Vec::new();
    for cmp in hist {
        let scale_factor: u32 = 1000;
        let max_value: &u32 = &(cmp.values().max().unwrap() / scale_factor);

        let mut s_out: String = String::from(format!("M 0 {}", max_value));

        for k in cmp.keys().sorted() {
            s_out.push_str(&format!(
                " L {} {}",
                {
                    if k == &0u8 {
                        0u32
                    } else {
                        ((*k as f32) * (*max_value as f32 / 255f32)).round() as u32
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
    const WINIT_BACKEND: bool = {
        if cfg!(windows) {
            true
        } else {
            false
        }
    };

    if WINIT_BACKEND {
        slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new()))
            .expect("Failed to set winit backend!");
    }

    let Window: LVIE = LVIE::new().unwrap();

    let loaded_image = Arc::new(Mutex::new(image::RgbImage::new(0, 0)));
    let low_res_preview = Arc::new(Mutex::new(image::RgbImage::new(0, 0)));

    // CALLBACKS:
    // open image:
    let img_weak = Arc::clone(&loaded_image);
    let low_res_weak = Arc::clone(&low_res_preview);
    let Window_weak = Window.as_weak();
    Window
        .global::<ToolbarCallbacks>()
        .on_open_file_callback(move || {
            let fd = FileDialog::new()
                .add_filter("jpg", &["jpg", "jpeg", "png"])
                .pick_file();
            let binding = fd.unwrap();
            let img =
                image::open(binding.as_path().to_str().unwrap()).expect("Failed to open the image");
            let mut _mt = img_weak.lock().expect("Cannot lock mutex");
            *_mt = img.to_rgb8();
            let mut _low_res = low_res_weak.lock().expect("Failed to lock");
            *_low_res = build_low_res_preview(img.to_rgb8());
            Window_weak
                .upgrade_in_event_loop(move |Window| {
                    let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                        img.as_rgb8().unwrap(),
                        img.width(),
                        img.height(),
                    );
                    let ww = Window.as_weak();
                    thread::spawn(move || {
                        let path = _create_svg_path(&img.to_rgb8());
                        ww.upgrade_in_event_loop(move |window| {
                            window.set_svg_path(path.into());
                        })
                        .expect("Failed to run in event loop");
                    });
                    Window.set_image(Image::from_rgb8(pix_buf));
                })
                .expect("Failed to call from event loop");
        });

    // close window:
    Window
        .global::<ToolbarCallbacks>()
        .on_close_window_callback(|| {
            slint::quit_event_loop().expect("Failed to stop the event loop");
        });

    // box blur filter
    let img_weak = Arc::clone(&loaded_image);
    let low_res_weak = Arc::clone(&low_res_preview);
    let Window_weak = Window.as_weak();
    Window
        .global::<ScreenCallbacks>()
        .on_add_box_blur(move |sigma: i32| {
            let mut low_res_kernel = Filters::BoxBlur((sigma / 3) as u32);
            //low res preview
            let processed = img_processing::apply_filter(
                low_res_weak.lock().unwrap().deref().clone(),
                &mut low_res_kernel,
            );
            Window_weak
                .upgrade_in_event_loop(move |Window: LVIE| {
                    let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                        &processed,
                        processed.width(),
                        processed.height(),
                    );
                    Window.set_image(Image::from_rgb8(pix_buf));
                    Window.set_AlertBoxType(AlertType::Warning);
                    Window.set_AlertText("Low Res preview".into());
                    let ww = Window.as_weak();
                    thread::spawn(move || {
                        let path = _create_svg_path(&processed);
                        ww.upgrade_in_event_loop(move |window| {
                            window.set_svg_path(path.into());
                        })
                        .expect("Failed to run in event loop");
                    });
                })
                .expect("Failed to call event loop");

            let mut kernel = Filters::BoxBlur(sigma as u32);
            let _w_w = Window_weak.clone();
            let _i_w = img_weak.clone();
            thread::spawn(move || {
                // full res
                let processed =
                    img_processing::apply_filter(_i_w.lock().unwrap().deref().clone(), &mut kernel);
                _w_w.upgrade_in_event_loop(move |Window: LVIE| {
                    let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                        &processed,
                        processed.width(),
                        processed.height(),
                    );
                    Window.set_image(Image::from_rgb8(pix_buf));
                    Window.set_AlertBoxType(AlertType::Null);
                    let ww = Window.as_weak();
                    thread::spawn(move || {
                        let path = _create_svg_path(&processed);
                        ww.upgrade_in_event_loop(move |window| {
                            window.set_svg_path(path.into());
                        })
                        .expect("Failed to run in event loop");
                    });
                })
                .expect("Failed to call event loop");
            });
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

    // startup procedure
    let l_weak: Weak<LVIE> = Window.as_weak();

    if WINIT_BACKEND {
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

    let _ = Window.show();
    slint::run_event_loop().expect("Cannnot run the evnt loop due to an error!");
    let _ = Window.hide();
}
