struct Hsl {
  h: f32,
  s: f32,
  l: f32,
}

@group(0) @binding(0) var <storage, read_write> color : array<f32>;


fn rgb_to_hsl(rgb: vec3<f32>, ) -> Hsl {
    // implememntation of LVIElib::hsl::rgb_to_hsl function

    var hsl = Hsl(0.0, 0.0, 0.0);

    let cmax = max(max(rgb.r, rgb.g), rgb.b);
    let cmin = min(min(rgb.r, rgb.g), rgb.b);

    hsl.l = clamp((cmax + cmin) / 2.0, 0.0, 1.0);

    let delta = cmax - cmin;

    if (delta != 0.0) {
      hsl.s = clamp(delta / (1.0 - abs((2.0 * hsl.l) - 1.0)), 0.0, 1.0);

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

    return hsl;
}

fn hsl_to_rgb(hsl: Hsl) -> vec3<f32> {
  // implememntation of LVIElib::hsl::hsl_to_rgb functions

    let h = hsl.h;
    let s = hsl.s;
    let l = hsl.l;
  
    let c = s * (1.0 - abs((2.0 * l) - 1.0));
    let x = c * (1.0 - abs(((h / 60.0) % 2.0) - 1.0));
    let m = l - (c / 2.0);

    var rgb = vec3<f32>(0.0);

    if (0.0 <= h && h < 60.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (60.0 <= h && h < 120.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (120.0 <= h && h < 180.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (180.0 <= h && h < 240.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (240.0 <= h && h < 300.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else if (300.0 <= h && h < 360.0) {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return vec3<f32>(rgb.r + m,
    rgb.g + m,
    rgb.b + m);
}

@compute @workgroup_size(1)
fn main() {
    let hsl = rgb_to_hsl(vec3<f32>(color[0], color[1], color[2]));
    let rgb = hsl_to_rgb(hsl);
    color[0] = rgb.r;
    color[1] = rgb.g;
    color[2] = rgb.b;
}