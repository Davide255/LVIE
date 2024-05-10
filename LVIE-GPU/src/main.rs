#![allow(non_snake_case)]

pub mod shader_compiler;

use image::{GenericImageView, Primitive, RgbaImage};
use shader_compiler::build;
use wgpu::util::DeviceExt;
use LVIElib::matrix::convolution::laplacian_of_gaussian;
use LVIE_GPU::GPU;

use std::time::Instant;

fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}

fn padded_bytes_per_row<T: Primitive>(width: u32) -> usize {
    let bytes_per_row = width as usize * std::mem::size_of::<T>()*4;
    let padding = (256 - bytes_per_row % 256) % 256;
    bytes_per_row + padding
}

fn convert_to_oklab_gpu(img: RgbaImage) -> Vec<f32>{
    use pollster::FutureExt;

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .block_on().expect("Cannot create adapter");

    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .block_on().expect("cannot create device ad queue");

    let (width, height) = img.dimensions();

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    
    let input_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("input texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    queue.write_texture(
        input_texture.as_image_copy(),
        bytemuck::cast_slice(img.as_raw()),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * width),
            rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
        },
        texture_size,
    );

    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("output texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Saturation shader"),
        source: wgpu::ShaderSource::Wgsl(build("LVIE-GPU/shaders/convert_to_oklab.wgsl").into()),
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Saturation pipeline"),
        layout: None, //Some(&pipeline_layout),
        module: &shader,
        entry_point: "shader_main",
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }
        ],
    });

    let mut encoder =
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {   let (dispatch_with, dispatch_height) =
            compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
    }

    let padded_bytes_per_row = padded_bytes_per_row::<f32>(width);
    let unpadded_bytes_per_row = width as usize * std::mem::size_of::<f32>()*4;

    let mut pixels = vec![0f32; padded_bytes_per_row * height as usize];

    //let mut pixels: Vec<f32> = vec![0.0; (padded_bytes_per_row / 4) * height as usize];

    let out_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor { 
        label: Some("out buff"), 
        size: (pixels.len() * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false 
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
                bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row as u32),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
        },
        texture_size,
    );

    queue.submit(Some(encoder.finish()));

    let buffer_slice = out_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    device.poll(wgpu::Maintain::Wait);

    let padded_data = buffer_slice.get_mapped_range();

    for (padded, p) in padded_data
        .chunks_exact(padded_bytes_per_row)
        .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row / std::mem::size_of::<f32>()))
    {
        p.copy_from_slice(&bytemuck::cast_slice(&padded[..unpadded_bytes_per_row]));
    }

    //for (padded, p) in u8b
    //    .chunks_exact(padded_bytes_per_row)
    //    .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row / 4))
    //{
    //    p.copy_from_slice(&bytemuck::cast_slice(&padded[..unpadded_bytes_per_row]));
    //}

    return pixels;
}

#[allow(unreachable_code)]
fn main() {

    let gpus = GPU::list_GPUs(None);

    for gpu in gpus {
        println!("{} - {:?}", gpu.info.name, gpu.info.backend);
    }

    let d_img = image::open("C:\\Users\\david\\Documents\\Projects\\workspaces\\original.jpg")
        .expect("cannot open the image");

    //let start = Instant::now();
//
    //let (mut vl, mut va, mut vb) = (Vec::<f32>::new(), Vec::<f32>::new(), Vec::<f32>::new());
    //let content = d_img.to_rgb8().to_vec();
    //for i in 0..content.len() / 3 {
    //    let o_color = Oklab::from(Rgb([
    //        content[3 * i] as f32 / 255f32,
    //        content[3 * i + 1] as f32 / 255f32,
    //        content[3 * i + 2] as f32 / 255f32,
    //    ]));
//
    //    let channels = o_color.channels();
//
    //    let (cl, ca, cb) = (channels[0], channels[1], channels[2]);
//
    //    vl.push(cl);
    //    va.push(ca);
    //    vb.push(cb);
    //}
//
    //println!("CPU: {}", start.elapsed().as_millis());

    let start = Instant::now();

    let out = convert_to_oklab_gpu(d_img.to_rgba8());

    println!("GPU: {}", start.elapsed().as_millis());

    use pollster::FutureExt;

    type Output = u8;

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .block_on().expect("Cannot create adapter");

    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .block_on().expect("cannot create device ad queue");

    let (width, height) = d_img.dimensions();

    let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("Kernel"),
        contents: bytemuck::cast_slice(&laplacian_of_gaussian(0.5, 5, 5).get_content().to_owned()),
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
    });

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    
    let input_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("input texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    queue.write_texture(
        input_texture.as_image_copy(),
        bytemuck::cast_slice(&out),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * width * std::mem::size_of::<f32>() as u32),
            rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
        },
        texture_size,
    );

    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("output texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Saturation shader"),
        source: wgpu::ShaderSource::Wgsl(build("LVIE-GPU/shaders/sharpening.wgsl").into()),
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Saturation pipeline"),
        layout: None, //Some(&pipeline_layout),
        module: &shader,
        entry_point: "shader_main",
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &input_texture.create_view(&wgpu::TextureViewDescriptor {
                        label: None,
                        format: Some(wgpu::TextureFormat::Rgba32Float),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None
                    }),
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
                resource: kernel_buffer.as_entire_binding(),
            }
        ],
    });

    let mut encoder =
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {   let (dispatch_with, dispatch_height) =
            compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
    }

    let padded_bytes_per_row = padded_bytes_per_row::<f32>(width);
    let unpadded_bytes_per_row = width as usize * std::mem::size_of::<Output>()*4;

    let mut pixels = vec![0 as Output; padded_bytes_per_row * height as usize];

    //let mut pixels: Vec<f32> = vec![0.0; (padded_bytes_per_row / 4) * height as usize];

    let out_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor { 
        label: Some("out buff"), 
        size: (pixels.len() * std::mem::size_of::<Output>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false 
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
                bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row as u32),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
        },
        texture_size,
    );

    queue.submit(Some(encoder.finish()));

    let buffer_slice = out_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    device.poll(wgpu::Maintain::Wait);

    let padded_data = buffer_slice.get_mapped_range();

    for (padded, p) in padded_data
        .chunks_exact(padded_bytes_per_row)
        .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row / std::mem::size_of::<Output>()))
    {
        p.copy_from_slice(&bytemuck::cast_slice(&padded[..unpadded_bytes_per_row]));
    }

    if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, &pixels[..])
    {
        output_image.save("original_out.jpg").expect("Failed to save the image");
    }

    std::process::exit(0);

    //let k_size = 4*4;
