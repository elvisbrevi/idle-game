//! Winit plumbing for the engine: window creation, event loop and input
//! forwarding. The game never sees this crate (ADR-0002); the engine calls
//! [`run`] with a frame handler and talks to the window via [`WindowHandle`].
//!
//! Platform-specific quirks live in cfg modules per ADR-0007: most behavior
//! comes from Winit itself, so only genuine per-SO differences get a module.

mod common;

#[cfg(all(target_os = "macos", feature = "desktop-pet"))]
mod macos;
#[cfg(all(target_os = "windows", feature = "desktop-pet"))]
mod windows;

pub use common::{FrameEvent, WindowHandle, run};
pub use winit::event::{ElementState, MouseButton as NativeMouseButton};
pub use winit::keyboard::KeyCode as NativeKeyCode;

/// Desktop pet window options, accepted by [`run`] only with the
/// `desktop-pet` feature (ADR-0009).
#[cfg(feature = "desktop-pet")]
pub use common::PetWindowOptions;
