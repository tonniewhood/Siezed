use std::{
    cell::RefCell,
    rc::Rc,
    time::{self, Duration},
};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalSize, Position, Size},
    event::{StartCause, WindowEvent},
    event_loop::{
        ActiveEventLoop,
        ControlFlow::{Wait, WaitUntil},
    },
    window::{Window, WindowId},
};

use super::image::interpolate::{InterpolationType, interpolate};
use super::{fill, image};

#[derive(Default)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub bg_color: u32,
}

pub struct SimpleApplication {
    window: Option<Rc<Window>>,
    fill: Option<fill::FillContext>,
    bg_color: u32,
    pub image: Rc<RefCell<image::Image>>,
    pub frame: Frame,
    pending_resize: Option<PhysicalSize<u32>>,
    resize_delay: time::Duration,
}

impl Frame {
    pub fn new(img: &Rc<RefCell<image::Image>>, bg_color: u32) -> Self {
        let img_ref = img.borrow();
        Self {
            width: img_ref.width as usize,
            height: img_ref.height as usize,
            buffer: img_ref
                .image_data
                .iter()
                .map(|&pixel| image::Image::to_argb(pixel))
                .collect(),
            bg_color,
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) -> bool {
        if new_width == self.width && new_height == self.height {
            return false; // No change needed
        }

        self.width = new_width;
        self.height = new_height;
        self.buffer.resize(new_width * new_height, self.bg_color);

        true // Resize occurred
    }
}

/// Represents a simple application with window, image, and frame management.
///
/// # Fields
/// - `window`: Optional window handle.
/// - `fill`: Optional fill property.
/// - `bg_color`: Background color as a `u32`.
/// - `image`: Reference-counted image.
/// - `frame`: Current frame to display.
/// - `pending_resize`: Optional pending resize event.
/// - `resize_delay`: Duration to delay resizing operations.
///
/// # Methods
/// - `new`: Creates a new `SimpleApplication` with the specified background color.
/// - `with_image`: Sets the application's image and updates the frame.
/// - `resize_frame`: Resizes the current frame using the specified interpolation type.
/// - `create_toolbar`: Initializes the application's toolbar (currently unimplemented).
impl SimpleApplication {
    pub fn new(new_color: u32) -> Self {
        Self {
            window: None,
            fill: None,
            bg_color: new_color,
            image: Rc::new(RefCell::new(image::Image::default())),
            frame: Frame::default(),
            pending_resize: None,
            resize_delay: Duration::new(0, 1000000),
        }
    }

    pub fn with_image(mut self, new_image: image::Image) -> Self {
        self.image = Rc::new(RefCell::new(new_image));
        self
    }

    pub fn make_frame(&mut self, win_width: usize, win_height: usize) {
        self.frame = Frame::new(&self.image, self.bg_color);
        if let Err(err) = interpolate(
            &mut self.frame,
            self.image.clone(),
            win_width,
            win_height,
            InterpolationType::Bilinear,
        ) {
            eprintln!("Error fitting image to the proper size: {}", err);
        }
    }

    fn resize_frame(
        &mut self,
        new_width: usize,
        new_height: usize,
        interpolation_type: InterpolationType,
    ) -> anyhow::Result<()> {
        interpolate(
            &mut self.frame,
            self.image.clone(),
            new_width,
            new_height,
            interpolation_type,
        )
    }

    /// Creates a basic toolbar on the window
    fn create_toolbar(&mut self) {
        println!("Not yet implemented");
    }
}

impl Default for SimpleApplication {
    fn default() -> Self {
        SimpleApplication {
            window: None,
            fill: None,
            bg_color: 0x000000,
            image: Rc::new(RefCell::new(image::Image::default())),
            frame: Frame::default(),
            pending_resize: None,
            resize_delay: Duration::new(0, 5000000),
        }
    }
}

impl ApplicationHandler for SimpleApplication {
    // Called once the application is ready to create windows
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed() called - creating window");
        let mut attrs = Window::default_attributes();
        attrs.visible = false;
        let img_ref = self.image.borrow();
        attrs.inner_size = Some(Size::Physical(
            LogicalSize::new(img_ref.width.max(800), img_ref.height.max(600))
                .to_physical::<u32>(1.0),
        ));
        drop(img_ref); // Release the borrow
        attrs.position = Some(Position::Physical(
            LogicalPosition::new(0, 0).to_physical::<i32>(1.0),
        ));

        let win = Rc::new(
            event_loop
                .create_window(attrs)
                .expect("Window creation failed"),
        );
        self.window = Some(win.clone());
        self.fill = Some(fill::FillContext::new(win.clone()).expect("FillContext creation failed"));
        let size = self
            .window
            .as_ref()
            .expect("Window context invalid")
            .inner_size();
        self.make_frame(size.width as usize, size.height as usize);

        self.create_toolbar();

        win.set_visible(true);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::ResumeTimeReached { .. } => match self.pending_resize {
                Some(new_size) => {
                    // Now perform the high-quality resize after the delay
                    if self
                        .resize_frame(
                            new_size.width as usize,
                            new_size.height as usize,
                            InterpolationType::Bilinear,
                        )
                        .is_err()
                    {
                        eprintln!(
                            "Failed to resize frame to {}x{}",
                            new_size.width, new_size.height
                        );
                    }
                    event_loop.set_control_flow(Wait);
                    self.pending_resize = None;
                    self.window
                        .as_ref()
                        .expect("window context invalid")
                        .request_redraw();
                }
                None => {
                    eprintln!("Pending size was None");
                }
            },
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window Close Requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested
                if Some(window_id) == self.window.as_ref().map(|w| w.id()) =>
            {
                let win = self.window.as_ref().unwrap().clone();
                self.fill
                    .as_mut()
                    .unwrap()
                    .fill(win, &mut self.frame, self.bg_color)
                    .unwrap();
            }
            WindowEvent::Resized(new_size) => {
                // Start the timer for delayed high-quality resize
                event_loop.set_control_flow(WaitUntil(time::Instant::now() + self.resize_delay));
                self.pending_resize = Some(new_size);
                // For immediate feedback, just request a redraw without resizing the frame
                // This will show the window border with the old frame centered inside
                self.window
                    .as_ref()
                    .expect("Window context lost")
                    .request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = event.logical_key.to_text() {
                    if key == "a" && !event.state.is_pressed() {
                        let current_aspect_locked = self.image.borrow().locked_aspect_ratio;
                        match current_aspect_locked {
                            true => {
                                println!("Unlocking aspect ratio");
                            }
                            false => {
                                println!("Locking aspect ratio");
                            }
                        }
                        self.image.borrow_mut().locked_aspect_ratio = !current_aspect_locked;
                        let size = self
                            .window
                            .as_ref()
                            .expect("Window context lost")
                            .inner_size();
                        if let Err(e) = self.resize_frame(
                            size.width as usize,
                            size.height as usize,
                            InterpolationType::Bilinear,
                        ) {
                            eprintln!("Failed to resize frame: {}", e);
                        }
                        self.window
                            .as_ref()
                            .expect("Window context lost")
                            .request_redraw();
                    }
                }
            }
            _ => (),
        }
    }
}
