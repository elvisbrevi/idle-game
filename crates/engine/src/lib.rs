pub mod color;
mod context;
pub mod draw;
pub mod keycode;
pub mod mouse;
pub mod window;

pub use color::Color;
pub use draw::clear_background;
pub use keycode::KeyCode;
pub use mouse::MouseButton;
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
                let renderer = graphics::Renderer::new(handle, (width.max(1), height.max(1)))
                    .expect("pet2d: failed to initialize renderer");
                context::init(
                    renderer,
                    background,
                    width as f32,
                    height as f32,
                );
            }
            platform::FrameEvent::Resized(width, height) => {
                context::with_mut(|ctx| {
                    ctx.width = width as f32;
                    ctx.height = height as f32;
                    ctx.renderer.resize(width, height);
                });
            }
            platform::FrameEvent::Redraw => {
                context::with_mut(|ctx| ctx.renderer.render(ctx.clear_color.to_rgba()));
            }
        },
    );
}
