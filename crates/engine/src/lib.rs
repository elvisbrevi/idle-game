pub mod color;
pub mod keycode;
pub mod mouse;
pub mod window;

pub use color::Color;
pub use keycode::KeyCode;
pub use mouse::MouseButton;
pub use window::WindowConfig;

pub fn run(config: WindowConfig, game_fn: impl FnOnce() + 'static) {
    platform::run(&config.title, config.width, config.height, game_fn);
}
