use LVIE_GPU::GPU;

#[derive(Debug)]
pub enum CoreBackends{
    GPU, CPU
}

pub enum Filter {
    Saturation(f32),
    Contrast(f32),
    Boxblur(f32),
    Sharpening(f32)
}

#[derive(Debug)]
pub struct Core{
    backend: CoreBackends,
    gpu: Option<GPU>,
}

impl Core{
    pub fn init(backend: CoreBackends) -> Core {
        let b: Core;
        match backend {
            CoreBackends::CPU => return Core{backend, gpu: None},
            CoreBackends::GPU => {},
        }

        let mut gpu = GPU::init(None).expect("Failed to init the GPU");
        gpu.compile_shaders();

        Core { backend, gpu: Some(gpu) }
    }

    pub fn render(&self, desc: Filter) {
        
    }
}