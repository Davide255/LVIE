use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    core::FilterType,
    history::{History, LogicOperationType},
};

use super::super::{
    super::ui::{ScreenCallbacks, LVIE},
    Data,
};
use image::{Pixel, Primitive};
use itertools::Itertools;
use num_traits::NumCast;
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer, SharedString, Weak};
use LVIElib::traits::ScaleImage;

use crate::img_processing::{Max, _collect_histogram_data_old};

const INTERNAL_CLOCK_TIME: u64 = 2;

fn create_svg_path<P: Pixel>(buff: &image::ImageBuffer<P, Vec<P::Subpixel>>) -> [SharedString; 3]
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

pub fn init_screen_callbacks<P>(
    Window: Weak<LVIE>,
    DATA: Arc<Mutex<Data<P>>>,
    HISTORY: Arc<Mutex<History>>,
    CLOCK: Arc<Mutex<slint::Timer>>,
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
        let img = data.full_res_preview.scale_image::<P, image::Rgba<u8>>();

        Window_weak
            .upgrade_in_event_loop(move |Window: LVIE| {
                Window.set_image(slint::Image::from_rgba8(
                    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        &img,
                        img.width(),
                        img.height(),
                    ),
                ));

                let ww = Window.as_weak();
                thread::spawn(move || {
                    let path = create_svg_path(&img);
                    ww.upgrade_in_event_loop(move |window| {
                        window.set_svg_path(path.into());
                    })
                    .expect("Failed to run in event loop");
                });
            })
            .expect("Failed to call event loop");
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
        let img = data.full_res_preview.scale_image::<P, image::Rgba<u8>>();

        Window_weak
            .upgrade_in_event_loop(move |Window: LVIE| {
                Window.set_image(slint::Image::from_rgba8(
                    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        &img,
                        img.width(),
                        img.height(),
                    ),
                ));

                let ww = Window.as_weak();
                thread::spawn(move || {
                    let path = create_svg_path(&img);
                    ww.upgrade_in_event_loop(move |window| {
                        window.set_svg_path(path.into());
                    })
                    .expect("Failed to run in event loop");
                });
            })
            .expect("Failed to call event loop");
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

            let processed = data.update_image().scale_image::<P, image::Rgba<u8>>();

            let pix_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &processed,
                data.image_dimensions().0,
                data.image_dimensions().1,
            );

            Window_weak
                .upgrade_in_event_loop(move |Window: LVIE| {
                    Window.set_image(slint::Image::from_rgba8(pix_buf));
                })
                .expect("Failed to call event loop");

            let ww = Window_weak.clone();
            thread::spawn(move || {
                let path = create_svg_path(&processed);
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
}
