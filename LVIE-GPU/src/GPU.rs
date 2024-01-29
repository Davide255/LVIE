#![allow(non_snake_case)]
use pollster::FutureExt;

#[derive(Debug)]
pub enum GPUError {
    ADAPTERNOTFOUND(),
    REQUESTDEVICEERROR(wgpu::RequestDeviceError)
}

pub enum GPUBackens {
    VULCAN,
    METAL,
    BRAWSER_WGPU,
    DX11,
    DX12,
    GL
}

pub enum GPUShaderType {
    Grayscale,
    Saturation
}

#[derive(Debug)]
pub struct GPU {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    encoder: wgpu::CommandEncoder,
    shaders: Vec<wgpu::ShaderModule>,
    texture: Option<wgpu::Texture>
}

impl GPU {
    pub fn init(backend: Option<GPUBackens>) -> Result<GPU, GPUError> {
        let wgpu_backend: wgpu::Backends;
        match backend {
            Some(GPUBackens::BRAWSER_WGPU) => wgpu_backend = wgpu::Backends::BROWSER_WEBGPU,
            Some(GPUBackens::DX11) => wgpu_backend = wgpu::Backends::DX11,
            Some(GPUBackens::DX12) => wgpu_backend = wgpu::Backends::DX12,
            Some(GPUBackens::GL) => wgpu_backend = wgpu::Backends::GL,
            Some(GPUBackens::METAL) => wgpu_backend = wgpu::Backends::METAL,
            Some(GPUBackens::VULCAN) => wgpu_backend = wgpu::Backends::VULKAN,
            None => wgpu_backend = wgpu::Backends::all(),
        }
        let instance = wgpu::Instance::new(wgpu_backend);
    
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase { power_preference: 
            wgpu::PowerPreference::HighPerformance, 
            force_fallback_adapter: false, compatible_surface: None })
            .block_on();

        if adapter.is_none() { return Err(GPUError::ADAPTERNOTFOUND()); }
        let adapter = adapter.unwrap();

        let dev_and_qu = adapter
        .request_device(&Default::default(),None).block_on();

        if dev_and_qu.is_err() {
            return Err(GPUError::REQUESTDEVICEERROR(dev_and_qu.err().unwrap()));
        }

        let (device, queue) = dev_and_qu.unwrap();

        let encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Ok(GPU {
            instance,
            adapter,
            device, 
            queue,
            encoder,
            shaders: Vec::new(),
            texture: None
        })
        
    }

    pub fn compile_shaders(&mut self) {

        let grayscale = self.device.create_shader_module(
            wgpu::ShaderModuleDescriptor { 
                label: Some("Grayscale shader"), 
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/grayscale.wgsl").into())
            });
        
        let saturation = self.device.create_shader_module(
            wgpu::ShaderModuleDescriptor { 
                label: Some("Saturation shader"), 
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/saturation.wgsl").into())
            });

        self.shaders = vec![grayscale, saturation];
    }

    pub fn create_texture(&mut self, img: image::RgbaImage) {
        let texture_size = wgpu::Extent3d {
            width: img.width(),
            height: img.height(),
            depth_or_array_layers: 1,
        };

        let input_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("input texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        self.queue.write_texture(
            input_texture.as_image_copy(),
            bytemuck::cast_slice(img.as_raw()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * img.width()),
                rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
            },
            texture_size,
        );
    }

    
}