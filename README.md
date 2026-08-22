# pet2d

Lightweight 2D engine for desktop games — idle games and desktop pets — built
on **Winit + WGPU** with a Macroquad-style API. Your game never touches the
graphics backend: it calls free functions (`clear_background`,
`load_texture`, `is_key_down`, ...) and drives frames with
`next_frame().await`.

- **Platforms:** macOS, Windows
- **Renderer:** WGPU with sprite batching (one draw call per texture)
- **Desktop pets:** opt-in `desktop-pet` feature for transparent,
  always-on-top, decoration-free windows

## Architecture

Layering drawn from the workspace layout and renderer design in
[`PLAN.md`](PLAN.md):

```
┌──────────────────────────────────────────┐
│ your game                                │
│   use engine::*;                         │
└───────────────┬──────────────────────────┘
                │ public API (free functions)
┌───────────────▼──────────────────────────┐
│ engine                                   │
│ window · input · time · draw · assets    │
│ camera · spritesheet · thread-local      │
│ Context + poll-based async frame loop    │
└───────────────┬───────────┬──────────────┘
        │               │
┌───────▼───────┐ ┌─────▼────────┐
│ graphics      │ │ platform     │
│ WGPU renderer │ │ Winit event  │
│ pipelines ·   │ │ loop · window│
│ SpriteBatch · │ │ input · per- │
│ Texture2D     │ │ OS quirks    │
└───────────────┘ └──────────────┘
```

The game only ever imports `engine`. `graphics` and `platform` are internal
crates; all engine state lives in a thread-local `Context` accessed by free
functions (ADR-0001). Decisions are recorded in [`docs/adr/`](docs/adr/).

## Quick start

Clone and run an example:

```sh
git clone https://github.com/elvisbrevi/idle-game.git
cd idle-game
cargo run -p engine --example basic
```

A minimal game:

```rust
use engine::{Color, KeyCode, WindowConfig, clear_background, get_frame_time, is_key_down, next_frame, run};

fn main() {
    run(WindowConfig::default(), || async {
        let mut x = 40.0;
        loop {
            clear_background(Color::from_rgb(30, 30, 46));

            if is_key_down(KeyCode::ArrowRight) {
                x += 200.0 * get_frame_time(); // frame-rate independent
            }

            next_frame().await; // present and yield to the event loop
        }
    });
}
```

`run` takes a `WindowConfig` and an async block; each `next_frame().await`
presents one frame. The loop ends when the closure returns or the user closes
the window.

## Examples

All run from the repo root (assets are resolved relative to it):

| Example | Command | Shows |
|---------|---------|-------|
| basic | `cargo run -p engine --example basic` | Open a window, clear to a color |
| sprite | `cargo run -p engine --example sprite` | Load a PNG, arrow-key movement, rotation, source-rect crop |
| animation | `cargo run -p engine --example animation` | Spritesheet slicing, delta-time animation |
| input | `cargo run -p engine --example input` | Keyboard, mouse buttons, cursor position |
| full | `cargo run -p engine --example full` | Every public API function together |
| desktop pet | `cargo run -p engine --features desktop-pet --example desktop_pet` | Transparent always-on-top floating character |

## API overview

```rust
// Window
WindowConfig { title, width, height, background }   // + WindowConfig::desktop_pet()
screen_width() -> f32            screen_height() -> f32
set_window_position(x, y)        set_window_size(w, h)

// Drawing
clear_background(color)
draw_texture(tex, x, y)          draw_texture_ex(tex, x, y, params)
draw_rectangle(x, y, w, h, color)                    draw_circle(x, y, r, color)

// Assets
load_texture(path) -> Result<Texture2D, EngineError> // cached by path, sync

// Input
is_key_down(key) -> bool         is_key_pressed(key) -> bool
is_mouse_button_down(btn) -> bool                    mouse_position() -> (f32, f32)

// Time
get_frame_time() -> f32          get_time() -> f64   get_fps() -> f32

// Camera & spritesheets
set_camera(camera)               set_default_camera()
SpriteSheet::new(texture, columns, rows)             Animation::new(frames, fps, looping)

// Frame loop
run(config, || async { ... })    next_frame().await
```

Types: `Texture2D` (handle to a GPU texture, `.size()`), `Color`, `Rect`,
`Vec2`, `Camera2D`, `KeyCode`, `MouseButton`, `DrawTextureParams`,
`EngineError`. See [docs.rs-style source docs](crates/engine/src/lib.rs) on
each function for details.

## Tests

```sh
cargo test --workspace
```

Unit tests cover engine internals; headless GPU tests render offscreen in
`graphics`; `engine/tests/api.rs` exercises the public API boundary inside a
real window (it briefly opens one on macOS — winit requires the main thread).

## Dependencies

| Crate | Version | Role |
|-------|---------|------|
| [winit](https://crates.io/crates/winit) | 0.30 | Window creation, event loop, input |
| [wgpu](https://crates.io/crates/wgpu) | 24 | Graphics device, pipelines, rendering |
| [glam](https://crates.io/crates/glam) | 0.29 | Vector/matrix math (re-exported as `engine::math`) |
| [image](https://crates.io/crates/image) | 0.25 | PNG/JPEG decoding for `load_texture` |
| [thiserror](https://crates.io/crates/thiserror) | 2 | Typed errors |

No async runtime: the engine polls the game future once per frame (ADR-0003).
