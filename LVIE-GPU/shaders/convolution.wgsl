@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var <storage, read> kernel : array<f32>; 
@group(0) @binding(3) var <storage, read_write> lenght : array<f32>;

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

    //sum = sum / k_size;

    return sum;
}

fn convolve_channels(coords: vec2<i32>, dimensions: vec2<i32>) -> vec3<f32> {
    var out = vec3<f32>();

    let k_size = sqrt(f32(arrayLength(&kernel)));

    for (var c = 0; c < 3; c++) {
        out[c] = single_conv(k_size, c, coords, dimensions);
    }

    return out;
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

    lenght[0] = 0.0;

    textureStore(output_texture, coords.xy, vec4<f32>(convolve_channels(coords, dimensions), color.a));
}