@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var <storage, read> kernel : array<f32>;

fn single_conv(k_size: f32, channel: i32, coords: vec2<i32>, dimensions: vec2<i32>) -> f32 {
    var sum = 0.0;
    let delta = (i32(k_size) - 1) / 2;

    for (var i = -delta; i <= delta; i++) {
        for (var j = -delta; j <= delta; j++) {
            var nc = vec2<i32>(coords.x + i, coords.y + j);
            if (nc.x >= 0 && nc.x < dimensions.x && nc.y >= 0 && nc.y < dimensions.y) {
                let v = textureLoad(input_texture, nc, 0)[channel];
                let kv = kernel[i32(k_size)*(j + delta) + (i + delta)];
                sum += v * kv;
            }
        }
    }

    return sum;
}

fn oklab_to_linsrgb(lab: vec3<f32>) -> vec3<f32> {
    let m1 = mat3x3<f32>(
        1.0,  1.0,  1.0,
        0.3963377774, -0.1055613458, -0.0894841775,
        0.2158037573, -0.0638541728, -1.2914855480
    );

    let m2 = mat3x3<f32>(
        4.0767416621, -1.2684380046, -0.0041960863,
        -3.3077115913,  2.6097574011, -0.7034186147,
        0.2309699292, -0.3413193965,  1.7076147010
    );

    let v = m1 * lab;

    let l = v[0] * v[0] * v[0];
    let m = v[1] * v[1] * v[1];
    let s = v[2] * v[2] * v[2];

    return m2 * vec3<f32>(l, m, s);
}

fn linsrgb_to_rgb(srgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        clamp(pow(srgb.r, 1.0 / 2.2), 0.0, 1.0),
        clamp(pow(srgb.g, 1.0 / 2.2), 0.0, 1.0),
        clamp(pow(srgb.b, 1.0 / 2.2), 0.0, 1.0)
    );
}

@compute @workgroup_size(16, 16)
fn shader_main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {
    let dimensions = textureDimensions(input_texture);
    let coords = vec2<u32>(global_id.xy);

    if(coords.x >= dimensions.x || coords.y >= dimensions.y) {
        return;
    }

    let color = textureLoad(input_texture, coords.xy, 0);

    let k_size = sqrt(f32(arrayLength(&kernel)));

    let out = single_conv(k_size, 2, coords, dimensions);

    textureStore(output_texture, coords.xy, vec4<f32>(linsrgb_to_rgb(oklab_to_linsrgb(vec3<f32>(color.r - out, color.g, color.b))), color.a));
}