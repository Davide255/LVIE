#![allow(non_snake_case)]
mod ui;
use crate::ui::*;

use i_slint_backend_winit::WinitWindowAccessor;

use slint::ComponentHandle;
use slint::{SharedString, Weak};

use std::sync::{Arc, Mutex};
use std::{thread, time};

use itertools::Itertools;

mod img_processing;
mod img_processing_generic;
mod raw_decoder;

mod core;
use crate::core::callbacks::*;
use crate::core::{Data, Rendering};

mod settings;
use crate::settings::{keyboard_shortcuts, load_settings};

mod history;
use history::{init_history_callbacks, History};

fn maximize_ui(ui: LVIE) {
    ui.window()
        .with_winit_window(
            |winit_window: &i_slint_backend_winit::winit::window::Window| {
                winit_window.set_maximized(true);
                winit_window.set_title("LVIE");
            },
        )
        .expect("Failed to use winit!");
}

build_shortcuts!(
"editor"
>"file"
--"open":(|Window: LVIE, _args: &[&str]| Window.global::<ToolbarCallbacks>().invoke_open_file())
--"close":(|Window: LVIE, _args: &[&str]| Window.global::<ToolbarCallbacks>().invoke_close_window())
>"image"
--"rotate-90-deg":(|Window: LVIE, _args: &[&str]| Window.global::<ToolbarCallbacks>().invoke_rotate_90_deg())
>"preview"
--"undo":(|Window: LVIE, _args: &[&str]| Window.global::<ScreenCallbacks>().invoke_undo() )
--"redo":(|Window: LVIE, _args: &[&str]| Window.global::<ScreenCallbacks>().invoke_redo() )
);

#[allow(unreachable_code)]
fn main() {
    const WINIT_BACKEND: bool = if cfg!(windows) { true } else { false };

    let s: crate::settings::Settings = load_settings(None).unwrap();

    let CORE: Rendering<image::Rgba<u8>> = Rendering::init(s.backend);

    let HISTORY = Arc::new(Mutex::new(History::init(
        Some(std::path::PathBuf::from(s.temp_file_directory.clone())),
        s.use_temp_file,
        Some(s.max_mem_size),
    )));

    let SETTINGS = Arc::new(Mutex::new(s));

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

    let CLOCK = Arc::new(Mutex::new(slint::Timer::default()));

    let ww = Window.as_weak();
    let sw = SETTINGS.clone();
    Window.on_handle_shortcut(
        move |kvalue: SharedString, alt: bool, ctrl: bool, shift: bool| {
            let settings = sw.lock().unwrap();

            let kvalue: String = kvalue.to_lowercase();

            let mut modifiers: Vec<keyboard_shortcuts::MODIFIER> = Vec::new();
            if alt {
                modifiers.push(keyboard_shortcuts::MODIFIER::ALT);
            }
            if ctrl {
                modifiers.push(keyboard_shortcuts::MODIFIER::CTRL);
            }
            if shift {
                modifiers.push(keyboard_shortcuts::MODIFIER::SHIFT);
            }

            for key in &settings.keyboard_shortcuts {
                if key.is(&kvalue) {
                    if let Some(b) = key.get_binding_by_modifiers(&modifiers) {
                        // the pattern is 'editor.*.*'
                        let _ts = b.action().clone();
                        let action: Vec<&str> = _ts.split(".").collect_vec();
                        handle_shortcut_action(ww.clone(), action);
                    }
                }
            }
        },
    );

    init_screen_callbacks(
        Window.as_weak(),
        DATA.clone(),
        HISTORY.clone(),
        CLOCK.clone(),
    );

    init_toolbar_callbacks(Window.as_weak(), DATA.clone(), HISTORY.clone());

    init_settings_callbacks(Window.as_weak(), SETTINGS.clone());

    init_curve_callbacks(Window.as_weak(), DATA.clone(), HISTORY.clone());

    init_mask_callbacks(Window.as_weak(), DATA.clone(), HISTORY.clone());

    init_history_callbacks(Window.as_weak(), DATA.clone(), HISTORY.clone());

    // startup procedure
    let l_weak: Weak<LVIE> = Window.as_weak();

    if WINIT_BACKEND && SETTINGS.lock().unwrap().start_maximized {
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
