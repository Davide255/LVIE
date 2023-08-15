mod lib;
mod helpers;
mod log_mask;

use lib::Buffer;
use palette::{Srgb, Hsv, Hsl, FromColor};

fn main() {
    let buff: Buffer<Hsl> = Buffer::<Hsl> {
        buffer: vec![Hsl::from_color(Srgb::new(100.0 / 255.0, 100.0 /255.0, 100.0/255.0))]
    };

    let new_buff: Buffer<Hsv> = buff.convert_to_color::<Hsv>();

    println!("{:#?}",new_buff.convert_to_f64());
}
