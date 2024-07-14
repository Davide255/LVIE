use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, SharedString, Weak};

use crate::settings::Settings;

use super::super::super::ui::{SettingsCallbacks, LVIE};

pub fn init_settings_callbacks(Window: Weak<LVIE>, SETTINGS: Arc<Mutex<Settings>>) {
    let Window = Window.unwrap();

    let sw = SETTINGS.clone();
    Window
        .global::<SettingsCallbacks>()
        .on_load_settings(move || {
            let settings = sw.lock().unwrap();
            ((
                SharedString::from(settings.backend.name()),
                settings.max_mem_size as i32,
                settings.use_temp_file,
                SharedString::from(settings.temp_file_directory.clone()),
                settings.start_maximized,
            ),)
        });
}
