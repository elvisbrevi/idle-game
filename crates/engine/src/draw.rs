use crate::color::Color;
use crate::context;
use crate::Texture2D;

/// Clears the frame with the given color and paints it immediately.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn clear_background(color: Color) {
    context::with_mut(|ctx| {
        ctx.clear_color = color;
        ctx.renderer.render(color.to_rgba());
    });
}

/// Draws a textured quad with its top-left corner at `(x, y)`, sized to the
/// texture dimensions in pixels.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn draw_texture(texture: &Texture2D, x: f32, y: f32) {
    context::with_mut(|ctx| ctx.renderer.draw_texture(texture, x, y));
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn panics_outside_of_run_context() {
        super::clear_background(crate::Color::RED);
    }
}
