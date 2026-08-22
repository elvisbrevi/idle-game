//! Sprite vertices: one textured quad per draw, batched by [`crate::batch`].

use crate::color::Color;
use glam::Vec2;

/// Axis-aligned rectangle in pixels: position plus size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

/// One corner of a textured quad: world-space position plus texture UV.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SpriteVertex {
    pub(crate) position: [f32; 2],
    pub(crate) uv: [f32; 2],
    pub(crate) color: [f32; 4],
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
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 2 * std::mem::size_of::<[f32; 2]>() as u64,
                shader_location: 2,
            },
        ],
    };
}

/// Two triangles per quad, shared by every sprite via `draw_indexed` base vertex.
pub(crate) const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

/// Optional tweaks for [`crate::Renderer::draw_texture_ex`].
///
/// Mirrors the Macroquad-style API: everything defaults to drawing the whole
/// texture at its pixel size, untinted and unrotated. The rotation `pivot` is
/// relative to the top-left corner of the destination rect and defaults to its
/// center.
#[derive(Debug, Clone)]
pub struct DrawTextureParams {
    /// Drawn size in world units; defaults to the source rect size in pixels.
    pub dest_size: Option<Vec2>,
    /// Sub-region of the texture to sample; defaults to the full texture.
    pub source: Option<Rect>,
    /// Rotation in radians around the pivot (clockwise on screen).
    pub rotation: f32,
    /// Rotation center relative to the top-left corner; defaults to center.
    pub pivot: Option<Vec2>,
    /// Mirrors the sprite horizontally.
    pub flip_x: bool,
    /// Mirrors the sprite vertically.
    pub flip_y: bool,
    /// Per-vertex tint multiplied with the sampled texel; defaults to white.
    pub color: Color,
}

impl Default for DrawTextureParams {
    fn default() -> Self {
        Self {
            dest_size: None,
            source: None,
            rotation: 0.0,
            pivot: None,
            flip_x: false,
            flip_y: false,
            color: Color::WHITE,
        }
    }
}

/// Builds the four corners (TL, TR, BR, BL) of a textured quad at `(x, y)`
/// from draw params, applying source-rect UVs, flips, tint and rotation.
pub(crate) fn sprite_quad(
    texture_size: (f32, f32),
    x: f32,
    y: f32,
    params: &DrawTextureParams,
) -> [SpriteVertex; 4] {
    let (tex_w, tex_h) = texture_size;
    let (u0, v0, u1, v1) = match params.source {
        Some(rect) => (
            rect.x / tex_w,
            rect.y / tex_h,
            (rect.x + rect.w) / tex_w,
            (rect.y + rect.h) / tex_h,
        ),
        None => (0.0, 0.0, 1.0, 1.0),
    };
    let (u_left, u_right) = if params.flip_x { (u1, u0) } else { (u0, u1) };
    let (v_top, v_bottom) = if params.flip_y { (v1, v0) } else { (v0, v1) };

    let size = params.dest_size.unwrap_or_else(|| match params.source {
        Some(rect) => Vec2::new(rect.w, rect.h),
        None => Vec2::new(tex_w, tex_h),
    });
    let pivot = params.pivot.unwrap_or_else(|| size * 0.5);
    let (sin, cos) = params.rotation.sin_cos();
    let tint = params.color.to_rgba();

    [
        (0.0, 0.0, u_left, v_top),
        (size.x, 0.0, u_right, v_top),
        (size.x, size.y, u_right, v_bottom),
        (0.0, size.y, u_left, v_bottom),
    ]
    .map(|(cx, cy, u, v)| {
        // rotate the corner around the pivot, then translate to (x, y)
        let dx = cx - pivot.x;
        let dy = cy - pivot.y;
        SpriteVertex {
            position: [
                x + pivot.x + dx * cos - dy * sin,
                y + pivot.y + dx * sin + dy * cos,
            ],
            uv: [u, v],
            color: tint,
        }
    })
}

