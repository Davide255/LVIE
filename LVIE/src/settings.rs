pub struct Settings {
    pub backend: crate::core::CoreBackends,
}

pub fn load_settings() -> Settings {
    Settings {
        backend: crate::core::CoreBackends::GPU,
    }
}