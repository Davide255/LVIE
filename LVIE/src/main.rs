#![allow(non_snake_case)]
slint::include_modules!();

use i_slint_backend_winit::WinitWindowAccessor;
use image;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Weak};

#[allow(unused_imports)]
use std::{thread, time};

use rfd::FileDialog;

mod loading;

fn read_image(path: &str) -> Image {
    let img = image::open(path).expect("Failed to open the image");

    let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        img.as_rgb8().unwrap(),
        img.width(),
        img.height(),
    );

    return Image::from_rgb8(pix_buf);
}

fn maximize_ui(ui: LVIE) {
    ui.window()
        .with_winit_window(|winit_window: &winit::window::Window| {
            winit_window.set_maximized(true);
            winit_window.set_title("LVIE");
        })
        .expect("Failed to use winit!");
}

#[allow(unreachable_code)]
fn main() {
    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new()))
        .expect("Failed to set winit backend!");

    let Window: LVIE = LVIE::new().unwrap();

    // CALLBACKS:
    // open image:
    let Window_weak = Window.as_weak();
    Window
        .global::<ToolbarCallbacks>()
        .on_open_file_callback(move || {
            let fd = FileDialog::new()
                .add_filter("jpg", &["jpg", "jpeg", "png"])
                .pick_file();
            let binding = fd.unwrap();
            Window_weak
                .upgrade_in_event_loop(move |Window| {
                    Window.set_image(read_image(binding.as_path().to_str().unwrap()))
                })
                .expect("Failed to call from event loop");
        });

    // close window:
    Window
        .global::<ToolbarCallbacks>()
        .on_close_window_callback(|| {
            slint::quit_event_loop().expect("Failed to stop the event loop");
        });

    // startup procedure
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
