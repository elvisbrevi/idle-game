use crate::color::Color;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub background: Color,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "pet2d".into(),
            width: 800,
            height: 600,
            background: Color::BLACK,
        }
    }
}

/// Current window width in pixels.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn screen_width() -> f32 {
    crate::context::with(|ctx| ctx.width)
}

/// Current window height in pixels.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn screen_height() -> f32 {
    crate::context::with(|ctx| ctx.height)
}

/// Moves the window so its top-left corner sits at physical screen
/// coordinates `(x, y)`.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn set_window_position(x: f32, y: f32) {
    crate::context::with(|ctx| ctx.window.set_outer_position(x, y));
}

/// Resizes the window to `(width, height)` in pixels. Screen dimensions and
/// the render surface follow immediately; the OS may confirm asynchronously
/// with a resize event carrying the same values.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn set_window_size(width: f32, height: f32) {
    crate::context::with_mut(|ctx| {
        let width = width.round().max(1.0) as u32;
        let height = height.round().max(1.0) as u32;
        // the OS confirms async via a Resized event with the same values on
        // every supported backend; apply locally so reads are consistent now
        ctx.window.set_inner_size(width, height);
        ctx.resize(width, height);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn screen_width_panics_outside_of_run_context() {
        super::screen_width();
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn screen_height_panics_outside_of_run_context() {
        super::screen_height();
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn set_window_position_panics_outside_of_run_context() {
        super::set_window_position(10.0, 20.0);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn set_window_size_panics_outside_of_run_context() {
        super::set_window_size(320.0, 240.0);
    }
}
