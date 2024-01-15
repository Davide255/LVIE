# LVIE - Light and Versatile Image Editor
A light-weight open source photo editor written in Rust

## Ambition
We are currently developing a photo editor thanks to the power of rust language and the slint ui framework!

## Features
This is only an embrional phase of this project so it hasn't even the GUI, but this list is a list of features that will surely be implemented in this editor!

## Color Spaces
This editor can work with the following color spaces:
- [X] *RGB* with 8 and 16 bit support
- [X] *HSL* used for the saturation adjustements
- [X] *OkLab* used in local contrast computation
- [ ] *OkLch* that will substitute the Hsl color space

### Image manipulation
Image manipulation is the core of this project, it includes all the values you need to adjust while processing an image!

This editor can adjust:
- [X] *Exposition (EV)* by increasing or decreasing the luminance of the color prensent in the image
- [X] *Saturation* thanks to the HSV color space conversion
- [X] *Contrast* (in grayscale images at the moment) expanding the color histogram's range
- [X] *White balance* (calculates White Points in uv chromacity coordinates from the correlated colour temperature, moves on the isothermal line according to tint difference and applies chromatic adaptation using the Bradford Transform)


### Image Filters
Later in the developement will be added filters, first with useful basic filters such as:
- [X] *B&W Filter* to convert an image into gray scale
- [X] *Box Blur* with parallel computation implementation
- [X] *Gaussian Blur* thanks to the formula for the gaussian blur interpolation
- [X] *Sharpening* via Laplacian over Gaussian convolution filter
- [ ] *Local contrast*
- [ ] *Wavelets denoise* (it would be cool to adjust different channels independently too)
- [ ] *Curves* for exposition, hue, color grading ecc
- [ ] *Graduated filters* of various shapes

### GPU Support
To make the code faster, we are starting to adapt some image manipulations to be GPU acelerated.
The library we are currently using is [wgpu](https://github.com/gfx-rs/wgpu) for a cross platform backend.

Currently we are trying to get access to GPU and to set rendering pipelines to render the image.
At the moment, we are writing simple shaders using [wgsl](https://www.w3.org/TR/WGSL/) to be compiled onto GPU.

Implemented shaders are:
- [X] *B&W Shader* to convert images to grayscale using the luminance method
- [ ] *Saturation Shader* with the conversion in Hsl color space
- [ ] *Gaussian Blur Shader* to compute the gaussian blur
- [ ] *Sharpening Shader* to compute the sharpening via laplacian of gaussian

To run the GPU aceleration tests use

<code>cargo run -r -p LVIE-GPU<\code>