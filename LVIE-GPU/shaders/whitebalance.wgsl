@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba8unorm, write>;

struct LinSrgb {
    r: f32,
    g: f32,
    b: f32
}

struct Xyz {
    x: f32,
    y: f32,
    z:f32
}

struct Lms {
    l: f32,
    m: f32,
    s: f32
}

fn lrgb_to_xyz(rgb: LinSrgb) -> Xyz {
    return Xyz(
        rgb.r * 0.4124564 + rgb.g * 0.3575761 + rgb.b * 0.1804375, 
        rgb.r * 0.2126729 + rgb.g * 0.7151522 + rgb.b * 0.0721750, 
        rgb.r * 0.0193339 + rgb.g * 0.1191920 + rgb.b * 0.9503041,
    );
}

fn xyz_to_linsrgb(xyz: Xyz) -> LinSrgb {

}

fn xyz_to_lms(xyz: Xyz) -> Lms {
    return Lms(
        xyz.x * 0.8951 + xyz.y * 0.2664 + xyz.z * -0.1614,
        xyz.x * -0.7502 + xyz.y * 1.7135 + xyz.z * 0.0367,
        xyz.x * 0.0389 + xyz.y * -0.0685 + xyz.z* 1.0296,
    );
}

@compute @workgroup_size(16, 16)
fn shader_main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {

}