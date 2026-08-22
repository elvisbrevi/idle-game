//! macOS quirks for desktop pet windows (ADR-0007). Winit resolves most
//! behavior; only what it does not cover lives here.

use winit::platform::macos::WindowAttributesExtMacOS;
use winit::window::WindowAttributes;

/// macOS composites an NSWindow drop shadow around the content bounds; on a
/// transparent pet it shows as a rectangular halo around the sprite, so pets
/// ship without a shadow.
pub(crate) fn refine_pet_attributes(attrs: WindowAttributes) -> WindowAttributes {
    attrs.with_has_shadow(false)
}
