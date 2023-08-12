# LVIE - Light and Versatile Image Editr
A light-weight open source photo editor written in Rust

## Ambition
We are currently developing a photo editor thanks to the power of rust language and the FLTK framework!

## Features
This is only an embrional phase of this project so it hasn't even the GUI, but this list is a list of features that will surely be implemented in this editor!

### Color spaces
Firstly this project use the rust palette library for color spaces management, meaning this photo editor can work with many color spaces!
At the moment we're focusing on implementing convertions between:
- [X] RGB
- [ ] HSV
- [ ] OKLAB
- [ ] OKLCH
- [ ] CIELAB

### Image manipulation
Image manipulation is the core of this project, it includes all the values you need to adjust while processing an image!

This editor can adjust:
- [X] *Exposition (EV)* by increasing or decreasing the luminance of the color prensent in the image
- [X] *Saturation* thanks to the HSV color space conversion
- [X] *Contrast* (in grayscale images at the monent) expanding the color histogram's range


### Image Filters
Later in the developement will be added filters, first with useful basic filters such as:
- [X] *B&W Filter* to convert an image into gray scale
- [X] *Gaussian Blur* thanks to the formula for the gaussian blur interpolation
