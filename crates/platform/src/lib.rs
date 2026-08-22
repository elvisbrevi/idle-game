use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
pub use winit::event::{ElementState, MouseButton as NativeMouseButton};
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
pub use winit::keyboard::KeyCode as NativeKeyCode;
use winit::window::{Window, WindowAttributes};

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
    /// reports. Returns the applied size; `None` on Wayland-like backends
    /// that only confirm asynchronously.
    pub fn set_inner_size(&self, width: u32, height: u32) -> Option<(u32, u32)> {
        let size = self
            .inner
            .request_inner_size(winit::dpi::PhysicalSize::new(width.max(1), height.max(1)))?;
        Some((size.width, size.height))
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
    KeyboardInput(NativeKeyCode, ElementState),
    CursorMoved(f32, f32),
    MouseInput(NativeMouseButton, ElementState),
}

struct App<F: FnMut(FrameEvent)> {
    title: String,
    width: u32,
    height: u32,
    window: Option<Arc<Window>>,
    frame_handler: F,
}

impl<F: FnMut(FrameEvent)> ApplicationHandler for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
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
                (self.frame_handler)(FrameEvent::Redraw);
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

pub fn run(title: &str, width: u32, height: u32, frame_handler: impl FnMut(FrameEvent) + 'static) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        title: title.into(),
        width,
        height,
        window: None,
        frame_handler,
    };
    event_loop.run_app(&mut app).unwrap();
}
