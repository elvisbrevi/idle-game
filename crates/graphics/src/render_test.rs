//! Headless GPU integration test: renders a textured quad to an offscreen
//! target and reads back the pixels. Proves the WGSL shaders, vertex/index
//! buffers, camera uniform, texture binding and alpha blending work without
//! opening a window.

#![cfg(test)]

use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

use wgpu::{
    Adapter, Device, DeviceDescriptor, Extent3d, Instance, InstanceDescriptor, LoadOp, MapMode,
    Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions,
    StoreOp, TexelCopyBufferLayout, TextureFormat, TextureUsages,
};

use crate::pipeline::{self, CAMERA_GROUP, TEXTURE_GROUP};
use crate::sprite::{DrawTextureParams, QUAD_INDICES, sprite_quad};
use crate::texture::{Samplers, Texture2D};

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

fn headless_device() -> (Device, Queue) {
    let instance = Instance::new(&InstanceDescriptor::default());
    let adapter: Adapter = block_on(instance.request_adapter(&RequestAdapterOptions::default()))
        .expect("no GPU adapter available");
    block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("pet2d-headless-test"),
            ..Default::default()
        },
        None,
    ))
    .expect("failed to request device")
}

/// Panics on any wgpu validation error instead of swallowing it.
fn trap_errors(device: &Device) {
    device.on_uncaptured_error(Box::new(|err| panic!("wgpu error: {err}")));
}

/// Default Camera2D projection over a 64x64 target, hand-computed as an
/// independent source of truth: scale 2/64 = 0.03125 per axis (y negated —
/// clip space is y-up), translation -1 on x and +1 on y.
const CAMERA_64PX: [[f32; 4]; 4] = [
    [0.03125, 0.0, 0.0, 0.0],
    [0.0, -0.03125, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0, 1.0],
];

fn pixel(data: &[u8], x: usize, y: usize) -> [u8; 4] {
    let offset = (y * 256) + x * 4;
    data[offset..offset + 4].try_into().unwrap()
}

#[test]
fn textured_quad_renders_with_alpha_blending() {
    let (device, queue) = headless_device();

    let pipeline = pipeline::create_pipeline(&device, TextureFormat::Rgba8Unorm);
    let camera_layout = pipeline.get_bind_group_layout(CAMERA_GROUP);
    let texture_layout = pipeline.get_bind_group_layout(TEXTURE_GROUP);

    // 2x1 texture: left texel opaque green, right texel fully transparent
    let samplers = Samplers::new(&device);
    let texture = Texture2D::from_rgba8(
        &device,
        &queue,
        &texture_layout,
        samplers.get(wgpu::FilterMode::Nearest),
        2,
        1,
        &[0, 255, 0, 255, 0, 0, 0, 0],
    );

    let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-camera"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&camera_buf, 0, bytemuck::bytes_of(&CAMERA_64PX));
    let camera_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pet2d-test-camera-group"),
        layout: &camera_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buf.as_entire_binding(),
        }],
    });

    // quad covering pixels 16..48 on both axes of the 64x64 target
    let vertices = sprite_quad(
        (2.0, 1.0),
        16.0,
        16.0,
        &DrawTextureParams {
            dest_size: Some(glam::Vec2::new(32.0, 32.0)),
            ..Default::default()
        },
    );
    let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-vertices"),
        size: std::mem::size_of_val(&vertices) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buf, 0, bytemuck::bytes_of(&vertices));
    let index_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-indices"),
        size: std::mem::size_of_val(&QUAD_INDICES) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&index_buf, 0, bytemuck::bytes_of(&QUAD_INDICES));

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pet2d-test-target"),
        size: Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-readback"),
        size: 256u64 * 64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("pet2d-test-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::BLUE),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(CAMERA_GROUP, &camera_group, &[]);
        pass.set_bind_group(TEXTURE_GROUP, texture.bind_group(), &[]);
        pass.set_vertex_buffer(0, vertex_buf.slice(..));
        pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: None,
            },
        },
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    block_on(async {
        slice.map_async(MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);
    });
    let data = slice.get_mapped_range().to_vec();
    staging.unmap();

    // opaque texel wins over the blue background inside the left half
    assert_eq!(pixel(&data, 20, 32), [0, 255, 0, 255]);
    // fully transparent texel lets the blue background through (alpha blending)
    assert_eq!(pixel(&data, 44, 32), [0, 0, 255, 255]);
    // outside the quad the clear color survives untouched
    assert_eq!(pixel(&data, 4, 4), [0, 0, 255, 255]);
}

