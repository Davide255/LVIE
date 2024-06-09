# Image Generator
this simple utility helps us to generate simple linear gradients in 3 different color spaces

### Usage
```
Usage: image_generator.exe [OPTIONS] --color-space <COLOR_SPACE> <WIDTH> <HEIGHT> <COMMAND>

Commands:
  solid     fill with a color
  gradient  fill with a gradient of colors
  help      Print this message or the help of the given subcommand(s)

Arguments:
  <WIDTH>   the width of the image
  <HEIGHT>  the height of the image

Options:
      --color-space <COLOR_SPACE>  select which color space should be used [possible values: rgb, rgba, hsl, hsla, oklab, oklaba]
      --path <PATH>                the path where the image will be saved
  -h, --help                       Print help
  -V, --version                    Print version
```
#### Creating a solid fill
<code>image_generator.exe 500 200 solid ff0000</code>

#### Creating a gradient fill
<code>image_generator.exe 500 200 gradient ff0000 0000ff</code> 
this will create a linear gradient from red to blue in the rgb color space:

![generated_500x200](https://github.com/Davide255/LVIE/assets/80689057/712df0b8-68f4-44ba-9a1e-c1c6e9784d1f)

we can choose another color space if we want:

<code>image_generator.exe 500 200 --color-space oklab gradient ff0000 0000ff</code> 

this example shows how to create a linear gradient from red to blue in the oklab color space:

![generated_500x200](https://github.com/Davide255/LVIE/assets/80689057/5b42a0fb-151a-4fd2-9d4a-851b2c5be586)

we can also add more colors and set intervals between them:

<code>image_generator.exe 500 200 --color-space rgb gradient 59C173 a17fe0 40 5D26C1</code> 

in this example we are taking 3 colors and creating a gradient: the first is at the start (0%) the second is at 40% of the width and the third is at the end (100%)

![generated_500x200](https://github.com/Davide255/LVIE/assets/80689057/cd89038c-8e53-4c7a-ab48-17463d8886f3)

now we can specify the angle of the gradient:

<code>image_generator.exe 500 200 --color-space rgb --path angled_gradient.png gradient 59C173 a17fe0 40 5D26C1 -a 15</code>

this is the same gradient as before but with an angle of 15 deg

![angled_gradient](https://github.com/Davide255/LVIE/assets/80689057/d6ceff8f-c9e1-4917-a693-ae732891016a)
