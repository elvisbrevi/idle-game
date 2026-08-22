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

use crate::batch::SpriteBatch;
use crate::pipeline;
use crate::sprite::{DrawTextureParams, QUAD_INDICES, SpriteVertex, sprite_quad};
use crate::texture::{Samplers, Texture2D, decode_image_file};

const CAMERA_UNIFORM_SIZE: u64 = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
const VERTEX_BYTES: u64 = std::mem::size_of::<SpriteVertex>() as u64;
const INDEX_BYTES: u64 = std::mem::size_of::<u32>() as u64;

/// Recreates `buffer` at double capacity when `needed_bytes` no longer fits.
fn grow_buffer(
    device: &Device,
    buffer: &wgpu::Buffer,
    capacity_bytes: u64,
    needed_bytes: u64,
    label: &str,
    usage: wgpu::BufferUsages,
) -> (wgpu::Buffer, u64) {
    if needed_bytes <= capacity_bytes {
        return (buffer.clone(), capacity_bytes);
    }
    let size = needed_bytes.max(capacity_bytes * 2);
    let grown = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    });
    (grown, size)
}

pub struct Renderer {
    _instance: Instance,
    _adapter: wgpu::Adapter,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    texture_layout: wgpu::BindGroupLayout,
    samplers: Samplers,
    camera_uniform: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity_bytes: u64,
    index_buffer: wgpu::Buffer,
    index_capacity_bytes: u64,
    // NOTE: deliberate simplification — the batch persists across redraws
    // because the game closure runs once today; it resets per frame once
    // next_frame() lands and drives a real loop
    batch: SpriteBatch<wgpu::BindGroup>,
    camera_matrix: [[f32; 4]; 4],
}

impl Renderer {
    pub fn new(
        window: impl WindowHandle + 'static,
        size: (u32, u32),
    ) -> Result<Self, crate::GraphicsError> {
        let instance = Instance::new(&InstanceDescriptor::default());
        let surface = instance.create_surface(SurfaceTarget::from(window))?;

        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
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
        let texture_layout = pipeline.get_bind_group_layout(pipeline::TEXTURE_GROUP);
        let samplers = Samplers::new(&device);

        let camera_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pet2d-camera-uniform"),
            size: CAMERA_UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pet2d-camera-bind-group"),
            layout: &pipeline.get_bind_group_layout(pipeline::CAMERA_GROUP),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform.as_entire_binding(),
            }],
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pet2d-index-buffer"),
            size: std::mem::size_of_val(&QUAD_INDICES) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&index_buffer, 0, bytemuck::bytes_of(&QUAD_INDICES));

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pet2d-vertex-buffer"),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            surface,
            config,
            pipeline,
            texture_layout,
            samplers,
            camera_uniform,
            camera_bind_group,
            vertex_buffer,
            vertex_capacity_bytes: 0,
            index_buffer,
            index_capacity_bytes: std::mem::size_of_val(&QUAD_INDICES) as u64,
            batch: SpriteBatch::new(),
            camera_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        })
    }

    /// Uploads RGBA8 data as a new GPU texture bound to the sprite pipeline,
    /// sampled with nearest filtering (pixel-art default).
    pub fn create_texture(&self, width: u32, height: u32, rgba: &[u8]) -> Texture2D {
        self.create_texture_filtered(wgpu::FilterMode::Nearest, width, height, rgba)
    }

    /// Like [`Renderer::create_texture`] with an explicit filter mode.
    pub fn create_texture_filtered(
        &self,
        filter: wgpu::FilterMode,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Texture2D {
        Texture2D::from_rgba8(
            &self.device,
            &self.queue,
            &self.texture_layout,
            self.samplers.get(filter),
            width,
            height,
            rgba,
        )
    }

    /// Decodes an image file from disk and uploads it as a GPU texture
    /// (ADR-0008). Synchronous; panics if the file cannot be loaded.
    pub fn load_texture(&self, path: &str) -> Texture2D {
        let (width, height, rgba) = decode_image_file(path);
        self.create_texture(width, height, &rgba)
    }

    /// Queues a textured quad with its top-left corner at (x, y), sized to the
    /// texture dimensions in pixels.
    pub fn draw_texture(&mut self, texture: &Texture2D, x: f32, y: f32) {
        self.draw_texture_ex(texture, x, y, DrawTextureParams::default());
    }

    /// Queues a textured quad with per-draw tweaks: source rect, destination
    /// size, rotation, pivot, flips and tint.
    pub fn draw_texture_ex(
        &mut self,
        texture: &Texture2D,
        x: f32,
        y: f32,
        params: DrawTextureParams,
    ) {
        let (width, height) = texture.size();
        let vertices = sprite_quad((width as f32, height as f32), x, y, &params);
        self.batch
            .push_quad(texture.bind_group().clone(), &vertices);
    }

    /// Sets the projection applied to every queued quad on the next render.
    ///
    /// The matrix is column-major (`Mat4::to_cols_array_2d`).
    pub fn set_camera(&mut self, projection: [[f32; 4]; 4]) {
        self.camera_matrix = projection;
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
            render_pass.set_bind_group(pipeline::CAMERA_GROUP, &self.camera_bind_group, &[]);
            self.queue.write_buffer(
                &self.camera_uniform,
                0,
                bytemuck::bytes_of(&self.camera_matrix),
            );

            if !self.batch.is_empty() {
                // ADR-0005: the batch uploads once per render, one write_buffer
                // per buffer, then one draw_indexed per texture run
                let (vertex_buffer, vertex_capacity) = grow_buffer(
                    &self.device,
                    &self.vertex_buffer,
                    self.vertex_capacity_bytes,
                    self.batch.vertices.len() as u64 * VERTEX_BYTES,
                    "pet2d-vertex-buffer",
                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                );
                self.vertex_buffer = vertex_buffer;
                self.vertex_capacity_bytes = vertex_capacity;
                self.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.batch.vertices),
                );

                let (index_buffer, index_capacity) = grow_buffer(
                    &self.device,
                    &self.index_buffer,
                    self.index_capacity_bytes,
                    self.batch.indices.len() as u64 * INDEX_BYTES,
                    "pet2d-index-buffer",
                    wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                );
                self.index_buffer = index_buffer;
                self.index_capacity_bytes = index_capacity;
                self.queue.write_buffer(
                    &self.index_buffer,
                    0,
                    bytemuck::cast_slice(&self.batch.indices),
                );

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                // one draw call per texture group
                for group in self.batch.groups() {
                    render_pass.set_bind_group(pipeline::TEXTURE_GROUP, &group.texture, &[]);
                    render_pass.draw_indexed(group.start..group.start + group.count, 0, 0..1);
                }
            }
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
        let modes = [
            CompositeAlphaMode::Opaque,
            CompositeAlphaMode::PreMultiplied,
        ];
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
