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
    pub is_grayscale: bool,
    pub inverted: bool,
    pub rotation: u16,
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

    pub fn to_argb(&self) -> u32 {
        (self.a as u32) << 24 | (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }

    pub fn to_grayscale(&self) -> Pixel {
        // Use luminance formula: 0.299*R + 0.587*G + 0.114*B
        let gray = (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) as u8;
        Pixel {
            a: self.a,
            r: gray,
            g: gray,
            b: gray,
            argb: (self.a as u32) << 24 | (gray as u32) << 16 | (gray as u32) << 8 | (gray as u32),
        }
    }

    pub fn to_display_color(&self, is_grayscale: bool, is_inverted: bool) -> u32 {
        let mut pixel = *self;

        // Apply grayscale transformation first
        if is_grayscale {
            pixel = pixel.to_grayscale();
        }

        // Apply inversion transformation
        if is_inverted {
            pixel = pixel.to_inverted();
        }

        pixel.argb
    }

    pub fn to_inverted(&self) -> Pixel {
        Pixel {
            a: self.a,
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
            argb: (self.a as u32) << 24
                | ((255 - self.r) as u32) << 16
                | ((255 - self.g) as u32) << 8
                | ((255 - self.b) as u32),
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
            is_grayscale: false,
            inverted: false,
            rotation: 0,
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.image_data[y * self.width as usize + x]
    }

    /// Get the effective width and height after rotation
    pub fn get_effective_dimensions(&self) -> (u32, u32) {
        match self.rotation % 180 {
            0 => (self.width, self.height), // 0° or 180° - no dimension swap
            _ => (self.height, self.width), // 90° or 270° - swap dimensions
        }
    }

    /// Get pixel with rotation applied
    pub fn get_rotated_pixel(&self, x: usize, y: usize) -> Pixel {
        let (rotated_x, rotated_y) = self.apply_rotation_to_coords(x, y);
        self.image_data[rotated_y * self.width as usize + rotated_x]
    }

    /// Apply rotation transformation to coordinates
    fn apply_rotation_to_coords(&self, x: usize, y: usize) -> (usize, usize) {
        let (eff_width, eff_height) = self.get_effective_dimensions();

        match self.rotation % 360 {
            0 => (x, y),
            90 => (y, (eff_height - 1 - x as u32) as usize),
            180 => (
                (eff_width - 1 - x as u32) as usize,
                (eff_height - 1 - y as u32) as usize,
            ),
            270 => ((eff_width - 1 - y as u32) as usize, x),
            _ => (x, y), // Fallback for non-90 degree rotations
        }
    }
}
