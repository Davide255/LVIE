#![allow(non_snake_case)]
slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use image::RgbImage;
use crate::img_processing::crop;
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
        let scale_factor: u32 = 1600;
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
    let loaded_preview = Arc::new(Mutex::new(image::RgbImage::new(0, 0)));
    let low_res_preview = Arc::new(Mutex::new(image::RgbImage::new(0, 0)));

    // CALLBACKS:
    // open image:
    let img_weak = Arc::clone(&loaded_image);
    let prev_weak = Arc::clone(&loaded_preview);
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
            crop(&img.to_rgb8(), 500, 500, 3000, 2000).save("prova.png").expect("Cannot save");
            let mut _mt = img_weak.lock().expect("Cannot lock mutex");
            *_mt = img.to_rgb8();
            let mut _low_res = low_res_weak.lock().expect("Failed to lock");
            *_low_res = build_low_res_preview(&img.to_rgb8());
            let mut _prev = prev_weak.lock().unwrap();
            *_prev = img.to_rgb8();
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

    //reset
    let img_weak = Arc::clone(&loaded_image);
    //let low_res_weak = Arc::clone(&low_res_preview);
    let prev_weak = Arc::clone(&loaded_preview);
    let low_res_weak = Arc::clone(&low_res_preview);
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_reset(move || {
        let img = img_weak.lock().unwrap().deref().clone();
        let mut _prev = prev_weak.lock().unwrap();
        *_prev = img.clone();
        Window_weak.unwrap().set_image(Image::from_rgb8(
            SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(&img, img.width(), img.height()),
        ));
        let mut lp = low_res_weak.lock().unwrap();
        *lp = build_low_res_preview(&img);
    });

    let prev_weak = Arc::clone(&loaded_preview);
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_preview_click(move|width: f32, height: f32, x: f32, y: f32| {
        let mut img = prev_weak.lock().unwrap();
        // check if there is an image loaded
        if img.dimensions() == (0, 0) { return; }

        let (real_w, real_h) = (*img).dimensions();
        let new_width = real_w - (real_w / 100u32 * 20u32);
        let new_height = (real_h * new_width) / real_w;

        let mut pos:(u32, u32) = (0u32, 0u32);

        // computation coefficients
        let coefficient = real_w / (width.round() as u32);
        let adjustement: u32 = (height.round() as u32 - (real_h * width.round() as u32) / real_w) / 2;

        let x = (x.round() as u32) * coefficient;
        let y = ({
            if adjustement <= (y.round() as u32) { y.round() as u32 - adjustement } else { 0u32 }
        }) * coefficient;
        
        if x < (new_width / 2) {
            pos.0 = 0u32;
        } else if x > real_w - (new_width / 2) {
            pos.0 = real_w - new_width;
        } else {
            pos.0 = x - (new_width / 2);
        }

        if y < (new_height / 2) {
            pos.1 = 0u32;
        } else if y > real_h - (new_height / 2) {
            pos.1 = real_h - new_height;
        } else {
            pos.1 = y - (new_height / 2);
        }

        let preview = crop(&img.deref(), pos.0, pos.1, new_width, new_height);
    
        let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
            &preview,
            preview.width(),
            preview.height(),
        );

        *img = preview;
        Window_weak.upgrade_in_event_loop(|Window: LVIE| Window.set_image(Image::from_rgb8(pix_buf))).expect("Failed to call event loop");

    });

    //saturation
    let prev_weak = Arc::clone(&loaded_preview);
    let low_res_weak = Arc::clone(&low_res_preview);
    let Window_weak = Window.as_weak();
    Window
        .global::<ScreenCallbacks>()
        .on_add_saturation(move |value: f32| {
            Window_weak
                .upgrade_in_event_loop(|w| w.global::<ScreenCallbacks>().invoke_reset())
                .expect("failed to reset");
            let mut prev = prev_weak.lock().unwrap();
            let satuarated = img_processing::saturate(&(*prev), value);
            *prev = satuarated;
            let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                &prev.deref(),
                prev.deref().width(),
                prev.deref().height(),
            );
            Window_weak.upgrade_in_event_loop(|Window: LVIE| Window.set_image(Image::from_rgb8(pix_buf))).expect("Failed to call event loop");
            let mut lp = low_res_weak.lock().unwrap();
            *lp = build_low_res_preview(&prev);
        });

    // apply filter
    let img_weak = Arc::clone(&loaded_image);
    let prev_weak = Arc::clone(&loaded_preview);
    let low_res_weak = Arc::clone(&low_res_preview);
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_apply_filters(
        move |box_blur: i32, _gaussian_blur: f32, sharpening: f32| {
            *low_res_weak.lock().unwrap() = build_low_res_preview(&img_weak.lock().unwrap());

            //low res preview
            let mut processed: image::ImageBuffer<image::Rgb<u8>, Vec<u8>>;

            if sharpening > 0.0 {
                processed = img_processing::sharpen(low_res_weak.lock().unwrap().deref(), sharpening, 3);
            }else {
                processed = build_low_res_preview(&img_weak.lock().unwrap());
            }

            if box_blur > 3 {
                let mut low_res_kernel = Filters::BoxBlur((box_blur / 3) as u32);
                processed = img_processing::apply_filter(
                    &processed,
                    &mut low_res_kernel,
                );
            } 

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

            println!("Done with low res preview");

            let _w_w = Window_weak.clone();
            let _i_w = img_weak.clone();
            let _p_w = prev_weak.clone();
            thread::spawn(move || {
                // full res
                let mut _prev = _i_w.lock().unwrap();

                let mut processed: image::ImageBuffer<image::Rgb<u8>, Vec<u8>>;

                if box_blur > 3 {
                    let mut kernel = Filters::BoxBlur(box_blur as u32);

                    processed = img_processing::apply_filter(_prev.deref(), &mut kernel);
                } else {
                    processed = _prev.clone();
                }

                if sharpening > 0.0 {
                    processed = img_processing::sharpen(&processed, sharpening, 3);
                }

                *_p_w.lock().unwrap() = processed.clone();

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
                println!("All done");
            });
        },
    );

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
    let img_weak = Arc::clone(&loaded_preview);
    Window
        .global::<ScreenCallbacks>()
        .on_save_file(move |path: SharedString| {
            img_weak
                .lock()
                .unwrap()
                .deref()
                .save(path.as_str())
                .expect("Failed to save file");
        });

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
