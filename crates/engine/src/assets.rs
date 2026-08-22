use std::collections::HashMap;

use crate::Texture2D;
use crate::context;
use crate::error::EngineError;

/// Cache of loaded assets keyed by path (CONTEXT.md: AssetServer). Generic
/// over the resource type so the caching behavior is testable without a GPU;
/// the engine only ever instantiates it with [`Texture2D`].
pub(crate) struct AssetServer<V: Clone> {
    resources: HashMap<String, V>,
}

impl<V: Clone> Default for AssetServer<V> {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl<V: Clone> AssetServer<V> {
    /// Returns the cached resource for `path`, if it was loaded before.
    pub(crate) fn get(&self, path: &str) -> Option<V> {
        self.resources.get(path).cloned()
    }

    /// Stores a successfully loaded resource under `path`.
    pub(crate) fn insert(&mut self, path: &str, resource: V) {
        self.resources.insert(path.to_string(), resource);
    }
}

/// Loads a texture from an image file (PNG, JPEG, ...) and caches it by path
/// (ADR-0008). Synchronous: blocks until the image is decoded and uploaded to
/// the GPU. Repeated calls with the same path return the cached texture;
/// failed loads are not cached and can be retried.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn load_texture(path: &str) -> Result<Texture2D, EngineError> {
    context::with_mut(|ctx| {
        if let Some(texture) = ctx.assets.get(path) {
            return Ok(texture);
        }
        // destructure to borrow the cache and the renderer disjointly only
        // where needed; a failed load must not insert anything
        let texture = ctx.renderer.load_texture(path)?;
        ctx.assets.insert(path, texture.clone());
        Ok(texture)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caches_each_path_after_first_insert() {
        let mut server = AssetServer::<u32>::default();
        assert_eq!(server.get("cat.png"), None);
        server.insert("cat.png", 7);
        assert_eq!(server.get("cat.png"), Some(7));
        assert_eq!(server.get("cat.png"), Some(7));
    }

    #[test]
    fn unknown_paths_miss_and_inserts_are_visible() {
        let mut server = AssetServer::<u32>::default();
        assert_eq!(server.get("a.png"), None);
        server.insert("a.png", 1);
        server.insert("b.png", 2);
        assert_eq!(server.get("a.png"), Some(1));
        assert_eq!(server.get("b.png"), Some(2));
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn load_texture_panics_outside_of_run_context() {
        let _ = load_texture("definitely/not/here.png");
    }
}
