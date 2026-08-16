# Spec: pet2d — Motor 2D para idle games y mascotas de escritorio

## Problem Statement

No existe en Rust una librería 2D ligera con API estilo Macroquad que permita desarrollar idle games y mascotas de escritorio sin exponer la complejidad de Winit y WGPU. Las alternativas actuales son o bien demasiado pesadas (Bevy), o no soportan desktop pet mode (Macroquad no tiene transparencia real en macOS), o requieren conocimiento del backend gráfico.

El desarrollador necesita una API de alto nivel donde `use pet2d::*;` sea suficiente para crear una ventana, cargar texturas, dibujar sprites, manejar input y tiempo, y ejecutar un game loop — sin importar si el backend real es Vulkan, Direct3D 12 o Metal.

## Solution

Construir `pet2d`, una librería 2D en Rust con API inspirada en Macroquad que abstrae completamente Winit y WGPU. El juego se escribe como un closure que se ejecuta dentro del event loop del motor. El estado global vive en `thread_local` y se accede via funciones libres (`clear_background()`, `draw_texture()`, `is_key_down()`, etc.).

La librería se organiza como Cargo workspace con 3 crates:
- **engine** — API pública para el juego
- **graphics** — renderer WGPU interno
- **platform** — abstracción Winit interna

El desarrollador del juego solo depende de `engine` y nunca importa Winit ni WGPU directamente.

## User Stories

