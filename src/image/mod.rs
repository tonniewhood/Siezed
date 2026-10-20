use std::default::Default;
use std::path::Path;

use anyhow;

use crate::parsers::{bmp, ppm};

pub mod interpolate;

#[derive(Default)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub image_data: Vec<u32>,
}

impl Image {
    pub fn new(width: u32, height: u32, fill: u32) -> Self {
        Image {
            width,
            height,
            image_data: vec![fill; (width * height) as usize],
        }
    }

    pub fn load_from_path<P: AsRef<Path>>(filepath: P) -> anyhow::Result<Image> {
        let path = filepath.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "ppm" => ppm::parse_ppm(path),
            "bmp" => bmp::parse_bmp(path),
            other => anyhow::bail!("Unsupported extension: '{}'", other),
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        self.image_data[y * self.width as usize + x]
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
