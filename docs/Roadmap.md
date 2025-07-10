## 1. Solidify Your Software-Rendering Pipeline (winit + Softbuffer)

1. **PPM parser (✅)**
   - ASCII & binary modes, 8-bit and 16-bit channels  
   - Centering & drawing into the Softbuffer  

2. **BMP parser**
   - Read BMP header, DIB header, pixel array  
   - Support uncompressed 24-bit; optionally RLE-8 for bonus practice  

3. **Toy RLE-based format** *(optional)*
   - Design a tiny run-length encoding scheme  
   - Parse & render it to see basic compression in action  

4. **Input handling**
   - Mouse: pan/zoom your image, pick pixels  
   - Keyboard: switch formats, apply filters (grayscale, invert)  
   - Bonus: use [`pixels`](https://crates.io/crates/pixels) to keep your CPU buffer but let it handle the GPU upload  

## 2. Dip into Native Windowing Protocols

> *Goal: understand what winit abstracts away.*

- **X11 (XCB or Xlib)**  
  1. Open a connection, create a simple window (e.g. `CreateWindow`), map it  
  2. Select & read `KeyPress`, `Expose`, `ConfigureNotify` events  
  3. Draw raw bytes to an XImage or shared memory segment  

- **Wayland**  
  1. Connect to the compositor, create a `wl_surface` & `wl_buffer`  
  2. Handle `wl_surface.enter` / `leave`, frame callbacks  
  3. Commit buffers to present frames  

- **Compare notes**  
  - How does the event loop work? Poll vs wait?  
  - What’s the “damage” model for redraws?  

## 3. Explore Low-Level GPU Context Creation

> *Goal: see how wgpu or Softbuffer gets its drawable.*

- **OpenGL context**  
  - Linux: GLX or EGL calls to get a `GLContext` from your X11/Wayland window  
  - Create a simple shader, draw a triangle  

- **Vulkan instance & surface**  
  - `vkCreateInstance`, `vkCreateSurfaceKHR`, pick a physical device  
  - Build a swapchain, allocate a command buffer, present one clear  

## 4. Jump into wgpu / WebGPU

1. **“Hello Triangle”**  
   - Set up adapter, device, queue  
   - Write a WGSL vertex + fragment shader  
   - Draw a 12-triangle cube; animate a rotation matrix  

2. **Texture & sampling**  
   - Load your PPM/BMP into a `wgpu::Texture`  
   - Sample it in a shader and apply it to geometry  

3. **Uniforms & cameras**  
   - Build a simple camera controller (position, look-at)  
   - Pass view/projection matrices as uniform buffers  

4. **Post-processing / compute** *(optional)*  
   - Write a compute shader to do a blur or color-grade on a fullscreen quad  

## 5. Higher-Level Engine Considerations

- **Event & input abstraction**  
  - Continue using winit for windowing & input  
  - Build your own small layer or adopt a crate (e.g. winit + egui / iced / macroquad)  

- **Asset & format strategy**  
  - DIY for learning: PPM, BMP, toy RLE  
  - Production: [`image`](https://crates.io/crates/image) + `png`, `jpeg-decoder`, `gif`, etc.  

- **Rendering backend**  
  - CPU-only (winit + Softbuffer / pixels) vs. GPU (wgpu)  

- **Project structure**  
  - Keep parsing, data, rendering, and input in separate modules/crates  
  - Define clear types (`Rgb`, `Image`, `Camera`) and minimize `u32` packing outside render code  

---

### Next Steps

- Pick one BMP+RLE mini-project and knock it out this week.  
- Schedule a weekend dive into X11 or Wayland by hand (Rust + XCB or C).  
- Carve out time to spin up a Vulkan/OpenGL “clear-triangle” demo.  
- Launch into your first wgpu cube renderer—apply your image-loading code as textures.  
