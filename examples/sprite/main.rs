//! Sprite example: loads a transparent PNG and draws it three ways — plain
//! (steer it with the arrow keys), rotated (spinning over time), and cropped
//! from a source rect with a tint.
//!
//! Run from the repo root: `cargo run -p engine --example sprite`
use engine::{
    Color, DrawTextureParams, KeyCode, Rect, Vec2, WindowConfig, clear_background, draw_texture,
    draw_texture_ex, get_frame_time, is_key_down, load_texture, next_frame, run,
};

fn main() {
    let config = WindowConfig {
        title: "pet2d sprite".into(),
        ..WindowConfig::default()
    };
    run(config, || async {
        // assets load once, synchronously, and stay cached
        let cat = load_texture("assets/textures/sprite.png").expect("missing sprite asset");
        let mut angle = 0.0f32;
        let mut x = 80.0f32;
        let mut y = 100.0f32;

        loop {
            clear_background(Color::from_rgb(30, 30, 46));

            // arrow keys steer the plain sprite, scaled by delta time so the
            // speed is the same at any frame rate
            let delta = get_frame_time();
            if is_key_down(KeyCode::ArrowLeft) {
                x -= 240.0 * delta;
            }
            if is_key_down(KeyCode::ArrowRight) {
                x += 240.0 * delta;
            }
            if is_key_down(KeyCode::ArrowUp) {
                y -= 240.0 * delta;
            }
            if is_key_down(KeyCode::ArrowDown) {
                y += 240.0 * delta;
            }

            // plain, at native pixel size
            draw_texture(&cat, x, y);

            // scaled up, rotating continuously around its center
            angle += get_frame_time();
            draw_texture_ex(
                &cat,
                260.0,
                100.0,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(128.0, 128.0)),
                    rotation: angle,
                    ..Default::default()
                },
            );

            // face crop from the source rect, tinted blue, flipped horizontally
            draw_texture_ex(
                &cat,
                460.0,
                100.0,
                DrawTextureParams {
                    source: Some(Rect::new(8.0, 8.0, 18.0, 16.0)),
                    dest_size: Some(Vec2::new(108.0, 96.0)),
                    color: Color::new(0.6, 0.8, 1.0, 1.0),
                    flip_x: true,
                    ..Default::default()
                },
            );

            next_frame().await;
        }
    });
}
