use std::num::{NonZeroU32, TryFromIntError};
use std::rc::Rc;

use clap::Parser;

use seized::math::Vec3;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use seized::canvas::{BlendMode, Canvas};
use seized::fill;
use seized::model::{Model, ModelError};

mod args;

struct SimpleApp {
    window: Option<Rc<Window>>,
    fill_ctx: Option<fill::FillContext>,
    width: NonZeroU32,
    height: NonZeroU32,
    canvas: Canvas,
    model: Model,
    dirty: bool,
}

impl SimpleApp {
    pub fn new() -> Self {
        SimpleApp {
            window: None,
            fill_ctx: None,
            width: NonZeroU32::new(800).unwrap(),
            height: NonZeroU32::new(600).unwrap(),
            canvas: Canvas::new(0, 0, 800, 600, BlendMode::Opaque),
            model: Model::default(),
            dirty: true,
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Result<Self, TryFromIntError> {
        self.width = NonZeroU32::try_from(width)?;
        self.height = NonZeroU32::try_from(height)?;

        Ok(self)
    }

    pub fn load_model(
        &mut self,
        model_filename: &str,
        scale_factor: f32,
    ) -> Result<(), ModelError> {
        if let Err(err) = self.model.load_model(model_filename) {
            return Err(err);
        }

        self.model.set_scale_factor(scale_factor);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), EventLoopError> {
        let event_loop: EventLoop<()> = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)
    }
}

impl ApplicationHandler for SimpleApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(self.width.get(), self.height.get())),
                )
                .unwrap(),
        );
        let window_id = window.as_ref().id();
        let mut fill_ctx =
            fill::FillContext::new(Rc::new(event_loop.owned_display_handle())).unwrap();

        match fill_ctx.register_window(window_id, window.clone()) {
            Ok(()) => (),
            Err(_) => eprintln!("Failed to register window with id: {:?}", window_id),
        };

        self.window = Some(window);
        self.fill_ctx = Some(fill_ctx);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.canvas
                    .resize(size.width as usize, size.height as usize);
                self.model.set_position(Vec3::new(
                    (size.width as f32) / 2.0,
                    (size.height as f32) / 2.0 * 3.0 / 2.0,
                    0.0,
                ));
                self.model.render(&mut self.canvas);
                self.dirty = true;
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if self.dirty {
                    self.canvas.buffer_mut().fill(0x0);
                    self.model.render(&mut self.canvas);
                    self.dirty = false;
                }
                if let Err(err) = self
                    .fill_ctx
                    .as_mut()
                    .unwrap()
                    .present(id, 0x0, &[&self.canvas])
                {
                    eprintln!("Failed to draw frame: {}", err);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => match event {
                _ => {}
            },
            _ => (),
        }
    }
}

fn main() {
    println!("Hallo Werld");
    let args = args::Args::parse();

    let mut app: SimpleApp = match SimpleApp::new().with_size(800, 600) {
        Ok(app) => app,
        Err(err) => panic!("Could not create app: {}", err),
    };

    match app.load_model(&args.model, args.scale_factor) {
        Ok(_) => println!("Loaded mesh '{}'", args.model),
        Err(err) => panic!("Could not load model '{}': {}", args.model, err),
    }

    match app.run() {
        Ok(_) => println!("Exit Success"),
        Err(err) => eprintln!("Error in exit: {}", err),
    }
}
