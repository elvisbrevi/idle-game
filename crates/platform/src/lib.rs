use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
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
}

impl HasWindowHandle for WindowHandle {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
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
}

struct App<F: FnMut(FrameEvent)> {
    title: String,
    width: u32,
    height: u32,
    window: Option<Arc<Window>>,
    game_fn: Option<Box<dyn FnOnce()>>,
    game_started: bool,
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
                if !self.game_started {
                    self.game_started = true;
                    if let Some(f) = self.game_fn.take() {
                        f();
                    }
                }
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

pub fn run(
    title: &str,
    width: u32,
    height: u32,
    game_fn: impl FnOnce() + 'static,
    frame_handler: impl FnMut(FrameEvent) + 'static,
) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        title: title.into(),
        width,
        height,
        window: None,
        game_fn: Some(Box::new(game_fn)),
        game_started: false,
        frame_handler,
    };
    event_loop.run_app(&mut app).unwrap();
}
