use std::path::Path;

use clap::Parser;

use winit::event_loop::{ControlFlow::Wait, EventLoop};

use seized::practice_utils::{args, image::parsers, simple_app};

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

    let filepath = args.image.unwrap();
    let bg_color = args.color.unwrap();
    if filepath.is_empty() {
        println!("No file provided; Window will be just be static");
        app = simple_app::SimpleApplication::new(args.color.unwrap());
    } else {
        app = match parsers::load_from_path(Path::new(&filepath), args.no_aspect) {
            Ok(img) => simple_app::SimpleApplication::new(bg_color).with_image(img),
            Err(err) => {
                eprintln!(
                    "Could not parse provided image ({err})\nDefaulting to static window with no image"
                );
                simple_app::SimpleApplication::new(bg_color)
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
