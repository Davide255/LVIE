#![allow(non_snake_case)]
slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use image;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Weak};

#[allow(unused_imports)]
use std::{thread, time};

mod loading;

#[allow(unreachable_code)]
fn main() {
    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new()))
        .expect("Failed to set winit backend!");

    let Window: LVIE = LVIE::new().unwrap();

    let img = image::open("original.jpg").expect("Failed to open the image");

    let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        img.as_rgb8().unwrap(),
        img.width(),
        img.height(),
    );

    let slint_image = Image::from_rgb8(pix_buf);

    Window.set_image(slint_image);

    Window
        .global::<ToolbarCallbacks>()
        .on_open_file_callback(|| {
            println!("Called!");
        });

    let maximize_ui = |ui: LVIE| {
        ui.window()
            .with_winit_window(|winit_window: &winit::window::Window| {
                winit_window.set_maximized(true);
                winit_window.set_title("LVIE");
            })
            .expect("Failed to use winit!");
    };

    let l_weak: Weak<LVIE> = Window.as_weak();

    thread::Builder::new()
        .name("waiter".to_string())
        .spawn(move || {
            thread::sleep(time::Duration::from_millis(100));
            l_weak
                .upgrade_in_event_loop(move |handle| {
                    while !handle.window().is_visible() {
                        thread::sleep(time::Duration::from_millis(10));
                    }
                    maximize_ui(handle);
                })
                .expect("Failed to call from the main thread");
        })
        .expect("Failed to spawn thread");

    let _ = Window.show();
    slint::run_event_loop().expect("Cannnot run the evnt loop due to an error!");
    let _ = Window.hide();
}
