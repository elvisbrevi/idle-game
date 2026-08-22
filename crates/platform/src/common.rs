use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

/// Options describing a desktop pet window; mirrors what the OS layer needs
/// to build a floating character (ADR-0009).
#[cfg(feature = "desktop-pet")]
#[derive(Clone, Copy, Debug)]
pub struct PetWindowOptions {
    pub always_on_top: bool,
    pub transparent: bool,
    pub click_through: bool,
    pub decorations: bool,
}

#[derive(Clone)]
pub struct WindowHandle {
    inner: Arc<Window>,
}

impl WindowHandle {
    pub(crate) fn new(inner: Arc<Window>) -> Self {
        Self { inner }
    }

    pub fn inner_size(&self) -> (u32, u32) {
        let size = self.inner.inner_size();
        (size.width, size.height)
    }

    /// Resizes the window in physical pixels, matching what `inner_size()`
    /// reports. Applied synchronously on macOS and Windows; Wayland-like
    /// backends (out of scope) only confirm via a later resize event.
    pub fn set_inner_size(&self, width: u32, height: u32) {
        let applied = self
            .inner
            .request_inner_size(winit::dpi::PhysicalSize::new(width.max(1), height.max(1)));
        // backends that apply immediately report the same size back
        debug_assert!(applied.is_none_or(|s| (s.width, s.height) == (width.max(1), height.max(1))));
    }

    /// Moves the window's top-left corner to physical screen coordinates.
    pub fn set_outer_position(&self, x: f32, y: f32) {
        self.inner
            .set_outer_position(winit::dpi::PhysicalPosition::new(
                f64::from(x),
                f64::from(y),
            ));
    }
}

impl HasWindowHandle for WindowHandle {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.inner.window_handle()
    }
}

impl HasDisplayHandle for WindowHandle {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.inner.display_handle()
    }
}

pub enum FrameEvent {
    WindowReady(WindowHandle, u32, u32),
    Resized(u32, u32),
    Redraw,
    KeyboardInput(crate::NativeKeyCode, crate::ElementState),
    CursorMoved(f32, f32),
    MouseInput(crate::NativeMouseButton, crate::ElementState),
}

struct App<F: FnMut(FrameEvent) -> bool> {
    title: String,
    width: u32,
    height: u32,
    #[cfg(feature = "desktop-pet")]
    pet: Option<PetWindowOptions>,
    window: Option<Arc<Window>>,
    /// Frame sink; returns false to end the event loop (the engine uses this
    /// when the game closure completes).
    frame_handler: F,
}

impl<F: FnMut(FrameEvent) -> bool> ApplicationHandler for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Self::build_attributes(self);
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        #[cfg(feature = "desktop-pet")]
        if self.pet.is_some_and(|pet| pet.click_through)
            && window.set_cursor_hittest(false).is_err()
        {
            // mouse events should fall through to the windows behind the pet
            eprintln!("pet2d: click-through unsupported on this platform");
        }
        let size = window.inner_size();
        (self.frame_handler)(FrameEvent::WindowReady(
            WindowHandle::new(window.clone()),
            size.width,
            size.height,
        ));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if !(self.frame_handler)(FrameEvent::Redraw) {
                    event_loop.exit();
                    return;
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                (self.frame_handler)(FrameEvent::Resized(size.width, size.height));
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // identified physical keys only; unnamed keys have no mapping
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    (self.frame_handler)(FrameEvent::KeyboardInput(code, event.state));
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                (self.frame_handler)(FrameEvent::CursorMoved(
                    position.x as f32,
                    position.y as f32,
                ));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                (self.frame_handler)(FrameEvent::MouseInput(button, state));
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {}
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}
    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {}
}

impl<F: FnMut(FrameEvent) -> bool> App<F> {
    /// Applies the base window attributes; without the desktop-pet feature
    /// every window is a normal application window.
    #[cfg(not(feature = "desktop-pet"))]
    fn build_attributes(&self) -> WindowAttributes {
        WindowAttributes::default()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height))
    }

    /// Applies the desktop pet settings to the base window attributes
    /// (transparent, borderless, topmost) plus the platform refinements of
    /// [`crate::macos`] / [`crate::windows`] (ADR-0007).
    #[cfg(feature = "desktop-pet")]
    fn build_attributes(&self) -> WindowAttributes {
        let attrs = WindowAttributes::default()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));
        let Some(pet) = self.pet else {
            return attrs;
        };
        let attrs = attrs
            .with_transparent(pet.transparent)
            .with_decorations(pet.decorations)
            .with_window_level(if pet.always_on_top {
                winit::window::WindowLevel::AlwaysOnTop
            } else {
                winit::window::WindowLevel::Normal
            });
        refine_pet_attributes(attrs)
    }
}

/// Per-SO tweaks Winit does not resolve on its own; no-op elsewhere.
#[cfg(all(feature = "desktop-pet", target_os = "macos"))]
fn refine_pet_attributes(attrs: WindowAttributes) -> WindowAttributes {
    crate::macos::refine_pet_attributes(attrs)
}

#[cfg(all(feature = "desktop-pet", target_os = "windows"))]
fn refine_pet_attributes(attrs: WindowAttributes) -> WindowAttributes {
    crate::windows::refine_pet_attributes(attrs)
}

#[cfg(all(
    feature = "desktop-pet",
    not(any(target_os = "macos", target_os = "windows"))
))]
fn refine_pet_attributes(attrs: WindowAttributes) -> WindowAttributes {
    attrs
}

/// Creates the window and runs the Winit event loop until it ends: either
/// the frame handler returns `false` (the engine signals this when the game
/// closure completes) or the user closes the window. Must be called from the
/// main thread on macOS.
pub fn run(
    title: &str,
    width: u32,
    height: u32,
    #[cfg(feature = "desktop-pet")] pet: Option<PetWindowOptions>,
    frame_handler: impl FnMut(FrameEvent) -> bool + 'static,
) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        title: title.into(),
        width,
        height,
        #[cfg(feature = "desktop-pet")]
        pet,
        window: None,
        frame_handler,
    };
    event_loop.run_app(&mut app).unwrap();
}
