use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CurveType {
    MONOTONE,
    SMOOTH,
}

#[derive(Debug)]
pub struct Curve {
    xs: Vec<f32>,
    ys: Vec<f32>,
    coefficients: Vec<[f32; 4]>,
    curve_type: CurveType,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum CurveError {
    OUT_OF_RANGE(String),
}

impl Curve {
    pub fn new(curve_type: CurveType) -> Curve {
        let mut c = Curve {
            xs: vec![0.0, 100.0],
            ys: vec![0.0, 100.0],
            coefficients: vec![],
            curve_type,
        };
        c.build_curve();
        return c;
    }

    pub fn to_image(&self, size: (u32, u32)) -> slint::Image {
        let mut buff = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(size.0, size.1);

        LVIElib::spline::create_plot_view(
            buff.make_mut_bytes(),
            size,
            &self.xs,
            &self.ys,
            Some(0.0..1.0),
            Some(0.0..1.0),
            (0, 0, 0, 0),
            Some(&self.coefficients),
        )
        .expect("Failed to create the plot");

        slint::Image::from_rgb8(buff)
    }

    pub fn add_point(&mut self, point: [f32; 2]) -> Result<usize, CurveError> {
        if point[0] < 0.0 || point[0] > 100.0 || point[1] < 0.0 || point[1] > 100.0 {
            return Err(CurveError::OUT_OF_RANGE(String::from(
                "Points coordinates out of range",
            )));
        }
        let mut ri = 0;
        for (i, x) in self.xs.clone().iter().enumerate() {
            if *x > point[0] {
                self.xs.insert(i, point[0]);
                self.ys.insert(i, point[1]);
                ri = i;
                break;
            }
        }
        self.build_curve();
        Ok(ri)
    }

    pub fn apply_curve(&self, val: f32) -> f32 {
        LVIElib::spline::apply_curve(val, &self.coefficients, &self.xs)
    }

    #[allow(dead_code)]
    pub fn from_points(xs: Vec<f32>, ys: Vec<f32>, curve_type: CurveType) -> Curve {
        let mut c = Curve {
            xs,
            ys,
            coefficients: vec![],
            curve_type,
        };
        c.build_curve();
        return c;
    }

    pub fn update_curve(&mut self, xs: Vec<f32>, ys: Vec<f32>) {
        self.xs = xs;
        self.ys = ys;
        self.build_curve();
    }

    pub fn update_curve_point(&mut self, index: usize, point: [f32; 2]) -> Result<(), CurveError> {
        if index < self.xs.len() {
            self.xs[index] = point[0];
            self.ys[index] = point[1];
            self.build_curve();
            Ok(())
        } else {
            Err(CurveError::OUT_OF_RANGE("this point does not exist".into()))
        }
    }

    pub fn get_raw_data(&self) -> (Vec<f32>, Vec<f32>) {
        (self.xs.clone(), self.ys.clone())
    }

    fn build_curve(&mut self) {
        self.xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.coefficients = {
            if self.curve_type == CurveType::SMOOTH {
                LVIElib::spline::spline_coefficients(
                    &self.ys,
                    &self.xs,
                    LVIElib::spline::SplineConstrains::FirstDerivatives(0.0, 0.0),
                )
            } else {
                LVIElib::spline::monotone_spline_coefficients(&self.ys, &self.xs)
            }
        };
    }

    pub fn into_rc_model(&self) -> slint::ModelRc<slint::ModelRc<f32>> {
        let mut c: Vec<slint::ModelRc<f32>> = vec![];
        for i in 0..self.xs.len() {
            c.push(std::rc::Rc::new(slint::VecModel::from(vec![self.xs[i], self.ys[i]])).into())
        }
        std::rc::Rc::new(slint::VecModel::from(c)).into()
    }

    pub fn get_point(&self, index: usize) -> [f32; 2] {
        [self.xs[index], self.ys[index]]
    }

    pub fn get_points(&self) -> Vec<[f32; 2]> {
        let mut c: Vec<[f32; 2]> = vec![];
        for i in 0..self.xs.len() {
            c.push([self.xs[i], self.ys[i]]);
        }
        c
    }

    pub fn remove_point(&mut self, index: usize) -> Result<(f32, f32), CurveError> {
        let x = self.xs.get(index);
        if index == 0 || index == self.xs.len() - 1 || x.is_none() {
            return Err(CurveError::OUT_OF_RANGE(format!(
                "{} is out of range",
                index
            )));
        } else {
            let x = self.xs.remove(index);
            let y = self.ys.remove(index);
            self.build_curve();
            return Ok((x, y));
        }
    }

    pub fn set_curve_type(&mut self, curve_type: CurveType) {
        self.curve_type = curve_type;
        self.build_curve();
    }

    pub fn get_curve_type(&self) -> &CurveType {
        &self.curve_type
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub filtertype: FilterType,
    pub parameters: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub fn default(&self) -> Vec<f32> {
        if *self == FilterType::WhiteBalance {
            return vec![6000.0, 0.0];
        }
        vec![0.0, 0.0]
    }

    pub fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone)]
// Struct to handle the application of filters
// it has an order of application of the filters
pub struct FilterArray {
    filters: Vec<Filter>,
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
    ($ty: expr) => {
        crate::core::Filter {
            filtertype: $ty,
            parameters: $ty.default()
        }
    }
}

impl FilterArray {
    pub fn new(filters: Option<Vec<Filter>>) -> FilterArray {
        let mut fa = vec![
            filter!(FilterType::Exposition, 0.0),
            filter!(FilterType::Sharpening),
            filter!(FilterType::WhiteBalance),
            filter!(FilterType::Contrast, 0.0),
            filter!(FilterType::Saturation, 0.0),
            filter!(FilterType::GaussianBlur),
            filter!(FilterType::Boxblur),
        ];

        if filters.is_some() {
            for f in filters.unwrap() {
                fa[f.filtertype.index()].parameters = f.parameters;
            }
        }

        FilterArray { filters: fa }
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
    type IntoIter = std::slice::Iter<'a, Filter>;

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
        out.filters[5].parameters[0] -= rhs.filters[5].parameters[0];
        out.filters[6].parameters[0] -= rhs.filters[6].parameters[0];

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
        out.filters[5].parameters[0] += rhs.filters[5].parameters[0];
        out.filters[6].parameters[0] += rhs.filters[6].parameters[0];

        out
    }
}
