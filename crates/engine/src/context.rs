use std::cell::RefCell;

use crate::camera::Camera2D;
use crate::color::Color;

pub(crate) struct Context {
    pub(crate) renderer: graphics::Renderer,
    pub(crate) clear_color: Color,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) camera: Camera2D,
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

pub(crate) fn init(
    renderer: graphics::Renderer,
    clear_color: Color,
    width: f32,
    height: f32,
) {
    CONTEXT.with(|cell| {
        *cell.borrow_mut() = Some(Context {
            renderer,
            clear_color,
            width,
            height,
            camera: Camera2D::default(),
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
