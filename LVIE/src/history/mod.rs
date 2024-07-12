#![allow(dead_code)]
use std::{
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    vec::Vec,
};

mod operations;
#[macro_use]
mod builder;

pub use operations::*;

use crate::core::FilterArray;

use image::Primitive;
use num_traits::NumCast;

use downcast_rs::DowncastSync;

build_operation!(
    (Filter, FilterArray),
    (Logic, LogicOperationType),
    (Geometric, GeometricOperationType),
    (Mask, (usize, MaskOperationType)),
    (Curve, CurveOperationType),
);

pub trait Operation: DowncastSync {
    fn get_type(&self) -> &OperationType;
}
downcast_rs::impl_downcast!(sync Operation);

impl Debug for dyn Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation: {:?}", self)
    }
}

#[derive(Debug)]
pub struct History {
    history: Vec<(bool, Box<dyn Operation>)>,
    use_temp_files: bool,
    file_handler: FileHandler,
    current_index: usize,
}

impl History {
    pub fn init(
        temp_file_directory: Option<PathBuf>,
        use_temp_files: bool,
        max_mem_size: Option<impl FileSize>,
    ) -> History {
        let fh = FileHandler::new(
            temp_file_directory.unwrap_or_else(|| {
                let d = std::env::current_dir().unwrap().join(".LVIE").join("temp");
                if d.is_dir() {
                    std::fs::create_dir(&d).expect("Failed to create directory");
                }
                d
            }),
            max_mem_size,
        );

        History {
            history: Vec::new(),
            use_temp_files,
            file_handler: fh,
            current_index: 0,
        }
    }

    pub fn next_undo_type(&self) -> Option<&OperationType> {
        if self.can_undo() {
            Some(
                self.history[self.history.len() - self.current_index - 1]
                    .1
                    .get_type(),
            )
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current_index < self.history.len()
    }

    pub fn undo(&mut self) -> Option<&Box<dyn Operation>> {
        if self.current_index < self.history.len() {
            self.current_index += 1;
            Some(&self.history[self.history.len() - self.current_index].1)
        } else {
            None
        }
    }

    pub fn can_redo(&self) -> bool {
        self.current_index > 0
    }

    pub fn redo(&mut self) -> Option<&Box<dyn Operation>> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.history[self.history.len() - 1 - self.current_index].1)
        } else {
            None
        }
    }

    pub fn preview_aviable(&self) -> bool {
        self.use_temp_files && self.history[self.history.len() - self.current_index].0
    }

    pub fn get_precomputed_preview<P>(
        &mut self,
    ) -> Option<std::io::Result<image::ImageBuffer<P, Vec<P::Subpixel>>>>
    where
        P: image::Pixel + std::fmt::Debug,
        P::Subpixel: image::Primitive + std::fmt::Debug + num_traits::ToBytes + bytemuck::Pod,
    {
        if self.history[self.history.len() - self.current_index].0 && self.use_temp_files {
            Some(self.file_handler.read())
        } else {
            None
        }
    }
}

pub trait FileSize {
    fn size_as_bytes(&self) -> usize;
}

pub enum FileSizes {
    MB(usize),
    GB(usize),
}

impl FileSize for FileSizes {
    fn size_as_bytes(&self) -> usize {
        const MB: usize = 1024 * 1024;
        match self {
            FileSizes::MB(x) => x * MB,
            FileSizes::GB(x) => x * 1024 * MB,
        }
    }
}

