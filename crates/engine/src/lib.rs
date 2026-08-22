//! pet2d engine: Macroquad-style public API over Winit + WGPU.
//!
//! The game imports everything from this crate (`use engine::*;`) and never
//! touches Winit or WGPU. All state lives in a thread-local [`context::Context`]
//! (ADR-0001) that the free functions access internally.

mod assets;
mod camera;
mod color;
mod context;
pub mod draw;
mod error;
mod frame;
mod input;
pub mod keycode;
mod math;
pub mod mouse;
pub mod spritesheet;
mod time;
pub mod window;

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Waker};

pub use assets::load_texture;
pub use camera::{Camera2D, set_camera, set_default_camera};
pub use color::Color;
pub use draw::{clear_background, draw_circle, draw_rectangle, draw_texture, draw_texture_ex};
pub use error::EngineError;
pub use frame::{NextFrame, next_frame};
pub use graphics::DrawTextureParams;
pub use graphics::Texture2D;
pub use input::{InputState, is_key_down, is_key_pressed, is_mouse_button_down, mouse_position};
pub use keycode::KeyCode;
pub use math::{Mat4, Rect, Vec2, Vec3};
pub use mouse::MouseButton;
pub use spritesheet::{Animation, SpriteSheet};
pub use time::{TimeState, get_fps, get_frame_time, get_time};
#[cfg(feature = "desktop-pet")]
pub use window::DesktopPetConfig;
pub use window::{WindowConfig, screen_height, screen_width, set_window_position, set_window_size};

/// Runs the game with the given configuration (ADR-0003).
///
/// `game` returns the game's main loop as an async block. The engine polls it
/// once per rendered frame until it completes or the window closes; each
/// [`next_frame`] call presents the queued draws and yields back to the event
/// loop:
///
/// ```no_run
/// use engine::{WindowConfig, next_frame, run};
/// # let config = WindowConfig::default();
/// run(config, || async {
///     loop {
///         // update + draw
///         next_frame().await;
///     }
/// });
/// ```
pub fn run<F, Fut>(config: WindowConfig, game: F)
where
    F: FnOnce() -> Fut + 'static,
    Fut: Future<Output = ()> + 'static,
{
    let background = config.background;
    let game: RefCell<Option<F>> = RefCell::new(Some(game));
    // ADR-0003: one active future at a time, polled once per frame by this
    // hand-rolled executor — no async runtime involved
    let future: RefCell<Option<Pin<Box<dyn Future<Output = ()>>>>> = RefCell::new(None);

    let handler = move |event: platform::FrameEvent| -> bool {
        match event {
            platform::FrameEvent::WindowReady(handle, width, height) => {
                // graphics clamps degenerate sizes before configuring the surface
                let renderer = graphics::Renderer::new(handle.clone(), (width, height))
                    .expect("pet2d: failed to initialize renderer");
                context::init(renderer, handle, background, width as f32, height as f32);
                // creating the async block runs no body code yet; the first
                // poll happens on the first Redraw below
                *future.borrow_mut() = Some(Box::pin(game
                    .borrow_mut()
                    .take()
                    .expect("pet2d: game closure consumed twice")(
                )));
                true
            }
            platform::FrameEvent::Resized(width, height) => {
                context::with_mut(|ctx| ctx.resize(width, height));
                true
            }
            platform::FrameEvent::Redraw => {
                context::with_mut(|ctx| ctx.time.update());

                // drive the game until it yields at next_frame() or ends; the
                // future is re-polled on every redraw (ADR-0003), so no waker
                // needs arming
                {
                    let mut slot = future.borrow_mut();
                    if let Some(active) = slot.as_mut() {
                        let mut cx = TaskContext::from_waker(Waker::noop());
                        if active.as_mut().poll(&mut cx).is_ready() {
                            *slot = None;
                        }
                    }
                }

                // present whatever this tick queued and close out the frame
                context::with_mut(|ctx| {
                    let projection = ctx.camera.projection_matrix(ctx.width, ctx.height);
                    ctx.renderer.set_camera(projection.to_cols_array_2d());
                    ctx.renderer.render(ctx.clear_color.to_rgba());
                    ctx.frame += 1;
                    // events accumulated since the last frame were visible to
                    // this one; drop the one-frame ones before the next tick
                    ctx.input.begin_frame();
                });
                // keep running while the game future lives; its completion
                // ends the loop so run() returns
                future.borrow().is_some()
            }
            platform::FrameEvent::KeyboardInput(native_key, state) => {
                context::with_mut(|ctx| {
                    if let Some(key) = keycode::KeyCode::from_native(native_key) {
                        ctx.input.on_key(key, state);
                    }
                });
                true
            }
            platform::FrameEvent::MouseInput(native_button, state) => {
                context::with_mut(|ctx| {
                    if let Some(button) = mouse::MouseButton::from_native(native_button) {
                        ctx.input.on_mouse_button(button, state);
                    }
                });
                true
            }
            platform::FrameEvent::CursorMoved(x, y) => {
                context::with_mut(|ctx| ctx.input.on_cursor_moved(x, y));
                true
            }
        }
    };

    #[cfg(feature = "desktop-pet")]
    {
        let pet = config.pet.map(|pet| platform::PetWindowOptions {
            always_on_top: pet.always_on_top,
            transparent: pet.transparent,
            click_through: pet.click_through,
            decorations: pet.decorations,
        });
        platform::run(&config.title, config.width, config.height, pet, handler);
    }
    #[cfg(not(feature = "desktop-pet"))]
    platform::run(&config.title, config.width, config.height, handler);
}
