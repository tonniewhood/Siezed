## 1. Solidify Your Software-Rendering Pipeline (winit + Softbuffer)

1. **PPM parser (✅)**
   - ASCII & binary modes, 8-bit and 16-bit channels  
   - Centering & drawing into the Softbuffer  

2. **BMP parser (✅)**
   - Read BMP header, DIB header, pixel array  
   - Support uncompressed 24-bit; optionally RLE-8 for bonus practice  

3. ~~**Toy RLE-based format** *(optional)*~~
   - ~~Design a tiny run-length encoding scheme~~  
   - ~~Parse & render it to see basic compression in action~~  

4. **Input handling (✅)**
   - Keyboard: switch formats, apply filters (grayscale, invert)  
   - Bonus: use [`pixels`](https://crates.io/crates/pixels) to keep your CPU buffer but let it handle the GPU upload  

## 2. Dip into Native Windowing Protocols

> *Goal: understand what winit abstracts away.*

> Note: This section is rather minimal

- **X11 (XCB or Xlib) (✅)**  
  1. Open a connection, create a simple window (e.g. `CreateWindow`), map it  
  2. Select & read `KeyPress`, `Expose`, `ConfigureNotify` events  
  3. Draw raw bytes to an XImage or shared memory segment  

- **Wayland (✅)**  
  1. Connect to the compositor, create a `wl_surface` & `wl_buffer`  
  2. Handle `wl_surface.enter` / `leave`, frame callbacks  
  3. Commit buffers to present frames  

- **Compare notes**  
  - How does the event loop work? Poll vs wait?  
  - What's the "damage" model for redraws?  

## 3. CPU Rasterization — Vertices to the Screen

> *Goal: write a software rasterizer that turns vertex data into pixels in the existing `Frame.canvas_buffer`. Start in 2D, grow the 3D path as additive features (not a rewrite). Both 2D and 3D stay first-class.*

1. **Module + types**
   - New `practice_utils::raster` module, sibling of `image` and `fill`
   - `Vec2`, `Color`, `Vertex2D { pos: Vec2, color: Color }` to start
   - `Mesh2D { verts: Vec<Vertex2D>, indices: Vec<u32> }`
   - A `RasterTarget` view over `Frame.canvas_buffer` (width, height, &mut [u32]) so primitives don't care about the rest of the frame

2. **2D primitives (screen-space, integer pixels first)**
   - `draw_point` — single pixel write with bounds check
   - `draw_line` — Bresenham, then DDA float variant for sub-pixel comparison
   - `draw_triangle_wireframe` — three lines
   - `draw_triangle_filled` — edge-function rasterizer with barycentric weights, color interpolation across vertices

3. **2D affine transforms**
   - `Mat3` (homogeneous 2D), translate / rotate / scale
   - Apply transform to a `Mesh2D` before rasterizing — first taste of a vertex stage

4. **Anti-aliasing pass (optional, before going 3D)**
   - Coverage-based AA on triangle edges (e.g. 4x supersample, then average) — useful comparison point against the bilinear scaling you already have

5. **3D math foundation (additive, doesn't replace 2D)**
   - `Vec3`, `Vec4`, `Mat4`
   - Translate / rotate (per-axis + axis-angle) / scale / look-at
   - Perspective + orthographic projection

6. **3D vertex pipeline**
   - `Vertex3D { pos: Vec3, normal: Vec3, color, uv }`, `Mesh3D`
   - Stages: model → world → view → clip → NDC → screen
   - Backface culling (CCW convention)
   - Near/far clipping (homogeneous; full Sutherland–Hodgman against the frustum is a stretch goal)

7. **Z-buffer**
   - Add a `depth: Vec<f32>` companion to the canvas, reset on each frame
   - Per-fragment depth test in the triangle rasterizer

8. **Camera**
   - `Camera { position, target, up, fov, aspect, near, far }` builds view + projection
   - Wire keyboard/mouse from §4 input layer to a fly-cam

9. **Texturing**
   - UV interpolation across triangles (perspective-correct in 3D path)
   - Sample from your existing `Image` types — your PPM/BMP loaders become textures
   - Nearest + bilinear sampling

10. **Lighting (optional)**
    - Per-vertex Gouraud shading first (cheap, runs in vertex stage)
    - Per-pixel Phong as a reach goal — sets up the fragment-shader mental model for §6

**Milestones to celebrate (each is a runnable demo / example):**
- A single colored triangle on screen
- Spinning 2D wireframe polygon driven by keyboard
- Spinning 3D wireframe cube
- Solid-shaded 3D cube with z-buffer
- Textured rotating cube using a PPM you already loaded
- (Stretch) Lit, textured cube with a movable camera

## 4. Input & Event Abstraction Layer

> *Goal: stop writing winit event handling inline. Build a small layer the rasterizer demos and any future engine code can sit on top of.*

- **Action-based input**
  - `InputState` snapshot per frame: keys held, just-pressed, just-released, mouse delta, scroll
  - Map raw winit events into actions (e.g. `Action::CameraForward`) so demos depend on actions, not key codes

- **Event/app loop trait**
  - A small `App` trait with `update(dt)` and `render(&mut RasterTarget)` that hides winit boilerplate
  - Convert `swiv` and any new demos to live on top of it

- **Frame timing**
  - Track `dt`, FPS, and an EWMA frame time — needed for fly-cam motion and for §5 profiling

## 5. Make the Rasterizer Suck Less (Profiling & Optimization)

> *Goal: get the CPU pipeline to a reasonable interactive frame rate; learn the profiling tooling along the way.*

- **Tooling**
  - Wire up `criterion` benches in `benches/` for the hot loops (line, triangle, blit)
  - Try `cargo flamegraph` and `perf` on a representative scene; record before/after
  - Add a debug HUD (frame time, triangles/frame, pixels/frame) drawn via the toolbar widget

- **Algorithmic wins (cheap, big payoff)**
  - Bounding-box scan instead of full-frame iteration in triangle raster
  - Edge-function with incremental updates (no per-pixel multiply)
  - Triangle setup outside the inner loop
  - Tile-based rasterization (16×16 or 32×32) to improve cache behavior

- **Memory & blitting**
  - Look at `fill::FillContext::fill` row-copy: is the slack math costing us?
  - Investigate using `pixels` crate for the present step while keeping our raster CPU-side
  - Consider `rayon` for per-tile or per-scanline parallelism

- **Numerical**
  - Replace `f32` matrix math hot paths with explicit unrolled forms where it matters
  - Prefer fixed-point edge functions for sub-pixel precision (classic trick)

## 6. Explore Low-Level GPU Context Creation

> *Goal: see how wgpu or Softbuffer gets its drawable.*

- **OpenGL context**  
  - Linux: GLX or EGL calls to get a `GLContext` from your X11/Wayland window  
  - Create a simple shader, draw a triangle  

- **Vulkan instance & surface**  
  - `vkCreateInstance`, `vkCreateSurfaceKHR`, pick a physical device  
  - Build a swapchain, allocate a command buffer, present one clear  

## 7. Jump into wgpu / WebGPU

1. **"Hello Triangle"**
   - Set up adapter, device, queue  
   - Write a WGSL vertex + fragment shader  
   - Draw a 12-triangle cube; animate a rotation matrix  

2. **Texture & sampling**
   - Load your PPM/BMP into a `wgpu::Texture`  
   - Sample it in a shader and apply it to geometry  

3. **Uniforms & cameras**
   - Reuse the camera controller from §3/§4
   - Pass view/projection matrices as uniform buffers  

4. **Post-processing / compute** *(optional)*
   - Write a compute shader to do a blur or color-grade on a fullscreen quad  

## 8. Higher-Level Engine Considerations

- **Asset & format strategy**  
  - DIY for learning: PPM, BMP, toy RLE  
  - Production: [`image`](https://crates.io/crates/image) + `png`, `jpeg-decoder`, `gif`, etc.  

- **Rendering backend swap**
  - CPU-only (winit + Softbuffer / pixels) vs. GPU (wgpu)
  - Hide both behind a `Renderer` trait so demos run on either

- **Project structure**  
  - Keep parsing, data, rendering, and input in separate modules/crates  
  - Define clear types (`Rgb`, `Image`, `Camera`) and minimize `u32` packing outside render code  
