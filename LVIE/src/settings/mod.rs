pub mod history;
pub mod keyboard_shortcuts;
use serde::Deserialize;
use std::io::prelude::*;

const DEFAULT: &str = r"<LVIESettings>
    <backend>GPU</backend>
</LVIESettings>";

#[derive(Deserialize, Debug)]
struct LVIESettings {
    pub backend: String,
}

pub struct Settings {
    pub backend: crate::core::RenderingBackends,
    pub keyboard_shortcuts: keyboard_shortcuts::Keyboard
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
        let s: LVIESettings = quick_xml::de::from_str(f.unwrap().as_str()).unwrap();

        return Ok(Settings {
            backend: {
                match s.backend.as_str() {
                    "GPU" => crate::core::RenderingBackends::GPU,
                    "CPU" => crate::core::RenderingBackends::CPU,
                    _ => unimplemented!() 
                }
            },
            keyboard_shortcuts: {
                let ks = keyboard_shortcuts::load_from_file(None)?;
                if ks.is_err() {
                    eprintln!("cannot get custom keyboard shortcuts, proceding with default...");
                    keyboard_shortcuts::load_from_xml(String::from(keyboard_shortcuts::DEFAULT)).unwrap()
                } else {
                    ks.unwrap()
                }
            },
        });
    }
}

pub fn first_startup() -> std::io::Result<()> {
    println!("First startup, creating \".LVIE\" directory and required files");
    std::fs::create_dir(".LVIE")?;
    std::fs::File::create(".LVIE/settings.xml")?
        .write_all(DEFAULT.as_bytes())?;
    std::fs::File::create(".LVIE/keyboard_shortcuts.xml")?
        .write_all(keyboard_shortcuts::DEFAULT.as_bytes())?;
    std::fs::File::create(".LVIE/history.xml")?;
    Ok(())
}