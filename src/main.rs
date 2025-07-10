
use clap::Parser;

use winit::{
    event_loop::{ControlFlow::Wait, EventLoop}
};

mod args;
mod fill;
mod image;
mod simple_app;


fn main() {

    // Explicitly match on the Result:
    let mut app: simple_app::SimpleApplication;
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(err) => {
            eprintln!("Failed to create event loop: {}", err);
            std::process::exit(1);
        }
    };
    
    let args = args::Args::parse();

    match (args.image, args.color) {
        (Some(image_filename), None) => {
            println!("Loading PPM from {}", image_filename);
            // eprintln!("PPM not implemented yet; Defaulting to background of 0x000000");
            match image::Image::new(image_filename) {
                Ok(img) => app = simple_app::SimpleApplication::default().with_image(img),
                Err(err) => {
                    eprintln!("Error creating Image: {}", err);
                    eprintln!("Using defualt window");
                    app = simple_app::SimpleApplication::default();
                }
            };
        },
        (None, Some(rgb)) => {
            println!("Filling window with color 0x{:06X}", rgb);
            app = simple_app::SimpleApplication::default().with_color(rgb);
        },
        _ => {
            eprint!("Either --image <FILE> or --color <RGB> must be provided");
            std::process::exit(1);
        }
    }

    event_loop.set_control_flow(Wait);
    if let Err(panic_info) = event_loop.run_app(&mut app){
        eprintln!("Error running event loop: {}", panic_info);
    }
}

