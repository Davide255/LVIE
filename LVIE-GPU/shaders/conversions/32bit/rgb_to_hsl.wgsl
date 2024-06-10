@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba32float, write>;

struct Hsl {
  h: f32,
  s: f32,
  l: f32,
}

fn rgb_to_hsl(rgb: vec3<f32>, ) -> vec3<f32> {

    var hsl = Hsl(0.0, 0.0, 0.0);

    let cmax = max(max(rgb.r, rgb.g), rgb.b);
    let cmin = min(min(rgb.r, rgb.g), rgb.b);

    hsl.l = (cmax + cmin) / 2.0;

    let delta = cmax - cmin;

    if (delta != 0.0) {
      hsl.s = delta / (1.0 - abs((2.0 * hsl.l) - 1.0));

      if (cmax == rgb.r) {
        hsl.h = ((rgb.g - rgb.b) / delta) % 6.0;
      } else if (cmax == rgb.g) {
        hsl.h = ((rgb.b - rgb.r) / delta) + 2.0;
      } else if (cmax == rgb.b) {
        hsl.h = ((rgb.r - rgb.g) / delta) + 4.0;
      }

      hsl.h = hsl.h * 60.0;
    }

    if (hsl.h < 0.0) {
        let m = hsl.h % 360.0;
        if (m != 0.0) {
          hsl.h = m + 360.0;
        }
    } else if (hsl.h == 0.0 && cmax != rgb.r) {
      hsl.h = 180.0;
    }

    return vec3<f32>(hsl.h, hsl.s, hsl.l);
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

    textureStore(output_texture, coords.xy, vec4<f32>(rgb_to_hsl(color.rgb), color.a));
}