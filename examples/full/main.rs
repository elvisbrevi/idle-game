//! Full API tour: every public engine function working together — window
//! control, shapes, sprites, input, time, camera, math types and the frame
//! loop.
//!
//! Run from the repo root: `cargo run -p engine --example full`
use engine::*;

fn main() {
    let config = WindowConfig {
        title: "pet2d full api".into(),
        ..WindowConfig::default()
    };

    run(config, || async {
        // assets: synchronous load, cached by path; failures are typed
        let cat = match load_texture("assets/textures/sprite.png") {
            Ok(texture) => texture,
            Err(err @ EngineError::Graphics(_)) => panic!("asset problem: {err}"),
        };

        // window control: place and size the OS window once at startup
        set_window_position(120.0, 90.0);
        set_window_size(640.0, 400.0);

        // math types re-exported so games never import glam directly
        let point: Vec3 = Vec2::new(40.0, 20.0).extend(0.0);
        let _projection_like: Mat4 = Mat4::IDENTITY;
        let crop = Rect::new(8.0, 8.0, 18.0, 16.0);
        assert_eq!(point.z, 0.0);

        let mut angle: f32 = 0.0;
        let mut frames: u64 = 0;

        loop {
            // time: delta time keeps motion frame-rate independent
            angle += get_frame_time();

            // camera pans with the arrow keys
            let pan = 160.0 * get_frame_time();
            let mut camera = Camera2D::default();
            if is_key_down(KeyCode::ArrowRight) {
                camera.position.x -= pan;
            }
            if is_key_down(KeyCode::ArrowLeft) {
                camera.position.x += pan;
            }
            if is_key_down(KeyCode::ArrowUp) {
                camera.position.y += pan;
            }
            if is_key_down(KeyCode::ArrowDown) {
                camera.position.y -= pan;
            }
            set_camera(camera);

            clear_background(Color::from_rgb(30, 30, 46));

            // shapes: a ground strip and a circle pulsing with elapsed time
            let w = screen_width();
            let h = screen_height();
            draw_rectangle(0.0, h - 48.0, w, 48.0, Color::from_rgb(49, 50, 68));
            let radius = 36.0 + 12.0 * get_time().sin() as f32;
            draw_circle(w / 2.0, h / 2.0, radius, Color::from_rgb(166, 227, 161));

            // sprite riding the circle's rim, cropped + tinted + flipped
            draw_texture_ex(
                &cat,
                w / 2.0 + angle.cos() * 96.0 - 32.0,
                h / 2.0 + angle.sin() * 96.0 - 32.0,
                DrawTextureParams {
                    source: Some(crop),
                    dest_size: Some(Vec2::splat(64.0)),
                    color: if is_mouse_button_down(MouseButton::Left) {
                        Color::from_rgb(243, 139, 168)
                    } else {
                        Color::WHITE
                    },
                    flip_x: angle.sin() < 0.0,
                    ..Default::default()
                },
            );

            // HUD back on the default pixel-perfect camera
            set_default_camera();
            draw_rectangle(12.0, 12.0, 150.0, 28.0, Color::new(0.07, 0.07, 0.11, 0.78));
            frames += 1;
            if frames.is_multiple_of(60) {
                println!(
                    "frame {frames} | {:.1} fps | cursor {:?}",
                    get_fps(),
                    mouse_position()
                );
            }

            // present everything queued above and yield to the event loop
            next_frame().await;
        }
    });
}
