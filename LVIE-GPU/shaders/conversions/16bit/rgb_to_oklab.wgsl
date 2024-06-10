@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba16float, write>;

struct LinSrgb {
    r: f32,
    g: f32,
    b: f32
}

struct Oklab {
    l: f32,
    a: f32,
    b: f32
}

fn rgb_to_linsrgb(rgb: vec3<f32>) -> LinSrgb {
    return LinSrgb(
        pow(rgb.r, 2.2),
        pow(rgb.g, 2.2),
        pow(rgb.b, 2.2)
    );
}

fn linsrgb_to_rgb(srgb: LinSrgb) -> vec3<f32> {
    return vec3<f32>(
        clamp(pow(srgb.r, 1.0 / 2.2), 0.0, 1.0),
        clamp(pow(srgb.g, 1.0 / 2.2), 0.0, 1.0),
        clamp(pow(srgb.b, 1.0 / 2.2), 0.0, 1.0)
    );
}

fn linsrgb_to_oklab(rgb: LinSrgb) -> Oklab {
    let r = rgb.r;
    let g = rgb.g;
    let b = rgb.b;

    let m1 = mat3x3<f32>(
        0.4122214708,
        0.2119034982,
        0.0883024619,
        0.5363325363,
        0.6806995451,
        0.2817188376,
        0.0514459929,
        0.1073969566,
        0.6299787005,
    );

    let m2 = mat3x3<f32>(
        0.2104542553,
        1.9779984951,
        0.0259040371,
        0.7936177850,
        -2.4285922050,
        0.7827717662,
        -0.0040720468,
        0.4505937099,
        -0.8086757660,
    );

    let v = m1 * vec3<f32>(r, g, b);

    let l = pow(v[0], 1.0/3.0); 
    let m = pow(v[1], 1.0/3.0);
    let s = pow(v[2], 1.0/3.0);

    let lab = m2 * vec3<f32>(l, m, s);

    return Oklab(
        lab[0], lab[1], lab[2]
    );
}

@compute @workgroup_size(16, 16)
fn shader_main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {
    let dimensions = textureDimensions(input_texture);
    let coords = vec2<i32>(global_id.xy);

    if(coords.x >= dimensions.x || coords.y >= dimensions.y) {
        return;
    }

    let color = textureLoad(input_texture, coords.xy, 0);
    
    let oklab = linsrgb_to_oklab(rgb_to_linsrgb(color.rgb));

    textureStore(output_texture, coords.xy, vec4<f32>(oklab.l, oklab.a, oklab.b, color.a));
}