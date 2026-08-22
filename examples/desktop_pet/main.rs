//! Desktop pet: a floating character on a transparent, always-on-top window
//! with no decorations (ADR-0009). The desktop shows through everywhere the
//! engine clears with `Color::TRANSPARENT`.
//!
//! Run from the repo root:
//! `cargo run -p engine --features desktop-pet --example desktop_pet`
use engine::*;

const SPRITE_SIZE: f32 = 48.0;

fn main() {
    let config = WindowConfig {
        title: "pet2d desktop pet".into(),
        width: 192,
        height: 192,
        ..WindowConfig::desktop_pet()
    };

    run(config, || async {
        let cat = match load_texture("assets/textures/sprite.png") {
            Ok(texture) => texture,
            Err(err @ EngineError::Graphics(_)) => panic!("asset problem: {err}"),
        };

        let mut position = Vec2::new(64.0, 64.0);
        let mut velocity = Vec2::new(120.0, 90.0);

        loop {
            // transparent clear: every pixel the sprite does not cover lets
            // the desktop show through
            clear_background(Color::TRANSPARENT);

            // bounce around the window, frame-rate independent
            let delta = get_frame_time();
            let max_x = (screen_width() - SPRITE_SIZE).max(0.0);
            let max_y = (screen_height() - SPRITE_SIZE).max(0.0);
            position += velocity * delta;
            if position.x < 0.0 || position.x > max_x {
                velocity.x = -velocity.x;
                position.x = position.x.clamp(0.0, max_x);
            }
            if position.y < 0.0 || position.y > max_y {
                velocity.y = -velocity.y;
                position.y = position.y.clamp(0.0, max_y);
            }

            draw_texture_ex(
                &cat,
                position.x,
                position.y,
                DrawTextureParams {
                    source: Some(Rect::new(8.0, 8.0, 18.0, 16.0)),
                    dest_size: Some(Vec2::splat(SPRITE_SIZE)),
                    flip_x: velocity.x < 0.0,
                    ..Default::default()
                },
            );

            next_frame().await;
        }
    });
}
