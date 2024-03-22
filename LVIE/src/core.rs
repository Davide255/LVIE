use std::slice::Iter;

use LVIElib::blurs::{boxblur::FastBoxBlur_rgba, gaussianblur::FastGaussianBlur_rgba};
use LVIE_GPU::{GPUShaderType, GPU};

use crate::img_processing::{build_low_res_preview, exposition_rgba, saturate_rgba, sharpen_rgba, whitebalance_rgba};

#[derive(Debug)]
pub struct Data {
    rendering: Rendering,
    pub full_res_preview: image::RgbaImage,
    filters: FilterArray,
    loaded_filters: FilterArray,
    loaded_image: image::RgbaImage,
    pub zoom: (u32, u32, f32)
}

impl Data {
    pub fn new(
        rendering: Rendering,
        image_to_load: Option<image::RgbaImage>,
        filters_to_load: Option<Vec<Filter>>
    ) -> Data {
        let img = {
            if image_to_load.is_none() {
                image::RgbaImage::new(0, 0)
            } else {
                image_to_load.unwrap()
            }
        };
        Data {
            rendering,
            full_res_preview: img.clone(),
            filters: FilterArray::new(filters_to_load),
            loaded_filters: FilterArray::new(None),
            loaded_image: img,
            zoom: (0,0, 1.0)
        }
    }

    pub fn image_dimensions(&self) -> (u32, u32) {
        self.loaded_image.dimensions()
    }

    pub fn load_image(&mut self, img: image::RgbaImage) {
        self.loaded_image = img.clone();
        self.full_res_preview = img;
        self.loaded_filters = FilterArray::new(None);
    }

    pub fn generate_preview(&mut self, nwidth: u32, nheight: u32) -> PreviewData {
        let prev = PreviewData::new(
            Some(self.rendering.clone()),
            Some(build_low_res_preview(&self.full_res_preview, nwidth, nheight)), 
            Some(self.loaded_filters.filters.clone())
        );

        prev
    }

    pub fn update_image(&mut self) -> image::RgbaImage {
        let mut filters = &self.filters - &self.loaded_filters;
        filters.update_filter(
            FilterType::WhiteBalance, 
            self.loaded_filters.get_filter(FilterType::WhiteBalance).clone().into_iter()
                .chain(
                    filters.get_filter(FilterType::WhiteBalance).clone().into_iter()
                )
                .collect()
            );
        let out = self.rendering.render_data(&self.full_res_preview, &filters).unwrap();
        self.full_res_preview = out.clone();
        self.loaded_filters = &self.loaded_filters + &filters;
        out
    }

    pub fn update_filter(&mut self, filtertype: FilterType, parameters: Vec<f32>) {
        self.filters.update_filter(filtertype, parameters);
    }

    pub fn reset(&mut self) {
        self.full_res_preview = self.loaded_image.clone();
        self.filters = FilterArray::new(None);
        self.loaded_filters = FilterArray::new(None)
    }
}

pub struct PreviewData {
    rendering: Option<Rendering>,
    pub preview: image::RgbaImage,
    filters: FilterArray,
    loaded_filters: FilterArray,
    zoom: (f32, f32, f32)
}

impl PreviewData {
    pub fn new(
        rendering: Option<Rendering>,
        preview: Option<image::RgbaImage>,
        filters_to_load: Option<Vec<Filter>>
    ) -> PreviewData {
        PreviewData {
            rendering,
            preview: {
                if preview.is_some() {
                    preview.unwrap()
                } else {
                    image::RgbaImage::new(0, 0)
                }
            },
            filters: FilterArray::new(filters_to_load),
            loaded_filters: FilterArray::new(None),
            zoom: (0.0, 0.0, 1.0)
        }
    }

