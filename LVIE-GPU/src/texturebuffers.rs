use half::f16;
use LVIElib::{oklab::OklabImage, utils::f32_vec_to_f16_vec};
use LVIElib::traits::Scale;
use LVIElib::hsl::HslaImage;

use image::{Primitive, Pixel};
use super::errors::GPUError;

#[allow(type_alias_bounds)]
pub type CRgbaImage<P: Pixel> = image::ImageBuffer<P, Vec<P::Subpixel>>;

#[derive(Debug)]
pub struct TexturesBuffer {
    rgb: wgpu::Texture,
    hsl: wgpu::Texture,
    oklab: wgpu::Texture,
    pub texture_size: wgpu::Extent3d,
    pub type_size: usize
}

impl TexturesBuffer {
    pub fn create_texture(device: &wgpu::Device, size: (u32, u32), type_size: usize) -> Result<TexturesBuffer, GPUError> 
    {   
        let texture_size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let rgb = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("input texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: {
                if type_size == 1 {
                    wgpu::TextureFormat::Rgba8Unorm
                } else if type_size == 2 {
                    panic!("This type is still not supported for GPU rendering, please use CPU rendering mode");
                    wgpu::TextureFormat::Rgba16Unorm
                } else if type_size == 4 {
                    panic!("This type is still not supported for GPU rendering, please use CPU rendering mode");
                    wgpu::TextureFormat::Rgba32Float
                } else {
                    return Err(GPUError::RENDERINGERROR())
                }
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let hsl = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hsl texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        }); 
        let oklab = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("oklab texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        }); 

        Ok(TexturesBuffer {
            rgb, hsl, oklab, texture_size, type_size
        })
    }

    pub fn allocate_rgb_texture<P>(&mut self, queue: &wgpu::Queue, img: &CRgbaImage<P>) -> Result<(), GPUError>
    where 
        P: Pixel + Send + Sync + 'static,
        P::Subpixel: Scale + Primitive + std::fmt::Debug + bytemuck::Pod
    {
        if self.texture_size.width == img.width() && self.texture_size.height == img.height() {
            queue.write_texture(
                self.rgb.as_image_copy(),
                bytemuck::cast_slice(img.as_raw()),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4* (std::mem::size_of::<P::Subpixel>() as u32) * img.width()),
                    rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
                },
                self.texture_size,
            );  
            Ok(())
        } else  {
            Err(GPUError::UNCOMPATIBLEIMAGESIZE((self.texture_size.width, self.texture_size.height), img.dimensions()))
        }
    }

    pub fn allocate_hsl_texture(&mut self, queue: &wgpu::Queue, img: &HslaImage) -> Result<(), GPUError> {   
        if self.texture_size.width == img.width() && self.texture_size.height == img.height() {
            queue.write_texture(
                self.hsl.as_image_copy(),
                bytemuck::cast_slice(&f32_vec_to_f16_vec(img.as_raw(), img.dimensions())),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4* (std::mem::size_of::<f16>() as u32) * img.width()),
                    rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
                },
                self.texture_size,
            );
            Ok(())
        } else  {
            Err(GPUError::UNCOMPATIBLEIMAGESIZE((self.texture_size.width, self.texture_size.height), img.dimensions()))
        }
    }

    pub fn allocate_oklab_texture(&mut self, queue: &wgpu::Queue, img: &OklabImage) -> Result<(), GPUError> {   
        if self.texture_size.width == img.width() && self.texture_size.height == img.height() {
            queue.write_texture(
                self.hsl.as_image_copy(),
                bytemuck::cast_slice(&f32_vec_to_f16_vec(img.as_raw(), img.dimensions())),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4* (std::mem::size_of::<f16>() as u32) * img.width()),
                    rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
                },
                self.texture_size,
            );
            Ok(())
        } else  {
            Err(GPUError::UNCOMPATIBLEIMAGESIZE((self.texture_size.width, self.texture_size.height), img.dimensions()))
        }
    }

    pub fn get_rgb_texture(&self) -> &wgpu::Texture {
        &self.rgb
    }

    pub fn get_hsl_texture(&self) -> &wgpu::Texture {
        &self.hsl
    }

    pub fn get_oklab_texture(&self) -> &wgpu::Texture {
        &self.oklab
    }
}