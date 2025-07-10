
use std::{
    collections::HashMap,
    num::NonZeroU32, num::TryFromIntError,
    rc::Rc
};

use softbuffer::{Context, SoftBufferError, Surface};

use winit::dpi::PhysicalSize;
use winit::window::{
    Window, 
    WindowId
};


use crate::image;


#[derive(thiserror::Error, Debug)]
pub enum FillError {
    #[error(transparent)]
    TryFromInt(std::num::TryFromIntError),
    #[error(transparent)]
    SoftBufferError(softbuffer::SoftBufferError),
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
    surfaces: HashMap<WindowId, Surface<Rc<Window>, Rc<Window>>>
}

impl FillContext {

    pub fn new(window: Rc<Window>) -> Result<Self, FillError> {
        let gc = Context::new(window.clone())?;
        Ok(Self {gc, surfaces: HashMap::new()})
    }

    pub fn fill_image(&mut self, window: Rc<Window>, image: Rc<image::Image>, background_color: u32) -> Result<(), FillError> {
        let size = window.inner_size();
        let (w, h) = (
            NonZeroU32::try_from(u32::max(size.width, image.width))?,
            NonZeroU32::try_from(u32::max(size.height, image.height))?
        );

        let _ = window.request_inner_size(PhysicalSize::new(w.get(), h.get()));
    
        let surf = self.surfaces.entry(window.id()).or_insert_with(|| {
                Surface::new(&self.gc, window.clone()).expect("Error: softbuffer Surface creation failed")
        });

        // TODO: Update the image to remove the black bar that appears on the bottom for some reason
        surf.resize(w, h)?;
        {
            let mut buf = surf.buffer_mut()?;
            let horizontal_slack = ((w.get() - image.width) / 2) as usize;
            let vertial_slack = ((h.get() - image.height) / 2) as usize;

            for (idx, pixel) in buf.iter_mut().enumerate() {
                let row_idx = idx / (h.get() as usize);
                let col_idx = idx % (w.get() as usize);

                *pixel = if col_idx < horizontal_slack || col_idx >= (image.width as usize) + horizontal_slack || row_idx < vertial_slack || row_idx >= (image.height as usize) + vertial_slack {
                    background_color 
                }
                else {
                    let image_row_idx = row_idx - vertial_slack;
                    let image_col_idx = col_idx - horizontal_slack;
                    image.image_data[(image.width as usize) * image_row_idx + image_col_idx]
                }
            }

            buf.present()?;
        }
        Ok(())
    }

    pub fn fill_solid(&mut self, window: Rc<Window>, color: u32) -> Result<(), FillError> {
        let size = window.inner_size();
        let (w, h) = (
            NonZeroU32::try_from(size.width)?,
            NonZeroU32::try_from(size.height)?
        );
        
        let surf = self.surfaces.entry(window.id()).or_insert_with(|| {
                Surface::new(&self.gc, window.clone()).expect("Error: softbuffer Surface creation failed")
        });

        surf.resize(w, h)?;
        {
            let mut buf = surf.buffer_mut()?;
            buf.fill(color);
            buf.present()?;
        }
        Ok(())
    }

}
