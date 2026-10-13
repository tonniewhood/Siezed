use std::default::Default;

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

    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        self.image_data[y * self.width as usize + x]
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
