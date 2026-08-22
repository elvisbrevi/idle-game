#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Space,
    Escape,
    Enter,
    Backspace,
    Tab,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
}

impl KeyCode {
    /// Maps a physical Winit key to its [`KeyCode`]; returns `None` for keys
    /// outside the engine's supported set.
    pub(crate) fn from_native(code: platform::NativeKeyCode) -> Option<Self> {
        use platform::NativeKeyCode as Winit;
        Some(match code {
            Winit::ArrowUp => Self::ArrowUp,
            Winit::ArrowDown => Self::ArrowDown,
            Winit::ArrowLeft => Self::ArrowLeft,
            Winit::ArrowRight => Self::ArrowRight,
            Winit::Space => Self::Space,
            Winit::Escape => Self::Escape,
            Winit::Enter => Self::Enter,
            Winit::Backspace => Self::Backspace,
            Winit::Tab => Self::Tab,
            Winit::KeyA => Self::A,
            Winit::KeyB => Self::B,
            Winit::KeyC => Self::C,
            Winit::KeyD => Self::D,
            Winit::KeyE => Self::E,
            Winit::KeyF => Self::F,
            Winit::KeyG => Self::G,
            Winit::KeyH => Self::H,
            Winit::KeyI => Self::I,
            Winit::KeyJ => Self::J,
            Winit::KeyK => Self::K,
            Winit::KeyL => Self::L,
            Winit::KeyM => Self::M,
            Winit::KeyN => Self::N,
            Winit::KeyO => Self::O,
            Winit::KeyP => Self::P,
            Winit::KeyQ => Self::Q,
            Winit::KeyR => Self::R,
            Winit::KeyS => Self::S,
            Winit::KeyT => Self::T,
            Winit::KeyU => Self::U,
            Winit::KeyV => Self::V,
            Winit::KeyW => Self::W,
            Winit::KeyX => Self::X,
            Winit::KeyY => Self::Y,
            Winit::KeyZ => Self::Z,
            Winit::Digit0 => Self::Key0,
            Winit::Digit1 => Self::Key1,
            Winit::Digit2 => Self::Key2,
            Winit::Digit3 => Self::Key3,
            Winit::Digit4 => Self::Key4,
            Winit::Digit5 => Self::Key5,
            Winit::Digit6 => Self::Key6,
            Winit::Digit7 => Self::Key7,
            Winit::Digit8 => Self::Key8,
            Winit::Digit9 => Self::Key9,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::KeyCode;
    use platform::NativeKeyCode;

    #[test]
    fn maps_arrows_space_and_escape() {
        let cases = [
            (NativeKeyCode::ArrowUp, KeyCode::ArrowUp),
            (NativeKeyCode::ArrowDown, KeyCode::ArrowDown),
            (NativeKeyCode::ArrowLeft, KeyCode::ArrowLeft),
            (NativeKeyCode::ArrowRight, KeyCode::ArrowRight),
            (NativeKeyCode::Space, KeyCode::Space),
            (NativeKeyCode::Escape, KeyCode::Escape),
        ];
        for (native, expected) in cases {
            assert_eq!(KeyCode::from_native(native), Some(expected));
        }
    }

    #[test]
    fn maps_enter_backspace_and_tab() {
        let cases = [
            (NativeKeyCode::Enter, KeyCode::Enter),
            (NativeKeyCode::Backspace, KeyCode::Backspace),
            (NativeKeyCode::Tab, KeyCode::Tab),
        ];
        for (native, expected) in cases {
            assert_eq!(KeyCode::from_native(native), Some(expected));
        }
    }

    #[test]
    fn maps_all_letters_a_to_z() {
        let natives = [
            NativeKeyCode::KeyA,
            NativeKeyCode::KeyB,
            NativeKeyCode::KeyC,
            NativeKeyCode::KeyD,
            NativeKeyCode::KeyE,
            NativeKeyCode::KeyF,
            NativeKeyCode::KeyG,
            NativeKeyCode::KeyH,
            NativeKeyCode::KeyI,
            NativeKeyCode::KeyJ,
            NativeKeyCode::KeyK,
            NativeKeyCode::KeyL,
            NativeKeyCode::KeyM,
            NativeKeyCode::KeyN,
            NativeKeyCode::KeyO,
            NativeKeyCode::KeyP,
            NativeKeyCode::KeyQ,
            NativeKeyCode::KeyR,
            NativeKeyCode::KeyS,
            NativeKeyCode::KeyT,
            NativeKeyCode::KeyU,
            NativeKeyCode::KeyV,
            NativeKeyCode::KeyW,
            NativeKeyCode::KeyX,
            NativeKeyCode::KeyY,
            NativeKeyCode::KeyZ,
        ];
        let letters = [
            KeyCode::A,
            KeyCode::B,
            KeyCode::C,
            KeyCode::D,
            KeyCode::E,
            KeyCode::F,
            KeyCode::G,
            KeyCode::H,
            KeyCode::I,
            KeyCode::J,
            KeyCode::K,
            KeyCode::L,
            KeyCode::M,
            KeyCode::N,
            KeyCode::O,
            KeyCode::P,
            KeyCode::Q,
            KeyCode::R,
            KeyCode::S,
            KeyCode::T,
            KeyCode::U,
            KeyCode::V,
            KeyCode::W,
            KeyCode::X,
            KeyCode::Y,
            KeyCode::Z,
        ];
        for (native, letter) in natives.into_iter().zip(letters) {
            assert_eq!(KeyCode::from_native(native), Some(letter));
        }
    }

    #[test]
    fn maps_all_digits_0_to_9() {
        let natives = [
            NativeKeyCode::Digit0,
            NativeKeyCode::Digit1,
            NativeKeyCode::Digit2,
            NativeKeyCode::Digit3,
            NativeKeyCode::Digit4,
            NativeKeyCode::Digit5,
            NativeKeyCode::Digit6,
            NativeKeyCode::Digit7,
            NativeKeyCode::Digit8,
            NativeKeyCode::Digit9,
        ];
        let digits = [
            KeyCode::Key0,
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];
        for (native, digit) in natives.into_iter().zip(digits) {
            assert_eq!(KeyCode::from_native(native), Some(digit));
        }
    }

    #[test]
    fn unsupported_keys_return_none() {
        assert_eq!(KeyCode::from_native(NativeKeyCode::F1), None);
        assert_eq!(KeyCode::from_native(NativeKeyCode::ShiftLeft), None);
    }
}
