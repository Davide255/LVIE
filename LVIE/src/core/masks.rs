use LVIElib::{traits::Scale, utils::boundary_fill};

#[derive(Debug)]
pub enum MaskError {
    PointNotFound,
    MaskNotClosed
}

#[derive(Debug, Default)]
pub struct Mask {
    // represent the points of the mask
    mask_points: Vec<[f32; 2]>,
    // represent the control points of the curves between two main points
    bezier_control_points: Vec<[[f32; 2]; 2]>,
    closed: bool
}

impl Mask {
    pub fn new() -> Mask {
        Mask { mask_points: Vec::new(), bezier_control_points: Vec::new(), closed: false }
    }

    pub fn close(&mut self) {
        if self.closed { return; }
        self.closed = true;

        self.bezier_control_points.push([
            [-1.0, -1.0],
            [-1.0, -1.0]
        ]);
    }

    pub fn add_point(&mut self, coords: [f32; 2]) -> usize {
        if self.mask_points.len() > 0 {
            self.bezier_control_points.push([
                [-1.0, -1.0],
                [-1.0, -1.0]
            ]);
        }
        self.mask_points.push(coords);
        self.mask_points.len() - 1
    }

    pub fn into_rc_model(&self) -> slint::ModelRc<slint::ModelRc<f32>> {
        let mut c: Vec<slint::ModelRc<f32>> = vec![];
        for i in &self.mask_points {
            c.push(std::rc::Rc::new(slint::VecModel::from(vec![i[0], i[1]])).into())
        };
        std::rc::Rc::new(slint::VecModel::from(c)).into()
    }

    pub fn generate_line_for_slint(&self) -> slint::ModelRc<slint::ModelRc<f32>> {
        let mut line: Vec<slint::ModelRc<f32>> = Vec::new();

        for i in 0..self.mask_points.len() - 1 {
            let curve = LVIElib::math::bezier_cubic_curve([
                self.mask_points[i], {
                    let k = self.bezier_control_points[i][0];
                    if k[0] == -1.0 {
                        let l = self.mask_points[i];
                        let dx = self.mask_points[i+1][0] - l[0];
                        let dy = self.mask_points[i+1][1] - l[1];
                        [l[0] + (dx / 3.0), l[1] + (dy / 3.0)]
                    } else {
                        k
                    }
                }, 
                {
                    let k = self.bezier_control_points[i][1];
                    if k[0] == -1.0 {
                        let l = self.mask_points[i];
                        let dx = self.mask_points[i+1][0] - l[0];
                        let dy = self.mask_points[i+1][1] - l[1];
                        [l[0] + (2.0*dx / 3.0), l[1] + (2.0*dy / 3.0)]
                    } else {
                        k
                    }
                }, 
                self.mask_points[i+1]
            ], Some(500));
            for k in curve {
                line.push(std::rc::Rc::new(slint::VecModel::from(vec![k[0], k[1]])).into())
            }
        }

        if self.closed {
            let curve = LVIElib::math::bezier_cubic_curve([
                *self.mask_points.last().unwrap(), {
                    let k = self.bezier_control_points.last().unwrap()[0];
                    if k[0] == -1.0 {
                        let l = self.mask_points.last().unwrap();
                        let dx = self.mask_points[0][0] - l[0];
                        let dy = self.mask_points[0][1] - l[1];
                        [l[0] + (dx / 3.0), l[1] + (dy / 3.0)]
                    } else {
                        k
                    }
                }, 
                {
                    let k = self.bezier_control_points.last().unwrap()[1];
                    if k[0] == -1.0 {
                        let l = self.mask_points.last().unwrap();
                        let dx = self.mask_points[0][0] - l[0];
                        let dy = self.mask_points[0][1] - l[1];
                        [l[0] + (2.0*dx / 3.0), l[1] + (2.0*dy / 3.0)]
                    } else {
                        k
                    }
                }, self.mask_points[0]
            ], Some(500));
            for k in curve {
                line.push(std::rc::Rc::new(slint::VecModel::from(vec![k[0], k[1]])).into())
            }
        }

        std::rc::Rc::new(slint::VecModel::from(line)).into()
    }

    pub fn generate_control_point_connection_lines_for_slint(&self) -> slint::ModelRc<slint::ModelRc<f32>> {
        let mut line: Vec<slint::ModelRc<f32>> = Vec::new();

        let p = self.get_control_points();

        for i in 0..self.mask_points.len() - 1 {
            line.append(&mut generate_linespace(
                self.mask_points[i][0], self.mask_points[i][1], 
                p[i][0][0], p[i][0][1], 100
                )
            );

            line.append(&mut generate_linespace( 
                p[i][1][0], p[i][1][1], 
                self.mask_points[i + 1][0], self.mask_points[i + 1][1], 100
                )
            );
        }

        if self.closed {
            line.append(&mut generate_linespace(
                self.mask_points.last().unwrap()[0], self.mask_points.last().unwrap()[1], 
                p.last().unwrap()[0][0], p.last().unwrap()[0][1], 100
                )
            );
            line.append(&mut generate_linespace(
                p.last().unwrap()[1][0], p.last().unwrap()[1][1], 
                self.mask_points[0][0], self.mask_points[0][1], 100
                )
            );
        }

        slint::ModelRc::new(slint::VecModel::from(line))
    }

