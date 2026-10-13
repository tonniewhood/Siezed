mod args;

use std::{
    path::Path,
    rc::Rc,
    time::{self, Duration},
};

use clap::Parser;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
    event::{MouseButton, StartCause, WindowEvent},
    event_loop::{
        ActiveEventLoop,
        ControlFlow::{Wait, WaitUntil},
        EventLoop,
    },
    window::{Window, WindowId},
};

use seized::{
    canvas::BlendMode,
    fill,
    image::{self, interpolate::InterpolationType},
    parsers,
    widgets::{ImageView, TOOLBAR_HEIGHT, Toolbar},
};

struct SimpleApplication {
    window: Option<Rc<Window>>,
    fill: Option<fill::FillContext>,
    bg_color: u32,
    image_view: ImageView,
    toolbar: Toolbar,
    pending_resize: Option<PhysicalSize<u32>>,
    resize_delay: time::Duration,
    cursor_position: PhysicalPosition<f64>,
}

impl SimpleApplication {
    pub fn new(new_color: u32, locked_aspect: bool) -> Self {
        Self {
            window: None,
            fill: None,
            bg_color: new_color,
            image_view: ImageView::new(0, 0, Some(locked_aspect), Some(BlendMode::Opaque)),
            toolbar: Toolbar::new(),
            pending_resize: None,
            resize_delay: Duration::new(0, 1000000),
            cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
        }
    }

    pub fn with_image(mut self, new_image: image::Image) -> Self {
        self.image_view.set_img(new_image);
        self
    }

    pub fn update_scene(&mut self, print_errs: bool) {
        let win_size = self
            .window
            .as_ref()
            .expect("Window context lost")
            .inner_size();
        // Without an image there's nothing to rebuild, so the window is just left static
        if self.image_view.img_ref().is_some()
            && let Err(err) = self.image_view.rebuild(
                win_size.width as usize,
                (win_size.height as usize).saturating_sub(TOOLBAR_HEIGHT),
                InterpolationType::Bilinear,
            )
            && print_errs
        {
            eprintln!("Error fitting image to the proper size: {}", err);
        }

        // Create the toolbar after the frame is set up
        self.toolbar
            .update(win_size.width as usize, win_size.height as usize);
    }

    fn resize_frame(
        &mut self,
        new_width: usize,
        new_height: usize,
        interpolation_type: InterpolationType,
    ) -> anyhow::Result<()> {
        // Recreate the toolbar for the new window size
        self.toolbar.update(new_width, new_height);

        self.image_view.rebuild(
            new_width,
            new_height.saturating_sub(TOOLBAR_HEIGHT),
            interpolation_type,
        )
    }

    fn toggle_aspect_ratio(&mut self) {
        self.image_view.toggle_locked_aspect();
        self.update_scene(true);
        self.window
            .as_ref()
            .expect("Window context lost")
            .request_redraw();
    }

    fn toggle_grayscale(&mut self) {
        self.image_view.toggle_grayscale();
        self.update_scene(true);
        self.window
            .as_ref()
            .expect("Window context lost")
            .request_redraw();
    }

    fn toggle_inversion(&mut self) {
        self.image_view.toggle_inverted();
        self.update_scene(true);
        self.window
            .as_ref()
            .expect("Window context lost")
            .request_redraw();
    }

    fn rotate(&mut self, rotation: i16) {
        self.image_view.rotate(rotation);
        self.update_scene(true);
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
            bg_color: 0x0,
            image_view: ImageView::new(0, 0, Some(true), Some(BlendMode::Opaque)),
            toolbar: Toolbar::new(),
            pending_resize: None,
            resize_delay: Duration::new(0, 1000000),
            cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
        }
    }
}

impl ApplicationHandler for SimpleApplication {
    // Called once the application is ready to create windows
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = Window::default_attributes();
        attrs.visible = false;
        // To get the size of the primary monitor, use ActiveEventLoop::primary_monitor() and MonitorHandle::size()
        let monitor_size = event_loop
            .primary_monitor()
            .and_then(|monitor| Some(monitor.size()))
            .unwrap_or(PhysicalSize::new(800, 600));
        let size = match self.image_view.img_size() {
            // At least 800x600 so small images still get a usable window, at most the
            // monitor size so large images don't open off-screen
            Ok((img_width, img_height)) => (
                img_width.max(800).min(monitor_size.width),
                (img_height + TOOLBAR_HEIGHT as u32)
                    .max(600)
                    .min(monitor_size.height),
            ),
            Err(_) => (monitor_size.width.min(800), monitor_size.height.min(600)),
        };

        attrs.inner_size = Some(Size::Physical(
            LogicalSize::new(size.0, size.1).to_physical::<u32>(1.0),
        ));
        attrs.position = Some(Position::Physical(
            LogicalPosition::new(0, 0).to_physical::<i32>(1.0),
        ));

        let win = Rc::new(
            event_loop
                .create_window(attrs)
                .expect("Window creation failed"),
        );
        self.window = Some(win.clone());
        let mut fill_ctx = fill::FillContext::new(Rc::new(event_loop.owned_display_handle()))
            .expect("FillContext creation failed");
        fill_ctx
            .register_window(win.id(), win.clone())
            .expect("Window registration failed");
        self.fill = Some(fill_ctx);

        self.update_scene(true);

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
                self.fill
                    .as_mut()
                    .unwrap()
                    .present(
                        window_id,
                        self.bg_color,
                        &[self.image_view.canvas(), self.toolbar.canvas()],
                    )
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
                let was_hovering = self.toolbar.on_hover(
                    position.x as usize,
                    position.y as usize,
                    window_size.height as usize,
                );

                // If not hovering over the button area, reset toolbar to white
                if !was_hovering && !self.toolbar.btn_pressed() {
                    self.toolbar.reset();
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
                    if self.toolbar.on_click(
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
                            self.rotate(-90);
                        }
                        "r" => {
                            self.rotate(90);
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

fn main() {
    // Explicitly match on the Result:
    let mut app: SimpleApplication;
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(err) => {
            eprintln!("Failed to create event loop: {}", err);
            std::process::exit(1);
        }
    };

    let args = args::Args::parse();

    let filepath = args.image.unwrap();
    let bg_color = args.color.unwrap();
    if filepath.is_empty() {
        println!("No file provided; Window will be just be static");
        app = SimpleApplication::new(args.color.unwrap(), true);
    } else {
        app = match parsers::load_from_path(Path::new(&filepath)) {
            Ok(img) => SimpleApplication::new(bg_color, !args.no_aspect).with_image(img),
            Err(err) => {
                eprintln!(
                    "Could not parse provided image ({err})\nDefaulting to static window with no image"
                );
                SimpleApplication::new(bg_color, !args.no_aspect)
            }
        }
    }
    println!(
        "Using image '{}' with background '0x{:06X}'",
        filepath, bg_color
    );

    event_loop.set_control_flow(Wait);
    if let Err(panic_info) = event_loop.run_app(&mut app) {
        eprintln!("Error running event loop: {}", panic_info);
    }
}
