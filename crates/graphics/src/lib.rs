mod batch;
mod color;
mod pipeline;
#[cfg(test)]
mod render_test;
mod renderer;
mod sprite;
mod texture;

pub use color::Color;
pub use renderer::Renderer;
pub use sprite::{DrawTextureParams, Rect};
pub use texture::Texture2D;

#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    #[error("no suitable GPU adapter found")]
    NoAdapter,
    #[error("failed to create graphics device")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("failed to create surface")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),
}
