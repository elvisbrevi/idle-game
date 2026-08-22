use wgpu::{BindGroup, BindGroupLayout, Device, Extent3d, TexelCopyBufferLayout, Queue, Sampler};

/// Handle to a texture uploaded to the GPU.
///
/// Wraps a [`wgpu::Texture`] and its view plus the bind group used by the
/// sprite pipeline. Internal fields are never exposed to the game.
pub struct Texture2D {
    _texture: wgpu::Texture,
    bind_group: BindGroup,
    pub(crate) size: (u32, u32),
}

impl Texture2D {
    /// Uploads RGBA8 pixel data as a new texture and pre-binds it against the
    /// sprite pipeline's texture layout (group 1).
    pub fn from_rgba8(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        sampler: &Sampler,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Self {
        debug_assert_eq!(rgba.len(), (width * height * 4) as usize);
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pet2d-texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pet2d-texture-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });
        Self {
            _texture: texture,
            bind_group,
            size: (width, height),
        }
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub(crate) fn size(&self) -> (u32, u32) {
        self.size
    }
}