/// Builds the four corners of a solid-color rectangle with its top-left
/// corner at `(x, y)`, sized `(w, h)` in world units. Shapes sample a shared
/// 1x1 white texture, so the tint alone decides their color.
pub(crate) fn solid_quad(x: f32, y: f32, w: f32, h: f32, color: Color) -> [SpriteVertex; 4] {
    sprite_quad(
        (1.0, 1.0),
        x,
        y,
        &DrawTextureParams {
            dest_size: Some(Vec2::new(w, h)),
            color,
            ..Default::default()
        },
    )
}

/// Builds a triangle fan approximating a filled circle centered at `(x, y)`
/// with `radius` in world units and `segments` rim vertices. Returns fan
/// vertices plus relative indices, ready for [`crate::batch::SpriteBatch`].
pub(crate) fn circle_fan(
    x: f32,
    y: f32,
    radius: f32,
    color: Color,
    segments: u16,
) -> (Vec<SpriteVertex>, Vec<u32>) {
    let tint = color.to_rgba();
    let mut vertices = Vec::with_capacity(segments as usize + 1);
    vertices.push(SpriteVertex {
        position: [x, y],
        uv: [0.5, 0.5],
        color: tint,
    });
    for i in 0..segments {
        let angle = i as f32 * std::f32::consts::TAU / segments as f32;
        vertices.push(SpriteVertex {
            position: [x + radius * angle.cos(), y + radius * angle.sin()],
            uv: [0.5, 0.5],
            color: tint,
        });
    }
    let mut indices: Vec<u32> = Vec::with_capacity(segments as usize * 3);
    for i in 1..=segments {
        indices.extend_from_slice(&[0, i as u32, u32::from(i % segments) + 1]);
    }
    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEX: (f32, f32) = (8.0, 8.0);

    fn quad(params: DrawTextureParams) -> [SpriteVertex; 4] {
        sprite_quad(TEX, 10.0, 20.0, &params)
    }

    fn positions(vertices: &[SpriteVertex; 4]) -> [[f32; 2]; 4] {
        vertices.map(|v| v.position)
    }

    #[test]
    fn default_params_draw_the_whole_texture_at_pixel_size() {
        let vertices = quad(DrawTextureParams::default());
        assert_eq!(
            positions(&vertices),
            [[10.0, 20.0], [18.0, 20.0], [18.0, 28.0], [10.0, 28.0]]
        );
        let uvs: Vec<[f32; 2]> = vertices.map(|v| v.uv).into_iter().collect();
        assert_eq!(uvs, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        assert!(vertices.iter().all(|v| v.color == [1.0, 1.0, 1.0, 1.0]));
    }

    #[test]
    fn dest_size_overrides_the_drawn_size() {
        let params = DrawTextureParams {
            dest_size: Some(Vec2::new(4.0, 16.0)),
            ..Default::default()
        };
        assert_eq!(
            positions(&quad(params)),
            [[10.0, 20.0], [14.0, 20.0], [14.0, 36.0], [10.0, 36.0]]
        );
    }

    #[test]
    fn source_rect_maps_uvs_and_shrinks_the_default_size() {
        let params = DrawTextureParams {
            source: Some(Rect::new(2.0, 2.0, 4.0, 4.0)),
            ..Default::default()
        };
        let vertices = quad(params);
        assert_eq!(
            positions(&vertices),
            [[10.0, 20.0], [14.0, 20.0], [14.0, 24.0], [10.0, 24.0]]
        );
        let uvs: Vec<[f32; 2]> = vertices.map(|v| v.uv).into_iter().collect();
        assert_eq!(
            uvs,
            [[0.25, 0.25], [0.75, 0.25], [0.75, 0.75], [0.25, 0.75]]
        );
    }

    #[test]
    fn flip_x_mirrors_horizontal_uv() {
        let params = DrawTextureParams {
            flip_x: true,
            ..Default::default()
        };
        let uvs: Vec<[f32; 2]> = quad(params).map(|v| v.uv).into_iter().collect();
        assert_eq!(uvs, [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
    }

    #[test]
    fn flip_y_mirrors_vertical_uv() {
        let params = DrawTextureParams {
            flip_y: true,
            ..Default::default()
        };
        let uvs: Vec<[f32; 2]> = quad(params).map(|v| v.uv).into_iter().collect();
        assert_eq!(uvs, [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
    }

    #[test]
    fn rotation_spins_the_quad_clockwise_around_its_center() {
        // quarter turn on an 8x8 quad at (10, 20): corners walk TL -> TR -> BR -> BL
        let params = DrawTextureParams {
            rotation: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        let got = positions(&quad(params));
        let expected = [[18.0, 20.0], [18.0, 28.0], [10.0, 28.0], [10.0, 20.0]];
        for (g, e) in got.iter().zip(expected) {
            assert!(
                (g[0] - e[0]).abs() < 1e-5 && (g[1] - e[1]).abs() < 1e-5,
                "{got:?}"
            );
        }
    }

    #[test]
    fn pivot_moves_the_rotation_center() {
        let params = DrawTextureParams {
            rotation: std::f32::consts::FRAC_PI_2,
            pivot: Some(Vec2::ZERO),
            ..Default::default()
        };
        let got = positions(&quad(params));
        let expected = [[10.0, 20.0], [10.0, 28.0], [2.0, 28.0], [2.0, 20.0]];
        for (g, e) in got.iter().zip(expected) {
            assert!(
                (g[0] - e[0]).abs() < 1e-5 && (g[1] - e[1]).abs() < 1e-5,
                "{got:?}"
            );
        }
    }

    #[test]
    fn color_tints_every_vertex() {
        let params = DrawTextureParams {
            color: Color::RED,
            ..Default::default()
        };
        assert!(quad(params).iter().all(|v| v.color == [1.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn solid_quad_is_a_tinted_rectangle_at_pixel_size() {
        let vertices = solid_quad(10.0, 20.0, 30.0, 40.0, Color::new(0.5, 0.25, 0.0, 1.0));
        assert_eq!(
            positions(&vertices),
            [[10.0, 20.0], [40.0, 20.0], [40.0, 60.0], [10.0, 60.0]]
        );
        // every corner carries the requested color; any uv inside [0, 1]
        // samples the same texel of the shared 1x1 white texture
        assert!(vertices.iter().all(|v| v.color == [0.5, 0.25, 0.0, 1.0]));
        assert!(vertices.iter().all(|v| (0.0..=1.0).contains(&v.uv[0])));
    }

    #[test]
    fn circle_fan_centers_the_fan_on_the_ring() {
        let (vertices, indices) = circle_fan(100.0, 50.0, 8.0, Color::WHITE, 4);

        assert_eq!(vertices.len(), 5);
        assert_eq!(vertices[0].position, [100.0, 50.0]);
        // 4 segments: ring vertices sit at angles 0, 90, 180, 270 degrees
        let ring: Vec<[f32; 2]> = vertices[1..].iter().map(|v| v.position).collect();
        for (angle, expected) in
            ring.iter()
                .zip([[108.0, 50.0], [100.0, 58.0], [92.0, 50.0], [100.0, 42.0]])
        {
            assert!(
                (angle[0] - expected[0]).abs() < 1e-5 && (angle[1] - expected[1]).abs() < 1e-5,
                "{ring:?}"
            );
        }
        assert_eq!(indices, &[0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1]);
    }

    #[test]
    fn circle_fan_indices_close_the_ring_for_any_segment_count() {
        let segments = 48;
        let (_, indices) = circle_fan(0.0, 0.0, 1.0, Color::WHITE, segments);
        assert_eq!(indices.len(), segments as usize * 3);
        // every triangle references the fan center and two adjacent rim
        // vertices, with the last triangle wrapping back to the first
        for (i, tri) in indices.chunks_exact(3).enumerate() {
            assert_eq!(
                tri,
                &[0, i as u32 + 1, (i as u32 + 1) % segments as u32 + 1]
            );
        }
    }

    #[test]
    fn circle_fan_tints_all_vertices() {
        let (vertices, _) = circle_fan(0.0, 0.0, 1.0, Color::BLUE, 6);
        assert!(vertices.iter().all(|v| v.color == [0.0, 0.0, 1.0, 1.0]));
    }
}
