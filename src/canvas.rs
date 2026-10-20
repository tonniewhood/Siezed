use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Opaque,
    Alpha,
}

pub struct PixelBuffer {
    height: usize,
    width: usize,
    buffer: Vec<u32>,
    blend: BlendMode,
}

pub struct Canvas {
    pos_x: i32,
    pos_y: i32,
    buffer: PixelBuffer,
    dirty: Cell<bool>,
}

impl PixelBuffer {
    fn new(width: usize, height: usize, blend: BlendMode) -> Self {
        Self {
            height,
            width,
            buffer: vec![0; height * width],
            blend,
        }
    }

    pub fn fill(&mut self, value: u32) {
        self.buffer.fill(value);
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn buff(&self) -> &[u32] {
        &self.buffer
    }

    pub fn buff_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend
    }
}

impl Canvas {
    pub fn new(pos_x: i32, pos_y: i32, width: usize, height: usize, blend: BlendMode) -> Self {
        Self {
            pos_x,
            pos_y,
            buffer: PixelBuffer::new(width, height, blend),
            dirty: Cell::new(true),
        }
    }

    pub fn buffer(&self) -> &PixelBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut PixelBuffer {
        self.dirty.set(true);
        &mut self.buffer
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) -> bool {
        if new_width == self.buffer.width && new_height == self.buffer.height {
            return false; // No change needed
        }

        self.buffer.width = new_width;
        self.buffer.height = new_height;
        self.buffer.buffer.resize(new_width * new_height, 0x0);
        self.dirty.set(true);

        true // Resize occurred
    }

    pub fn get_position(&self) -> (i32, i32) {
        (self.pos_x, self.pos_y)
    }

    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.buffer.width, self.buffer.height)
    }

    pub fn get_pixel_value(&self, x: usize, y: usize) -> u32 {
        let (width, _) = self.get_dimensions();
        self.buffer.buff()[x + y * width]
    }

    pub fn set_pixels(&mut self, pixels: Vec<u32>) {
        if pixels.len() != self.buffer.height * self.buffer.width {
            panic!("Pixel data does not match canvas dimensions");
        }

        self.buffer.buffer = pixels;
        self.dirty.set(true);
    }

    pub fn set_position(&mut self, pos_x: i32, pos_y: i32) {
        self.pos_x = pos_x;
        self.pos_y = pos_y;
        self.dirty.set(true);
    }

    pub fn set_x(&mut self, x: i32) {
        self.pos_x = x;
        self.dirty.set(true);
    }

    pub fn set_y(&mut self, y: i32) {
        self.pos_y = y;
        self.dirty.set(true);
    }

    pub fn set_pixel_value(&mut self, x: usize, y: usize, value: u32) -> bool {
        let (width, _) = self.get_dimensions();

        let buff_mut = self.buffer.buff_mut();
        if x + y * width < buff_mut.len() {
            buff_mut[x + y * width] = value;
            self.dirty.set(true);
            return true;
        }

        false
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    pub(crate) fn set_clean(&self) {
        self.dirty.set(false);
    }
}
