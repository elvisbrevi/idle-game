//! Windows quirks for desktop pet windows (ADR-0007). Winit resolves most
//! behavior; only what it does not cover lives here.

use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::WindowAttributes;

/// A pet must not own a taskbar button, and undecorated Windows windows keep
/// drawing a DWM shadow unless it is explicitly disabled (same halo problem
/// as macOS around transparent content).
pub(crate) fn refine_pet_attributes(attrs: WindowAttributes) -> WindowAttributes {
    attrs.with_skip_taskbar(true).with_undecorated_shadow(false)
}