1. As a game developer, I want to create a window with a single function call (`run()`), so that I can start building my game immediately without boilerplate.
2. As a game developer, I want to load a PNG texture with `load_texture("path.png")`, so that I can display sprites on screen.
3. As a game developer, I want to draw a texture at a position with `draw_texture(&texture, x, y)`, so that I can render sprites.
4. As a game developer, I want to draw a texture with custom params (size, rotation, flip, source rect) via `draw_texture_ex()`, so that I can have fine control over sprite rendering.
5. As a game developer, I want to clear the screen with `clear_background(color)`, so that I can set the background each frame.
6. As a game developer, I want to draw colored rectangles with `draw_rectangle()`, so that I can render simple shapes without textures.
7. As a game developer, I want to draw circles with `draw_circle()`, so that I can render simple shapes.
8. As a game developer, I want to check if a key is currently held down with `is_key_down(KeyCode)`, so that I can implement continuous movement.
9. As a game developer, I want to check if a key was just pressed this frame with `is_key_pressed(KeyCode)`, so that I can implement discrete actions like jumping.
10. As a game developer, I want to get the mouse position with `mouse_position()`, so that I can implement mouse interaction.
11. As a game developer, I want to check mouse button state with `is_mouse_button_down()`, so that I can implement clicking.
12. As a game developer, I want to get delta time with `get_frame_time()`, so that I can make movement framerate-independent.
13. As a game developer, I want to get total elapsed time with `get_time()`, so that I can implement time-based animations.
14. As a game developer, I want to get current FPS with `get_fps()`, so that I can display performance info.
15. As a game developer, I want to get screen dimensions with `screen_width()` / `screen_height()`, so that I can position elements relative to the window.
16. As a game developer, I want to set window position with `set_window_position()`, so that I can control where the window appears.
17. As a game developer, I want to set window size with `set_window_size()`, so that I can resize the window programmatically.
18. As a game developer, I want to use a Camera2D with `set_camera()` / `set_default_camera()`, so that I can implement scroll and zoom.
19. As a game developer, I want to call `next_frame()` at the end of my loop, so that the engine can present the frame and wait for the next tick.
20. As a game developer, I want the engine to handle the Winit event loop internally, so that I never interact with `ApplicationHandler` or `EventLoop` directly.
21. As a game developer, I want a transparent window with no decorations via `WindowConfig::desktop_pet()`, so that I can build desktop pet applications.
22. As a game developer, I want the desktop pet window to stay always-on-top, so that the pet is always visible.
23. As a game developer, I want to optionally make the desktop pet click-through, so that mouse events pass to windows behind.
24. As a game developer, I want the engine to automatically batch sprites by texture, so that I get good performance without managing draw calls.
25. As a game developer, I want to load spritesheets and play animations with `SpriteSheet` and `Animation`, so that I can create animated characters.
26. As a game developer, I want the animation system to be time-based (not frame-based), so that animations look correct at any FPS.
27. As a game developer, I want `load_texture()` to be synchronous (blocking internally), so that my code stays simple without async/await.
28. As a game developer, I want the engine to cache loaded textures, so that loading the same path twice doesn't re-read from disk.
29. As a game developer, I want error types from the engine to be descriptive, so that I can debug issues easily.
30. As a game developer, I want the engine to work on both Windows and macOS with the same API, so that I don't need platform-specific code.
31. As a game developer, I want the engine to detect supported alpha modes dynamically, so that transparency works correctly on different hardware.
32. As a game developer, I want pixel-art textures to render crisply with nearest-neighbor sampling, so that sprites look sharp.
33. As a game developer, I want the ability to choose between nearest and linear texture filtering, so that I can support both pixel art and smooth graphics.
34. As a game developer, I want the engine to use orthographic projection by default, so that 2D coordinates map directly to screen pixels.
35. As a game developer, I want `DrawTextureParams` to support source rectangles, so that I can render sub-regions of a spritesheet.
36. As a game developer, I want `DrawTextureParams` to support `dest_size`, so that I can scale sprites.
37. As a game developer, I want `DrawTextureParams` to support rotation and pivot, so that I can rotate sprites around a point.
38. As a game developer, I want `DrawTextureParams` to support flip_x and flip_y, so that I can mirror sprites.
39. As a game developer, I want `DrawTextureParams` to support tinting via color, so that I can recolor sprites.
40. As a game developer, I want the engine to use alpha blending for PNG transparency, so that sprites compose correctly over the desktop.
41. As a game developer, I want the engine to use `wgsl` shaders, so that the shading language is portable across backends.
42. As a game developer, I want the engine to handle window resize events automatically, so that the rendering adapts to the new size.
43. As a game developer, I want the engine to handle window close events, so that the application exits cleanly.
44. As a game developer, I want the `DesktopPetConfig` to be behind a feature flag, so that games that don't need it don't pay the compile cost.
45. As a game developer, I want the engine to use `glam` for math types (Vec2, Mat4), so that I have standard 2D math without extra dependencies.
46. As a game developer, I want the engine to re-export math types from `pet2d::*`, so that I don't need to import `glam` separately.
47. As a game developer, I want a `Color` type with constants like `Color::TRANSPARENT`, `Color::WHITE`, `Color::BLACK`, so that I can use common colors easily.
48. As a game developer, I want the `run()` function to accept a closure, so that I can define my game loop inline.
49. As a game developer, I want the game loop to call my closure repeatedly, so that I can implement update/draw cycles.
50. As a game developer, I want the engine to request redraws automatically, so that I don't need to call `window.request_redraw()` myself.
51. As a game developer, I want a `KeyCode` enum that maps to common keys (arrows, space, letters, numbers), so that I can handle keyboard input portably.
52. As a game developer, I want a `MouseButton` enum (Left, Right, Middle), so that I can handle mouse input.
53. As a game developer, I want the engine to detect and report the active graphics backend (Vulkan/Metal/D3D12), so that I can debug rendering issues.
54. As a game developer, I want the engine to configure the surface format based on hardware capabilities, so that rendering works on different GPUs.
55. As a game developer, I want the engine to select the best supported composite alpha mode, so that transparency works on all platforms.
56. As a game developer, I want the engine to panic with a clear message if functions are called outside the game loop, so that I can debug misuse easily.
57. As a game developer, I want a `WindowConfig` struct with sensible defaults, so that I can customize the window without specifying every option.
58. As a game developer, I want `WindowConfig::default()` to create a normal 800x600 window, so that I get started quickly.
59. As a game developer, I want `WindowConfig::desktop_pet()` to configure all transparent/popup settings, so that I don't have to set them individually.
60. As a game developer, I want to be able to set the window title via `WindowConfig`, so that the window has a meaningful name.
61. As a game developer, I want the engine to handle `AboutToWait` by requesting a redraw, so that the game loop runs continuously.
62. As a game developer, I want the engine to handle `Resumed` by creating the window and initializing WGPU, so that I don't manage the initialization sequence.
63. As a game developer, I want the SpriteBatch to flush automatically at the end of each frame, so that I don't need to manage batch lifecycle.
64. As a game developer, I want the SpriteBatch to group draws by texture, so that draw calls are minimized automatically.
65. As a game developer, I want the engine to support both individual textures and spritesheets, so that I can choose the right abstraction for my needs.
66. As a game developer, I want the `SpriteSheet` to calculate UV coordinates automatically from columns/rows, so that I don't do manual UV math.
67. As a game developer, I want the `Animation` struct to track current frame, fps, and looping, so that I can implement sprite animations easily.
68. As a game developer, I want animations to advance based on delta time, so that they look correct regardless of framerate.
69. As a game developer, I want the engine to be a Cargo workspace, so that the codebase is modular and each concern is isolated.
70. As a game developer, I want the engine to compile on stable Rust, so that I don't need nightly features.

