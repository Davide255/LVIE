#![allow(non_snake_case)]
slint::include_modules!();

use slint::{Image, SharedPixelBuffer, Rgb8Pixel};
use image;
use i_slint_backend_winit::WinitWindowAccessor;

fn main() -> Result<(), slint::PlatformError> {
    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new()))
        .expect("Failed to set winit backend!");

    let Window = HelloWorld::new()?;

    let Window_weak = Window.as_weak();

    let img = image::open("D:\\PhotoEditorRust\\rustlib\\src\\original.jpg")
        .expect("Failed to open the image");

    let pix_buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        img.as_rgb8().unwrap(), img.width(), img.height()
    );

    let slint_image = Image::from_rgb8(pix_buf);

    Window.set_image(slint_image);

    Window.on_rust_init(move ||{
        let ui = Window_weak.unwrap();
        ui.window().with_winit_window(|winit_window: &winit::window::Window| {
            winit_window.set_maximized(true);
            winit_window.set_title("LVIE");
        }).expect("Failed to use winit!");
    });

    //Window.window().with_winit_window(|winit_window: &winit::window::Window| {
    //    winit_window.set_minimized(true);
    //    winit_window.set_title("LVIE");
    //}).expect("Failed to use winit!");


    Window.run()
}