use std::default::Default;
use std::path::Path;

use crate::practice_utils::image_parsers::ppm::parse_ppm;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Pixel {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Default)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub image_data: Vec<Pixel>,
    pub argb_data: Vec<u32>,
}

impl Pixel {
    pub fn from_argb(argb: u32) -> Self {
        Self {
            a: (argb >> 24) as u8,
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: ((argb) & 0xFF) as u8,
        }
    }
}

impl Image {
    pub fn from_filepath(image_filename: String) -> anyhow::Result<Self> {
        parse_ppm(Path::new(&image_filename))
    }

    pub fn new(width: u32, height: u32, fill: Pixel) -> Self {
        Image {
            width,
            height,
            image_data: vec![fill; (width * height) as usize],
            argb_data: vec![],
        }
    }

    pub fn to_argb(pixel: Pixel) -> u32 {
        (pixel.a as u32) << 24 | (pixel.r as u32) << 16 | (pixel.g as u32) << 8 | (pixel.b as u32)
    }
}
