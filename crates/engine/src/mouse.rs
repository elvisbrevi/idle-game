#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    /// Maps a Winit mouse button; returns `None` for buttons the engine does
    /// not model (back, forward, extra buttons).
    pub(crate) fn from_native(button: platform::NativeMouseButton) -> Option<Self> {
        Some(match button {
            platform::NativeMouseButton::Left => Self::Left,
            platform::NativeMouseButton::Right => Self::Right,
            platform::NativeMouseButton::Middle => Self::Middle,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::MouseButton;
    use platform::NativeMouseButton;

    #[test]
    fn maps_left_right_and_middle() {
        let cases = [
            (NativeMouseButton::Left, MouseButton::Left),
            (NativeMouseButton::Right, MouseButton::Right),
            (NativeMouseButton::Middle, MouseButton::Middle),
        ];
        for (native, expected) in cases {
            assert_eq!(MouseButton::from_native(native), Some(expected));
        }
    }

    #[test]
    fn unsupported_buttons_return_none() {
        assert_eq!(MouseButton::from_native(NativeMouseButton::Back), None);
        assert_eq!(MouseButton::from_native(NativeMouseButton::Other(4)), None);
    }
}
