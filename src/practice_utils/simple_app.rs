use std::{
    cell::RefCell,
    rc::Rc,
    time::{self, Duration},
};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
    event::{MouseButton, StartCause, WindowEvent},
    event_loop::{
        ActiveEventLoop,
        ControlFlow::{Wait, WaitUntil},
    },
    window::{Window, WindowId},
};

use super::image::interpolate::{InterpolationType, interpolate};
use super::{fill, image, widgets};

const TOOLBAR_HEIGHT: usize = 40;

#[derive(Default)]
pub struct Frame {
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub canvas_buffer: Vec<u32>,
    pub toolbar: widgets::Toolbar,
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
    cursor_position: PhysicalPosition<f64>,
}

impl Frame {
    pub fn new(img: &Rc<RefCell<image::Image>>, bg_color: u32) -> Self {
        let img_ref = img.borrow();
        Self {
            canvas_width: img_ref.width as usize,
            canvas_height: img_ref.height as usize,
            canvas_buffer: img_ref
                .image_data
                .iter()
                .map(|&pixel| pixel.to_argb())
                .collect(),
            toolbar: widgets::Toolbar::default(),
            bg_color,
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
            cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
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
            win_height - TOOLBAR_HEIGHT,
            InterpolationType::Bilinear,
        ) {
            eprintln!("Error fitting image to the proper size: {}", err);
        }

        // Create the toolbar after the frame is set up
        self.frame.toolbar.update(win_width);
    }

    fn update_frame(&mut self) -> anyhow::Result<()> {
        let old_width = self.frame.canvas_width;
        let old_height = self.frame.canvas_height;
        interpolate(
            &mut self.frame,
            self.image.clone(),
            old_width,
            old_height,
            InterpolationType::Bilinear,
        )
    }

    fn resize_frame(
        &mut self,
        new_width: usize,
        new_height: usize,
        interpolation_type: InterpolationType,
    ) -> anyhow::Result<()> {
        let result = interpolate(
            &mut self.frame,
            self.image.clone(),
            new_width,
            new_height - TOOLBAR_HEIGHT,
            interpolation_type,
        );

        // Recreate the toolbar for the new window size
        self.frame.toolbar.update(new_width);

        result
    }

    fn toggle_aspect_ratio(&mut self) {
        let current_aspect_locked = self.image.borrow().locked_aspect_ratio;
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

    fn toggle_grayscale(&mut self) {
        let current_grayscale = self.image.borrow().is_grayscale;
        self.image.borrow_mut().is_grayscale = !current_grayscale;
        if let Err(e) = self.update_frame() {
            eprintln!("Failed to regenerate frame: {}", e);
        }

        self.window
            .as_ref()
            .expect("Window context lost")
            .request_redraw();
    }

    fn toggle_inversion(&mut self) {
        let current_inverted = self.image.borrow().inverted;
        self.image.borrow_mut().inverted = !current_inverted;
        if let Err(e) = self.update_frame() {
            eprintln!("Failed to regenerate frame: {}", e);
        }

        self.window
            .as_ref()
            .expect("Window context lost")
            .request_redraw();
    }

    fn rotate(&mut self, rotation: i16, window_size: PhysicalSize<u32>) {
        let current_rotation = self.image.borrow().rotation;
        let new_rotation = (current_rotation as i16 + rotation).rem_euclid(360) as u16;

        // Actually store the new rotation value
        self.image.borrow_mut().rotation = new_rotation;

        // Recalculate frame size based on rotated dimensions
        if let Err(e) = self.resize_frame(
            window_size.width as usize,
            window_size.height as usize,
            InterpolationType::Bilinear,
        ) {
            eprintln!("Failed to resize frame after rotation: {}", e);
        }

        self.window
            .as_ref()
            .expect("Window context lost")
            .request_redraw();
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
            cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
        }
    }
}

impl ApplicationHandler for SimpleApplication {
    // Called once the application is ready to create windows
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = Window::default_attributes();
        attrs.visible = false;
        let img_ref = self.image.borrow();
        // To get the size of the primary monitor, use ActiveEventLoop::primary_monitor() and MonitorHandle::size()
        let monitor_size = event_loop
            .primary_monitor()
            .and_then(|monitor| Some(monitor.size()))
            .unwrap_or(PhysicalSize::new(800, 600));
        let width = img_ref.width.max(800).min(monitor_size.width);
        let height = (img_ref.height + TOOLBAR_HEIGHT as u32)
            .max(600)
            .min(monitor_size.height);
        attrs.inner_size = Some(Size::Physical(
            LogicalSize::new(width, height).to_physical::<u32>(1.0),
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
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = position;
                let window_size = self
                    .window
                    .as_ref()
                    .expect("Window context lost")
                    .inner_size();

                // Check if hovering over toolbar and update accordingly
                let was_hovering = self.frame.toolbar.on_hover(
                    position.x as usize,
                    position.y as usize,
                    window_size.height as usize,
                );

                // If not hovering over the button area, reset toolbar to white
                if !was_hovering && !self.frame.toolbar.button_pressed {
                    self.frame.toolbar.reset();
                }

                // Always request redraw to update the visual state
                self.window
                    .as_ref()
                    .expect("Window context lost")
                    .request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let window_size = self
                    .window
                    .as_ref()
                    .expect("Window context lost")
                    .inner_size();

                // println!("Button: {:?}, State: {:?}", button, state);

                // Check if hovering over toolbar and update accordingly
                if !state.is_pressed() && button == MouseButton::Left {
                    if self.frame.toolbar.on_click(
                        self.cursor_position.x as usize,
                        self.cursor_position.y as usize,
                        window_size.height as usize,
                    ) {
                        self.toggle_inversion();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = event.logical_key.to_text()
                    && !event.state.is_pressed()
                {
                    match key {
                        "a" => {
                            self.toggle_aspect_ratio();
                        }
                        "g" => {
                            self.toggle_grayscale();
                        }
                        "i" => {
                            self.toggle_inversion();
                        }
                        "l" => {
                            self.rotate(
                                -90,
                                self.window
                                    .as_ref()
                                    .expect("Window context lost")
                                    .inner_size(),
                            );
                        }
                        "r" => {
                            self.rotate(
                                90,
                                self.window
                                    .as_ref()
                                    .expect("Window context lost")
                                    .inner_size(),
                            );
                        }
                        "q" => {
                            event_loop.exit();
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }
}
