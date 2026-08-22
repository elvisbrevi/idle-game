use crate::Texture2D;
use crate::color::Color;
use crate::context;
use graphics::DrawTextureParams;

/// Sets the color the next frame is cleared with. Unlike the draw calls it
/// does not paint immediately: [`crate::next_frame`] presents a frame filled
/// with this color behind everything drawn on top.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn clear_background(color: Color) {
    context::with_mut(|ctx| ctx.clear_color = color);
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

/// Draws a filled solid-color rectangle with its top-left corner at
/// `(x, y)`, sized `(width, height)` in world units.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn draw_rectangle(x: f32, y: f32, width: f32, height: f32, color: Color) {
    context::with_mut(|ctx| ctx.renderer.draw_rectangle(x, y, width, height, color));
}

/// Draws a filled solid-color circle centered at `(x, y)` with `radius` in
/// world units.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {
    context::with_mut(|ctx| ctx.renderer.draw_circle(x, y, radius, color));
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn clear_background_panics_outside_of_run_context() {
        super::clear_background(crate::Color::RED);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn draw_rectangle_panics_outside_of_run_context() {
        super::draw_rectangle(0.0, 0.0, 10.0, 10.0, crate::Color::RED);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn draw_circle_panics_outside_of_run_context() {
        super::draw_circle(0.0, 0.0, 10.0, crate::Color::RED);
    }
}
