use std::num::{NonZeroU32, TryFromIntError};
use std::rc::Rc;
use std::time;

use clap::Parser;

use seized::camera::Camera;
use seized::math::transform::Pose;
use seized::math::{Quat, Vec3};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::error::EventLoopError;
use winit::event::{DeviceEvent, ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

use seized::canvas::{BlendMode, Canvas};
use seized::fill;
use seized::model::{Model, ModelError};

mod args;

const SPEED: f32 = 5.0;
// Radians of rotation per unit of raw mouse motion
const SENSITIVITY: f32 = 0.002;

#[derive(Default)]
struct CameraState {
    pub z_vel: f32,
    pub x_vel: f32,
    pub cursor_locked: bool,
    pub phi: f32,
    pub theta: f32,
}

struct SimpleApp {
    window: Option<Rc<Window>>,
    fill_ctx: Option<fill::FillContext>,
    width: NonZeroU32,
    height: NonZeroU32,
    canvas: Canvas,
    model: Model,
    camera: Camera,
    last_tick: time::Instant,
    camera_state: CameraState,
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
            camera: Camera::new(Pose::with_position(Vec3::new(0.0, 0.0, 10.0)), 60.0),
            last_tick: time::Instant::now(),
            camera_state: CameraState::default(),
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

        if let Some(window) = &self.window {
            let win_size = window.inner_size();
            self.model.set_screen_height(win_size.height as f32);
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

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.camera_state.cursor_locked {
                    // Vertical motion pitches (about x), horizontal motion yaws (about y)
                    self.camera_state.phi -= delta.1 as f32 * SENSITIVITY;
                    self.camera_state.theta -= delta.0 as f32 * SENSITIVITY;
                    // Keep pitch just short of straight up/down so the view can't flip over
                    self.camera_state.phi = self.camera_state.phi.clamp(-1.55, 1.55);
                }
            }
            _ => {}
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.canvas
                    .resize(size.width as usize, size.height as usize);
                self.model.set_screen_height(size.height as f32);
                self.dirty = true;
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if self.dirty {
                    self.canvas.buffer_mut().fill(0x0);
                    self.model.render(&mut self.canvas, &self.camera);
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
                    physical_key: PhysicalKey::Code(KeyCode::KeyQ),
                    repeat: false,
                    ..
                } => {
                    event_loop.exit();
                }
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyW),
                    repeat: false,
                    ..
                } => {
                    if event.state.is_pressed() {
                        self.camera_state.z_vel = -SPEED;
                    } else {
                        self.camera_state.z_vel = 0.0;
                    }
                }
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyA),
                    repeat: false,
                    ..
                } => {
                    if event.state.is_pressed() {
                        self.camera_state.x_vel = -SPEED;
                    } else {
                        self.camera_state.x_vel = 0.0;
                    }
                }
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyS),
                    repeat: false,
                    ..
                } => {
                    if event.state.is_pressed() {
                        self.camera_state.z_vel = SPEED;
                    } else {
                        self.camera_state.z_vel = 0.0;
                    }
                }
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyD),
                    repeat: false,
                    ..
                } => {
                    if event.state.is_pressed() {
                        self.camera_state.x_vel = SPEED;
                    } else {
                        self.camera_state.x_vel = 0.0;
                    }
                }
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Tab),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                } => {
                    if let Some(window) = &self.window {
                        if !self.camera_state.cursor_locked {
                            window
                                .set_cursor_grab(CursorGrabMode::Locked)
                                .expect("Why?");

                            let win_size = window.inner_size();
                            window
                                .set_cursor_position(PhysicalPosition::new(
                                    win_size.width / 2,
                                    win_size.height / 2,
                                ))
                                .expect("You too?");
                            window.set_cursor_visible(false);

                            self.camera_state.cursor_locked = true;
                        } else {
                            window.set_cursor_grab(CursorGrabMode::None).expect("Why?");
                            window.set_cursor_visible(true);

                            self.camera_state.cursor_locked = false;
                        }
                    }
                }
                _ => {}
            },
            _ => (),
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if cause == StartCause::Poll {
            let now = time::Instant::now();
            let dt = now.duration_since(self.last_tick).as_secs_f32();

            // phi/theta are absolute angles, so the rotation is rebuilt from them each frame
            // (camera -> world: yaw about world up, then pitch about the camera's own x)
            let q = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), self.camera_state.theta)
                * Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), self.camera_state.phi);
            let p = Vec3::new(
                self.camera_state.x_vel * dt,
                0.0,
                self.camera_state.z_vel * dt,
            );

            self.camera.transform(q.rotate(p), q);
            self.dirty = true;
            self.last_tick = time::Instant::now();
            self.window.as_ref().unwrap().request_redraw();
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
