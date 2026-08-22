use crate::Texture2D;
use crate::color::Color;
use crate::context;
use graphics::DrawTextureParams;

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
    draw_texture_ex(texture, x, y, DrawTextureParams::default());
}

/// Draws a textured quad with per-draw tweaks: source rect, destination size,
/// rotation around a pivot, flips and tint. See [`DrawTextureParams`].
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn draw_texture_ex(texture: &Texture2D, x: f32, y: f32, params: DrawTextureParams) {
    context::with_mut(|ctx| ctx.renderer.draw_texture_ex(texture, x, y, params));
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn clear_background_panics_outside_of_run_context() {
        super::clear_background(crate::Color::RED);
    }
}
