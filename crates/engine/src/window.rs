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
}
