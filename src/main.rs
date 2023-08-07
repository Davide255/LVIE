use std::vec;

use rust_backend::shift_hue;
use fltk::{app, enums::FrameType, frame::Frame, image::{SvgImage, JpegImage}, prelude::*, window::Window};

fn main() {
    let app = app::App::default();
    let mut win = Window::new(100, 100, 400, 300, "Image Editor");

    let mut frame = Frame::default().with_size(360, 260).center_of(&win);
    frame.set_frame(FrameType::EngravedBox);
    let mut image = JpegImage::load("test.jpg").unwrap();
    frame.set_image(Some(image.clone()));

    let buf = image.to_rgb_data();
    //let mut i = 0;
    //for pix in buf {
    //    i += 1;
    //    if i % 3 == 2 {
    //        print!("{}\n", pix)
    //    } else {
    //        print!("{} ", pix)
    //    }
    //}

    win.make_resizable(true);
    win.end();
    win.show();

    app.run().unwrap();
}