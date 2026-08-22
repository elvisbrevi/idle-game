use std::collections::HashMap;

use crate::Texture2D;
use crate::context;

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
    /// Returns the cached resource for `path`, loading it through `load` on
    /// first access only.
    pub(crate) fn entry(&mut self, path: &str, load: impl FnOnce(&str) -> V) -> V {
        self.resources
            .entry(path.to_string())
            .or_insert_with(|| load(path))
            .clone()
    }
}

/// Loads a texture from an image file (PNG, JPEG, ...) and caches it by path
/// (ADR-0008). Synchronous: blocks until the image is decoded and uploaded to
/// the GPU. Repeated calls with the same path return the cached texture.
///
/// Panics if the file is missing or undecodable, and if called outside of
/// [`crate::run`] (ADR-0001).
pub fn load_texture(path: &str) -> Texture2D {
    context::with_mut(|ctx| {
        // destructure to borrow the cache and the renderer disjointly
        let context::Context {
            assets, renderer, ..
        } = ctx;
        assets.entry(path, |path| renderer.load_texture(path))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn loads_each_path_once_and_caches_it() {
        let loads = Rc::new(Cell::new(0));
        let mut server = AssetServer::<u32>::default();
        let counter = loads.clone();
        let first = server.entry("cat.png", |path| {
            assert_eq!(path, "cat.png");
            counter.set(counter.get() + 1);
            7
        });
        let second = server.entry("cat.png", |_| unreachable!("cache hit must not reload"));
        assert_eq!(first, 7);
        assert_eq!(second, 7);
        assert_eq!(loads.get(), 1);
    }

    #[test]
    fn different_paths_load_separately() {
        let mut server = AssetServer::<u32>::default();
        assert_eq!(server.entry("a.png", |_| 1), 1);
        assert_eq!(server.entry("b.png", |_| 2), 2);
        assert_eq!(server.entry("a.png", |_| unreachable!()), 1);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn load_texture_panics_outside_of_run_context() {
        load_texture("definitely/not/here.png");
    }
}
