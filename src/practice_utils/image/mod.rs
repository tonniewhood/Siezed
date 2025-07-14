use std::default::Default;

use super::simple_app::Frame;

pub mod interpolate;
pub mod parsers;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Pixel {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub argb: u32,
}

#[derive(Default)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub image_data: Vec<Pixel>,
    pub locked_aspect_ratio: bool,
}

impl Pixel {
    pub fn from_argb(argb: u32) -> Self {
        Self {
            a: (argb >> 24) as u8,
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: ((argb) & 0xFF) as u8,
            argb: argb,
        }
    }

    pub fn lerp(&self, other: &Pixel, weight: f32) -> Pixel {
        fn blend(a: u8, b: u8, weight: f32) -> u8 {
            ((a as f32) * (1.0 - weight) + (b as f32) * weight)
                .round()
                .clamp(0.0, 255.0) as u8
        }

        let a = blend(self.a, other.a, weight);
        let r = blend(self.r, other.r, weight);
        let g = blend(self.g, other.g, weight);
        let b = blend(self.b, other.b, weight);
        Pixel {
            a,
            r,
            g,
            b,
            argb: (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32),
        }
    }
}

impl Image {
    pub fn new(width: u32, height: u32, fill: Pixel, no_aspect: bool) -> Self {
        Image {
            width,
            height,
            image_data: vec![fill; (width * height) as usize],
            locked_aspect_ratio: !no_aspect,
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.image_data[y * self.width as usize + x]
    }

    pub fn to_argb(pixel: Pixel) -> u32 {
        (pixel.a as u32) << 24 | (pixel.r as u32) << 16 | (pixel.g as u32) << 8 | (pixel.b as u32)
    }
}
