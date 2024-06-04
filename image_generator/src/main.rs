mod generator;
use clap::{Parser, Subcommand, ValueEnum};
use generator::{linear_gradient, solid_fill};

#[derive(Debug, Clone)]
struct Color(String);

impl Color {
    pub fn parse(&self) -> Vec<u8> {
        let v = self.0.as_bytes().chunks(2).map(|b| {
            u8::from_str_radix(&String::from_utf8(b.to_vec()).unwrap(), 16).unwrap_or_else(|_| {
                panic!("color \"{}\" cannot be decoded from hex value", String::from_utf8(b.to_vec()).unwrap());
            })
        }).collect::<Vec<u8>>();
        v
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    width: u32,
    height: u32,

    #[command(subcommand)]
    fillmode: FillMode,

    #[arg(long, value_enum, help = "select which color space should be used between RGB, RGBA, HSL, HSLA, OKLAB, OKLABA")]
    color_space: ColorSpace,

    #[arg(long, help = "the path where the image will be saved")]
    path: Option<String>
}

#[derive(Debug, Clone, ValueEnum, Copy)]
enum ColorSpace {
    RGB, RGBA, HSL, HSLA, OKLAB, OKLABA
}

#[derive(Subcommand, Debug)]
enum FillMode {
    Solid {
        color: String
    },
    Gradient {
        points: Vec<String>,
        #[arg(short)]
        angle: f32
    }
}

fn main() {
    let args = Args::parse();

    match args.fillmode {
        FillMode::Gradient { points, angle } => {
            linear_gradient(
                args.width, args.height, args.color_space, 
                Color(points[0].clone()), Color(points[1].clone()), angle
            )
            .save(&args.path.unwrap_or(format!("generated_{}x{}.png", args.width, args.height).into()))
            .expect("Failed to save");
        },
        FillMode::Solid { color } => {
            solid_fill(
                args.width, args.height, 
                Color(color), args.color_space
            )
            .save(&args.path.unwrap_or(format!("generated_{}x{}.png", args.width, args.height).into()))
            .expect("Failed to save");
        }
    }
}