//
    //let kernel = vec![1.0/k_size as f32; k_size];
//
    //let instance = wgpu::Instance::new(wgpu::Backends::all());
    //let adapter = instance
    //    .request_adapter(&wgpu::RequestAdapterOptionsBase {
    //        power_preference: wgpu::PowerPreference::HighPerformance,
    //        force_fallback_adapter: false,
    //        compatible_surface: None,
    //    })
    //    .block_on().expect("Cannot create adapter");
//
    //let (device, queue) = adapter
    //    .request_device(&Default::default(), None)
    //    .block_on().expect("cannot create device ad queue");    
    //    
    //let img = d_img.to_rgba8();
//
    //println!("{:?}", img.dimensions());
//
    //let (width, height) = img.dimensions();
//
    //let texture_size = wgpu::Extent3d {
    //    width,
    //    height,
    //    depth_or_array_layers: 1,
    //};
    //
    //let input_texture = device.create_texture(&wgpu::TextureDescriptor {
    //    label: Some("input texture"),
    //    size: texture_size,
    //    mip_level_count: 1,
    //    sample_count: 1,
    //    dimension: wgpu::TextureDimension::D2,
    //    format: wgpu::TextureFormat::Rgba8Unorm,
    //    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    //});
//
    //queue.write_texture(
    //    input_texture.as_image_copy(),
    //    bytemuck::cast_slice(img.as_raw()),
    //    wgpu::ImageDataLayout {
    //        offset: 0,
    //        bytes_per_row: std::num::NonZeroU32::new(4 * width),
    //        rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
    //    },
    //    texture_size,
    //);
//
    //let output_texture = device.create_texture(&wgpu::TextureDescriptor {
    //    label: Some("output texture"),
    //    size: texture_size,
    //    mip_level_count: 1,
    //    sample_count: 1,
    //    dimension: wgpu::TextureDimension::D2,
    //    format: wgpu::TextureFormat::Rgba8Unorm,
    //    usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
    //});
//
    //let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    //    label: Some("Saturation shader"),
    //    source: wgpu::ShaderSource::Wgsl(build("LVIE-GPU/shaders/convolution.wgsl").into()),
    //});
//
    //let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
    //    label: Some("Kernel"),
    //    contents: bytemuck::cast_slice(&kernel),
    //    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
    //});
//
    //let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    //    label: Some("Saturation pipeline"),
    //    layout: None, //Some(&pipeline_layout),
    //    module: &shader,
    //    entry_point: "shader_main",
    //});
//
    //let mut local_buffer = vec![0f32; 6];
//
    //let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //    label: Some("Storage Buffer"),
    //    size: std::mem::size_of_val(&local_buffer) as u64,
    //    usage: wgpu::BufferUsages::STORAGE
    //        | wgpu::BufferUsages::COPY_SRC
    //        | wgpu::BufferUsages::COPY_DST,
    //    mapped_at_creation: false,
    //});
//
    //let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //    label: Some("Texture bind group"),
    //    layout: &pipeline.get_bind_group_layout(0),
    //    entries: &[
    //        wgpu::BindGroupEntry {
    //            binding: 0,
    //            resource: wgpu::BindingResource::TextureView(
    //                &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
    //            ),
    //        },
    //        wgpu::BindGroupEntry {
    //            binding: 1,
    //            resource: wgpu::BindingResource::TextureView(
    //                &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
    //            ),
    //        },
    //        wgpu::BindGroupEntry {
    //            binding: 2,
    //            resource: kernel_buffer.as_entire_binding(),
    //        },
    //        wgpu::BindGroupEntry {
    //            binding: 3,
    //            resource: storage_buffer.as_entire_binding(),
    //        }
    //    ],
    //});
