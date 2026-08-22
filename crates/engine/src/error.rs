use graphics::GraphicsError;

/// Typed error for engine operations that can fail (ADR-0004). The variants
/// mirror the underlying subsystems: texture loading surfaces the path and
/// decode cause; GPU/surface failures come through [`GraphicsError`].
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// A graphics-level failure: GPU adapter/device init, surface creation
    /// or a texture load naming the offending path.
    #[error(transparent)]
    Graphics(#[from] GraphicsError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_graphics_errors_transparently() {
        let err = EngineError::from(GraphicsError::NoAdapter);
        // #[error(transparent)]: the message comes straight from GraphicsError
        assert!(err.to_string().contains("GPU adapter"), "{err}");
    }
}
