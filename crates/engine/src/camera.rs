use crate::math::{Mat4, Vec2};

/// Orthographic 2D camera: position, zoom and rotation of the view.
///
/// The default camera maps world coordinates to screen pixels 1:1 with the
/// origin at the top-left corner and `y` growing downwards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: Vec2,
    pub rotation: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: Vec2::ONE,
            rotation: 0.0,
        }
    }
}

impl Camera2D {
    /// Builds the orthographic projection matrix for a window of the given
    /// pixel size.
    pub fn projection_matrix(&self, screen_width: f32, screen_height: f32) -> Mat4 {
        let scale_x = self.zoom.x * 2.0 / screen_width;
        let scale_y = self.zoom.y * 2.0 / screen_height;
        let (sin, cos) = self.rotation.sin_cos();

        // column-major columns. World space is y-down (origin top-left);
        // clip space is y-up on every backend wgpu supports, so the y axis
        // flips here once — sprites stay upright without per-draw fixes.
        Mat4::from_cols_array_2d(&[
            [scale_x * cos, -scale_y * sin, 0.0, 0.0],
            [-scale_x * sin, -scale_y * cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                -scale_x * (cos * self.position.x - sin * self.position.y) - 1.0,
                scale_y * (sin * self.position.x + cos * self.position.y) + 1.0,
                0.0,
                1.0,
            ],
        ])
    }
}

/// Sets the active camera used to render the scene.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn set_camera(camera: Camera2D) {
    crate::context::with_mut(|ctx| ctx.camera = camera);
}

/// Restores the default camera, which maps screen pixels 1:1.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn set_default_camera() {
    crate::context::with_mut(|ctx| ctx.camera = Camera2D::default());
}

#[cfg(test)]
mod tests {
    use super::Camera2D;
    use crate::math::Vec2;

    fn project(camera: &Camera2D, screen: (f32, f32), point: (f32, f32)) -> (f32, f32) {
        let p = camera
            .projection_matrix(screen.0, screen.1)
            .project_point3(Vec2::from(point).extend(0.0));
        (p.x, p.y)
    }

    #[test]
    fn default_camera_maps_screen_pixels_one_to_one() {
        let camera = Camera2D::default();
        // world origin is the top-left corner; clip space y is up
        assert_eq!(project(&camera, (800.0, 600.0), (0.0, 0.0)), (-1.0, 1.0));
        assert_eq!(
            project(&camera, (800.0, 600.0), (800.0, 600.0)),
            (1.0, -1.0)
        );
        assert_eq!(project(&camera, (800.0, 600.0), (400.0, 300.0)), (0.0, 0.0));
    }

    #[test]
    fn default_camera_matches_any_window_size() {
        let camera = Camera2D::default();
        assert_eq!(project(&camera, (1920.0, 1080.0), (0.0, 0.0)), (-1.0, 1.0));
        assert_eq!(
            project(&camera, (1920.0, 1080.0), (960.0, 540.0)),
            (0.0, 0.0)
        );
    }

    #[test]
    fn zoom_scales_world_coordinates() {
        let camera = Camera2D {
            zoom: Vec2::new(2.0, 2.0),
            ..Default::default()
        };
        assert_eq!(
            project(&camera, (800.0, 600.0), (400.0, 300.0)),
            (1.0, -1.0)
        );
    }

    #[test]
    fn position_recenters_the_view() {
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            ..Default::default()
        };
        assert_eq!(
            project(&camera, (800.0, 600.0), (400.0, 300.0)),
            (-1.0, 1.0)
        );
        // the visible window shifted right by half a screen width
        assert_eq!(project(&camera, (800.0, 600.0), (800.0, 300.0)), (0.0, 1.0));
        assert_eq!(
            project(&camera, (800.0, 600.0), (1200.0, 300.0)),
            (1.0, 1.0)
        );
    }

    #[test]
    fn rotation_rotates_the_view() {
        let camera = Camera2D {
            rotation: std::f32::consts::FRAC_PI_2,
            ..Default::default()
        };
        // +x world rotates onto +y world (down on screen); half screen height
        // maps to NDC 0
        let (x, y) = project(&camera, (800.0, 600.0), (300.0, 0.0));
        assert!((x - (-1.0)).abs() < 1e-6, "x was {x}");
        assert!(y.abs() < 1e-6, "y was {y}");
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn set_camera_panics_outside_of_run_context() {
        super::set_camera(Camera2D::default());
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn set_default_camera_panics_outside_of_run_context() {
        super::set_default_camera();
    }
}