    pub fn generate_line(&self, width: f32, height: f32) -> Vec<[f32; 2]> {
        let mut line: Vec<[f32; 2]> = Vec::new();

        let mp: Vec<[f32; 2]> = (&self.mask_points).into_iter().map(|x| {
            if *x != [-1.0, -1.0] {
                [x[0]*width / 100.0, (100.0 - x[1])*height / 100.0]
            } else {
                *x
            }
        }).collect();

        let bcp: Vec<[[f32; 2]; 2]> = (&self.bezier_control_points).into_iter().map(|k| {
            let m: Vec<[f32;2]> = k.into_iter().map(|x|  {
                if *x != [-1.0, -1.0] {
                    [x[0]*width / 100.0, (100.0 - x[1])*height / 100.0]
                } else {
                    *x
                }
            }).collect();
            [m[0], m[1]]
        }).collect();

        for i in 0..mp.len() - 1 {
            let curve = LVIElib::math::bezier_cubic_curve([
                mp[i], {
                    let k = bcp[i][0];
                    if k[0] == -1.0 {
                        let l = mp[i];
                        let dx = mp[i+1][0] - l[0];
                        let dy = mp[i+1][1] - l[1];
                        [l[0] + (dx / 3.0), l[1] + (dy / 3.0)]
                    } else {
                        k
                    }
                }, 
                {
                    let k = bcp[i][1];
                    if k[0] == -1.0 {
                        let l = mp[i];
                        let dx = mp[i+1][0] - l[0];
                        let dy = mp[i+1][1] - l[1];
                        [l[0] + (2.0*dx / 3.0), l[1] + (2.0*dy / 3.0)]
                    } else {
                        k
                    }
                }, 
                mp[i+1]
            ], None);
            for k in curve {
                line.push(k);
            }
        }

        if self.closed {
            let curve = LVIElib::math::bezier_cubic_curve([
                *mp.last().unwrap(), {
                    let k = bcp.last().unwrap()[0];
                    if k[0] == -1.0 {
                        let l = mp.last().unwrap();
                        let dx = mp[0][0] - l[0];
                        let dy = mp[0][1] - l[1];
                        [l[0] + (dx / 3.0), l[1] + (dy / 3.0)]
                    } else {
                        k
                    }
                }, 
                {
                    let k = bcp.last().unwrap()[1];
                    if k[0] == -1.0 {
                        let l = mp.last().unwrap();
                        let dx = mp[0][0] - l[0];
                        let dy = mp[0][1] - l[1];
                        [l[0] + (2.0*dx / 3.0), l[1] + (2.0*dy / 3.0)]
                    } else {
                        k
                    }
                }, mp[0]
            ], None);
            for k in curve {
                line.push(k);
            }
        }

        line
    }

    fn generate_track<P>(&self, width: u32, height: u32) -> Result<image::ImageBuffer<P, Vec<P::Subpixel>>, MaskError> 
    where 
        P: image::Pixel + std::fmt::Debug + LVIElib::traits::ToHsl + 'static,
        P::Subpixel: image::Primitive + std::fmt::Debug + Send + Sync + Default + LVIElib::traits::Scale
    {
        if !self.closed { 
            Err(MaskError::MaskNotClosed)
        } else {
            let mut points = vec![P::Subpixel::default(); (width*height) as usize];

            for k in self.generate_line(width as f32, height as f32) {
                let x = k[0].round() as isize;
                let y = k[1].round() as isize;

                for i in -1..=1 {
                    for j in -1..=1 {
                        points[width as usize *(y + j) as usize + (x + i) as usize] = 255u8.scale::<P::Subpixel>();
                    }
                }
            }

            let out = image::ImageBuffer::from_vec(width, height, {
                let mut out: Vec<P::Subpixel> = Vec::new();

                for k in &points {
                    let mut channels = vec![*k; P::CHANNEL_COUNT as usize];
                    out.append(&mut channels);
                }

                out
            }).unwrap();

            image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_vec(width, height, {
                let mut out: Vec<u8> = Vec::new();

                for k in &points {
                    let channels = [k.scale(); 3];
                    out.push(channels[0]);
                    out.push(channels[1]);
                    out.push(channels[2]);
                }

                out
            }).unwrap().save("track.png").expect("Failed to save");

            Ok(out)
        }
    }

