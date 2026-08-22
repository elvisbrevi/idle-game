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
        self.push_vertices(texture, quad, &[0, 1, 2, 0, 2, 3]);
    }

    /// Adds an indexed vertex run (e.g. a circle fan) under the given texture
    /// key; indices are relative to `vertices` and rebased onto the batch.
    pub(crate) fn push_vertices(&mut self, texture: K, vertices: &[SpriteVertex], indices: &[u32]) {
        let count = indices.len() as u32;
        let base = self.vertices.len() as u32;
        match self.groups.last_mut() {
            Some(group) if group.texture == texture => group.count += count,
            _ => self.groups.push(DrawGroup {
                texture,
                start: self.indices.len() as u32,
                count,
            }),
        }
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|&i| base + i));
    }

    pub(crate) fn groups(&self) -> &[DrawGroup<K>] {
        &self.groups
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Drops every queued primitive so the next frame starts empty.
    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.groups.clear();
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

    #[test]
    fn push_vertices_rebases_indices_onto_the_batch() {
        let mut batch = SpriteBatch::new();
        batch.push_quad(7, &sample_quad());
        batch.push_vertices(
            7,
            &[
                sample_quad()[0],
                sample_quad()[1],
                sample_quad()[2],
                sample_quad()[3],
                sample_quad()[0],
            ],
            &[0, 1, 2, 0, 2, 3, 0, 3, 4],
        );

        // the fan continues the same texture group, with indices shifted by
        // the four vertices already queued for the quad
        assert_eq!(batch.groups().len(), 1);
        assert_eq!(batch.groups()[0].count, 15);
        assert_eq!(&batch.indices[6..], &[4, 5, 6, 4, 6, 7, 4, 7, 8]);
    }

    #[test]
    fn clear_resets_vertices_indices_and_groups() {
        let mut batch = SpriteBatch::<u32>::new();
        batch.push_quad(7, &sample_quad());
        batch.clear();

        assert!(batch.is_empty());
        assert!(batch.groups().is_empty());
        assert_eq!(batch.indices.len(), 0);

        // after a clear the next push starts from zero again
        batch.push_quad(7, &sample_quad());
        assert_eq!(batch.groups().len(), 1);
        assert_eq!((batch.groups()[0].start, batch.groups()[0].count), (0, 6));
    }
}
