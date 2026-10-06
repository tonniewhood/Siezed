use super::widgets;

#[derive(Default)]
pub struct Frame {
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub canvas_buffer: Vec<u32>,
    pub toolbar: widgets::Toolbar,
    pub bg_color: u32,
}

impl Frame {
    pub fn new(bg_color: u32) -> Self {
        Self {
            bg_color,
            ..Default::default()
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) -> bool {
        if new_width == self.canvas_width && new_height == self.canvas_height {
            return false; // No change needed
        }

        self.canvas_width = new_width;
        self.canvas_height = new_height;
        self.canvas_buffer
            .resize(new_width * new_height, self.bg_color);

        true // Resize occurred
    }
}
