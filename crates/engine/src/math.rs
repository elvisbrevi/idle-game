/// Math types re-exported so the game doesn't import glam directly.
///
/// [`Rect`] is not a glam type — glam has none — so it comes from the sprite
/// system (`graphics`): an axis-aligned pixel rectangle with `x, y, w, h`.
pub use glam::{Mat4, Vec2, Vec3};
pub use graphics::Rect;
