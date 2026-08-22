use std::cell::RefCell;
use std::task::Waker;

use crate::assets::AssetServer;
use crate::camera::Camera2D;
use crate::color::Color;
use crate::input::InputState;
use crate::time::TimeState;

pub(crate) struct Context {
    pub(crate) renderer: graphics::Renderer,
    pub(crate) assets: AssetServer<graphics::Texture2D>,
    pub(crate) clear_color: Color,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) camera: Camera2D,
    pub(crate) input: InputState,
    pub(crate) time: TimeState,
    /// Handle to the OS window for runtime window control
    /// ([`crate::set_window_position`], [`crate::set_window_size`]).
    pub(crate) window: platform::WindowHandle,
    /// Number of frames presented so far; drives [`crate::next_frame`].
    pub(crate) frame: u64,
    /// Waker of the [`crate::next_frame`] future currently waiting on the
    /// next presented frame.
    pub(crate) frame_waker: Option<Waker>,
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

pub(crate) fn init(
    renderer: graphics::Renderer,
    window: platform::WindowHandle,
    clear_color: Color,
    width: f32,
    height: f32,
) {
    CONTEXT.with(|cell| {
        *cell.borrow_mut() = Some(Context {
            renderer,
            assets: AssetServer::default(),
            clear_color,
            width,
            height,
            camera: Camera2D::default(),
            input: InputState::default(),
            time: TimeState::default(),
            window,
            frame: 0,
            frame_waker: None,
        });
    });
}

pub(crate) fn with<R>(f: impl FnOnce(&Context) -> R) -> R {
    CONTEXT.with(|cell| {
        let borrowed = cell.borrow();
        let context = borrowed.as_ref().expect("pet2d: fuera de contexto");
        f(context)
    })
}

pub(crate) fn with_mut<R>(f: impl FnOnce(&mut Context) -> R) -> R {
    CONTEXT.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        let context = borrowed.as_mut().expect("pet2d: fuera de contexto");
        f(context)
    })
}
