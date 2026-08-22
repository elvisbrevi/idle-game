/// One corner of a textured quad: world-space position plus texture UV.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SpriteVertex {
    pub(crate) position: [f32; 2],
    pub(crate) uv: [f32; 2],
}

impl SpriteVertex {
    pub(crate) const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SpriteVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: std::mem::size_of::<[f32; 2]>() as u64,
                shader_location: 1,
            },
        ],
    };
}

/// Two triangles per quad, shared by every sprite via `draw_indexed` base vertex.
pub(crate) const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

/// Corners of the destination rect, clockwise from top-left, with full-texture UVs.
pub(crate) fn quad_vertices(x: f32, y: f32, width: f32, height: f32) -> [SpriteVertex; 4] {
    [
        SpriteVertex { position: [x, y], uv: [0.0, 0.0] },
        SpriteVertex { position: [x + width, y], uv: [1.0, 0.0] },
        SpriteVertex { position: [x + width, y + height], uv: [1.0, 1.0] },
        SpriteVertex { position: [x, y + height], uv: [0.0, 1.0] },
    ]
}

#[cfg(test)]
mod tests {
    use super::QUAD_INDICES;

    #[test]
    fn index_buffer_is_the_canonical_quad_pattern() {
        assert_eq!(QUAD_INDICES, [0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn quad_vertices_cover_the_destination_rect() {
        let vertices = super::quad_vertices(10.0, 20.0, 30.0, 40.0);
        let positions: Vec<[f32; 2]> = vertices.map(|v| v.position).into_iter().collect();
        assert_eq!(
            positions,
            [[10.0, 20.0], [40.0, 20.0], [40.0, 60.0], [10.0, 60.0]]
        );
    }

    #[test]
    fn quad_vertices_map_the_full_texture() {
        let vertices = super::quad_vertices(0.0, 0.0, 8.0, 8.0);
        let uvs: Vec<[f32; 2]> = vertices.map(|v| v.uv).into_iter().collect();
        assert_eq!(uvs, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    }
}
