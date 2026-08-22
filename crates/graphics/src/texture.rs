use wgpu::{
    BindGroup, BindGroupLayout, Device, Extent3d, FilterMode, Queue, TexelCopyBufferLayout,
};

/// The shared sampler pair: nearest keeps pixel art crisp, linear smooths
/// scaled sprites. Textures pick one at creation time.
pub(crate) struct Samplers {
    nearest: wgpu::Sampler,
    linear: wgpu::Sampler,
}

impl Samplers {
    pub(crate) fn new(device: &Device) -> Self {
        Self {
            nearest: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("pet2d-sampler-nearest"),
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                ..Default::default()
            }),
            linear: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("pet2d-sampler-linear"),
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..Default::default()
            }),
        }
    }

    pub(crate) fn get(&self, filter: FilterMode) -> &wgpu::Sampler {
        match filter {
            FilterMode::Nearest => &self.nearest,
            FilterMode::Linear => &self.linear,
        }
    }
}

/// Handle to a texture uploaded to the GPU.
///
/// Wraps a [`wgpu::Texture`] and its view plus the bind group used by the
/// sprite pipeline. Internal fields are never exposed to the game.
#[derive(Clone)]
pub struct Texture2D {
    _texture: wgpu::Texture,
    bind_group: BindGroup,
    pub(crate) size: (u32, u32),
}

impl Texture2D {
    /// Uploads RGBA8 pixel data as a new texture sampled through `sampler`,
    /// pre-bound against the sprite pipeline's texture layout (group 1).
    pub(crate) fn from_rgba8(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        sampler: &wgpu::Sampler,
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

/// Decodes an image file from disk into raw RGBA8 pixels (ADR-0008).
pub(crate) fn decode_image_file(path: &str) -> Result<(u32, u32, Vec<u8>), image::ImageError> {
    let img = image::open(path)?.to_rgba8();
    let (width, height) = img.dimensions();
    Ok((width, height, img.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_from_disk_decodes_to_rgba_with_transparency() {
        // 2x1: left opaque green, right fully transparent
        let png = image::RgbaImage::from_raw(2, 1, vec![0, 255, 0, 255, 0, 0, 0, 0]).unwrap();
        let path = std::env::temp_dir().join(format!("pet2d-test-{}.png", std::process::id()));
        png.save(&path).expect("failed to write test PNG");

        let (width, height, rgba) = decode_image_file(path.to_str().unwrap()).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!((width, height), (2, 1));
        assert_eq!(rgba, vec![0, 255, 0, 255, 0, 0, 0, 0]);
    }

    #[test]
    fn missing_file_reports_an_error() {
        let err = decode_image_file("definitely/not/here.png").unwrap_err();
        assert!(matches!(err, image::ImageError::IoError(_)));
    }
}
