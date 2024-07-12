use std::fmt::Debug;

use image::{Pixel, Primitive};
use LVIE_GPU::Pod;

use LVIElib::traits::*;
use LVIE_GPU::CRgbaImage;

use super::filters::*;
use super::rendering::*;
use super::ImageBuffers;

use super::masks::Mask;

#[derive(Debug)]
pub struct Data<P>
where
    P: Pixel + Send + Sync + Debug + ToHsl + ToOklab + 'static,
    P::Subpixel: Scale + Primitive + Debug + Pod + Send + Sync + AsFloat,
{
    rendering: Rendering<P>,
    pub full_res_preview: CRgbaImage<P>,
    filters: FilterArray,
    loaded_filters: FilterArray,
    loaded_image: CRgbaImage<P>,
    pub curve: Curve,
    pub masks: Vec<Mask>,
    pub rotation: f32,
}

impl<P> Data<P>
where
    P: Pixel + Send + Sync + 'static + Debug + ToOklab + ToHsl,
    P::Subpixel: Scale + Primitive + Debug + Pod + Send + Sync + AsFloat,
{
    pub fn new(
        rendering: Rendering<P>,
        image_to_load: Option<CRgbaImage<P>>,
        filters_to_load: Option<Vec<Filter>>,
    ) -> Data<P> {
        let img = {
            if image_to_load.is_none() {
                CRgbaImage::<P>::new(0, 0)
            } else {
                image_to_load.unwrap()
            }
        };
        let mut imagebuffers = ImageBuffers::from_rgb(img.clone());
        imagebuffers.set_updates(true, true);

        let mut data = Data {
            rendering,
            full_res_preview: img.clone(),
            filters: FilterArray::new(filters_to_load),
            loaded_filters: FilterArray::new(None),
            loaded_image: img,
            curve: Curve::new(CurveType::MONOTONE),
            masks: vec![Mask::new()],
            rotation: 0.0,
        };

        data.rendering.attach_image_buffers(imagebuffers);

        data
    }

    pub fn get_loaded_filters(&self) -> &FilterArray {
        &self.loaded_filters
    }

    pub fn manual_reset_rendering(&mut self) {
        self.rendering.imagebuffers.set_updated(true, false, false);
    }

    pub fn update_all_color_spaces(&mut self) {
        self.rendering.imagebuffers.update();
    }

    pub fn image_dimensions(&self) -> (u32, u32) {
        self.loaded_image.dimensions()
    }

    pub fn load_image(&mut self, img: CRgbaImage<P>) {
        self.loaded_image = img.clone();
        self.rendering.imagebuffers.replace_rgb(img.clone());
        self.rendering.imagebuffers.update();
        self.full_res_preview = img;
        self.loaded_filters = FilterArray::new(None);
    }

    pub fn update_filters(&mut self, filters: FilterArray) {
        self.filters = filters;
    }

    pub fn norm_filters(&mut self) {
        let mut filters = &self.filters - &self.loaded_filters;
        filters.update_filter(
            FilterType::WhiteBalance,
            self.loaded_filters
                .get_filter(FilterType::WhiteBalance)
                .clone()
                .into_iter()
                .chain(
                    filters
                        .get_filter(FilterType::WhiteBalance)
                        .clone()
                        .into_iter(),
                )
                .collect(),
        );
        self.loaded_filters = &self.loaded_filters + &filters;
    }

    pub fn update_image(&mut self) -> CRgbaImage<P> {
        let mut filters = &self.filters - &self.loaded_filters;
        filters.update_filter(
            FilterType::WhiteBalance,
            self.loaded_filters
                .get_filter(FilterType::WhiteBalance)
                .clone()
                .into_iter()
                .chain(
                    filters
                        .get_filter(FilterType::WhiteBalance)
                        .clone()
                        .into_iter(),
                )
                .collect(),
        );
        self.full_res_preview = self
            .rendering
            .render_data(&(self.full_res_preview), &filters)
            .unwrap();
        self.loaded_filters = &self.loaded_filters + &filters;
        self.full_res_preview.clone()
    }

    pub fn update_filter(&mut self, filtertype: FilterType, parameters: Vec<f32>) {
        self.filters.update_filter(filtertype, parameters);
    }

    pub fn reset(&mut self) {
        self.full_res_preview = self.loaded_image.clone();
        self.filters = FilterArray::new(None);
        self.loaded_filters = FilterArray::new(None);
        self.rendering.imagebuffers.reset();
        self.rendering
            .imagebuffers
            .replace_rgb(self.full_res_preview.clone());
        self.rendering.imagebuffers.update();
    }

    pub fn export(&mut self) -> CRgbaImage<P> {
        let mut filters = self.filters.clone();
        filters.update_filter(
            FilterType::WhiteBalance,
            vec![6500.0, 0.0]
                .into_iter()
                .chain(
                    filters
                        .get_filter(FilterType::WhiteBalance)
                        .clone()
                        .into_iter(),
                )
                .collect(),
        );
        println!("{:?}", filters);
        return self
            .rendering
            .render_data(&self.full_res_preview, &filters)
            .unwrap();
    }
}
