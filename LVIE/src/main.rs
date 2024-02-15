#![allow(non_snake_case)]
slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use image::{RgbaImage, GenericImageView, ImageBuffer, Pixel, Primitive};
use crate::img_processing::crop;
use img_processing::{build_low_res_preview, collect_histogram_data, Max};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString, Weak};
use num_traits::NumCast;

use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, time};

use rfd::FileDialog;

use itertools::Itertools;

mod history;
mod img_processing;

mod core;
use crate::core::{Core, Data, FilterArray, FilterType, PreviewData};

mod settings;
use crate::settings::load_settings;

fn maximize_ui(ui: LVIE) {
    ui.window()
        .with_winit_window(|winit_window: &i_slint_backend_winit::winit::window::Window| {
            winit_window.set_maximized(true);
            winit_window.set_title("LVIE");
        })
        .expect("Failed to use winit!");
}

fn _create_svg_path<P>(buff: &ImageBuffer<P, Vec<P::Subpixel>>) -> [SharedString; 3] 
where 
    P: Pixel, P::Subpixel: Primitive + Max + std::cmp::Eq + std::hash::Hash + std::cmp::Ord,
    std::ops::RangeInclusive<P::Subpixel>: IntoIterator,
    <std::ops::RangeInclusive<<P as Pixel>::Subpixel> as IntoIterator>::Item: num_traits::ToPrimitive
{
    let hist = collect_histogram_data(&buff);
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
                        (<f32 as NumCast>::from(*k).unwrap() * (*max_value as f32 / 255f32)).round() as u32
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

    let SETTINGS: crate::settings::Settings = load_settings();

    let CORE: Core = Core::init(SETTINGS.backend);

    if WINIT_BACKEND {
        slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap()))
            .expect("Failed to set winit backend!");
    }

    let Window: LVIE = LVIE::new().unwrap();

    let DATA = Arc::new(Mutex::new(Data{
        core: CORE, 
        loaded_image: image::RgbaImage::new(0, 0), 
        full_res_preview: image::RgbaImage::new(0, 0),
        filters: FilterArray::new(None)
    }));

    let preview = Arc::new(Mutex::new(PreviewData{
        preview: image::RgbaImage::new(0, 0),
        preview_filters: FilterArray::new(None),
        zoom: (0.0, 0.0, 0.0)
    }));

    // CALLBACKS:
    // open image:
    let data_weak = DATA.clone();
    let prev_w = preview.clone();
    let Window_weak = Window.as_weak();
    Window
        .global::<ToolbarCallbacks>()
        .on_open_file_callback(move || {
            // get the file with native file dialog
            let fd = FileDialog::new()
                .add_filter("jpg", &["jpg", "jpeg", "png"])
                .pick_file();
            let binding = fd.unwrap();

            let img =
                image::open(binding.as_path().to_str().unwrap()).expect("Failed to open the image");
            let (real_w, real_h) = img.dimensions();

            let mut data = data_weak.lock().unwrap();

            // store the image
            data.loaded_image = img.to_rgba8();

            // set the full res preview
            data.full_res_preview = img.to_rgba8();

            let lpw = prev_w.clone();
            Window_weak
                .upgrade_in_event_loop(move |Window| {
                    // build the low resolution preview based on the effective sizes on the screen
                    let nw: u32 = Window.get_image_space_size_width().round() as u32;
                    let nh: u32 = (real_h * nw) / real_w;
                    
                    let mut _low_res = &mut lpw.lock().expect("Failed to lock").preview;
                    *_low_res = build_low_res_preview(&img.to_rgba8(), nw, nh);
                    
                    // loading the image into the UI
                    let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        &_low_res,
                        _low_res.width(),
                        _low_res.height(),
                    );

                    // create the histogram and update the UI
                    let ww = Window.as_weak();
                    thread::spawn(move || {
                        let path = _create_svg_path(&img.to_rgb8());
                        ww.upgrade_in_event_loop(move |window| {
                            window.set_svg_path(path.into());
                        })
                        .expect("Failed to run in event loop");
                    });
                    Window.set_image(Image::from_rgba8(pix_buf));
                })
                .expect("Failed to call from event loop");
        });

    // close window: (quit the slint event loop)
    Window
        .global::<ToolbarCallbacks>()
        .on_close_window_callback(|| {
            slint::quit_event_loop().expect("Failed to stop the event loop");
        });

    //reset
    let data_weak = DATA.clone();
    let prev_w = preview.clone();
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_reset(move || {
        let mut data = data_weak.lock().expect("Failed to lock data");

        // restore filters
        data.filters = FilterArray::new(None);

        // restore all the previews to the original image
        let img = data.loaded_image.clone();
        let (real_w, real_h) = img.dimensions();
        data.full_res_preview = img.clone();

        let lpw = prev_w.clone();
        Window_weak.upgrade_in_event_loop(move |Window: LVIE| {
            Window.set_image(
                Image::from_rgba8(
            SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&img, img.width(), img.height())));

            // re-build the preview based on the effective screen sizes
            let nw: u32 = Window.get_image_space_size_width().round() as u32;
            let nh: u32 = (real_h * nw) / real_w;
                    
            let mut _low_res = lpw.lock().expect("Failed to lock");
            _low_res.preview_filters = FilterArray::new(None);
            _low_res.preview = build_low_res_preview(&img, nw, nh);

            let ww = Window.as_weak();
            thread::spawn(move || {
                let path = _create_svg_path(&img);
                ww.upgrade_in_event_loop(move |window| {
                    window.set_svg_path(path.into());
                })
                .expect("Failed to run in event loop");
            });
        }).expect("Failed to call event loop");
    });

    // handle the zoom
    let data_weak = DATA.clone();
    let prev_w = preview.clone();
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_preview_click(move|width: f32, height: f32, x: f32, y: f32| {

        // check the aviability of the full resolution image
        // if not, utilize temporary the low resolution one
        let data: MutexGuard<'_, Data>;
        let prevdata: MutexGuard<'_, PreviewData>;
        let img: &RgbaImage;
        if data_weak.try_lock().is_ok() {
            data = data_weak.lock().unwrap();
            img = &data.full_res_preview;
            prevdata = prev_w.lock().expect("Failed to lock");
        } else {
            prevdata = prev_w.lock().expect("Failed to lock");
            img = &prevdata.preview;
        }

        // check if there is an image loaded
        if img.dimensions() == (0, 0) { return; }
        
        let mut zoom = prevdata.zoom;
        let (img_w, img_h) = img.dimensions();

        // zoomed_rectangle_width / image_width  
        let mut prop: f32 = zoom.2;

        // retrive the current zoomed rectange sizes
        let real_w = (img_w as f32 * prop) as u32;
        let real_h = (img_h as f32 * prop) as u32;

        // compute the new zoom rectangle sizes
        prop = prop - (0.1 * (prop / 2f32));
        let new_width = (real_w as f32 * prop) as u32;
        let new_height = (real_h * new_width) / real_w;
        zoom.2 = prop;

        let mut pos:(u32, u32) = (0u32, 0u32);

        // get the x and y coordinates of the click into the real image 
        let coefficient = real_w / (width.round() as u32);
        let adjustement: u32 = (height.round() as u32 - (real_h * width.round() as u32) / real_w) / 2;

        let x = (x.round() as u32) * coefficient;
        let y = ({
            if adjustement <= (y.round() as u32) { y.round() as u32 - adjustement } else { 0u32 }
        }) * coefficient;
        
        // centering the rectangle x
        if x < (new_width / 2) {
            pos.0 = (img_w as f32 * zoom.0) as u32;
        } else if x > real_w - (new_width / 2) {
            pos.0 = (img_w as f32 * zoom.0) as u32 + real_w - new_width;
        } else {
            pos.0 = (img_w as f32 * zoom.0) as u32 + x - (new_width / 2);
        }

        // centering the rectangle y
        if y < (new_height / 2) {
            pos.1 = (img_h as f32 * zoom.1) as u32;
        } else if y > real_h - (new_height / 2) {
            pos.1 = (img_h as f32 * zoom.1) as u32 + real_h - new_height;
        } else {
            pos.1 = (img_h as f32 * zoom.1) as u32 + y - (new_height / 2);
        }

        // update the position
        zoom.0 = pos.0 as f32 / img_w as f32;
        zoom.1 = pos.1 as f32 / img_h as f32;

        // crop and display the image
        let preview = crop(&img, pos.0, pos.1, new_width, new_height);
    
        let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            &preview,
            preview.width(),
            preview.height(),
        );

        Window_weak.upgrade_in_event_loop(|Window: LVIE| Window.set_image(Image::from_rgba8(pix_buf))).expect("Failed to call event loop");

    });

    //saturation
    let data_weak = DATA.clone();
    let prev_weak = preview.clone();
    let Window_weak = Window.as_weak();
    Window
        .global::<ScreenCallbacks>()
        .on_add_saturation(move |value: f32| {
            /* Window_weak
                .upgrade_in_event_loop(|w| w.global::<ScreenCallbacks>().invoke_reset())
                .expect("failed to reset"); */
            let mut data = data_weak.lock().unwrap();

            let mut prevdata = prev_weak.lock().expect("Failed to lock");
            let prev = &mut prevdata.preview;

            data.filters.set_filter(FilterType::Saturation, vec![value]);

            let filters = data.filters.clone();

            *prev = data.core.render_data(&prev, &filters).unwrap();

            let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                prev,
                prev.width(),
                prev.height(),
            );

            Window_weak.upgrade_in_event_loop(move |Window: LVIE| {
                Window.set_image(Image::from_rgba8(pix_buf));
            }).expect("Failed to call event loop");

            let pw = data_weak.clone();
            thread::spawn(move || {
                let mut data = pw.lock().expect("Failed to lock");
                let filters = data.filters.clone();
                let img_data = data.full_res_preview.clone();
                data.full_res_preview = data.core.render_data(&img_data, &filters).unwrap();
            });
            
        });

    // apply filter
    let data_weak = DATA.clone();
    let prev_weak = preview.clone();
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_apply_filters(
        move |exposition:f32, box_blur: f32, gaussian_blur: f32, sharpening: f32| {
            //low res preview
            let mut data = data_weak.lock().expect("Failed to lock");
            let mut prevdata = prev_weak.lock().expect("Failed to lock");

            let rw = data.full_res_preview.dimensions().0 as f32;
            let lrw = prevdata.preview.dimensions().0 as f32;

            prevdata.preview_filters.set_filter(FilterType::Exposition, vec![exposition]);
            prevdata.preview_filters.set_filter(FilterType::Sharpening, vec![sharpening * (lrw / rw)]);
            prevdata.preview_filters.set_filter(FilterType::Boxblur, vec![box_blur * (lrw / rw)]);
            prevdata.preview_filters.set_filter(FilterType::GaussianBlur, vec![gaussian_blur * (lrw / rw), 5f32]);

            let filters = prevdata.preview_filters.clone();
            let lr = &mut prevdata.preview;

            let processed = data.core.render_data(&lr, &filters).unwrap();
            *lr = processed.clone();

            Window_weak
                .upgrade_in_event_loop(move |Window: LVIE| {
                    let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        &processed,
                        processed.width(),
                        processed.height(),
                    );
                    Window.set_image(Image::from_rgba8(pix_buf));

                    /* no longer needed                     
                    Window.set_AlertBoxType(AlertType::Warning);
                    Window.set_AlertText("Low Res preview".into());*/
                    
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

            drop(data);
            drop(prevdata);
            
            // start computing the full resolution image
            let _w_w = Window_weak.clone();
            let _p_w = data_weak.clone();
            thread::spawn(move || {
                // full res
                let mut data = _p_w.lock().unwrap();

                data.filters.set_filter(FilterType::Exposition, vec![exposition]);
                data.filters.set_filter(FilterType::Sharpening, vec![sharpening]);
                data.filters.set_filter(FilterType::Boxblur, vec![box_blur]);
                data.filters.set_filter(FilterType::GaussianBlur, vec![gaussian_blur]);

                let filters = data.filters.clone();

                let processed = data.full_res_preview.clone();
                // re allocate the variable to assure that's immutable
                let processed = data.core.render_data(&processed, &filters).unwrap();

                data.full_res_preview = processed.clone();

                let ww = _w_w.clone();
                thread::spawn(move || {
                    let path = _create_svg_path(&processed);
                    ww.upgrade_in_event_loop(move |window| {
                        window.set_svg_path(path.into());
                    })
                    .expect("Failed to run in event loop");
                });

                /* there's no needing to load the full resolution image into the UI because the result
                   seen by the user is the same as the low resolution preview!!

                _w_w.upgrade_in_event_loop(move |Window: LVIE| {
                    let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
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
                println!("All done");*/
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
    let data_weak = DATA.clone();
    Window
        .global::<ScreenCallbacks>()
        .on_save_file(move |path: SharedString| {
            data_weak
                .lock()
                .unwrap()
                .full_res_preview
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
