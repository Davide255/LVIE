#![allow(non_snake_case)]
slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use image::{ImageBuffer, Pixel, Primitive};
use crate::raw_decoder::{decode, supported_formats};
use img_processing::{collect_histogram_data, Max};
use slint::{Image, Model, Rgba8Pixel, SharedPixelBuffer, SharedString, Weak};
use num_traits::NumCast;

use std::sync::{Arc, Mutex};
use std::{thread, time};

use rfd::FileDialog;

use itertools::Itertools;

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

    let SETTINGS: crate::settings::Settings = load_settings(None).unwrap();

    println!("{:?}", SETTINGS.keyboard_shortcuts);

    let CORE: Rendering = Rendering::init(SETTINGS.backend);

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

    let preview = Arc::new(Mutex::new(PreviewData::new(None, None, None)));

    // CALLBACKS:
    // open image:
    let data_weak = DATA.clone();
    let prev_w = preview.clone();
    let Window_weak = Window.as_weak();
    Window
        .global::<ToolbarCallbacks>()
        .on_open_file(move || {
            // get the file with native file dialog
            let fd = FileDialog::new()
                .add_filter("all image formats", 
                &[supported_formats().as_slice(), ["jpg", "jpeg", "png"].as_slice()].concat())
                .pick_file();
            if fd.is_none() { return; }
            let binding = fd.unwrap();

            let img: image::RgbaImage;

            if supported_formats().contains(&binding.as_path().extension().unwrap().to_str().unwrap().to_uppercase().as_str()) {
                let buff = decode(binding.as_path());
                if buff.is_none() { 
                    println!("Cannot decode file {}", binding.as_path().display());
                    return; 
                }
                img = buff.unwrap();
            } else {
                let buff = image::open(binding.as_path().to_str().unwrap());
                if buff.is_err() {
                    println!("Cannot decode file {}", binding.as_path().display());
                    return;
                }
                img = buff.unwrap().to_rgba8();
            }

            let (real_w, real_h) = img.dimensions();

            let mut data = data_weak.lock().unwrap();

            // load the image
            data.load_image(img.clone());

            let nw: u32 = Window_weak.unwrap().get_image_space_size_width().round() as u32;
            let nh: u32 = (real_h * nw) / real_w;

            *prev_w.lock().unwrap() = data.generate_preview(nw, nh);

            //let lpw = prev_w.clone();
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

    let data_weak = DATA.clone();
    let Window_weak = Window.as_weak();
    Window
        .global::<ToolbarCallbacks>()
        .on_rotate_90_deg(move || {
            let mut data = data_weak.lock().unwrap();
            let img = image::imageops::rotate90(&data.full_res_preview);
            // load the image
            data.load_image(img.clone());

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
    Window
        .global::<ToolbarCallbacks>()
        .on_close_window_callback(|| {
            slint::quit_event_loop().expect("Failed to stop the event loop");
        });

    Window.on_to_lowercase(move |s: SharedString| {
        s.to_lowercase().into()
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

    let ww = Window.as_weak();
    let dw = DATA.clone();
    Window.global::<ScreenCallbacks>().on_update_curve(move |points: slint::ModelRc<slint::ModelRc<f32>>| {
        let mut xs: Vec<f32> = Vec::new();
        let mut ys: Vec<f32> = Vec::new();

        for point in points.iter() {
            let p: Vec<f32> = point.iter().collect();
            xs.push(p[0]);
            ys.push(p[1]);
        }

        let mut data = dw.lock().unwrap();
        data.curve.update_curve(xs, ys);

        let W = ww.unwrap();

        W.set_curve(data.curve.to_image((300, 300)));
        W.set_curve_points(data.curve.into_rc_model()); 
    });

    let dw = DATA.clone();
    let ww = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_add_curve_point(move |x: f32, y: f32| {
        let mut d = dw.lock().unwrap();
        d.curve.add_point([x, y]).expect("Failed to add a point");
        let Window = ww.unwrap();
        Window.set_curve(d.curve.to_image((300, 300)));
        Window.set_curve_points(d.curve.into_rc_model());
    });

    let d_w = DATA.clone();
    Window.global::<ScreenCallbacks>().on_there_is_a_point(move |x: f32, y: f32, width: f32, height: f32, size: f32| {
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
    });

    // apply filters
    let data_weak = DATA.clone();
    let Window_weak = Window.as_weak();
    Window.global::<ScreenCallbacks>().on_apply_filters(
        move |exposition:f32, box_blur: f32, gaussian_blur: f32, sharpening: f32, temp: f32, tint: f32, saturation: f32| {
            //low res preview
            let mut data = data_weak.lock().expect("Failed to lock");

            data.update_filter(FilterType::Exposition, vec![exposition]);
            data.update_filter(FilterType::Saturation, vec![saturation]);
            data.update_filter(FilterType::Sharpening, vec![sharpening, 5.0]);
            data.update_filter(FilterType::Boxblur, vec![box_blur, 5.0]);
            data.update_filter(FilterType::GaussianBlur, vec![gaussian_blur, 5.0]);
            data.update_filter(FilterType::WhiteBalance, vec![2000f32*temp + 6000f32, tint*50.0]);

            let processed = data.update_image();

            let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &data.full_res_preview,
                data.image_dimensions().0,
                data.image_dimensions().1,
            );

            Window_weak.upgrade_in_event_loop(move |Window: LVIE| {
                Window.set_image(Image::from_rgba8(pix_buf));
            }).expect("Failed to call event loop");

            let ww = Window_weak.clone();
            thread::spawn(move || {
                let path = _create_svg_path(&processed);
                ww.upgrade_in_event_loop(|Window| Window.set_svg_path(path.into()) )
                    .expect("Cannot update the histogram");
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
    slint::run_event_loop().expect("Failed to create the event loop");
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