use std::io::Error;
use std::num::{NonZeroU32, TryFromIntError};
use std::path::Path;
use std::rc::Rc;

use clap::Parser;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use seized::canvas::{BlendMode, Canvas};
use seized::fill;
use seized::geometry::{DrawType, Mesh};

mod args;

struct SimpleApp {
    window: Option<Rc<Window>>,
    fill_ctx: Option<fill::FillContext>,
    width: NonZeroU32,
    height: NonZeroU32,
    canvas: Canvas,
    mesh: Mesh,
    dirty: bool,
    draw_type: DrawType,
}

impl SimpleApp {
    pub fn new(draw_type: DrawType) -> Self {
        SimpleApp {
            window: None,
            fill_ctx: None,
            width: NonZeroU32::new(800).unwrap(),
            height: NonZeroU32::new(600).unwrap(),
            canvas: Canvas::new(0, 0, 800, 600, BlendMode::Opaque),
            mesh: Mesh::default(),
            dirty: true,
            draw_type: draw_type,
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Result<Self, TryFromIntError> {
        self.width = NonZeroU32::try_from(width)?;
        self.height = NonZeroU32::try_from(height)?;
        Ok(self)
    }

    pub fn load_mesh(&mut self, mesh_filepath: &Path) -> Result<(), Error> {
        if let Err(err) = self.mesh.load_mesh(mesh_filepath) {
            return Err(err);
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), EventLoopError> {
        let event_loop: EventLoop<()> = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(self)
    }

    fn reset_mesh() {}
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
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.canvas
                    .resize(size.width as usize, size.height as usize);
                self.dirty = true;
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if self.dirty {
                    self.canvas.buffer_mut().fill(0x0);
                    self.mesh.draw_2d_shape(&mut self.canvas, self.draw_type);
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
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = event.logical_key.to_text()
                    && !event.state.is_pressed()
                {
                    match key {
                        "r" => {
                            self.reset_mesh();
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
    println!("Hallo Werld");
    let args = args::Args::parse();

    let draw_type = if args.wire {
        DrawType::Wire
    } else {
        DrawType::Fill
    };
    let mesh_filepath = Path::new(&args.mesh);

    let mut app: SimpleApp = match SimpleApp::new(draw_type).with_size(800, 600) {
        Ok(app) => app,
        Err(err) => panic!("Could not create app: {}", err),
    };

    match app.load_mesh(Path::new(&mesh_filepath)) {
        Ok(_) => println!("Loaded mesh '{}'", args.mesh),
        Err(err) => panic!("Could not load mesh '{}': {}", args.mesh, err),
    }

    match app.run() {
        Ok(_) => println!("Exit Success"),
        Err(err) => eprintln!("Error in exit: {}", err),
    }
}
