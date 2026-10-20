use std::io::Error;
use std::num::{NonZeroU32, TryFromIntError};
use std::path::Path;
use std::rc::Rc;
use std::time::{self, Instant};

use clap::Parser;

use rand;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use seized::canvas::{BlendMode, Canvas};
use seized::fill;
use seized::geometry::{DrawType, Mesh};
use seized::math::Vec2;

mod args;

// Moves at 50 pixels/second
const SPEED: f32 = 50.0;

struct SimpleApp {
    window: Option<Rc<Window>>,
    fill_ctx: Option<fill::FillContext>,
    width: NonZeroU32,
    height: NonZeroU32,
    canvas: Canvas,
    mesh: Mesh,
    dirty: bool,
    draw_type: DrawType,
    t: time::Instant,
    running: bool,
    p0_dir: Vec2,
    p1_dir: Vec2,
    p2_dir: Vec2,
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
            t: time::Instant::now(),
            running: false,
            p0_dir: Vec2::ZERO,
            p1_dir: Vec2::ZERO,
            p2_dir: Vec2::ZERO,
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Result<Self, TryFromIntError> {
        self.width = NonZeroU32::try_from(width)?;
        self.height = NonZeroU32::try_from(height)?;

        self.mesh.set_bounds((width, height));

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
                self.mesh.set_bounds((size.width, size.height));
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
            WindowEvent::KeyboardInput { event, .. } => match event {
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                } => {
                    if !self.running {
                        self.running = true;
                        self.t = Instant::now();

                        println!("yeet");

                        self.p0_dir = Vec2 {
                            x: rand::random_range(-1.0..1.0),
                            y: rand::random_range(-1.0..1.0),
                        };
                        self.p0_dir = self.p0_dir / self.p0_dir.norm();

                        self.p1_dir = Vec2 {
                            x: rand::random_range(-1.0..1.0),
                            y: rand::random_range(-1.0..1.0),
                        };
                        self.p1_dir = self.p1_dir / self.p1_dir.norm();

                        self.p2_dir = Vec2 {
                            x: rand::random_range(-1.0..1.0),
                            y: rand::random_range(-1.0..1.0),
                        };
                        self.p2_dir = self.p2_dir / self.p2_dir.norm();

                        println!(
                            "p0_dir: {:?}, p1_dir: {:?}, p2_dir: {:?}",
                            self.p0_dir, self.p1_dir, self.p2_dir
                        );
                    } else {
                        self.running = false;
                    }
                }
                _ => {}
            },
            _ => (),
        }
    }

    // Ignoring cause for now. With more causes later, I'll probably use it
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        if !self.running {
            return;
        }

        // println!("yote");

        // Get the change since last update
        let dt = self.t.elapsed();
        self.t = Instant::now();
        let step = dt.as_secs_f32() * SPEED;
        self.mesh.transform(
            &mut [&mut self.p0_dir, &mut self.p1_dir, &mut self.p2_dir],
            step,
        );

        self.dirty = true;
        self.window.as_ref().unwrap().request_redraw();
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
