use std::fmt::Debug;

use image::{Pixel, Primitive};
use LVIElib::blurs::{boxblur::FastBoxBlur, gaussianblur::FastGaussianBlur};
use LVIE_GPU::{GPUShaderType, Pod, GPU};

use serde::{Deserialize, Serialize};

use LVIElib::traits::*;

use super::processors::{exposition, saturate, sharpen, whitebalance};
pub use LVIE_GPU::CRgbaImage;

use super::filters::*;
use super::ImageBuffers;

#[allow(dead_code)]
#[derive(Debug)]
pub enum RenderingError<'a> {
    GENERICERROR(&'a str),
    GPUERROR(LVIE_GPU::errors::GPUError),
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize, Default)]
pub enum RenderingBackends {
    #[default]
    GPU,
    CPU,
}

impl RenderingBackends {
    pub fn name(&self) -> &'static str {
        match self {
            &RenderingBackends::CPU => "CPU",
            &RenderingBackends::GPU => "GPU",
        }
    }
}

#[derive(Debug)]
pub struct Rendering<P>
where
    P: Pixel + Send + Sync + Debug + ToHsl + 'static,
    P::Subpixel: Scale + Primitive + Debug + Pod + Send + Sync + AsFloat,
{
    backend: RenderingBackends,
    gpu: Option<GPU>,
    pub imagebuffers: ImageBuffers<P>,
}

impl<P> Rendering<P>
where
    P: Pixel + Send + Sync + Debug + ToHsl + ToOklab + 'static,
    P::Subpixel: Scale + Primitive + Debug + Pod + Send + Sync + AsFloat,
{
    pub fn init(backend: RenderingBackends) -> Rendering<P> {
        match backend {
            RenderingBackends::CPU => {
                return Rendering {
                    backend,
                    gpu: None,
                    imagebuffers: ImageBuffers::new(),
                }
            }
            RenderingBackends::GPU => {}
        }

        let mut gpu = GPU::init(None, None).expect("Failed to init the GPU");
        gpu.compile_shaders();

        Rendering {
            backend,
            gpu: Some(gpu),
            imagebuffers: ImageBuffers::new(),
        }
    }

    pub fn render_data(
        &mut self,
        img: &CRgbaImage<P>,
        filters: &FilterArray,
    ) -> Result<CRgbaImage<P>, crate::core::RenderingError> {
        let mut out = img.clone();

        for filter in filters {
            if filter.parameters[0] != filter.filtertype.default()[0] {
                //println!("applying {:?} with values: {:?}", filter.filtertype, filter.parameters);
                let gpu_filter: Option<GPUShaderType> = {
                    match filter.filtertype {
                        FilterType::Saturation => Some(LVIE_GPU::GPUShaderType::Saturation),
                        FilterType::Exposition => Some(LVIE_GPU::GPUShaderType::Exposition),
                        FilterType::WhiteBalance => Some(LVIE_GPU::GPUShaderType::WhiteBalance),
                        _ => None,
                    }
                };

                if self.backend == RenderingBackends::GPU && gpu_filter.is_some() {
                    let gpu = self.gpu.as_mut().unwrap();
                    gpu.create_rgb_texture(&out)
                        .expect("Failed to create a texture!");
                    let res = gpu.render(&gpu_filter.unwrap(), &filter.parameters);
                    if res.is_err() {
                        return Err(RenderingError::GPUERROR(res.unwrap_err()));
                    } else {
                        out = res.unwrap();
                    }
                } else {
                    match filter.filtertype {
                        FilterType::Saturation => {
                            saturate(
                                self.imagebuffers.get_hsl_mut_updated(),
                                filter.parameters[0],
                            );
                            self.imagebuffers.set_updated(false, true, false);
                            out = self.imagebuffers.get_rgb_updated().clone();
                        }
                        FilterType::Exposition => {
                            exposition(
                                self.imagebuffers.get_hsl_mut_updated(),
                                filter.parameters[0],
                            );
                            self.imagebuffers.set_updated(false, true, false);
                            out = self.imagebuffers.get_rgb_updated().clone();
                        }
                        FilterType::Boxblur => {
                            out = FastBoxBlur(&out, filter.parameters[0] as u32);
                        }
                        FilterType::Sharpening => {
                            sharpen(
                                self.imagebuffers.get_oklab_mut_updated(),
                                filter.parameters[0],
                                filter.parameters[1] as usize,
                            );
                            self.imagebuffers.set_updated(false, false, true);
                            out = self.imagebuffers.get_rgb_updated().clone();
                        }
                        FilterType::GaussianBlur => {
                            out = FastGaussianBlur(
                                &out,
                                filter.parameters[0],
                                filter.parameters[1] as u8,
                            )
                        }
                        FilterType::WhiteBalance => {
                            whitebalance(
                                self.imagebuffers.get_rgb_mut_updated(),
                                filter.parameters[0],
                                filter.parameters[1],
                                filter.parameters[2],
                                filter.parameters[3],
                            );
                            self.imagebuffers.set_updated(true, false, false);
                            out = self.imagebuffers.get_rgb_mut_updated().clone();
                        }
                        _ => unimplemented!(),
                    }
                }
            }
        }

        Ok(out)
    }

    pub fn attach_image_buffers(&mut self, imagebuffers: ImageBuffers<P>) {
        self.imagebuffers = imagebuffers;
    }
}

impl<P> Clone for Rendering<P>
where
    P: Pixel + Send + Sync + Debug + ToHsl + 'static,
    P::Subpixel: Scale + Primitive + Debug + Pod + Send + Sync + AsFloat,
{
    fn clone(&self) -> Self {
        let gpu: Option<GPU>;
        if self.gpu.is_none() {
            gpu = None;
        } else {
            let mut _gpu = GPU::clone_from(self.gpu.as_ref().unwrap()).unwrap();
            _gpu.compile_shaders();
            gpu = Some(_gpu);
        }

        Rendering {
            backend: self.backend.clone(),
            gpu,
            imagebuffers: self.imagebuffers.clone(),
        }
    }
}
