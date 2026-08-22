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
    #[error("failed to load texture '{path}'")]
    TextureLoad {
        path: String,
        #[source]
        source: image::ImageError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_load_error_names_the_path() {
        let source =
            image::ImageError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        let err = GraphicsError::TextureLoad {
            path: "cat.png".into(),
            source,
        };
        let message = err.to_string();
        assert!(message.contains("cat.png"), "{message}");
        // the underlying cause stays reachable through the error chain
        assert!(std::error::Error::source(&err).is_some());
    }
}
