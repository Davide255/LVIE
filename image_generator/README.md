# Image Generator
this simple utility helps us to generate simple linear gradients in 3 different color spaces

### Usage
```
Usage: image_generator.exe [OPTIONS] --color-space <COLOR_SPACE> <WIDTH> <HEIGHT> <COMMAND>

Commands:
  solid
  gradient
  help      Print this message or the help of the given subcommand(s)

Arguments:
  <WIDTH>
  <HEIGHT>

Options:
      --color-space <COLOR_SPACE>  select which color space should be used [possible values: rgb, rgba, hsl, hsla, oklab, oklaba]
      --path <PATH>                the path where the image will be saved
  -h, --help                       Print help
  -V, --version                    Print version
```
