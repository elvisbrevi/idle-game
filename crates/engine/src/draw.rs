use crate::color::Color;
use crate::context;

/// Clears the frame with the given color and paints it immediately.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn clear_background(color: Color) {
    context::with_mut(|ctx| {
        ctx.clear_color = color;
        ctx.renderer.render(color.to_rgba());
    });
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn panics_outside_of_run_context() {
        super::clear_background(crate::Color::RED);
    }
}
