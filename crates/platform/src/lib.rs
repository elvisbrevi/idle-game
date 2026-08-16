use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

struct App {
    title: String,
    width: u32,
    height: u32,
    window: Option<Arc<Window>>,
    game_fn: Option<Box<dyn FnOnce()>>,
    game_started: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));
        self.window = Some(Arc::new(event_loop.create_window(attrs).unwrap()));
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
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::Resized(_size) => {
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
) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        title: title.into(),
        width,
        height,
        window: None,
        game_fn: Some(Box::new(game_fn)),
        game_started: false,
    };
    event_loop.run_app(&mut app).unwrap();
}
