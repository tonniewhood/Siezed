use rusttype::Font;

use crate::{
    canvas::{BlendMode, Canvas},
    color::blend_over,
    image::{
        Image,
        interpolate::{InterpolationType, ViewOptions, interpolate, rotated_size},
    },
};

pub const TOOLBAR_HEIGHT: usize = 40;

pub struct Toolbar {
    width: usize,
    canvas: Canvas,
    button_pressed: bool,
}

pub struct ImageView {
    image: Option<Image>,
    canvas: Canvas,
    options: ViewOptions,
    locked_aspect: bool,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            width: 0,
            canvas: Canvas::new(0, 0, 0, TOOLBAR_HEIGHT, BlendMode::Opaque),
            button_pressed: false,
        }
    }

    pub fn at(mut self, x: i32, y: i32) -> Toolbar {
        self.canvas.set_position(x, y);
        self
    }

    pub fn with_size(mut self, width: usize) -> Toolbar {
        self.width = width;
        self.canvas.resize(width, TOOLBAR_HEIGHT);
        self
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn btn_pressed(&self) -> bool {
        self.button_pressed
    }

    pub fn add_label(&mut self, text: &str, x_offset: usize, y_offset: usize, color: u32) {
        let font = Font::try_from_bytes(include_bytes!("../assets/fonts/SUPERSIMF.TTF")).unwrap();
        let scale = rusttype::Scale::uniform(35.0); // Smaller font to fit in toolbar
        let v_metrics = font.v_metrics(scale);

        let glyphs: Vec<_> = font
            .layout(
                text,
                scale,
                rusttype::point(x_offset as f32, v_metrics.ascent + y_offset as f32),
            )
            .collect();

        let buffer = self.canvas.buffer_mut().buff_mut();

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
                        if idx < buffer.len() && v > 0.0 {
                            // Glyph coverage is the text color's alpha
                            let coverage = (v * 255.0) as u32;
                            let text_color = (coverage << 24) | (color & 0x00FFFFFF);
                            buffer[idx] = blend_over(text_color, buffer[idx]);
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

    pub fn update(&mut self, win_width: usize, win_height: usize) {
        self.width = win_width;
        self.canvas.resize(win_width, TOOLBAR_HEIGHT);
        self.canvas.set_y(win_height as i32 - TOOLBAR_HEIGHT as i32);
        self.reset();
    }

    pub fn reset(&mut self) {
        // Reset the entire toolbar to white
        self.canvas.buffer_mut().buff_mut().fill(0xFFFFFFFF);
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
        if y_pos < window_height.saturating_sub(TOOLBAR_HEIGHT) {
            return false;
        }

        // Check if mouse is in the left 40x40 button area
        let relative_y = y_pos - (window_height.saturating_sub(TOOLBAR_HEIGHT));
        if x_pos > 40 || relative_y >= TOOLBAR_HEIGHT {
            return false;
        }

        // First reset the toolbar to white
        self.reset();

        let buffer = self.canvas.buffer_mut().buff_mut();
        // Draw gray hover effect in a 30x30 area with 5px margin from edges
        for row_idx in 5..35 {
            for col_idx in 5..35 {
                let idx = row_idx * self.width + col_idx;
                if idx < buffer.len() {
                    buffer[idx] = 0xFF808080; // Gray color
                }
            }
        }

        true
    }

    pub fn on_click(&mut self, x_pos: usize, y_pos: usize, window_height: usize) -> bool {
        // Check if mouse is in the toolbar area (bottom TOOLBAR_HEIGHT pixels)
        if y_pos < window_height.saturating_sub(TOOLBAR_HEIGHT) {
            return false;
        }

        // Check if mouse is in the left 40x40 button area
        let relative_y = y_pos - (window_height.saturating_sub(TOOLBAR_HEIGHT));
        if x_pos > 40 || relative_y >= TOOLBAR_HEIGHT {
            return false;
        }

        // Toggle button state
        self.button_pressed = !self.button_pressed;

        // First reset the toolbar to white
        self.reset();

        let buffer = self.canvas.buffer_mut().buff_mut();

        // Draw button in a 30x30 area with 5px margin from edges
        for row_idx in 5..35 {
            for col_idx in 5..35 {
                let idx = row_idx * self.width + col_idx;
                if idx < buffer.len() {
                    buffer[idx] = if self.button_pressed {
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

impl ImageView {
    pub fn new(
        pos_x: i32,
        pos_y: i32,
        locked_aspect: Option<bool>,
        blend: Option<BlendMode>,
    ) -> ImageView {
        ImageView {
            image: None,
            canvas: Canvas::new(pos_x, pos_y, 0, 0, blend.unwrap_or(BlendMode::Opaque)),
            options: ViewOptions::default(),
            locked_aspect: locked_aspect.unwrap_or(true),
        }
    }

    pub fn with_img(mut self, img: Image) -> ImageView {
        self.image = Some(img);
        self
    }

    pub fn set_img(&mut self, img: Image) {
        self.image = Some(img);
    }

    pub fn img_ref(&self) -> &Option<Image> {
        &self.image
    }

    pub fn img_mut(&mut self) -> &mut Option<Image> {
        &mut self.image
    }

    pub fn img_size(&self) -> anyhow::Result<(u32, u32)> {
        let Some(image) = self.image.as_ref() else {
            anyhow::bail!("No image to check size of")
        };

        Ok(image.get_size())
    }

    /// Resizes the canvas to fit within `max_width` x `max_height` (keeping the
    /// image's aspect ratio if locked) and resamples the image into it
    pub fn rebuild(
        &mut self,
        max_width: usize,
        max_height: usize,
        interpolation_type: InterpolationType,
    ) -> anyhow::Result<()> {
        let Some(image) = self.image.as_ref() else {
            anyhow::bail!("No image to rebuild with")
        };

        let (effective_width, effective_height) = rotated_size(image, self.options.rotation);
        if effective_width == 0 || effective_height == 0 {
            return Ok(());
        }

        let (dst_w, dst_h) = if self.locked_aspect {
            let scale = (max_width as f32 / effective_width as f32)
                .min(max_height as f32 / effective_height as f32);
            (
                (effective_width as f32 * scale).round() as usize,
                (effective_height as f32 * scale).round() as usize,
            )
        } else {
            (max_width, max_height)
        };

        self.canvas.resize(dst_w, dst_h);
        // Center the image in the available area
        self.canvas.set_position(
            (max_width.saturating_sub(dst_w) / 2) as i32,
            (max_height.saturating_sub(dst_h) / 2) as i32,
        );

        // `image` and `canvas` are separate fields, so borrowing one immutably
        // and the other mutably at the same time is allowed
        interpolate(
            image,
            self.canvas.buffer_mut(),
            &self.options,
            interpolation_type,
        )
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    pub fn rotate(&mut self, degrees: i16) {
        self.options.rotation = (self.options.rotation + degrees).rem_euclid(360);
    }

    pub fn toggle_grayscale(&mut self) {
        self.options.grayscale = !self.options.grayscale;
    }

    pub fn toggle_inverted(&mut self) {
        self.options.inverted = !self.options.inverted;
    }

    pub fn toggle_locked_aspect(&mut self) {
        self.locked_aspect = !self.locked_aspect;
    }

    pub fn rotation(&self) -> i16 {
        self.options.rotation
    }

    pub fn grayscale(&self) -> bool {
        self.options.grayscale
    }

    pub fn inverted(&self) -> bool {
        self.options.inverted
    }

    pub fn locked_aspect(&self) -> bool {
        self.locked_aspect
    }

    pub fn view_options(&self) -> &ViewOptions {
        &self.options
    }

    pub fn view_options_mut(&mut self) -> &mut ViewOptions {
        &mut self.options
    }
}
