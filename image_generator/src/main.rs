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
        #[arg(short, default_value_t = 0.0, required = false)]
        angle: f32
    }
}

fn get_color_at(v: &Vec<f32>, p: &Vec<f32>, size: f32, pos: f32) -> Option<f32>{
    let mut i = p.len() + 1;
    for k in 0..p.len() {
        if size*p[k]/100.0 <= pos && pos < size*p[k+1]/100.0 {
            i = k;
            break;
        }
    }

    if i > p.len() { return None; }
    let d = size*p[i+1]/100.0 - size*p[i]/100.0;
    let np = pos - size*p[i]/100.0;

    Some(v[i] - (v[i]-v[i+1])*np/d)
}

fn main() {

    let values: Vec<f32> = vec![0.0, 10.0, 5.0, 7.0];
    let positions: Vec<f32> = vec![0.0, 20.0, 80.0, 100.0];

    let steps = 20;

    let mut gradient = vec![];

    for k in 0..steps {
        gradient.push(get_color_at(&values, &positions, steps as f32, k as f32).unwrap())
    }

    println!("{:?}", gradient);

    std::process::exit(0);
    let args = Args::parse();

    println!("colors: {:?}", args.fillmode);

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