impl<T: Primitive> FileSize for T {
    fn size_as_bytes(&self) -> usize {
        NumCast::from(*self).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct FileHandler {
    root_path: PathBuf,
    max_mem_size: usize,
    current_mem_size: Vec<usize>,
    current_files: Vec<PathBuf>,
    current_index: usize,
}

impl FileHandler {
    pub fn new(root: PathBuf, max_mem_size: Option<impl FileSize>) -> FileHandler {
        let max_mem_size: usize = match max_mem_size {
            Some(max_mem_size) => {
                if max_mem_size.size_as_bytes() > FileSizes::GB(10).size_as_bytes() {
                    println!("required max memory size exceeds the limit of 10GB! Using 10GB as max memory size.");
                    FileSizes::GB(10).size_as_bytes()
                } else if max_mem_size.size_as_bytes() == 0 {
                    FileSizes::GB(3).size_as_bytes()
                } else {
                    max_mem_size.size_as_bytes()
                }
            }
            None => FileSizes::GB(3).size_as_bytes(),
        };

        FileHandler {
            root_path: root,
            max_mem_size,
            current_mem_size: Vec::new(),
            current_files: Vec::new(),
            current_index: 0,
        }
    }

    pub fn write<P>(
        &mut self,
        buffer: &image::ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> std::io::Result<()>
    where
        P: image::Pixel + std::fmt::Debug,
        P::Subpixel: image::Primitive + std::fmt::Debug + num_traits::ToBytes + bytemuck::Pod,
    {
        let t_size = std::mem::size_of::<P::Subpixel>() as u8;
        let ch_count = P::CHANNEL_COUNT;
        let dimensions = vec![buffer.width(), buffer.height()];

        let mut header = vec![t_size, ch_count];
        header.append(&mut bytemuck::cast_slice(&dimensions).to_vec());

        let mut content: Vec<u8> = bytemuck::cast_slice(&buffer.to_vec()).to_vec();
        header.append(&mut bytemuck::cast_slice(&[content.len()]).to_vec());
        header.append(&mut content);

        let out_path = self.root_path.join(uuid::Uuid::new_v4().to_string());

        if self.current_index != 0 {
            if self.current_index == self.current_files.len() {
                self.current_files.clear();
                self.current_mem_size.clear();
            } else {
                let (new, old) = self
                    .current_files
                    .split_at(self.current_files.len() - self.current_index);
                for f in old {
                    if f.is_file() {
                        std::fs::remove_file(f)?;
                    }
                }
                self.current_mem_size
                    .resize(self.current_files.len() - self.current_index, 0);
                self.current_files = new.to_vec();
            }
        }

        self.current_files.push(out_path.clone());
        self.current_mem_size.push(header.len());
        self.current_index = 0;

        if (&self.current_mem_size).into_iter().sum::<usize>() > self.max_mem_size {
            let f = self.current_files.pop().unwrap();
            if f.is_file() {
                std::fs::remove_file(f)?;
            }
            self.current_mem_size.pop();
        }

        let mut fs = File::create(out_path)?;
        fs.write(&header)?;

        Ok(())
    }

    pub fn read<P>(&mut self) -> std::io::Result<image::ImageBuffer<P, Vec<P::Subpixel>>>
    where
        P: image::Pixel + std::fmt::Debug,
        P::Subpixel: image::Primitive + std::fmt::Debug + num_traits::ToBytes + bytemuck::Pod,
    {
        if self.current_files.len() == 0 {
            return Ok(image::ImageBuffer::new(0, 0));
        }
        let mut fs = File::open(
            &self.current_files[self.current_files.len() - {
                if self.current_index == self.current_files.len() {
                    self.current_files.len()
                } else {
                    self.current_index + 1
                }
            }],
        )?;

        let mut header = vec![0u8; 18];
        fs.read_exact(&mut header)?;

        let _t_size = header.remove(0);
        let ch_count = header.remove(0);

        if ch_count == P::CHANNEL_COUNT {
            let width = u32::from_ne_bytes([header[0], header[1], header[2], header[3]]);
            for _ in 0..4 {
                header.remove(0);
            }
            let height = u32::from_ne_bytes([header[0], header[1], header[2], header[3]]);
            for _ in 0..4 {
                header.remove(0);
            }

            let buffer_size = usize::from_ne_bytes([
                header[0], header[1], header[2], header[3], header[4], header[5], header[6],
                header[7],
            ]);

            let mut buf = vec![0u8; buffer_size];
            fs.read(&mut buf)?;

            self.current_index =
                num_traits::clamp(self.current_index + 1, 0, self.current_files.len());

            Ok(
                image::ImageBuffer::from_vec(width, height, bytemuck::cast_slice(&buf).to_vec())
                    .unwrap(),
            )
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "cannot decode data",
            ))
        }
    }
}
