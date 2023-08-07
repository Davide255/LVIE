use std::vec::Vec;
use fltk::{app, enums::FrameType, frame::Frame, image::{JpegImage}, prelude::*, window::Window, group::{Flex, FlexType}};

fn normalize_buffer(buffer: Vec<i32>) -> Vec<Vec<i32>> {
    let mut out_v: Vec<Vec<i32>> = Vec::new();

    for x in 0..(buffer.len() / 3) {
        out_v.append(&mut vec![buffer[x..(x + 3)].to_vec()]);
    }

    out_v
}

pub fn gui() {
    let app = app::App::default();
    let mut win = Window::new(100, 100, 800, 450, "Image Editor");

    let mut flex = Flex::new(0, 0, 500, 450, None);
    flex.set_type(FlexType::Column);

    let mut frame = Frame::default().size_of_parent().center_of(&win);
    frame.set_frame(FrameType::BorderBox);
    let mut image = JpegImage::load("test.jpg").unwrap();
    let mut preview = image.clone();
    preview.scale(frame.height(), frame.width(), true, true);
    frame.set_image(Some(preview));

    flex.fixed(&mut frame, 500);
    flex.end();

    win.make_resizable(false);
    win.end();
    win.show();

    app.run().unwrap();
}
