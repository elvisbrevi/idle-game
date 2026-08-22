use crate::sprite::SpriteVertex;

/// One run of consecutive quads sharing a texture key.
#[derive(Debug)]
pub(crate) struct DrawGroup<K> {
    pub(crate) texture: K,
    pub(crate) start: u32,
    pub(crate) count: u32,
}

/// Immediate-mode sprite batch (ADR-0005): accumulates quads as vertices and
/// indices in CPU vectors, grouped per texture so the flush issues one
/// `draw_indexed` per texture run. Submission order is preserved — sprites are
/// never reordered across textures, because that would corrupt painter's
/// algorithm for overlapping transparent sprites.
#[derive(Debug)]
pub(crate) struct SpriteBatch<K: Clone + PartialEq> {
    pub(crate) vertices: Vec<SpriteVertex>,
    pub(crate) indices: Vec<u32>,
    groups: Vec<DrawGroup<K>>,
}

impl<K: Clone + PartialEq> SpriteBatch<K> {
    pub(crate) fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            groups: Vec::new(),
        }
    }

    /// Adds one quad under the given texture key, extending the current group
    /// when the key matches the previous quad.
    pub(crate) fn push_quad(&mut self, texture: K, quad: &[SpriteVertex; 4]) {
        let base = self.vertices.len() as u32;
        match self.groups.last_mut() {
            Some(group) if group.texture == texture => group.count += 6,
            _ => self.groups.push(DrawGroup {
                texture,
                start: self.indices.len() as u32,
                count: 6,
            }),
        }
        self.vertices.extend_from_slice(quad);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub(crate) fn groups(&self) -> &[DrawGroup<K>] {
        &self.groups
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_quad() -> [SpriteVertex; 4] {
        crate::sprite::sprite_quad(
            (8.0, 8.0),
            0.0,
            0.0,
            &crate::sprite::DrawTextureParams::default(),
        )
    }

    #[test]
    fn adjacent_quads_with_the_same_texture_share_one_draw_group() {
        let mut batch = SpriteBatch::<u32>::new();
        batch.push_quad(7, &sample_quad());
        batch.push_quad(7, &sample_quad());

        assert_eq!(batch.groups().len(), 1);
        assert_eq!(batch.vertices.len(), 8);
        assert_eq!(batch.indices.len(), 12);
    }

    #[test]
    fn a_texture_change_starts_a_new_draw_group() {
        let mut batch = SpriteBatch::<u32>::new();
        batch.push_quad(1, &sample_quad());
        batch.push_quad(2, &sample_quad());
        batch.push_quad(2, &sample_quad());

        let groups = batch.groups();
        assert_eq!(groups.len(), 2);
        assert_eq!((groups[0].start, groups[0].count), (0, 6));
        assert_eq!((groups[1].start, groups[1].count), (6, 12));
    }

    #[test]
    fn submission_order_is_preserved_even_when_textures_interleave() {
        let mut batch = SpriteBatch::<u32>::new();
        batch.push_quad(1, &sample_quad());
        batch.push_quad(2, &sample_quad());
        batch.push_quad(1, &sample_quad());

        // interleaved textures stay as three groups: reordering would break
        // painter's-algorithm transparency for overlapping sprites
        let textures: Vec<u32> = batch.groups().iter().map(|g| g.texture).collect();
        assert_eq!(textures, [1, 2, 1]);
    }
}
