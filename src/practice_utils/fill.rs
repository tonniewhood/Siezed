use std::{collections::HashMap, num::NonZeroU32, num::TryFromIntError, rc::Rc};

use softbuffer::{Context, SoftBufferError, Surface};

use winit::{
    dpi::PhysicalSize,
    window::{Window, WindowId},
};

use crate::practice_utils::simple_app;

#[derive(thiserror::Error, Debug)]
pub enum FillError {
    #[error(transparent)]
    TryFromInt(std::num::TryFromIntError),
    #[error(transparent)]
    SoftBufferError(softbuffer::SoftBufferError),
    #[error("Mismatched sizes: {0}")]
    InvalidDimensionError(String),
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
    gc: Context<Rc<Window>>,
    surfaces: HashMap<WindowId, Surface<Rc<Window>, Rc<Window>>>,
}

impl FillContext {
    pub fn new(window: Rc<Window>) -> Result<Self, FillError> {
        let gc = Context::new(window.clone())?;
        Ok(Self {
            gc,
            surfaces: HashMap::new(),
        })
    }

    pub fn fill(
        &mut self,
        window: Rc<Window>,
        frame: &mut simple_app::Frame,
        bg_color: u32,
    ) -> Result<(), FillError> {
        let size = window.inner_size();

        let (w, h) = (
            NonZeroU32::try_from(u32::max(size.width, frame.width as u32)).unwrap(),
            NonZeroU32::try_from(u32::max(size.height, frame.height as u32)).unwrap(),
        );

        let surf = self.surfaces.entry(window.id()).or_insert_with(|| {
            Surface::new(&self.gc, window.clone())
                .expect("Error: softbuffer Surface creation failed")
        });

        let _ = window.request_inner_size(PhysicalSize::new(w.get(), h.get()));
        surf.resize(w, h)?;
        let mut buf = surf.buffer_mut()?;
        buf.fill(bg_color);

        let vertical_slack = (h.get() as usize - frame.height) / 2;
        let horizontal_slack = (w.get() as usize - frame.width) / 2;

        for row in 0..frame.height {
            let src_start = row * frame.width;
            let src_end = src_start + frame.width;
            let dest_start = (row + vertical_slack) * w.get() as usize + horizontal_slack;
            let dest_end = dest_start + frame.width;

            buf[dest_start..dest_end].copy_from_slice(&frame.buffer[src_start..src_end]);
        }

        buf.present()?;
        Ok(())
    }
}
