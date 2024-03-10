#![allow(non_snake_case)]
slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use image::{RgbaImage, GenericImageView, ImageBuffer, Pixel, Primitive};
use crate::img_processing::crop;
use crate::raw_decoder::{decode, supported_formats};
use img_processing::{collect_histogram_data, Max};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString, Weak};
use num_traits::NumCast;

use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, time};

use rfd::FileDialog;

use itertools::Itertools;

mod history;
mod img_processing;
mod raw_decoder;

mod core;
use crate::core::{Rendering, Data, FilterType, PreviewData};

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

    let CORE: Rendering = Rendering::init(SETTINGS.backend);

    if WINIT_BACKEND {
        slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap()))
            .expect("Failed to set winit backend!");
    }

    let Window: LVIE = LVIE::new().unwrap();

    let DATA = Arc::new(Mutex::new(Data::new(CORE, None, None)));

    let preview = Arc::new(Mutex::new(PreviewData::new(None, None, None)));

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
                .add_filter("all image formats", 
                &[supported_formats().as_slice(), ["jpg", "jpeg", "png"].as_slice()].concat())
                .pick_file();
            let binding = fd.unwrap();

            let img: image::RgbaImage;

            if supported_formats().contains(&binding.as_path().extension().unwrap().to_str().unwrap().to_uppercase().as_str()) {
                img = decode(binding.as_path()).unwrap();
            } else {
                img = image::open(binding.as_path().to_str().unwrap()).expect("Failed to open the image").to_rgba8();
            }

            let (real_w, real_h) = img.dimensions();

            let mut data = data_weak.lock().unwrap();

            // load the image
            data.load_image(img.clone());

            let nw: u32 = Window_weak.unwrap().get_image_space_size_width().round() as u32;
            let nh: u32 = (real_h * nw) / real_w;

            *prev_w.lock().unwrap() = data.generate_preview(nw, nh);

            let lpw = prev_w.clone();
            Window_weak
                .upgrade_in_event_loop(move |Window| {
                    let prevdata = lpw.lock().unwrap();
                    
                    // loading the image into the UI
                    let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        &prevdata.preview,
                        prevdata.preview.width(),
                        prevdata.preview.height(),
                    );

                    // create the histogram and update the UI
                    let ww = Window.as_weak();
                    thread::spawn(move || {
                        let path = _create_svg_path(&img);
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
        data.reset();

        // restore all the previews to the original image
        let img = data.full_res_preview.clone();
        let (real_w, real_h) = img.dimensions();

        let lpw = prev_w.clone();
        let dw = data_weak.clone();
        Window_weak.upgrade_in_event_loop(move |Window: LVIE| {
            Window.set_image(
                Image::from_rgba8(
            SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&img, img.width(), img.height())));

            // re-build the preview based on the effective screen sizes
            let nw: u32 = Window.get_image_space_size_width().round() as u32;
            let nh: u32 = (real_h * nw) / real_w;
                    
            *lpw.lock().unwrap() = dw.lock().unwrap().generate_preview(nw, nh);

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
        let mut prevdata: MutexGuard<'_, PreviewData>;
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
        
        let mut zoom = prevdata.zoom().clone();
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

        prevdata.set_zoom(zoom);
    });

    //saturation
    //let data_weak = DATA.clone();
    //let prev_weak = preview.clone();
    //let Window_weak = Window.as_weak();
    //Window
    //    .global::<ScreenCallbacks>()
    //    .on_add_saturation(move |value: f32| {
    //        /* Window_weak
    //            .upgrade_in_event_loop(|w| w.global::<ScreenCallbacks>().invoke_reset())
    //            .expect("failed to reset"); */
    //        let mut data = data_weak.lock().unwrap();
//
    //        let mut prevdata = prev_weak.lock().expect("Failed to lock");
    //        let prev = &mut prevdata.preview;
//
    //        data.filters.update_filter(FilterType::Saturation, vec![value]);
//
    //        let filters = data.filters.clone();
//
    //        *prev = data.core.render_data(&prev, &filters).unwrap();
//
    //        let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
    //            prev,
    //            prev.width(),
    //            prev.height(),
    //        );
//
    //        Window_weak.upgrade_in_event_loop(move |Window: LVIE| {
    //            Window.set_image(Image::from_rgba8(pix_buf));
    //        }).expect("Failed to call event loop");
//
    //        let pw = data_weak.clone();
    //        thread::spawn(move || {
    //            let mut data = pw.lock().expect("Failed to lock");
    //            let filters = data.filters.clone();
    //            let img_data = data.full_res_preview.clone();
    //            data.full_res_preview = data.core.render_data(&img_data, &filters).unwrap();
    //        });
    //        
    //    });

    // apply filter
    let data_weak = DATA.clone();
    let prev_weak = preview.clone();
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_apply_filters(
        move |exposition:f32, box_blur: f32, gaussian_blur: f32, sharpening: f32, temp: f32, tint: f32, saturation: f32| {
            //low res preview
            let data = data_weak.lock().expect("Failed to lock");
            let mut prevdata = prev_weak.lock().expect("Failed to lock");

            let rw = data.image_dimensions().0 as f32;
            let lrw = prevdata.preview.dimensions().0 as f32;

            prevdata.update_filter(FilterType::Exposition, vec![exposition]);
            prevdata.update_filter(FilterType::Saturation, vec![saturation]);
            prevdata.update_filter(FilterType::Sharpening, vec![sharpening * (lrw / rw), 5.0]);
            prevdata.update_filter(FilterType::Boxblur, vec![box_blur * (lrw / rw), 5.0]);
            prevdata.update_filter(FilterType::GaussianBlur, vec![gaussian_blur * (lrw / rw), 5f32]);
            prevdata.update_filter(FilterType::WhiteBalance, vec![6500f32, 0.0, 2000f32*temp + 6500f32, tint]);

            let start = std::time::Instant::now();
            let processed = prevdata.update_image();
            println!("Filters applyed in {} ms (low res)", start.elapsed().as_millis());

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

                data.update_filter(FilterType::Exposition, vec![exposition]);
                data.update_filter(FilterType::Saturation, vec![saturation]);
                data.update_filter(FilterType::Sharpening, vec![sharpening, 5.0]);
                data.update_filter(FilterType::Boxblur, vec![box_blur, 5.0]);
                data.update_filter(FilterType::GaussianBlur, vec![gaussian_blur, 5.0]);
                data.update_filter(FilterType::WhiteBalance, vec![6500f32, 0.0, 2000f32*temp + 6000f32, tint]);
                
                let start = std::time::Instant::now();
                let processed = data.update_image();
                println!("Filters applyed in {} ms", start.elapsed().as_millis());

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

#[cfg(test)]
#[macro_use]
mod tests {
    use crate::core::*;

    macro_rules! filter {
        ($ty:expr, $($param:expr), *) => {{
            let mut parameters = Vec::new();
            $(
                parameters.push($param);
            )*
            crate::core::Filter {
                filtertype: $ty,
                parameters
            }}
        };
    }

    #[test]
    fn white_balance(){
        let fromtemp = 6500.0;
        let fromtint = 0.0;
        let totemp = 9900.0;
        let totint = 1.23;

        let mut cpu = Rendering::init(crate::core::RenderingBackends::CPU);
        let mut gpu = Rendering::init(crate::core::RenderingBackends::GPU);

        let filters = FilterArray::new(Some(vec![filter!(FilterType::WhiteBalance, fromtemp, fromtint, totemp, totint)]));

        let img = image::open("C:\\Users\\david\\Documents\\workspaces\\original.jpg").unwrap().to_rgba8();

        cpu.render_data(&img, &filters).unwrap().save("prova_cpu.jpg").expect("Failed to save the image");
        gpu.render_data(&img, &filters).unwrap().save("prova_gpu.jpg").expect("Failed to save the image");
    }
}