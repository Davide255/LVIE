mod helpers;

use helpers::norm_range;

fn main() {
    println!("{}", norm_range(1.0..=10.0, -3.0));
}