## Implementation Decisions

### Workspace structure
The project is a Cargo workspace with 3 crates: `engine` (public API), `graphics` (WGPU internals), `platform` (Winit internals). No `math` crate — `glam` is used directly and re-exported from engine.

### Global state via thread_local
The engine uses a `thread_local! { static CONTEXT: RefCell<Option<Context>> }` pattern. Public functions like `clear_background()`, `draw_texture()`, etc. are free functions that access this context internally. This mirrors Macroquad's ergonomic API. The context is set once when `run()` starts and cleared when the game loop exits.

### Async bridge
Winit 0.30 is synchronous. The engine implements a minimal poll-based executor for the game closure. `next_frame()` is synchronous from the game's perspective — it triggers `window.request_redraw()` and yields control back to the event loop. No external async runtime (tokio, async-std) is used.

### load_texture is synchronous
`load_texture(path: &str) -> Texture2D` blocks internally using the engine's own executor. This keeps the API simple. The function reads the image file, creates a GPU texture, and returns a `Texture2D` handle. Textures are cached in the `AssetServer` so repeated loads return the cached version.

### Error handling
The library crates (`graphics`, `platform`) use `thiserror` for typed errors. Applications use `anyhow`. The engine exposes an `EngineError` enum. `load_texture()` returns `Result<Texture2D, EngineError>`.

### Sprite batching
A single `SpriteBatch` with dynamic `Vec<SpriteVertex>` and `Vec<u32>` accumulates all draw calls per frame. Vertices are uploaded to GPU in one `write_buffer` call. One `render_pass` with one `draw_indexed` per texture group. Sufficient for ~1000 sprites.

### Camera2D from Phase 4
`Camera2D` with `position: Vec2`, `zoom: Vec2`, `rotation: f32` generates an orthographic projection matrix passed to the vertex shader as a uniform. `set_camera()` / `set_default_camera()` switch between custom and default views.

### Platform abstraction via cfg modules
Platform-specific code uses `#[cfg(target_os = "...")]` modules in `platform/src/`. Winit handles 95% of cross-platform behavior. Platform modules only exist for cases where Winit doesn't resolve the behavior (certain desktop pet quirks on macOS).

### Desktop pet as feature flag
`DesktopPetConfig` and `WindowConfig::desktop_pet()` are behind `#[cfg(feature = "desktop-pet")]`. The feature enables transparent window, no decorations, always-on-top, and optional click-through. Games that don't need it don't compile this code.

