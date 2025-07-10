
use std::{
    collections::HashMap,
    num::NonZeroU32, num::TryFromIntError,
    rc::Rc
};

use softbuffer::{Context, SoftBufferError, Surface};

use winit::window::{
    Window, 
    WindowId
};

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

    pub fn fill(&mut self, window: Rc<Window>, color: u32) -> Result<(), FillError> {
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
