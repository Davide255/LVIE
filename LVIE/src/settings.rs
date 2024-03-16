pub struct Settings {
    pub backend: crate::core::RenderingBackends,
}

pub fn load_settings() -> Settings {
    Settings {
        backend: crate::core::RenderingBackends::GPU,
    }
}