    pub fn update_image(&mut self) -> image::RgbaImage {
        let mut filters = &self.filters - &self.loaded_filters;
        filters.update_filter(
            FilterType::WhiteBalance, 
            self.loaded_filters.get_filter(FilterType::WhiteBalance).clone().into_iter()
                .chain(
                    filters.get_filter(FilterType::WhiteBalance).clone().into_iter()
                )
                .collect()
            );
        let out = self.rendering.as_mut().unwrap().render_data(&self.preview, &filters).unwrap();
        self.preview = out.clone();
        self.loaded_filters = &self.loaded_filters + &filters;
        out
    }

    pub fn update_filter(&mut self, filtertype: FilterType, parameters: Vec<f32>) {
        self.filters.update_filter(filtertype, parameters);
    }

    pub fn zoom(&self) -> &(f32, f32, f32) {
        &self.zoom
    }

    pub fn set_zoom(&mut self, zoom: (f32, f32, f32)) {
        self.zoom = zoom;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RenderingError<'a> {
    GENERICERROR(&'a str),
    GPUERROR(LVIE_GPU::GPUError)
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RenderingBackends{
    GPU, CPU
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub filtertype: FilterType,
    pub parameters: Vec<f32>
}

#[derive(Clone, Copy, Debug)]
pub enum FilterType {
    Exposition,
    Sharpening,
    WhiteBalance,
    Contrast,
    Saturation,
    GaussianBlur,
    Boxblur,
}

impl FilterType {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone)]
// Struct to handle the application of filters
// it has an order of application of the filters
pub struct FilterArray {
    filters: Vec<Filter>
}

#[macro_export]
macro_rules! filter {
    ($ty:expr, $($param:expr), *) => {{
        let mut parameters = Vec::new();
        $(
            parameters.push($param);
        )*
        crate::core::Filter {
            filtertype: $ty,
            parameters
        }}
    };
}

impl FilterArray {
    pub fn new(filters: Option<Vec<Filter>>) -> FilterArray {
        let mut fa = vec![
            filter!(FilterType::Exposition, 0.0),
            filter!(FilterType::Sharpening, 0.0, 0.0),
            filter!(FilterType::WhiteBalance, 6500.0, 0.0),
            filter!(FilterType::Contrast, 0.0),
            filter!(FilterType::Saturation, 0.0),
            filter!(FilterType::GaussianBlur, 0.0, 0.0),
            filter!(FilterType::Boxblur, 0.0, 0.0),
        ];

        if filters.is_some() {
            for f in filters.unwrap() {
                fa[f.filtertype.index()].parameters = f.parameters;
            }
        }

        FilterArray {
            filters: fa
        }
    }

    pub fn update_filter(&mut self, filtertype: FilterType, parameters: Vec<f32>) {
        self.filters[filtertype.index()].parameters = parameters;
    }

    pub fn get_filter(&self, filtertype: FilterType) -> &Vec<f32> {
        &self.filters[filtertype.index()].parameters
    }
}

impl IntoIterator for FilterArray {
    type Item = Filter;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.filters.into_iter()
    }
}

impl<'a> IntoIterator for &'a FilterArray {
    type Item = &'a Filter;
    type IntoIter = Iter<'a, Filter>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.filters).into_iter()
    }
}

impl std::ops::Sub for &FilterArray {
    type Output = FilterArray;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut out = FilterArray::new(Some(self.filters.clone()));

        // Exposition
        // difference between expositions
        out.filters[0].parameters[0] -= rhs.filters[0].parameters[0];
        // Sharpening
        // difference between the amounts
        out.filters[1].parameters[0] -= rhs.filters[1].parameters[0];
        // Contrast
        // difference in contrast
        out.filters[3].parameters[0] -= rhs.filters[3].parameters[0];
        // Saturation
        // difference in saturation
        out.filters[4].parameters[0] -= rhs.filters[4].parameters[0];

        // Gaussian Blur and box blur cannot be trasformed

        out
    }
}

impl std::ops::Add for &FilterArray {
    type Output = FilterArray;

