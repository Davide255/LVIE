use std::sync::{Arc, Mutex};

use crate::{
    core::{FilterArray, FilterType},
    history::{History, LogicOperationType},
};

use super::super::{
    super::ui::{ScreenCallbacks, LVIE},
    Data,
};
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer, SharedString, Weak};
use LVIElib::traits::ScaleImage;

const INTERNAL_CLOCK_TIME: u64 = 2;

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
            })
            .expect("Failed to call event loop");
    });

    // apply filters
    let data_weak = DATA.clone();
    let Window_weak = Window.as_weak();
    let clock_w = CLOCK.clone();
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

            if !cfg!(debug_assertions) {
                let clock = clock_w.lock().unwrap();
                if clock.running() {
                    clock.restart();
                } else {
                    let dw = data_weak.clone();
                    clock.start(
                        slint::TimerMode::SingleShot,
                        std::time::Duration::from_secs(INTERNAL_CLOCK_TIME),
                        move || {
                            let mut data = dw.lock().unwrap();
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

    let dw = DATA.clone();
    let hw = HISTORY.clone();
    Window.global::<ScreenCallbacks>().on_update_history(
        move |exposition: f32,
              box_blur: f32,
              gaussian_blur: f32,
              sharpening: f32,
              temp: f32,
              tint: f32,
              saturation: f32| {
            let data = dw.lock().unwrap();

            let filters = data.get_loaded_filters().clone();

            let mut old_f = FilterArray::new(None);

            old_f.update_filter(FilterType::Exposition, vec![exposition]);
            old_f.update_filter(FilterType::Saturation, vec![saturation]);
            old_f.update_filter(FilterType::Sharpening, vec![sharpening, 5.0]);
            old_f.update_filter(FilterType::Boxblur, vec![box_blur, 5.0]);
            old_f.update_filter(FilterType::GaussianBlur, vec![gaussian_blur, 5.0]);
            old_f.update_filter(
                FilterType::WhiteBalance,
                vec![2000f32 * temp + 6000f32, tint * 50.0],
            );

            hw.lock()
                .unwrap()
                .register_Filter_Operation_and_save(&(old_f, filters), &data.full_res_preview)
                .expect("Failed to register filter operation");
        },
    );
}
