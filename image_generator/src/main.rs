mod generator;
use std::{fmt::Error, path::PathBuf};

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
    #[arg(help="the width of the image")]
    width: u32,
    #[arg(help="the height of the image")]
    height: u32,

    #[command(subcommand, help = "select how the image will be filled")]
    fillmode: FillMode,

    #[arg(long, default_value_t=ColorSpace::RGB, value_enum, help = "select which color space should be used")]
    color_space: ColorSpace,

    #[arg(long, help = "the path where the image will be saved")]
    path: Option<PathBuf>
}

#[derive(Debug, Clone, ValueEnum, Copy)]
enum ColorSpace {
    RGB, RGBA, HSL, HSLA, OKLAB, OKLABA
}

#[derive(Subcommand, Debug)]
enum FillMode {
    #[command(about = "fill with a color")]
    Solid {
        #[arg(help="select the color by copying his hex value")]
        color: String
    },
    #[command(about = "fill with a gradient of colors", 
    long_about = "This command creates a linear gradient over the image, you can choose which colors should be used and their position (%)")]
    Gradient {
        #[arg(help="input a bunch of colors: the format should be:\n(the color in hex values) \"xxxxxx\" (optionally the position in percentage) [0] \"xxxxxx\" [15] ... \nmissing percentages will be calculated automatically")]
        points: Vec<String>,
        #[arg(short, default_value_t = 0.0, required = false, help = "the angle of the linear gradient")]
        angle: f32
    }
}

fn str_is_color(s: &str) -> bool {
    if s.parse::<f32>().is_ok() && s.len() != 6 && s.len() != 8 {
        false
    } else { true }
}

fn parse_color(s: Vec<String>) -> Result<Vec<(Color, f32)>, Error> {
    let mut s = s.clone();
    let mut out = vec![];

    out.push((Color(s.remove(0)), 0.0));
    if !str_is_color(&s[0]) {
        out[0].1 = s.remove(0).parse::<f32>().unwrap();
    }
    
    let mut c = true;
    for k in s {
        if c && str_is_color(&k) {
            out.push((Color(k), -1.0));
        } else {
            let v = k.parse::<f32>();
            if v.is_ok() {
                out.last_mut().unwrap().1 = v.unwrap();
            } else if v.is_err() && str_is_color(&k) {
                out.push((Color(k), -1.0));
                c = !c;
            } else {
                return Err(Error);
            }
        }
        c = !c;
    }

    if out.last().unwrap().1 == -1.0 {
        out.last_mut().unwrap().1 = 100.0;
    }

    let mut i = 1;
    while i <= out.len() - 1  {
        let k = &out[i];
        if k.1 == -1.0 {
            let mut c = 1;
            let mut t = out.get(i+c).unwrap_or(&(Color("000000".into()), 100.0)).1;
            while t == -1.0 {
                c += 1;
                t = out.get(i+c).unwrap_or(&(Color("000000".into()), 100.0)).1;
            }

            let s = (out[i+c].1 - out[i-1].1) / (c+1) as f32;
            for k in 1..=c { 
                out[i+k-1].1 = out[i-1].1 + s*k as f32;
            }
            i += c;
        }
        i += 1;
    }

    Ok(out)
}

fn main() {
    
    let args = Args::parse();

    match args.fillmode {
        FillMode::Gradient { points, angle } => {
            let colors: Vec<(Color, f32)> = parse_color(points).unwrap();
            linear_gradient(
                args.width, args.height, args.color_space, 
                colors, angle
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

#[cfg(test)]
mod tests {
    use super::parse_color;
    use image::Pixel;
    #[test]
    fn test_points_gradient() {
        LVIElib::math::linear_gradient_more_points(
            (1000, 1000), 
            parse_color(
                vec!["ff0000".into(), "0".into(), "00ff00".into(), "15.0".into(), "0000ff".into()]
            ).unwrap().into_iter().map(|(c, p)| {
                (image::Rgb::from_slice(&c.parse()).to_rgba(), p)
            }).collect(),
            0.0
        ).save("prova.png").expect("Failed to save image");
    }
}