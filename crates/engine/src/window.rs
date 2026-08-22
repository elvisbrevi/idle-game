use crate::color::Color;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub background: Color,
    /// Desktop pet options; only present with the `desktop-pet` feature
    /// (ADR-0009). `None` keeps the window a normal application window.
    #[cfg(feature = "desktop-pet")]
    pub pet: Option<DesktopPetConfig>,
}

/// Options describing a desktop pet window (ADR-0009): a transparent,
/// decoration-free, always-visible floating character.
#[cfg(feature = "desktop-pet")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopPetConfig {
    /// Keeps the window above every normal window.
    pub always_on_top: bool,
    /// Lets the desktop show through wherever nothing is drawn.
    pub transparent: bool,
    /// Passes mouse events through to the windows behind the pet.
    pub click_through: bool,
    /// Draws OS title bar and borders (off for pets).
    pub decorations: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "pet2d".into(),
            width: 800,
            height: 600,
            background: Color::BLACK,
            #[cfg(feature = "desktop-pet")]
            pet: None,
        }
    }
}

impl WindowConfig {
    /// Config for a floating character on the desktop: transparent, no
    /// decorations, always on top. Click-through stays opt-in via
    /// [`DesktopPetConfig::click_through`] (ADR-0009).
    ///
    /// Only available with the `desktop-pet` feature.
    #[cfg(feature = "desktop-pet")]
    pub fn desktop_pet() -> Self {
        Self {
            background: Color::TRANSPARENT,
            pet: Some(DesktopPetConfig {
                always_on_top: true,
                transparent: true,
                click_through: false,
                decorations: false,
            }),
            ..Self::default()
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

    #[cfg(feature = "desktop-pet")]
    #[test]
    fn desktop_pet_configures_transparent_topmost_borderless_window() {
        let config = super::WindowConfig::desktop_pet();
        let pet = config.pet.expect("desktop_pet sets the pet options");
        assert!(pet.transparent);
        assert!(!pet.decorations);
        assert!(pet.always_on_top);
        // click-through is opt-in, not part of the default pet window
        assert!(!pet.click_through);
        // a floating pet starts with a see-through clear color
        assert_eq!(config.background, crate::Color::TRANSPARENT);
    }
}