    fn add(self, rhs: Self) -> Self::Output {
        let mut out = FilterArray::new(Some(self.filters.clone()));

        // Exposition
        // difference between expositions
        out.filters[0].parameters[0] += rhs.filters[0].parameters[0];
        // Sharpening
        // difference between the amounts
        out.filters[1].parameters[0] += rhs.filters[1].parameters[0];
        // White balance
        // copy the values from the other
        out.filters[2].parameters = rhs.filters[2].parameters[2..=3].to_vec();
        // Contrast
        // difference in contrast
        out.filters[3].parameters[0] += rhs.filters[3].parameters[0];
        // Saturation
        // difference in saturation
        out.filters[4].parameters[0] += rhs.filters[4].parameters[0];

        // Gaussian Blur and box blur cannot be trasformed

        out
    }
}


#[derive(Debug)]
pub struct Rendering{
    backend: RenderingBackends,
    gpu: Option<GPU>,
}

impl Rendering {
    pub fn init(backend: RenderingBackends) -> Rendering {
        match backend {
            RenderingBackends::CPU => return Rendering{backend, gpu: None},
            RenderingBackends::GPU => {},
        }

        let mut gpu = GPU::init(None, None).expect("Failed to init the GPU");
        gpu.compile_shaders();

        Rendering { backend, gpu: Some(gpu) }
    }

    pub fn render_data(&mut self, img: &image::RgbaImage, filters: &FilterArray) -> Result<image::RgbaImage, crate::core::RenderingError> {

        let mut out = img.clone();

        for filter in filters {
            if filter.parameters[0] != 0.0 {
                //println!("applying {:?} with values: {:?}", filter.filtertype, filter.parameters);
                let gpu_filter: Option<GPUShaderType> = {
                    match filter.filtertype {
                        FilterType::Saturation => Some(LVIE_GPU::GPUShaderType::Saturation),
                        FilterType::Exposition => Some(LVIE_GPU::GPUShaderType::Exposition),
                        FilterType::WhiteBalance => Some(LVIE_GPU::GPUShaderType::WhiteBalance),
                        _ => None,
                    }
                };

                if self.backend == RenderingBackends::GPU && gpu_filter.is_some(){
                    let gpu = self.gpu.as_mut().unwrap();
                    gpu.create_texture(&out);
                    let res = gpu.render(&gpu_filter.unwrap(), &filter.parameters);
                    if res.is_err() {
                        return Err(RenderingError::GPUERROR(res.unwrap_err()));
                    } else {
                        out = res.unwrap();
                    }
                } else {
                    match filter.filtertype {
                        FilterType::Saturation => {
                            out = saturate_rgba(&out, filter.parameters[0]);
                        }
                        FilterType::Exposition => {
                            out = exposition_rgba(&out, filter.parameters[0]);
                        }
                        FilterType::Boxblur => {
                            out = FastBoxBlur_rgba(&out, filter.parameters[0] as u32);
                        }
                        FilterType::Sharpening => {
                            out = sharpen_rgba(&out, filter.parameters[0], filter.parameters[1] as usize)
                        }
                        FilterType::GaussianBlur => {
                            out = FastGaussianBlur_rgba(&out, filter.parameters[0], filter.parameters[1] as u8)
                        }
                        FilterType::WhiteBalance => {
                            out = whitebalance_rgba(&out, filter.parameters[0], filter.parameters[1], filter.parameters[2], filter.parameters[3]);                       }
                        _ => unimplemented!()
                    }
                }
            }
        }

        Ok(out)

    }
}

impl Clone for Rendering {
    fn clone(&self) -> Self {
        if self.gpu.is_none() {
            Rendering {
                backend: self.backend,
                gpu: None,
            }
        } else {
            let mut gpu = GPU::clone_from(self.gpu.as_ref().unwrap()).unwrap();
            gpu.compile_shaders();
            Rendering {
                backend: self.backend,
                gpu: Some(gpu)
            }
        }
    }
}