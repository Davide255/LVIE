#[cfg(debug_assertions)]
use std::env;

fn main() {
    #[cfg(debug_assertions)]
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg=/stack:{}", 3 * 1024 * 1024);
    }

    slint_build::compile("ui/LVIE.slint").unwrap();
}
