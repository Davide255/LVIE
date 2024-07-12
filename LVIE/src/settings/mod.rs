pub mod keyboard_shortcuts;

use keyboard_shortcuts::{prettify_keyboard_xml, Keyboard};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

use crate::history::{FileSize, FileSizes};

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub backend: crate::core::RenderingBackends,
    pub start_maximized: bool,
    pub temp_file_directory: String,
    pub use_temp_file: bool,
    pub max_mem_size: usize,
    #[serde(default, skip_serializing)]
    pub keyboard_shortcuts: keyboard_shortcuts::Keyboard,
}

pub fn load_settings(fd: Option<String>) -> std::io::Result<Settings> {
    let f = {
        if fd.is_none() {
            let f = std::fs::read_to_string(".LVIE/settings.xml");

            if !std::path::Path::new(".LVIE").exists() || f.is_err() {
                first_startup()?;
                let f = std::fs::read_to_string(".LVIE/settings.xml");
                f
            } else {
                f
            }
        } else {
            std::fs::read_to_string(fd.unwrap())
        }
    };

    if f.is_err() {
        return Err(f.unwrap_err());
    } else {
        let mut s: Settings = quick_xml::de::from_str(f.unwrap().as_str()).unwrap();

        s.keyboard_shortcuts = {
            let ks = keyboard_shortcuts::load_from_file(None)?;
            if ks.is_err() {
                eprintln!("cannot get custom keyboard shortcuts, proceding with default...");
                keyboard_shortcuts::Keyboard::default()
            } else {
                ks.unwrap()
            }
        };

        Ok(s)
    }
}

fn prettify_settings_xml(content: String) -> String {
    content
        .replace("<", "\n\t<")
        .replace("\n\t</", "</")
        .replace("\n\t<Settings", "<Settings")
        .replace("</Settings", "\n</Settings")
        .replace("\n\n", "\n")
}

pub fn first_startup() -> std::io::Result<()> {
    println!("First startup, creating \".LVIE\" directory and required files");
    std::fs::create_dir(".LVIE")?;

    let s = Settings::default();
    std::fs::File::create(".LVIE/keyboard_shortcuts.xml")?.write_all(
        prettify_keyboard_xml(quick_xml::se::to_string(&s.keyboard_shortcuts).unwrap()).as_bytes(),
    )?;
    std::fs::File::create(".LVIE/settings.xml")?
        .write_all(prettify_settings_xml(quick_xml::se::to_string(&s).unwrap()).as_bytes())?;
    //std::fs::File::create(".LVIE/history.xml")?;
    Ok(())
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            backend: crate::core::RenderingBackends::GPU,
            start_maximized: false,
            temp_file_directory: ".LVIE\\temp".into(),
            use_temp_file: true,
            max_mem_size: FileSizes::GB(3).size_as_bytes(),
            keyboard_shortcuts: Keyboard::default(),
        }
    }
}
