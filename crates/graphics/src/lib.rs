mod pipeline;
mod renderer;

pub use renderer::Renderer;

#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    #[error("no suitable GPU adapter found")]
    NoAdapter,
    #[error("failed to create graphics device")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("failed to create surface")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),
}
