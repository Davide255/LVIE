use std::slice::Iter;

use LVIE_GPU::GPU;

use crate::img_processing::saturate_rgba;

#[derive(Debug)]
pub struct Data {
    pub core: Core,
    pub loaded_image: image::RgbaImage,
    pub full_res_preview: image::RgbaImage,
    pub filters: FilterArray
}

pub struct PreviewData {
    pub preview: image::RgbaImage,
    pub zoom: (f32, f32, f32)
}

#[derive(Debug)]
pub enum CoreError<'a> {
    GENERICERROR(&'a str)
}


#[derive(Debug)]
pub enum CoreBackends{
    GPU, CPU
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub filtertype: FilterType,
    pub parameters: Vec<f32>
}

#[derive(Clone, Copy, Debug)]
pub enum FilterType {
    Sharpening,
    GaussianBlur,
    Boxblur,
    Contrast,
    Saturation,
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
            filter!(FilterType::Sharpening, 0.0),
            filter!(FilterType::GaussianBlur, 0.0),
            filter!(FilterType::Boxblur, 0.0),
            filter!(FilterType::Contrast, 0.0),
            filter!(FilterType::Saturation, 0.0)
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

    pub fn set_filter(&mut self, filtertype: FilterType, parameters: Vec<f32>) {
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

#[derive(Debug)]
pub struct Core{
    backend: CoreBackends,
    gpu: Option<GPU>,
}

impl Core{
    pub fn init(backend: CoreBackends) -> Core {
        match backend {
            CoreBackends::CPU => return Core{backend, gpu: None},
            CoreBackends::GPU => {},
        }

        let mut gpu = GPU::init(None, None).expect("Failed to init the GPU");
        gpu.compile_shaders();

        Core { backend, gpu: Some(gpu) }
    }

    pub fn render_data(&mut self, img: &image::RgbaImage, filters: &FilterArray) -> Result<image::RgbaImage, crate::core::CoreError> {

        match self.backend {
            CoreBackends::CPU => {
                let mut out = img.clone();
                for filter in filters{
                    match filter.filtertype {
                        FilterType::Saturation => {
                            out = saturate_rgba(&out, filter.parameters[0]);
                        }
                        _ => unimplemented!()
                    }
                }
                return Ok(out);

            }
            CoreBackends::GPU => {
                let mut out = img.clone();
                for filter in filters {
                    match filter.filtertype {
                        FilterType::Saturation => {
                            let gpu = self.gpu.as_mut().unwrap();
                            gpu.create_texture(&out);
                            let res = gpu.render(&LVIE_GPU::GPUShaderType::Saturation, &filter.parameters);
                            if res.is_err() {
                                println!("{:?}", res.unwrap_err());
                            } else {
                                out = res.unwrap();
                            }
                        }
                        _ => unimplemented!()
                    }
                }

                return Ok(out);
            }
        }
    }
}