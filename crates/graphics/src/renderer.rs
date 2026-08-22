use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

use wgpu::{
    CompositeAlphaMode, Device, DeviceDescriptor, Instance, InstanceDescriptor, Limits, LoadOp,
    Operations, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceCapabilities, SurfaceConfiguration,
    SurfaceError, SurfaceTarget, TextureUsages, WindowHandle,
};

use crate::pipeline;

pub struct Renderer {
    _instance: Instance,
    _adapter: wgpu::Adapter,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(window: impl WindowHandle + 'static, size: (u32, u32)) -> Result<Self, crate::GraphicsError> {
        let instance = Instance::new(&InstanceDescriptor::default());
        let surface = instance.create_surface(SurfaceTarget::from(window))?;

        let adapter =
            block_on(instance.request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .ok_or(crate::GraphicsError::NoAdapter)?;

        let device_descriptor = DeviceDescriptor {
            label: Some("pet2d-device"),
            required_features: wgpu::Features::empty(),
            required_limits: Limits::default(),
            ..Default::default()
        };
        let (device, queue) = block_on(adapter.request_device(&device_descriptor, None))?;

        let capabilities = surface.get_capabilities(&adapter);
        let config = surface_configuration(&capabilities, size);
        surface.configure(&device, &config);

        let pipeline = pipeline::create_pipeline(&device, config.format);

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            surface,
            config,
            pipeline,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, clear_color: [f32; 4]) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(SurfaceError::OutOfMemory) => {
                panic!("graphics: out of memory acquiring surface frame");
            }
            Err(_) => return,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pet2d-encoder"),
            });

        let clear = wgpu::Color {
            r: clear_color[0] as f64,
            g: clear_color[1] as f64,
            b: clear_color[2] as f64,
            a: clear_color[3] as f64,
        };

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("pet2d-clear"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(clear),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn surface_configuration(
    capabilities: &SurfaceCapabilities,
    size: (u32, u32),
) -> SurfaceConfiguration {
    SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: capabilities.formats[0],
        width: size.0.max(1),
        height: size.1.max(1),
        present_mode: PresentMode::Fifo,
        alpha_mode: choose_alpha_mode(&capabilities.alpha_modes),
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

fn choose_alpha_mode(modes: &[CompositeAlphaMode]) -> CompositeAlphaMode {
    if modes.contains(&CompositeAlphaMode::PreMultiplied) {
        CompositeAlphaMode::PreMultiplied
    } else if modes.contains(&CompositeAlphaMode::Auto) {
        CompositeAlphaMode::Auto
    } else {
        modes[0]
    }
}

// Minimal std-only executor to block on wgpu's async initialization from
// inside the sync event loop. The poll-based frame bridge (ADR-0003)
// lands together with next_frame().
fn block_on<F: Future>(future: F) -> F::Output {
    struct ThreadWaker(Thread);

    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let mut future = pin!(future);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::TextureFormat;

    #[test]
    fn prefers_premultiplied() {
        let modes = [CompositeAlphaMode::Opaque, CompositeAlphaMode::PreMultiplied];
        assert_eq!(choose_alpha_mode(&modes), CompositeAlphaMode::PreMultiplied);
    }

    #[test]
    fn falls_back_to_auto() {
        let modes = [CompositeAlphaMode::Opaque, CompositeAlphaMode::Auto];
        assert_eq!(choose_alpha_mode(&modes), CompositeAlphaMode::Auto);
    }

    #[test]
    fn uses_first_available_as_last_resort() {
        let modes = [CompositeAlphaMode::Inherit, CompositeAlphaMode::Opaque];
        assert_eq!(choose_alpha_mode(&modes), CompositeAlphaMode::Inherit);
    }

    #[test]
    fn configuration_takes_format_from_capabilities() {
        let capabilities = SurfaceCapabilities {
            formats: vec![TextureFormat::Bgra8UnormSrgb],
            alpha_modes: vec![CompositeAlphaMode::Auto],
            present_modes: vec![PresentMode::Fifo],
            usages: TextureUsages::RENDER_ATTACHMENT,
        };
        let config = surface_configuration(&capabilities, (800, 600));
        assert_eq!(config.format, TextureFormat::Bgra8UnormSrgb);
        assert_eq!(config.alpha_mode, CompositeAlphaMode::Auto);
        assert_eq!((config.width, config.height), (800, 600));
    }

    #[test]
    fn configuration_clamps_zero_size() {
        let capabilities = SurfaceCapabilities {
            formats: vec![TextureFormat::Bgra8Unorm],
            alpha_modes: vec![CompositeAlphaMode::PreMultiplied],
            present_modes: vec![PresentMode::Fifo],
            usages: TextureUsages::RENDER_ATTACHMENT,
        };
        let config = surface_configuration(&capabilities, (0, 0));
        assert_eq!((config.width, config.height), (1, 1));
    }
}
