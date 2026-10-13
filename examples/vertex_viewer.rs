use std::num::{NonZeroU32, TryFromIntError};
use std::rc::Rc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use seized::canvas::{BlendMode, Canvas};
use seized::fill;

struct SimpleApp {
    window: Option<Rc<Window>>,
    fill_ctx: Option<fill::FillContext>,
    width: NonZeroU32,
    height: NonZeroU32,
    canvases: Vec<Canvas>,
}

impl SimpleApp {
    pub fn new() -> Self {
        SimpleApp {
            window: None,
            fill_ctx: None,
            width: NonZeroU32::new(800).unwrap(),
            height: NonZeroU32::new(600).unwrap(),
            canvases: vec![],
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Result<Self, TryFromIntError> {
        self.width = NonZeroU32::try_from(width)?;
        self.height = NonZeroU32::try_from(height)?;
        Ok(self)
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
        let mut box_1 = Canvas::new(-25, -25, 50, 50, BlendMode::Opaque);
        let mut box_2 = Canvas::new(-25, 500, 1000, 200, BlendMode::Alpha);
        box_1.buffer_mut().fill(0x00FF0000);
        box_2.buffer_mut().fill(0x7F0000FF);

        self.canvases.push(box_1);
        self.canvases.push(box_2);

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
                println!("Bye bye");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.fill_ctx.as_mut().unwrap().present(
                    id,
                    0x00FFFFFF,
                    &self.canvases.iter().collect::<Vec<_>>(),
                ) {
                    eprintln!("Failed to draw frame: {}", err);
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    println!("Hallo Werld");

    let mut app: SimpleApp = match SimpleApp::new().with_size(800, 600) {
        Ok(app) => app,
        Err(err) => panic!("Could not create app: {}", err),
    };

    match app.run() {
        Ok(_) => println!("Exit Success"),
        Err(err) => eprintln!("Error in exit: {}", err),
    }
}
