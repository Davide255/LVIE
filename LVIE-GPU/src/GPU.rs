#![allow(non_snake_case)]
use std::{
    any::TypeId,
    path::{Path, PathBuf},
};

use image::Pixel;
use image::Primitive;
use pollster::FutureExt;
use texturebuffers::TexturesBuffer;
use wgpu::util::DeviceExt;

pub mod errors;
mod texturebuffers;

use errors::GPUError;

use LVIElib::traits::Scale;

pub use bytemuck::Pod;
pub use texturebuffers::CRgbaImage;

fn COMPILER_PATH() -> (PathBuf, PathBuf) {
    (
        PathBuf::from(file!()).ancestors().collect::<Vec<&Path>>()[2]
            .join("shaders")
            .join("compiler")
            .join("dxil.dll"),
        PathBuf::from(file!()).ancestors().collect::<Vec<&Path>>()[2]
            .join("shaders")
            .join("compiler")
            .join("dxcompiler.dll"),
    )
}

#[allow(non_camel_case_types)]
pub enum GPUBackens {
    VULCAN,
    METAL,
    BRAWSER_WGPU,
    DX12,
    GL,
}

#[derive(Clone, Copy)]
pub enum GPUShaderType {
    Exposition,
    Saturation,
    Grayscale,
    WhiteBalance,
}

impl GPUShaderType {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct GPU {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shaders: Vec<wgpu::ShaderModule>,
    texture: Option<(wgpu::Texture, wgpu::Extent3d)>,
    textures: TexturesBuffer,
}

pub struct AdapterInfo {
    adapter: wgpu::Adapter,
    pub info: wgpu::AdapterInfo,
}

fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}

fn padded_bytes_per_row<T: Primitive>(width: u32) -> usize {
    let bytes_per_row = width as usize * 4 * std::mem::size_of::<T>();
    let padding = (256 - bytes_per_row % 256) % 256;
    bytes_per_row + padding
}

