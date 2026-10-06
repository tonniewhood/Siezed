use std::{collections::HashMap, num::NonZeroU32, num::TryFromIntError, rc::Rc};

use softbuffer::{Context, SoftBufferError, Surface};

use winit::{
    event_loop::OwnedDisplayHandle,
    window::{Window, WindowId},
};

use super::frame::Frame;

#[derive(thiserror::Error, Debug)]
pub enum FillError {
    #[error(transparent)]
    TryFromInt(std::num::TryFromIntError),
    #[error(transparent)]
    SoftBufferError(softbuffer::SoftBufferError),
    #[error("Mismatched sizes: {0}")]
    InvalidDimensionError(String),
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

    pub fn prepare(_window_id: WindowId) -> Result<(), ()> {
        Ok(())
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

    pub fn draw(&mut self, window_id: WindowId, frame: &Frame) -> Result<(), FillError> {
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
        buf.fill(frame.bg_color);

        // Center the frame in the window
        let vertical_slack = if size.height as usize > frame.canvas_height {
            (size.height as usize - frame.canvas_height) / 2
        } else {
            0
        };
        let horizontal_slack = if size.width as usize > frame.canvas_width {
            (size.width as usize - frame.canvas_width) / 2
        } else {
            0
        };

        // Only draw the frame if it fits within the window
        let drawable_height = frame.canvas_height.min(size.height as usize);
        let drawable_width = frame.canvas_width.min(size.width as usize);

        for row in 0..drawable_height {
            let src_start = row * frame.canvas_width;
            let src_end = src_start + drawable_width;
            let dest_start = (row + vertical_slack) * w.get() as usize + horizontal_slack;
            let dest_end = dest_start + drawable_width;

            if dest_end <= buf.len() && src_end <= frame.canvas_buffer.len() {
                buf[dest_start..dest_end].copy_from_slice(&frame.canvas_buffer[src_start..src_end]);
            }
        }

        // Draw the toolbar at the bottom of the window
        if !frame.toolbar.buffer.is_empty() {
            let toolbar_height = frame.toolbar.buffer.len() / size.width as usize;
            let toolbar_start = (size.height as usize - toolbar_height) * size.width as usize;
            let toolbar_end = toolbar_start + frame.toolbar.buffer.len();

            if toolbar_end <= buf.len() {
                buf[toolbar_start..toolbar_end].copy_from_slice(&frame.toolbar.buffer);
            }
        }

        buf.present()?;
        Ok(())
    }
}
