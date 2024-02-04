#![allow(non_snake_case)]
use wgpu::util::DeviceExt;

use image::Rgb;

pub fn test_conversion(color: Rgb<f32>) {

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

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("test shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("C:\\Users\\david\\Documents\\workspaces\\LVIE-GPU\\shaders\\test_hsl_conversion.wgsl").into()),
    });

    let mut local_buffer = color.0;

    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Storage Buffer"),
        size: std::mem::size_of_val(&local_buffer) as u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let in_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("input"),
        contents: bytemuck::cast_slice(&local_buffer),
        usage:  wgpu::BufferUsages::MAP_WRITE
        | wgpu::BufferUsages::COPY_SRC,
    });

    let out_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: std::mem::size_of_val(&local_buffer) as u64,
        usage: wgpu::BufferUsages::MAP_READ
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
        label: Some("Bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry{
                visibility: wgpu::ShaderStages::COMPUTE,
                binding: 0,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None},
                count: None
            }
        ]
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
        label: Some("layout"), 
        bind_group_layouts: &[ &bind_group_layout, ], push_constant_ranges: &[] });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    let input_buffer_slice = in_buffer.slice(..);
    input_buffer_slice.map_async(wgpu::MapMode::Write, move |r| {
        if r.is_err() {
            panic!("failed to map input staging buffer");
        }
    });
    device.poll(wgpu::Maintain::Wait);
    input_buffer_slice.get_mapped_range_mut().clone_from_slice(bytemuck::cast_slice(&local_buffer));

    in_buffer.unmap();

    let mut encoder =
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_buffer(
        &in_buffer, 0,
        &storage_buffer, 0,
        4*3
    );
    {   
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Saturation pass"),
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }

    encoder.copy_buffer_to_buffer(
        &storage_buffer, 0,
        &out_buffer, 0,
        4*3
    );

    queue.submit(Some(encoder.finish()));

    let output_buffer_slice = out_buffer.slice(..);
    output_buffer_slice.map_async(wgpu::MapMode::Read, |r| {
        if r.is_err() {
            panic!("failed to map output staging buffer");
        }
    });
    device.poll(wgpu::Maintain::Wait);
    local_buffer.copy_from_slice(
        &bytemuck::cast_slice(&*output_buffer_slice.get_mapped_range())
    );

    out_buffer.unmap();

    println!("Result: {:?}", local_buffer)

}