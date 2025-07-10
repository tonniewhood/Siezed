# Seized

Just a side project where I make a game engine based on Rust. It's mainly a tool to learn more about GUIs, Rust, and 3D Rendering...but it's fun cause it's a game.

(The name is a play on 'Rust' being in an engine, so the engine is seized. Sue me)

## Table of Contents

- [Features](#features)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [License](#license)

## Features

- Written in Rust for safety and performance (with some practice code writeen in C)
- Basic 3D rendering pipeline
- GUI integration
- Modular architecture for easy extension
- Example projects and demos

## Getting Started

1. **Clone the repository:**
    ```sh
    git clone https://github.com/yourusername/seized.git
    cd seized
    ```

2. **Build the project:**
    ```sh
    cargo build --release
    ```

## Project Structure

```
Siezed
├── assets
│   └── images
│       ├── field.ppm
│       ├── tree.ppm
│       └── west.ppm
├── benches
├── Cargo.lock
├── Cargo.toml
├── docs
│   └── Roadmap.md
├── examples
├── LICENSE
├── practice
│   ├── c
│   └── rust
├── README.md
├── src
│   ├── args.rs
│   ├── fill.rs
│   ├── image.rs
│   ├── main.rs
│   └── simple_app.rs
└── testing
```

## License

This project is licensed under the terms described in the [LICENSE](./LICENSE) file. ~~See the NOTICE file for additional information.~~ Notice file is currently unavailable.