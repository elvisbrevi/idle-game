mod assets;
mod camera;
mod color;
mod context;
pub mod draw;
mod input;
pub mod keycode;
mod math;
pub mod mouse;
mod time;
pub mod window;

pub use assets::load_texture;
pub use camera::{Camera2D, set_camera, set_default_camera};
pub use color::Color;
pub use draw::{clear_background, draw_texture, draw_texture_ex};
pub use graphics::Texture2D;
pub use graphics::{DrawTextureParams, Rect};
pub use input::{InputState, is_key_down, is_key_pressed, is_mouse_button_down, mouse_position};
pub use keycode::KeyCode;
pub use math::{Mat4, Vec2};
pub use mouse::MouseButton;
pub use time::{TimeState, get_fps, get_frame_time, get_time};
pub use window::{WindowConfig, screen_height, screen_width};

/// Runs the game with the given configuration.
///
/// The game closure is executed once when the first frame starts.
pub fn run(config: WindowConfig, game_fn: impl FnOnce() + 'static) {
    let background = config.background;
    platform::run(
        &config.title,
        config.width,
        config.height,
        game_fn,
        move |event| match event {
            platform::FrameEvent::WindowReady(handle, width, height) => {
                // graphics clamps degenerate sizes before configuring the surface
                let renderer = graphics::Renderer::new(handle, (width, height))
                    .expect("pet2d: failed to initialize renderer");
                context::init(renderer, background, width as f32, height as f32);
            }
            platform::FrameEvent::Resized(width, height) => {
                context::with_mut(|ctx| {
                    ctx.width = width as f32;
                    ctx.height = height as f32;
                    ctx.renderer.resize(width, height);
                });
            }
            platform::FrameEvent::Redraw => {
                context::with_mut(|ctx| {
                    ctx.time.update();
                    let projection = ctx.camera.projection_matrix(ctx.width, ctx.height);
                    ctx.renderer.set_camera(projection.to_cols_array_2d());
                    ctx.renderer.render(ctx.clear_color.to_rgba());
                    // events accumulated since the last frame were visible to
                    // this one; drop the one-frame ones before the next tick
                    ctx.input.begin_frame();
                });
            }
            platform::FrameEvent::KeyboardInput(native_key, state) => {
                context::with_mut(|ctx| {
                    if let Some(key) = keycode::KeyCode::from_native(native_key) {
                        ctx.input.on_key(key, state);
                    }
                });
            }
            platform::FrameEvent::MouseInput(native_button, state) => {
                context::with_mut(|ctx| {
                    if let Some(button) = mouse::MouseButton::from_native(native_button) {
                        ctx.input.on_mouse_button(button, state);
                    }
                });
            }
            platform::FrameEvent::CursorMoved(x, y) => {
                context::with_mut(|ctx| ctx.input.on_cursor_moved(x, y));
            }
        },
    );
}
