use rusttype::Font;

const TOOLBAR_HEIGHT: usize = 40;

#[derive(Default)]
pub struct Toolbar {
    pub width: usize,
    pub buffer: Vec<u32>,
    pub button_pressed: bool,
}

impl Toolbar {
    pub fn new(width: usize) -> Self {
        let mut buf = Vec::new();
        buf.resize(TOOLBAR_HEIGHT * width, 0xFFFFFFFF);
        Self {
            width: width,
            buffer: buf,
            button_pressed: false,
        }
    }

    pub fn add_label(&mut self, text: &str, x_offset: usize, y_offset: usize, color: u32) {
        let font =
            Font::try_from_bytes(include_bytes!("../../assets/fonts/SUPERSIMF.TTF")).unwrap();
        let scale = rusttype::Scale::uniform(35.0); // Smaller font to fit in toolbar
        let v_metrics = font.v_metrics(scale);

        let glyphs: Vec<_> = font
            .layout(
                text,
                scale,
                rusttype::point(x_offset as f32, v_metrics.ascent + y_offset as f32),
            )
            .collect();

        // Rasterize each glyph and draw it to the toolbar buffer
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph if it has a bounding box
                glyph.draw(|x, y, v| {
                    let pixel_x = bounding_box.min.x as usize + x as usize;
                    let pixel_y = bounding_box.min.y as usize + y as usize;

                    // Check bounds
                    if pixel_x < self.width && pixel_y < TOOLBAR_HEIGHT {
                        let idx = pixel_y * self.width + pixel_x;
                        if idx < self.buffer.len() && v > 0.0 {
                            // Alpha blend the text color with the background
                            let alpha = (v * 255.0) as u32;
                            let bg_color = self.buffer[idx];

                            // Extract RGB components
                            let bg_r = (bg_color >> 16) & 0xFF;
                            let bg_g = (bg_color >> 8) & 0xFF;
                            let bg_b = bg_color & 0xFF;

                            let text_r = (color >> 16) & 0xFF;
                            let text_g = (color >> 8) & 0xFF;
                            let text_b = color & 0xFF;

                            // Alpha blend
                            let r = (text_r * alpha + bg_r * (255 - alpha)) / 255;
                            let g = (text_g * alpha + bg_g * (255 - alpha)) / 255;
                            let b = (text_b * alpha + bg_b * (255 - alpha)) / 255;

                            self.buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                        }
                    }
                });
            }
        }
    }

    /// Add a label with predefined styling for button states
    pub fn add_button_label(&mut self, text: &str) {
        self.add_label(text, 50, 5, 0xFF000000);
    }

    pub fn update(&mut self, width: usize) {
        self.width = width;
        self.buffer.resize(TOOLBAR_HEIGHT * width, 0xFFFFFFFF);
    }

    pub fn reset(&mut self) {
        // Reset the entire toolbar to white
        self.buffer.fill(0xFFFFFFFF);
        if self.button_pressed {
            self.add_button_label("Inverted");
        } else {
            self.add_button_label("Normal");
        }
    }

    pub fn on_hover(&mut self, x_pos: usize, y_pos: usize, window_height: usize) -> bool {
        if self.button_pressed {
            return false;
        }
        // Check if mouse is in the toolbar area (bottom TOOLBAR_HEIGHT pixels)
        if y_pos < window_height - TOOLBAR_HEIGHT {
            return false;
        }

        // Check if mouse is in the left 40x40 button area
        let relative_y = y_pos - (window_height - TOOLBAR_HEIGHT);
        if x_pos > 40 || relative_y >= TOOLBAR_HEIGHT {
            return false;
        }

        // First reset the toolbar to white
        self.reset();

        // Draw gray hover effect in a 30x30 area with 5px margin from edges
        for row_idx in 5..35 {
            for col_idx in 5..35 {
                let idx = row_idx * self.width + col_idx;
                if idx < self.buffer.len() {
                    self.buffer[idx] = 0xFF808080; // Gray color
                }
            }
        }

        true
    }

    pub fn on_click(&mut self, x_pos: usize, y_pos: usize, window_height: usize) -> bool {
        // Check if mouse is in the toolbar area (bottom TOOLBAR_HEIGHT pixels)
        if y_pos < window_height - TOOLBAR_HEIGHT {
            return false;
        }

        // Check if mouse is in the left 40x40 button area
        let relative_y = y_pos - (window_height - TOOLBAR_HEIGHT);
        if x_pos > 40 || relative_y >= TOOLBAR_HEIGHT {
            return false;
        }

        // Toggle button state
        self.button_pressed = !self.button_pressed;

        // First reset the toolbar to white
        self.reset();

        // Draw button in a 30x30 area with 5px margin from edges
        for row_idx in 5..35 {
            for col_idx in 5..35 {
                let idx = row_idx * self.width + col_idx;
                if idx < self.buffer.len() {
                    self.buffer[idx] = if self.button_pressed {
                        0xFF000000 // Black when pressed
                    } else {
                        0xFF808080 // Gray when not pressed
                    };
                }
            }
        }

        true
    }
}
