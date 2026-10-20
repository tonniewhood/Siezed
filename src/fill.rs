use std::cmp::{max, min};
use std::collections::HashMap;
use std::num::{NonZeroU32, TryFromIntError};
use std::rc::Rc;

use softbuffer::{Context, SoftBufferError, Surface};

use winit::{
    event_loop::OwnedDisplayHandle,
    window::{Window, WindowId},
};

use crate::canvas::{BlendMode, Canvas, PixelBuffer};
use crate::color::blend_over;

#[derive(thiserror::Error, Debug)]
pub enum FillError {
    #[error(transparent)]
    TryFromInt(std::num::TryFromIntError),
    #[error(transparent)]
    SoftBufferError(softbuffer::SoftBufferError),
    #[error("Window ID {0:?} wasn't registered")]
    UnknownWindow(WindowId),
}

// Implement From for SoftBufferError
impl From<SoftBufferError> for FillError {
    fn from(err: SoftBufferError) -> Self {
        FillError::SoftBufferError(err)
    }
}

// Implement From for TryFromIntError
impl From<TryFromIntError> for FillError {
    fn from(err: TryFromIntError) -> Self {
        FillError::TryFromInt(err)
    }
}

pub struct FillContext {
    gc: Context<Rc<OwnedDisplayHandle>>,
    surfaces: HashMap<WindowId, Surface<Rc<OwnedDisplayHandle>, Rc<Window>>>,
}

impl FillContext {
    pub fn new(display_handle: Rc<OwnedDisplayHandle>) -> Result<Self, FillError> {
        let gc = Context::new(display_handle.clone())?;
        Ok(Self {
            gc: gc,
            surfaces: HashMap::new(),
        })
    }

    pub fn register_window(
        &mut self,
        window_id: WindowId,
        window: Rc<Window>,
    ) -> Result<(), FillError> {
        let win_surf = Surface::new(&self.gc, window)?;
        self.surfaces.insert(window_id, win_surf);
        Ok(())
    }

    fn blit_buffer(
        canvas_buff: &PixelBuffer,
        canvas_x: i64,
        canvas_y: i64,
        window_buff: &mut [u32],
        window_width: i64,
        window_height: i64,
    ) {
        let (canvas_width, canvas_height) = {
            let (width, height) = canvas_buff.get_size();
            (width as i64, height as i64)
        };
        let overhang_left = max(0, min(-canvas_x, canvas_width));
        let overhang_right = max(0, min(canvas_x + canvas_width - window_width, canvas_width));
        let overhang_top = max(0, min(-canvas_y, canvas_height));
        let overhang_bottom = max(
            0,
            min(canvas_y + canvas_height - window_height, canvas_height),
        );

        let valid_rows = canvas_height - overhang_top - overhang_bottom;
        let valid_cols = canvas_width - overhang_left - overhang_right;
        let window_x = max(canvas_x, 0);
        let window_y = max(canvas_y, 0);
        let canvas_x = -min(0, canvas_x);
        let canvas_y = -min(0, canvas_y);

        if valid_rows <= 0 || valid_cols <= 0 {
            return;
        }

        let (valid_rows, valid_cols) = (valid_rows as usize, valid_cols as usize);
        let (canvas_x, canvas_y) = (canvas_x as usize, canvas_y as usize);
        let (window_x, window_y) = (window_x as usize, window_y as usize);
        let (canvas_width, window_width) = (canvas_width as usize, window_width as usize);

        for n in 0..valid_rows {
            let canvas_start_idx = canvas_x + canvas_width * (canvas_y + n);
            let window_start_idx = window_x + window_width * (window_y + n);
            let window_slice = &mut window_buff[window_start_idx..(window_start_idx + valid_cols)];
            let canvas_slice =
                &canvas_buff.buff()[canvas_start_idx..(canvas_start_idx + valid_cols)];

            if canvas_buff.blend_mode() == BlendMode::Opaque {
                window_slice.copy_from_slice(canvas_slice);
            } else {
                for col_idx in 0..valid_cols {
                    window_slice[col_idx] =
                        blend_over(canvas_slice[col_idx], window_slice[col_idx]);
                }
            }
        }
    }

    pub fn present(
        &mut self,
        window_id: WindowId,
        bg_color: u32,
        canvases: &[&Canvas],
    ) -> Result<(), FillError> {
        let surf = self
            .surfaces
            .get_mut(&window_id)
            .ok_or(FillError::UnknownWindow(window_id))?;

        let size = surf.window().inner_size();
        let (w, h) = (
            NonZeroU32::try_from(size.width)?,
            NonZeroU32::try_from(size.height)?,
        );

        surf.resize(w, h)?;
        let mut buf = surf.buffer_mut()?;
        buf.fill(bg_color);

        if !canvases.iter().any(|c| c.is_dirty()) {
            return Ok(());
        }

        for canvas in canvases {
            let (canvas_x, canvas_y) = {
                let (x, y) = canvas.get_position();
                (x as i64, y as i64)
            };
            Self::blit_buffer(
                canvas.buffer(),
                canvas_x,
                canvas_y,
                buf.as_mut(),
                i64::from(w.get()),
                i64::from(h.get()),
            );
            canvas.set_clean();
        }

        buf.present()?;
        Ok(())
    }
}
