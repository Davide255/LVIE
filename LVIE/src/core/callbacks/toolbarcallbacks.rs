use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::history::{GeometricOperationType, History};

use super::super::{
    super::ui::{ToolbarCallbacks, LVIE},
    Data,
};
use image::Pixel;
use itertools::{max, Itertools};
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer, Weak};
use LVIElib::traits::ScaleImage;

use crate::core::CRgbaImage;
use crate::raw_decoder::*;

use crate::img_processing::collect_histogram_data;
use LVIElib::utils::{graph, GraphColor};

fn create_hist<P>(buff: &image::ImageBuffer<P, Vec<P::Subpixel>>) -> [slint::Image; 4]
where
    P: Pixel,
    P::Subpixel: image::Primitive + crate::img_processing::Max + std::cmp::Eq + std::hash::Hash,
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

pub fn init_toolbar_callbacks<P>(
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
        + LVIElib::traits::AsFloat,
{
    let Window = Window.unwrap();

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
        data.load_image(img.scale_image::<image::Rgba<u8>, P>(), true);

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
                        let path = create_hist(&img);
                        window.set_new_histogram(path.into());
                    })
                    .expect("Failed to run in event loop");
                });
                Window.set_image(slint::Image::from_rgba8(pix_buf));
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
            data.load_image(img.clone(), false);

            let img = img.scale_image::<P, image::Rgba<u8>>();

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

                    Window.set_image(slint::Image::from_rgba8(pix_buf));
                })
                .expect("Failed to call from event loop");
        });

    // close window: (quit the slint event loop)
    Window.global::<ToolbarCallbacks>().on_close_window(|| {
        slint::quit_event_loop().expect("Failed to stop the event loop");
    });
}
