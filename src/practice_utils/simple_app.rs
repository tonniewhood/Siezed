use std::rc::Rc;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, Position, Size},
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::practice_utils::fill;
use crate::practice_utils::image;

pub struct SimpleApplication {
    window: Option<Rc<Window>>,
    fill: Option<fill::FillContext>,
    bg_color: u32,
    pub image: Rc<image::Image>,
}

impl SimpleApplication {
    pub fn new(new_color: u32) -> Self {
        Self {
            window: None,
            fill: None,
            bg_color: new_color,
            image: Rc::new(image::Image::default()),
        }
    }

    pub fn with_image(mut self, new_image: image::Image) -> Self {
        self.image = Rc::new(new_image);
        self
    }
}

impl Default for SimpleApplication {
    fn default() -> Self {
        SimpleApplication {
            window: None,
            fill: None,
            bg_color: 0x000000,
            image: Rc::new(image::Image::default()),
        }
    }
}

impl ApplicationHandler for SimpleApplication {
    // Called once the application is ready to create windows
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = Window::default_attributes();
        attrs.visible = false;
        attrs.inner_size = Some(Size::Physical(
            LogicalSize::new(800, 600).to_physical::<u32>(1.0),
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
        self.fill = Some(fill::FillContext::new(win.clone()).expect("FillContext creation failed"));

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
            }
            WindowEvent::RedrawRequested
                if Some(window_id) == self.window.as_ref().map(|w| w.id()) =>
            {
                let win = self.window.as_ref().unwrap().clone();
                self.fill
                    .as_mut()
                    .unwrap()
                    .fill(win, self.image.clone(), self.bg_color)
                    .unwrap();
            }
            _ => (),
        }
    }
}
