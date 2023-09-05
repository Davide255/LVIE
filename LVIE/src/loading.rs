slint::slint! {
import "fonts\\SFProDisplayBlackItalic.otf";

import { VerticalBox, Button } from "std-widgets.slint";

export component Loading inherits Window {
    //default-font-family: "SF Pro Display";
    callback close_window();
    in-out property <int> counter: 0;
    Text {
        font-size: 150px;
        text: "LVIE";
        color: @linear-gradient(90deg, #fc5c7d 0%, #6a82fb 100%);
        //font-family: "SF Pro Display Black Italic";
    }
}
}

use std::thread;

#[allow(dead_code)]
pub fn loading_window(duration: std::time::Duration) {
    let loading = Loading::new().unwrap();

    /*thread::Builder::new()
    .name("monitor".to_string())
    .spawn(move || {
        thread::sleep(time::Duration::from_millis(100));
        l_weak
            .upgrade_in_event_loop(move |ui| {
                while ui.window().is_visible() != true {
                    thread::sleep(time::Duration::from_millis(10));
                }
                ui.window()
                    .with_winit_window(|winit_window: &winit::window::Window| {
                        winit_window.set_enabled_buttons(WindowButtons::empty());
                    })
                    .expect("Failed to use winit!");
            })
            .expect("Failed to call the main thread");
    })
    .expect("Failed to create the thread");*/

    let l_weak = loading.as_weak();
    thread::Builder::new()
        .name("quit_thread".to_string())
        .spawn(move || {
            thread::sleep(duration);
            slint::invoke_from_event_loop(move || {
                let ui = l_weak.unwrap();
                slint::quit_event_loop().expect("Failed to stop the window!");
                let _ = ui.hide();
            })
            .expect("Failed to call the main thread");
        })
        .expect("Failed to create the thread");

    let _ = loading.run();
}
