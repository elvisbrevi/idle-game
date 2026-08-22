use std::collections::HashSet;

use crate::context;
use crate::keycode::KeyCode;
use crate::math::Vec2;
use crate::mouse::MouseButton;
use platform::ElementState;

/// Snapshot of keyboard and mouse state, updated from Winit window events.
/// The game reads it through the free functions ([`is_key_down`],
/// [`mouse_position`], ...); the fields are public for direct inspection.
pub struct InputState {
    /// Keys currently held down.
    pub keys_down: HashSet<KeyCode>,
    /// Keys pressed since the previous frame; cleared by [`Self::begin_frame`].
    pub keys_pressed: HashSet<KeyCode>,
    /// Mouse buttons currently held down.
    pub mouse_buttons_down: HashSet<MouseButton>,
    /// Cursor position in window pixels, origin at the top-left corner.
    pub mouse_position: Vec2,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_buttons_down: HashSet::new(),
            mouse_position: Vec2::ZERO,
        }
    }
}

impl InputState {
    pub(crate) fn on_key(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                // insert() returns false on OS key repeat; pressed stays a
                // transition event, not a per-repeat one
                if self.keys_down.insert(key) {
                    self.keys_pressed.insert(key);
                }
            }
            ElementState::Released => {
                self.keys_down.remove(&key);
                self.keys_pressed.remove(&key);
            }
        }
    }

    pub(crate) fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_buttons_down.insert(button);
            }
            ElementState::Released => {
                self.mouse_buttons_down.remove(&button);
            }
        }
    }

    pub(crate) fn on_cursor_moved(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
    }

    /// Clears one-frame events so [`is_key_pressed`] reports each press
    /// during a single frame. The engine calls this after every rendered frame
    /// (ADR-0003: events accumulate between frames, one game tick consumes them).
    pub(crate) fn begin_frame(&mut self) {
        self.keys_pressed.clear();
    }
}

/// Returns whether `key` is currently held down.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn is_key_down(key: KeyCode) -> bool {
    context::with(|ctx| ctx.input.keys_down.contains(&key))
}

/// Returns whether `key` was pressed since the previous frame; true for
/// exactly one frame per press.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn is_key_pressed(key: KeyCode) -> bool {
    context::with(|ctx| ctx.input.keys_pressed.contains(&key))
}

/// Returns whether `button` is currently held down.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn is_mouse_button_down(button: MouseButton) -> bool {
    context::with(|ctx| ctx.input.mouse_buttons_down.contains(&button))
}

/// Returns the cursor position in window pixels, origin at the top-left corner.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn mouse_position() -> (f32, f32) {
    context::with(|ctx| {
        let position = ctx.input.mouse_position;
        (position.x, position.y)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_key_stays_down_until_released() {
        let mut input = InputState::default();
        input.on_key(KeyCode::ArrowRight, ElementState::Pressed);
        assert!(input.keys_down.contains(&KeyCode::ArrowRight));

        // OS key repeat sends more Pressed events while held; state must not flip
        input.on_key(KeyCode::ArrowRight, ElementState::Pressed);
        assert!(input.keys_down.contains(&KeyCode::ArrowRight));

        input.on_key(KeyCode::ArrowRight, ElementState::Released);
        assert!(!input.keys_down.contains(&KeyCode::ArrowRight));
    }

    #[test]
    fn press_is_a_one_frame_event_cleared_by_begin_frame() {
        let mut input = InputState::default();
        input.on_key(KeyCode::Space, ElementState::Pressed);
        assert!(input.keys_pressed.contains(&KeyCode::Space));

        input.begin_frame();
        assert!(input.keys_pressed.is_empty());
        // keys_down persists while held; only the one-shot event is cleared
        assert!(input.keys_down.contains(&KeyCode::Space));
    }

    #[test]
    fn releasing_a_key_drops_it_from_pressed() {
        let mut input = InputState::default();
        input.on_key(KeyCode::A, ElementState::Pressed);
        input.on_key(KeyCode::A, ElementState::Released);
        assert!(input.keys_pressed.is_empty());
    }

    #[test]
    fn mouse_button_tracks_press_and_release() {
        let mut input = InputState::default();
        input.on_mouse_button(MouseButton::Left, ElementState::Pressed);
        assert!(input.mouse_buttons_down.contains(&MouseButton::Left));
        input.on_mouse_button(MouseButton::Left, ElementState::Released);
        assert!(input.mouse_buttons_down.is_empty());
    }

    #[test]
    fn cursor_moved_updates_position() {
        let mut input = InputState::default();
        input.on_cursor_moved(120.0, 64.0);
        assert_eq!(input.mouse_position, Vec2::new(120.0, 64.0));
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn is_key_down_panics_outside_of_run_context() {
        super::is_key_down(KeyCode::Space);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn is_key_pressed_panics_outside_of_run_context() {
        super::is_key_pressed(KeyCode::Escape);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn is_mouse_button_down_panics_outside_of_run_context() {
        super::is_mouse_button_down(MouseButton::Left);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn mouse_position_panics_outside_of_run_context() {
        super::mouse_position();
    }
}