    pub fn apply_to_image<P>(&self, image: &image::ImageBuffer<P, Vec<P::Subpixel>>) -> Result<image::ImageBuffer<P, Vec<P::Subpixel>>, MaskError>
    where 
        P: image::Pixel + std::fmt::Debug + LVIElib::traits::ToHsl + 'static + Send + Sync,
        P::Subpixel: image::Primitive + std::fmt::Debug + Send + Sync + Default + LVIElib::traits::Scale + Send + Sync
    {
        let track: image::ImageBuffer<P, Vec<P::Subpixel>> = self.generate_track(image.width(), image.height())?;

        let out = boundary_fill(
            &track, 
            None, None,
            image,
            P::from_slice(&vec![255u8.scale(); P::CHANNEL_COUNT as usize]),
            true
        );

        Ok(out)
    }

    pub fn get_points(&self) -> &Vec<[f32;2]> {
        &self.mask_points
    }

    pub fn get_control_points(&self) -> Vec<[[f32;2]; 2]> {
        let mut out = Vec::new();
        for (i, k) in (&self.bezier_control_points).into_iter().enumerate() {
            let mut step = [[0f32; 2]; 2];
            for (j, z) in k.into_iter().enumerate() {
                if *z == [-1.0, -1.0] {
                    step[j] = {
                        let l = self.mask_points[i];
                        let dx = self.mask_points[(i+1) % self.mask_points.len()][0] - l[0];
                        let dy = self.mask_points[(i+1) % self.mask_points.len()][1] - l[1];
                        [l[0] + ((1.0 + 1.0*j as f32)*dx / 3.0), l[1] + ((1.0 + 1.0*j as f32)*dy / 3.0)]
                    };
                } else {
                    step[j] = *z;
                }
            }
            out.push(step);
        }
        out
    }

    pub fn get_control_points_model_rc(&self) -> slint::ModelRc<slint::ModelRc<f32>> {

        let mut out = Vec::new();
        for (i, k) in (&self.bezier_control_points).into_iter().enumerate() {
            for (j, z) in k.into_iter().enumerate() {
                if *z == [-1.0, -1.0] {
                    out.push(slint::ModelRc::new(slint::VecModel::from({
                        let l = self.mask_points[i];
                        let dx = self.mask_points[(i+1) % self.mask_points.len()][0] - l[0];
                        let dy = self.mask_points[(i+1) % self.mask_points.len()][1] - l[1];
                        vec![l[0] + ((1.0 + 1.0*j as f32)*dx / 3.0), l[1] + ((1.0 + 1.0*j as f32)*dy / 3.0)]
                    })));
                } else {
                    out.push(slint::ModelRc::new(slint::VecModel::from(vec![z[0], z[1]])));
                }
            }
        }

        slint::ModelRc::new(slint::VecModel::from(out))
    }

    pub fn update_control_point(&mut self, index: [usize; 2], point: [f32; 2]) -> Result<(), MaskError> {
        let p = self.bezier_control_points.get_mut(index[0]);
        if p.is_none() {
            return Err(MaskError::PointNotFound);
        }
        let k = p.unwrap().get_mut(index[1]);
        if k.is_none() {
            return Err(MaskError::PointNotFound);
        }
        *k.unwrap() = point;
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn update_points(&mut self, points: Vec<[f32; 2]>) {
        self.mask_points = points;
    }

    pub fn update_point(&mut self, index: usize, point: [f32; 2]) -> Result<(), MaskError> {
        if index >= self.mask_points.len() {
            Err(MaskError::PointNotFound)
        } else {
            let old = self.mask_points[index];
            let dxy = [point[0] - old[0], point[1]-old[1]];
            self.mask_points[index] = point;

            if index < self.bezier_control_points.len() && self.bezier_control_points[index][0] != [-1.0, -1.0] {
                self.bezier_control_points[index][0] = [
                    self.bezier_control_points[index][0][0] + dxy[0],
                    self.bezier_control_points[index][0][1] + dxy[1],
                ];
            }
            let ni = if index != 0 { index - 1 } else { self.mask_points.len() - 1 };
            if self.bezier_control_points[ni][1] != [-1.0, -1.0] {
                self.bezier_control_points[ni][1] = [
                    self.bezier_control_points[ni][1][0] + dxy[0],
                    self.bezier_control_points[ni][1][1] + dxy[1],
                ];
            }
            Ok(())
        }
    }

    pub fn remove_point(&mut self, index: usize) -> Result<(), MaskError> {
        if index >= self.mask_points.len() {
            Err(MaskError::PointNotFound)
        } else {
            self.mask_points.remove(index);
            self.bezier_control_points.remove(index-1);
            Ok(())
        }
    }
}

fn generate_linespace(from_x: f32, from_y: f32, to_x: f32, to_y: f32, steps: usize) -> Vec<slint::ModelRc<f32>> {
    let mut out: Vec<slint::ModelRc<f32>> = Vec::new();

    let s = (
        (to_x - from_x) / steps as f32,
        (to_y - from_y) / steps as f32
    );

    for k in 0..steps {
        out.push(slint::ModelRc::new(slint::VecModel::from(vec![from_x + s.0*k as f32, from_y + s.1*k as f32])));
    }

    out
}