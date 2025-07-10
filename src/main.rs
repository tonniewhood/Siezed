
use std::{
    default::Default,
    env,
    rc::Rc
};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, Position, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow::Wait, EventLoop},
    window::{Cursor, Fullscreen, Icon, Theme, Window, WindowButtons, WindowId, WindowLevel, WindowAttributes}
};

use crate::fill::FillContext;

#[path = "fill.rs"]
mod fill;

struct HelloApp {
    window: Option<Rc<Window>>,
    fill: Option<FillContext>,
    color: u32
}

impl Default for HelloApp {
    fn default() -> Self {
        HelloApp { window: None, fill: None, color: 0x000000 }
    }
}

impl ApplicationHandler for HelloApp {
    // Called once the application is ready to create windows
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = Window::default_attributes();
        attrs.visible = false;
        attrs.inner_size = Some(Size::Physical(LogicalSize::new(800, 600).to_physical::<u32>(1.0)));
        attrs.position = Some(Position::Physical(LogicalPosition::new(0, 0).to_physical::<i32>(1.0)));

        let win= Rc::new(event_loop.create_window(attrs).expect("Window creation failed"));
        self.window = Some(win.clone());
        self.fill = Some(FillContext::new(win.clone()).expect("FillContext creation failed"));

        win.set_visible(true);
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
            },
            WindowEvent::RedrawRequested if Some(window_id) == self.window.as_ref().map(|w| w.id()) => {
                let win = self.window.as_ref().unwrap().clone();
                self.fill.as_mut().unwrap().fill(win, self.color).unwrap();
            }
            _ => (),
        }
    }
}

fn main() {
    // Explicitly match on the Result:
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(err) => {
            eprintln!("Failed to create event loop: {}", err);
            std::process::exit(1);
        }
    };

    let mut app = HelloApp::default();
    
    // let args = env::args();
    if let Some(hex_color) = env::args().nth(1) {
        let color = hex_color.trim_start_matches('#').trim_start_matches("0x");
        match u32::from_str_radix(color, 16) {
            Ok(valid_color) => { app.color = valid_color },
            Err(_) => {
                eprintln!("Could not parse '{hex_color}' as a valid hex color; Defualting to black (0x000000)");
            }
        }
    }

    event_loop.set_control_flow(Wait);
    if let Err(panic_info) = event_loop.run_app(&mut app){
        eprintln!("Error running event loop: {}", panic_info);
    }
}