/// End-to-end: a PNG with transparency on disk is decoded, uploaded, drawn
/// twice through the SpriteBatch (same texture, one merged draw group) and
/// alpha-blended over the clear color. The second sprite carries a red tint
/// that turns the green texel black, proving the vertex tint reaches the GPU.
#[test]
fn png_with_transparency_draws_through_sprite_batch() {
    let (device, queue) = headless_device();
    trap_errors(&device);

    let pipeline = pipeline::create_pipeline(&device, TextureFormat::Rgba8Unorm);
    let camera_layout = pipeline.get_bind_group_layout(CAMERA_GROUP);
    let texture_layout = pipeline.get_bind_group_layout(TEXTURE_GROUP);

    // 2x1 PNG on disk: left texel opaque green, right fully transparent
    let path = std::env::temp_dir().join(format!("pet2d-render-{}.png", std::process::id()));
    image::RgbaImage::from_raw(2, 1, vec![0, 255, 0, 255, 0, 0, 0, 0])
        .unwrap()
        .save(&path)
        .expect("failed to write test PNG");
    let (width, height, rgba) = crate::texture::decode_image_file(path.to_str().unwrap()).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!((width, height), (2, 1));

    let samplers = Samplers::new(&device);
    let texture = Texture2D::from_rgba8(
        &device,
        &queue,
        &texture_layout,
        samplers.get(wgpu::FilterMode::Nearest),
        width,
        height,
        &rgba,
    );

    let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-camera"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&camera_buf, 0, bytemuck::bytes_of(&CAMERA_64PX));
    let camera_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pet2d-test-camera-group"),
        layout: &camera_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buf.as_entire_binding(),
        }],
    });

    // two sprites sharing the texture: SpriteBatch must merge them into one
    // draw group. Untinted at 16..48; red-tinted at 0..16.
    let mut batch = crate::batch::SpriteBatch::new();
    batch.push_quad(
        texture.bind_group().clone(),
        &sprite_quad(
            (2.0, 1.0),
            16.0,
            16.0,
            &DrawTextureParams {
                dest_size: Some(glam::Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
        ),
    );
    batch.push_quad(
        texture.bind_group().clone(),
        &sprite_quad(
            (2.0, 1.0),
            0.0,
            0.0,
            &DrawTextureParams {
                dest_size: Some(glam::Vec2::new(16.0, 16.0)),
                color: crate::color::Color::RED,
                ..Default::default()
            },
        ),
    );
    assert_eq!(batch.groups().len(), 1);

    let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-vertices"),
        size: batch.vertices.len() as u64
            * std::mem::size_of::<crate::sprite::SpriteVertex>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buf, 0, bytemuck::cast_slice(&batch.vertices));
    let index_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-indices"),
        size: batch.indices.len() as u64 * 4,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&index_buf, 0, bytemuck::cast_slice(&batch.indices));

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pet2d-test-target"),
        size: Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pet2d-test-readback"),
        size: 256u64 * 64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("pet2d-test-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::BLUE),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(CAMERA_GROUP, &camera_group, &[]);
        pass.set_vertex_buffer(0, vertex_buf.slice(..));
        pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint32);
        for group in batch.groups() {
            pass.set_bind_group(TEXTURE_GROUP, &group.texture, &[]);
            pass.draw_indexed(group.start..group.start + group.count, 0, 0..1);
        }
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: None,
            },
        },
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    block_on(async {
        slice.map_async(MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);
    });
    let data = slice.get_mapped_range().to_vec();
    staging.unmap();

    // untinted sprite: opaque green texel wins over blue, transparent half doesn't
    assert_eq!(pixel(&data, 20, 32), [0, 255, 0, 255]);
    assert_eq!(pixel(&data, 44, 32), [0, 0, 255, 255]);
    // tinted sprite: green texel * red tint = black over blue background
    assert_eq!(pixel(&data, 4, 4), [0, 0, 0, 255]);
    assert_eq!(pixel(&data, 12, 4), [0, 0, 255, 255]);
}