impl GPU {
    pub fn list_GPUs(backend: Option<GPUBackens>) -> Vec<AdapterInfo> {
        let wgpu_backend: wgpu::Backends;
        match backend {
            Some(GPUBackens::BRAWSER_WGPU) => wgpu_backend = wgpu::Backends::BROWSER_WEBGPU,
            Some(GPUBackens::DX12) => wgpu_backend = wgpu::Backends::DX12,
            Some(GPUBackens::GL) => wgpu_backend = wgpu::Backends::GL,
            Some(GPUBackens::METAL) => wgpu_backend = wgpu::Backends::METAL,
            Some(GPUBackens::VULCAN) => wgpu_backend = wgpu::Backends::VULKAN,
            None => wgpu_backend = wgpu::Backends::all(),
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu_backend,
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
                dxil_path: Some(COMPILER_PATH().0),
                dxc_path: Some(COMPILER_PATH().1),
            },
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });
        let adapters = instance.enumerate_adapters(wgpu_backend);

        let mut infos: Vec<AdapterInfo> = Vec::new();

        for adapter in adapters {
            let info = adapter.get_info();
            infos.push(AdapterInfo { adapter, info })
        }

        infos
    }

    pub fn init(
        backend: Option<GPUBackens>,
        adapter: Option<AdapterInfo>,
    ) -> Result<GPU, GPUError> {
        let wgpu_backend: wgpu::Backends;
        match backend {
            Some(GPUBackens::BRAWSER_WGPU) => wgpu_backend = wgpu::Backends::BROWSER_WEBGPU,
            Some(GPUBackens::DX12) => wgpu_backend = wgpu::Backends::DX12,
            Some(GPUBackens::GL) => wgpu_backend = wgpu::Backends::GL,
            Some(GPUBackens::METAL) => wgpu_backend = wgpu::Backends::METAL,
            Some(GPUBackens::VULCAN) => wgpu_backend = wgpu::Backends::VULKAN,
            None => wgpu_backend = wgpu::Backends::all(),
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu_backend,
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
                dxil_path: Some(COMPILER_PATH().0),
                dxc_path: Some(COMPILER_PATH().1),
            },
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let a: wgpu::Adapter;
        if adapter.is_none() {
            let _adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptionsBase {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    force_fallback_adapter: false,
                    compatible_surface: None,
                })
                .block_on();
            if _adapter.is_none() {
                return Err(GPUError::ADAPTERNOTFOUND());
            }
            a = _adapter.unwrap();
        } else {
            a = adapter.unwrap().adapter
        }

        let dev_and_qu = a.request_device(&Default::default(), None).block_on();

        if dev_and_qu.is_err() {
            return Err(GPUError::REQUESTDEVICEERROR(dev_and_qu.err().unwrap()));
        }

        let (device, queue) = dev_and_qu.unwrap();
        let textures = TexturesBuffer::create_texture(&device, (1, 1), 1)?;

        Ok(GPU {
            instance,
            adapter: a,
            device,
            queue,
            shaders: Vec::new(),
            texture: None,
            textures,
        })
    }

    pub fn clone_from(gpu: &GPU) -> Result<GPU, GPUError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: gpu.adapter.get_info().backend.into(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
                dxil_path: Some(COMPILER_PATH().0),
                dxc_path: Some(COMPILER_PATH().1),
            },
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let mut adapter: wgpu::Adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .block_on()
            .unwrap();

        for a in instance.enumerate_adapters(gpu.adapter.get_info().backend.into()) {
            if a.get_info().name == gpu.adapter.get_info().name {
                adapter = a;
            }
        }

        let dev_and_qu = adapter.request_device(&Default::default(), None).block_on();

        if dev_and_qu.is_err() {
            return Err(GPUError::REQUESTDEVICEERROR(dev_and_qu.err().unwrap()));
        }

        let (device, queue) = dev_and_qu.unwrap();

        let textures = TexturesBuffer::create_texture(
            &device,
            (
                gpu.textures.texture_size.width,
                gpu.textures.texture_size.height,
            ),
            gpu.textures.type_size,
        )?;

        Ok(GPU {
            instance,
            adapter,
            device,
            queue,
            shaders: Vec::new(),
            texture: None,
            textures,
        })
    }

    pub fn compile_shaders(&mut self) {
        let grayscale = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Grayscale shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/grayscale.wgsl").into()),
            });

        let exposition = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Exposition shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/exposition.wgsl").into()),
            });

        let saturation = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Saturation shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/saturation.wgsl").into()),
            });
        let whitebalance = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("White Balance shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/whitebalance.wgsl").into(),
                ),
            });

        self.shaders = vec![exposition, saturation, grayscale, whitebalance];
    }

    #[allow(unreachable_code)]
    pub fn create_rgb_texture<P>(&mut self, img: &CRgbaImage<P>) -> Result<(), GPUError>
    where
        P: Pixel + Send + Sync + 'static,
        P::Subpixel: Scale + Primitive + std::fmt::Debug + bytemuck::Pod,
    {
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
            format: {
                if TypeId::of::<P::Subpixel>() == TypeId::of::<u8>() {
                    wgpu::TextureFormat::Rgba8Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<u16>() {
                    panic!("This type is still not supported for GPU rendering, please use CPU rendering mode");
                    wgpu::TextureFormat::Rgba16Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<f32>() {
                    panic!("This type is still not supported for GPU rendering, please use CPU rendering mode");
                    wgpu::TextureFormat::Rgba32Float
                } else {
                    return Err(GPUError::RENDERINGERROR())
                }
            },
            view_formats: &[{
                if TypeId::of::<P::Subpixel>() == TypeId::of::<u8>() {
                    wgpu::TextureFormat::Rgba8Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<u16>() {
                    panic!("This type is still not supported for GPU rendering, please use CPU rendering mode");
                    wgpu::TextureFormat::Rgba16Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<f32>() {
                    panic!("This type is still not supported for GPU rendering, please use CPU rendering mode");
                    wgpu::TextureFormat::Rgba32Float
                } else {
                    return Err(GPUError::RENDERINGERROR())
                }
            }],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        self.queue.write_texture(
            input_texture.as_image_copy(),
            bytemuck::cast_slice(img.as_raw()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * (std::mem::size_of::<P::Subpixel>() as u32) * img.width()),
                rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
            },
            texture_size,
        );
        self.texture = Some((input_texture, texture_size));
        Ok(())
    }

    pub fn render<T, P>(
        &mut self,
        shader: &GPUShaderType,
        parameters: &Vec<T>,
    ) -> Result<CRgbaImage<P>, GPUError>
    where
        P: Pixel + Send + Sync + 'static,
        P::Subpixel: Scale + Primitive + std::fmt::Debug + bytemuck::Pod,
        T: Primitive + bytemuck::Pod,
    {
        if self.shaders.len() == 0 {
            return Err(GPUError::SHADERSNOTCOMPILED());
        }

        let shader = &self.shaders[shader.index()];

        let (texture, texture_size) = &self.texture.as_ref().unwrap();

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Pipeline"),
                layout: None, //Some(&pipeline_layout),
                module: shader,
                entry_point: "shader_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            });

        let output_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("output texture"),
            size: *texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: {
                if TypeId::of::<P::Subpixel>() == TypeId::of::<u8>() {
                    wgpu::TextureFormat::Rgba8Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<u16>() {
                    wgpu::TextureFormat::Rgba16Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<f32>() {
                    wgpu::TextureFormat::Rgba32Float
                } else {
                    return Err(GPUError::RENDERINGERROR());
                }
            },
            view_formats: &[{
                if TypeId::of::<P::Subpixel>() == TypeId::of::<u8>() {
                    wgpu::TextureFormat::Rgba8Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<u16>() {
                    wgpu::TextureFormat::Rgba16Unorm
                } else if TypeId::of::<P::Subpixel>() == TypeId::of::<f32>() {
                    wgpu::TextureFormat::Rgba32Float
                } else {
                    return Err(GPUError::RENDERINGERROR());
                }
            }],
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
        });

        let parameter_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Parameter Buffer"),
                contents: bytemuck::cast_slice(parameters),
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            });

        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: parameter_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let (dispatch_with, dispatch_height) =
                compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &texture_bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
        }

        let padded_bytes_per_row = padded_bytes_per_row::<P::Subpixel>(texture_size.width);
        let unpadded_bytes_per_row =
            texture_size.width as usize * 4 * std::mem::size_of::<P::Subpixel>();

        let mut pixels: Vec<P::Subpixel> = vec![
            P::Subpixel::DEFAULT_MIN_VALUE;
            padded_bytes_per_row * texture_size.height as usize
        ];

        let out_buffer: wgpu::Buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("out buff"),
            size: pixels.len() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &out_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: Some(texture_size.height),
                },
            },
            *texture_size,
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = out_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

        self.device.poll(wgpu::Maintain::Wait);

        let padded_data = buffer_slice.get_mapped_range();

        for (padded, pixels) in padded_data
            .chunks_exact(padded_bytes_per_row)
            .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row))
        {
            pixels.copy_from_slice(bytemuck::cast_slice(&padded[..unpadded_bytes_per_row]));
        }

        if let Some(output_image) = CRgbaImage::<P>::from_raw(
            texture_size.width,
            texture_size.height,
            (&pixels[..]).to_vec(),
        ) {
            Ok(output_image)
        } else {
            Err(GPUError::RENDERINGERROR())
        }
    }
}

impl Clone for GPU {
    fn clone(&self) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: self.adapter.get_info().backend.into(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
                dxil_path: Some(COMPILER_PATH().0),
                dxc_path: Some(COMPILER_PATH().1),
            },
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let mut adapter: wgpu::Adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .block_on()
            .unwrap();

        for a in instance.enumerate_adapters(self.adapter.get_info().backend.into()) {
            if a.get_info().name == self.adapter.get_info().name {
                adapter = a;
            }
        }

        let dev_and_qu = adapter.request_device(&Default::default(), None).block_on();

        let (device, queue) = dev_and_qu.unwrap();

        let textures = TexturesBuffer::create_texture(
            &device,
            (
                self.textures.texture_size.width,
                self.textures.texture_size.height,
            ),
            self.textures.type_size,
        )
        .unwrap_or(TexturesBuffer::create_texture(&device, (0, 0), 1).unwrap());

        GPU {
            instance,
            adapter,
            device,
            queue,
            shaders: Vec::new(),
            texture: None,
            textures,
        }
    }
}
