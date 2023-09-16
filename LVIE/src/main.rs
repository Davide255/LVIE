mod hsl;

use hsl::{hslf32_to_rgbf32, rgbf32_to_hslf32};

fn main() {
    let c = rgbf32_to_hslf32(127.0, 177.0, 77.0).0;
    println!("{:?}", c);
    println!("{:?}", hslf32_to_rgbf32(c[0], c[1], c[2]));
}