//
    //let start = Instant::now();
//
    //let mut encoder =
    //device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
//
    //{   let (dispatch_with, dispatch_height) =
    //        compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
    //    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
    //        label: Some("Saturation pass"),
    //    });
    //    compute_pass.set_pipeline(&pipeline);
    //    compute_pass.set_bind_group(0, &texture_bind_group, &[]);
    //    compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
    //}
//
    //let padded_bytes_per_row = padded_bytes_per_row::<u8>(width);
    //let unpadded_bytes_per_row = width as usize * std::mem::size_of::<u8>()*4;
//
    //let mut u8b = vec![0u8; padded_bytes_per_row * height as usize];
//
    ////let mut pixels: Vec<f32> = vec![0.0; (padded_bytes_per_row / 4) * height as usize];
//
    //let out_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor { 
    //    label: Some("out buff"), 
    //    size: u8b.len() as u64,
    //    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    //    mapped_at_creation: false 
    //});
//
    //let out_buffer2 = device.create_buffer(&wgpu::BufferDescriptor {
    //    label: Some("Output Buffer"),
    //    size: std::mem::size_of_val(&local_buffer) as u64,
    //    usage: wgpu::BufferUsages::MAP_READ
    //        | wgpu::BufferUsages::COPY_DST,
    //    mapped_at_creation: false,
    //});
//
    //encoder.copy_texture_to_buffer(
    //    wgpu::ImageCopyTexture {
    //        aspect: wgpu::TextureAspect::All,
    //        texture: &output_texture,
    //        mip_level: 0,
    //        origin: wgpu::Origin3d::ZERO,
    //    },
    //    wgpu::ImageCopyBuffer {
    //        buffer: &out_buffer,
    //        layout: wgpu::ImageDataLayout {
    //            offset: 0,
    //            bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row as u32),
    //            rows_per_image: std::num::NonZeroU32::new(height),
    //        },
    //    },
    //    texture_size,
    //);
//
    //encoder.copy_buffer_to_buffer(
    //    &storage_buffer, 0,
    //    &out_buffer2, 0,
    //    std::mem::size_of::<f32>() as u64
    //);
//
    //queue.submit(Some(encoder.finish()));
//
    //let buffer_slice = out_buffer.slice(..);
    //buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
//
    //device.poll(wgpu::Maintain::Wait);
//
    //let padded_data = buffer_slice.get_mapped_range();
//
    //for (padded, p) in padded_data
    //    .chunks_exact(padded_bytes_per_row)
    //    .zip(u8b.chunks_exact_mut(unpadded_bytes_per_row))
    //{
    //    p.copy_from_slice(&padded[..unpadded_bytes_per_row]);
    //}
//
    ////for (padded, p) in u8b
    ////    .chunks_exact(padded_bytes_per_row)
    ////    .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row / 4))
    ////{
    ////    p.copy_from_slice(&bytemuck::cast_slice(&padded[..unpadded_bytes_per_row]));
    ////}
//
    //if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, &u8b[..])
    //{
    //    output_image.save("original_out.jpg").expect("Failed to save the image");
    //}
//
    //let output_buffer_slice = out_buffer2.slice(..);
    //output_buffer_slice.map_async(wgpu::MapMode::Read, |r| {
    //    if r.is_err() {
    //        panic!("failed to map output staging buffer");
    //    }
    //});
    //device.poll(wgpu::Maintain::Wait);
    //local_buffer.copy_from_slice(
    //    &bytemuck::cast_slice(&*output_buffer_slice.get_mapped_range())
    //);
//
    //out_buffer2.unmap();
//
    //println!("Result: {:?}", local_buffer);
//
    //println!("GPU time: {}", start.elapsed().as_millis());
//
    //let start = Instant::now();
//
    //let mut kernel = Matrix::new(kernel, (k_size as f32).sqrt() as usize, (k_size as f32).sqrt() as usize);
    //kernel.pad(width as usize, height as usize, 0.0);
//
    //let out = apply_convolution(Matrix::new(d_img.to_rgb8().to_vec(), height as usize, width as usize * 3), &kernel).get_content().to_owned();
    //ImageBuffer::<Rgb<u8>, _>::from_raw(d_img.width(), d_img.height(), out).unwrap().save("prova_conv.jpg");
//
    //println!("CPU time: {}", start.elapsed().as_millis());

    //for x in 0..9 {
    //    println!("[{}, {}, {}, {}], {:?}", pixels[4*x], pixels[4*x +1], pixels[4*x+2], pixels[4*x+3], Oklaba::from(*img.get_pixel(x as u32, 0)).to_vec());
    //}

    //let start = Instant::now();
    //
    //convert_hsl_to_rgb(convert_rgb_to_hsl(&d_img.to_rgb8()).map(|hsl|{
    //    *hsl.saturation_mut() = norm_range_f32(0.0..=1.0, hsl.saturation() + saturation[0]);
    //})).save("prova_dalla_lib.jpg").expect("Failed to save");
    //
    //println!("CPU time: {}", start.elapsed().as_millis());
}