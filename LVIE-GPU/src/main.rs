#![allow(non_snake_case)]

pub mod shader_compiler;
pub mod test_conversion;

pub mod img2win_convolution;

use shader_compiler::build;
use wgpu::util::DeviceExt;

use LVIElib::{generic_color::PixelMapping, utils::{convert_hsl_to_rgb, convert_rgb_to_hsl, norm_range_f32}};

fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let x = (width + workgroup_width - 1) / workgroup_width;
    let y = (height + workgroup_height - 1) / workgroup_height;

    (x, y)
}

fn padded_bytes_per_row(width: u32) -> usize {
    let bytes_per_row = width as usize * 4;
    let padding = (256 - bytes_per_row % 256) % 256;
    bytes_per_row + padding
}

use crate::img2win_convolution::conv;

#[allow(unreachable_code)]
fn main() {
    /*let color: Rgb<f32> = Rgb([0.5, 0.02, 0.76]);

    test_conversion::test_conversion(color);

    println!("{:?}", color.0);

    exit(0);*/

    let kernel: Vec<u8> = vec![
        56, 78, 99, 141, 2, 156, 
        255, 254, 134, 23, 1, 23, 
        68, 45, 77, 89, 100, 2,
        34, 145, 178, 199, 2, 34,
        123, 167, 178, 99, 89, 2,
        65, 87, 89, 234, 56, 134
    ];

    let filter = vec![
        23, 11, 45, 1
    ];

    let out: Vec<u8> = conv(kernel, (1, 6, 6), filter, (1, 2, 2), 2);

    println!("{:?}", out);

    std::process::exit(0);

    use std::time::Instant;

    let saturation: Vec<f32> = vec![0.1];
    let d_img = image::open("C:\\Users\\david\\Documents\\workspaces\\original.jpg")
        .expect("cannot open the image");

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
        
    let img = d_img.to_rgba8();

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
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Saturation shader"),
        source: wgpu::ShaderSource::Wgsl(build("LVIE-GPU/shaders/saturation.wgsl").into()),
    });

    let saturation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("Saturation Buffer"),
        contents: bytemuck::cast_slice(&saturation),
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
    });

    //let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //    label: None,
    //    entries: &[
    //        wgpu::BindGroupLayoutEntry {
    //            binding: 0,
    //            visibility: wgpu::ShaderStages::COMPUTE,
    //            ty: wgpu::BindingType::Buffer {
    //                ty: wgpu::BufferBindingType::Storage {
    //                    read_only: false,
    //                },
    //                has_dynamic_offset: false,
    //                min_binding_size: None,
    //            },
    //            count: None,
    //        }
    //    ]
    //});

    //let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //    label: None,
    //    layout: &bind_group_layout,
    //    entries: &[
    //        wgpu::BindGroupEntry {
    //            binding: 0,
    //            resource: saturation_buffer.as_entire_binding(),
    //        }
    //    ],
    //});

    //let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    //    label: None,
    //    bind_group_layouts: &[
    //        &bind_group_layout,
    //    ],
    //    push_constant_ranges: &[],
    //});

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Saturation pipeline"),
        layout: None, //Some(&pipeline_layout),
        module: &shader,
        entry_point: "saturation_main",
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
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: saturation_buffer.as_entire_binding(),
            }
        ],
    });

    let start = Instant::now();

    let mut encoder =
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {   let (dispatch_with, dispatch_height) =
            compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Saturation pass"),
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
    }

    let padded_bytes_per_row = padded_bytes_per_row(width);
    let unpadded_bytes_per_row = width as usize * 4;

    let mut pixels: Vec<u8> = vec![0; padded_bytes_per_row * height as usize];

    let out_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor { 
        label: Some("out buff"), 
        size: pixels.len() as u64,
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

    for (padded, pixels) in padded_data
        .chunks_exact(padded_bytes_per_row)
        .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row))
    {
        pixels.copy_from_slice(&padded[..unpadded_bytes_per_row]);
    }

    if let Some(output_image) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, &pixels[..])
    {
        output_image.save("shader-saturation.png").expect("Failed to save the image");
    }

    println!("GPU time: {}", start.elapsed().as_millis());

    let start = Instant::now();

    convert_hsl_to_rgb(convert_rgb_to_hsl(&d_img.to_rgb8()).map(|hsl|{
        *hsl.saturation_mut() = norm_range_f32(0.0..=1.0, hsl.saturation() + saturation[0]);
    })).save("prova_dalla_lib.jpg").expect("Failed to save");

    println!("CPU time: {}", start.elapsed().as_millis());
}