### Alpha mode detection
The engine queries `surface.get_capabilities(&adapter).alpha_modes` and selects the best supported mode (prefer PreMultiplied, fallback to Auto, then first available). Never hardcodes `Bgra8Unorm` or `Opaque`.

### Naming
The crate is named `pet2d`. Import as `use pet2d::*;`.

### WGSL shaders
All shaders are written in WGSL for portability across Vulkan/Metal/D3D12. The vertex shader applies camera transformation. The fragment shader samples the texture and outputs with alpha blending.

### API surface
The public API is defined in `engine/src/lib.rs` and re-exports types from submodules. The full API:
- Window: `screen_width`, `screen_height`, `set_window_position`, `set_window_size`
- Drawing: `clear_background`, `draw_texture`, `draw_texture_ex`, `draw_rectangle`, `draw_circle`
- Assets: `load_texture`
- Input: `is_key_down`, `is_key_pressed`, `is_mouse_button_down`, `mouse_position`
- Time: `get_frame_time`, `get_time`, `get_fps`
- Camera: `set_camera`, `set_default_camera`
- Frame: `next_frame`

### Game loop model
`run(config, || { ... })` accepts a closure that runs in a loop. Each iteration: update game state, call draw functions, call `next_frame()`. The engine manages the event loop, window creation, WGPU initialization, and frame presentation internally.

## Testing Decisions

### Seam
The primary testing seam is the **engine public API boundary** — the free functions and types that the game imports via `use pet2d::*;`. This is the highest seam in the codebase. All internal WGPU/Winit complexity is hidden behind this seam.

Testing at this seam means:
- Verifying that `run()` creates a window and executes the game closure
- Verifying that `load_texture()` returns a valid `Texture2D`
- Verifying that `draw_texture()` doesn't panic
- Verifying that `is_key_down()` reflects keyboard state
- Verifying that `get_frame_time()` returns positive values
- Verifying that `clear_background(Color::TRANSPARENT)` doesn't crash on transparent windows

### What makes a good test
Tests should verify external behavior (API contracts), not internal implementation details. A test should answer: "Does this function do what the documentation says?" not "Does this function call the right internal method?"

### Which modules will be tested
- `engine` — API contract tests (integration tests that call the public API)
- `graphics` — renderer initialization and texture creation (unit tests with mock or real GPU)
- `platform` — window creation and input mapping (unit tests)

### Prior art
No existing tests in the codebase. Tests will be created alongside implementation. Integration tests in `examples/` serve as visual/manual tests.

## Out of Scope

- Linux support (future)
- Web/WASM support (future)
- Audio system
- Physics engine
- ECS framework
- UI system
- Tilemap rendering
- Particle system
- Font rendering / text drawing
- Multiple windows
- Scene graph
- Resource hot-reloading
- Serialization / save-load
- Networking
- Macroquad compatibility layer (the API is inspired by, not compatible with)

## Further Notes

### Milestone 1 (Minimum Viable)
The first milestone is: transparent window (Winit) + WGPU clear with alpha = 0 + one textured quad with PNG transparency + works on Windows and macOS. This validates the entire stack before building higher-level features.

### Dependencies
```toml
winit = "0.30"
wgpu = "24"
glam = "0.29"
image = "0.25"
```

### Architecture layers
```
Game
  ↓
Engine (public API — free functions, thread_local state)
  ↓
Graphics (WGPU — renderer, pipeline, batch)
Platform (Winit — window, input, events)
  ↓
WGPU → Vulkan / Metal / D3D12
Winit → OS windowing
```

### Key invariants
1. The game never imports `wgpu` or `winit` directly.
2. All engine state lives in `thread_local` — no context passing.
3. The engine selects GPU configuration dynamically from hardware capabilities.
4. Desktop pet mode is opt-in via feature flag.
5. The API works identically on Windows and macOS.
