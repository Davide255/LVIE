#![allow(non_snake_case)]

fn main() {}

//use half::f16;
//use image::{GenericImageView, RgbaImage};
//use LVIElib::hsl::Hsla;
//
//use std::{fs::read_to_string, time::Instant};
//
//const WORKGROUP_SIZE: (u32, u32) = (32, 32);
//
//fn compute_work_group_count(
//    (width, height): (u32, u32),
//    (workgroup_width, workgroup_height): (u32, u32),
//) -> (u32, u32) {
//    let x = (width + workgroup_width - 1) / workgroup_width;
//    let y = (height + workgroup_height - 1) / workgroup_height;
//
//    (x, y)
//}
//
//fn padded_bytes_per_row<T>(width: u32) -> usize {
//    let bytes_per_row = width as usize * std::mem::size_of::<T>()*4;
//    let padding = (256 - bytes_per_row % 256) % 256;
//    bytes_per_row + padding
//}
//
//fn convert_to_hsl_gpu(img: RgbaImage) -> Vec<f16>{
//    use pollster::FutureExt;
//
//    type O = f16;
//
//    let instance = wgpu::Instance::new(wgpu::Backends::all());
//    let adapter = instance
//        .request_adapter(&wgpu::RequestAdapterOptionsBase {
//            power_preference: wgpu::PowerPreference::HighPerformance,
//            force_fallback_adapter: false,
//            compatible_surface: None,
//        })
//        .block_on().expect("Cannot create adapter");
//
//    let (device, queue) = adapter
//        .request_device(&Default::default(), None)
//        .block_on().expect("cannot create device ad queue");
//
//    let (width, height) = img.dimensions();
//
//    let texture_size = wgpu::Extent3d {
//        width,
//        height,
//        depth_or_array_layers: 1,
//    };
//    
//    let input_texture = device.create_texture(&wgpu::TextureDescriptor {
//        label: Some("input texture"),
//        size: texture_size,
//        mip_level_count: 1,
//        sample_count: 1,
//        dimension: wgpu::TextureDimension::D2,
//        format: wgpu::TextureFormat::Rgba8Unorm,
//        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//    });
//
//    queue.write_texture(
//        input_texture.as_image_copy(),
//        bytemuck::cast_slice(img.as_raw()),
//        wgpu::ImageDataLayout {
//            offset: 0,
//            bytes_per_row: std::num::NonZeroU32::new(4 * width),
//            rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
//        },
//        texture_size,
//    );
//
//    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
//        label: Some("output texture"),
//        size: texture_size,
//        mip_level_count: 1,
//        sample_count: 1,
//        dimension: wgpu::TextureDimension::D2,
//        format: wgpu::TextureFormat::Rgba16Float,
//        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
//    });
//
//    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//        label: Some("Saturation shader"),
//        source: wgpu::ShaderSource::Wgsl(read_to_string("LVIE-GPU/shaders/conversions/rgb_to_hsl_f16.wgsl").unwrap().into()),
//    });
//
//    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
//        label: Some("Saturation pipeline"),
//        layout: None, //Some(&pipeline_layout),
//        module: &shader,
//        entry_point: "shader_main",
//    });
//
//    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//        label: Some("Texture bind group"),
//        layout: &pipeline.get_bind_group_layout(0),
//        entries: &[
//            wgpu::BindGroupEntry {
//                binding: 0,
//                resource: wgpu::BindingResource::TextureView(
//                    &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
//                ),
//            },
//            wgpu::BindGroupEntry {
//                binding: 1,
//                resource: wgpu::BindingResource::TextureView(
//                    &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
//                ),
//            }
//        ],
//    });
//
//    let mut encoder =
//    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//
//    {   let (dispatch_with, dispatch_height) =
//            compute_work_group_count((texture_size.width, texture_size.height), WORKGROUP_SIZE);
//        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
//            label: None,
//        });
//        compute_pass.set_pipeline(&pipeline);
//        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
//        compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
//    }
//
//    let padded_bytes_per_row = padded_bytes_per_row::<O>(width);
//    let unpadded_bytes_per_row = width as usize * std::mem::size_of::<f16>()*4;
//
//    let mut pixels = vec![O::NAN; padded_bytes_per_row * height as usize];
//
//    //let mut pixels: Vec<f32> = vec![0.0; (padded_bytes_per_row / 4) * height as usize];
//
//    let out_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor { 
//        label: Some("out buff"), 
//        size: (pixels.len() * std::mem::size_of::<O>()) as u64,
//        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
//        mapped_at_creation: false 
//    });
//
//    encoder.copy_texture_to_buffer(
//        wgpu::ImageCopyTexture {
//            aspect: wgpu::TextureAspect::All,
//            texture: &output_texture,
//            mip_level: 0,
//            origin: wgpu::Origin3d::ZERO,
//        },
//        wgpu::ImageCopyBuffer {
//            buffer: &out_buffer,
//            layout: wgpu::ImageDataLayout {
//                offset: 0,
//                bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row as u32),
//                rows_per_image: std::num::NonZeroU32::new(height),
//            },
//        },
//        texture_size,
//    );
//
//    queue.submit(Some(encoder.finish()));
//
//    let buffer_slice = out_buffer.slice(..);
//    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
//
//    device.poll(wgpu::Maintain::Wait);
//
//    let padded_data = buffer_slice.get_mapped_range();
//
//    for (padded, p) in padded_data
//        .chunks_exact(padded_bytes_per_row)
//        .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row / std::mem::size_of::<O>()))
//    {
//        p.copy_from_slice(&bytemuck::cast_slice(&padded[..unpadded_bytes_per_row]));
//    }
//
//    return pixels;
//}
//
//#[allow(unreachable_code)]
//fn main() {
//
//    let d_img = image::open("C:\\Users\\david\\Documents\\Projects\\workspaces\\original.jpg")
//        .expect("cannot open the image");
//
//    let start = Instant::now();
//
//    let out = convert_to_hsl_gpu(d_img.to_rgba8());
//
//    println!("GPU: {}", start.elapsed().as_millis());
//
//    println!("{:?}, {:?}", &out[0..4], Hsla::from(*d_img.to_rgba8().get_pixel(0, 0)));
//
//    use pollster::FutureExt;
//
//    type Output = u8;
//
//    let instance = wgpu::Instance::new(wgpu::Backends::all());
//    let adapter = instance
//        .request_adapter(&wgpu::RequestAdapterOptionsBase {
//            power_preference: wgpu::PowerPreference::HighPerformance,
//            force_fallback_adapter: false,
//            compatible_surface: None,
//        })
//        .block_on().expect("Cannot create adapter");
//
//    let (device, queue) = adapter
//        .request_device(&Default::default(), None)
//        .block_on().expect("cannot create device ad queue");
//
//    let (width, height) = d_img.dimensions();
//
//    let texture_size = wgpu::Extent3d {
//        width,
//        height,
//        depth_or_array_layers: 1,
//    };
//    
//    let input_texture = device.create_texture(&wgpu::TextureDescriptor {
//        label: Some("input texture"),
//        size: texture_size,
//        mip_level_count: 1,
//        sample_count: 1,
//        dimension: wgpu::TextureDimension::D2,
//        format: wgpu::TextureFormat::Rgba16Float,
//        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//    });
//
//    queue.write_texture(
//        input_texture.as_image_copy(),
//        bytemuck::cast_slice(&out),
//        wgpu::ImageDataLayout {
//            offset: 0,
//            bytes_per_row: std::num::NonZeroU32::new(4 * width * 2),
//            rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
//        },
//        texture_size,
//    );
//
//    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
//        label: Some("output texture"),
//        size: texture_size,
//        mip_level_count: 1,
//        sample_count: 1,
//        dimension: wgpu::TextureDimension::D2,
//        format: wgpu::TextureFormat::Rgba8Unorm,
//        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
//    });
//
//    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//        label: Some("Saturation shader"),
//        source: wgpu::ShaderSource::Wgsl(read_to_string("LVIE-GPU/shaders/conversions/hsl_to_rgb.wgsl").unwrap().into()),
//    });
//
//    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
//        label: Some("Saturation pipeline"),
//        layout: None, //Some(&pipeline_layout),
//        module: &shader,
//        entry_point: "shader_main",
//    });
//
//    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//        label: Some("Texture bind group"),
//        layout: &pipeline.get_bind_group_layout(0),
//        entries: &[
//            wgpu::BindGroupEntry {
//                binding: 0,
//                resource: wgpu::BindingResource::TextureView(
//                    &input_texture.create_view(&wgpu::TextureViewDescriptor {
//                        label: None,
//                        format: Some(wgpu::TextureFormat::Rgba16Float),
//                        dimension: Some(wgpu::TextureViewDimension::D2),
//                        aspect: wgpu::TextureAspect::All,
//                        base_mip_level: 0,
//                        mip_level_count: None,
//                        base_array_layer: 0,
//                        array_layer_count: None
//                    }),
//                ),
//            },
//            wgpu::BindGroupEntry {
//                binding: 1,
//                resource: wgpu::BindingResource::TextureView(
//                    &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
//                ),
//            }
//        ],
//    });
//
//    let mut encoder =
//    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//
//    {   let (dispatch_with, dispatch_height) =
//            compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
//        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
//            label: None,
//        });
//        compute_pass.set_pipeline(&pipeline);
//        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
//        compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
//    }
//
//    let padded_bytes_per_row = padded_bytes_per_row::<f16>(width);
//    let unpadded_bytes_per_row = width as usize * std::mem::size_of::<Output>()*4;
//
//    let mut pixels = vec![0 as Output; padded_bytes_per_row * height as usize];
//
//    //let mut pixels: Vec<f32> = vec![0.0; (padded_bytes_per_row / 4) * height as usize];
//
//    let out_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor { 
//        label: Some("out buff"), 
//        size: (pixels.len() * std::mem::size_of::<Output>()) as u64,
//        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
//        mapped_at_creation: false 
//    });
//
//    encoder.copy_texture_to_buffer(
//        wgpu::ImageCopyTexture {
//            aspect: wgpu::TextureAspect::All,
//            texture: &output_texture,
//            mip_level: 0,
//            origin: wgpu::Origin3d::ZERO,
//        },
//        wgpu::ImageCopyBuffer {
//            buffer: &out_buffer,
//            layout: wgpu::ImageDataLayout {
//                offset: 0,
//                bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row as u32),
//                rows_per_image: std::num::NonZeroU32::new(height),
//            },
//        },
//        texture_size,
//    );
//
//    queue.submit(Some(encoder.finish()));
//
//    let buffer_slice = out_buffer.slice(..);
//    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
//
//    device.poll(wgpu::Maintain::Wait);
//
//    let padded_data = buffer_slice.get_mapped_range();
//
//    for (padded, p) in padded_data
//        .chunks_exact(padded_bytes_per_row)
//        .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row / std::mem::size_of::<Output>()))
//    {
//        p.copy_from_slice(&bytemuck::cast_slice(&padded[..unpadded_bytes_per_row]));
//    }
//
//    println!("{:?}, {:?}", &pixels[0..4], d_img.to_rgba8().get_pixel(0, 0));
//
//    if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, &pixels[..])
//    {
//        output_image.save("original_out.jpg").expect("Failed to save the image");
//    }
//}