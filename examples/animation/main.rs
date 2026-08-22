//! Spritesheet animation example: an 8-frame blob bounce cycle driven by
//! delta time, so playback speed is the same at any framerate. The bottom
//! strip shows every frame of the grid; the big blob plays them in sequence.
//!
//! Run from the repo root: `cargo run -p engine --example animation`
use engine::{
    Animation, Color, DrawTextureParams, SpriteSheet, Vec2, WindowConfig, clear_background,
    draw_texture_ex, get_frame_time, load_texture, next_frame, run,
};

fn main() {
    let config = WindowConfig {
        title: "pet2d animation".into(),
        ..WindowConfig::default()
    };
    run(config, || async {
        // 128x64 sheet sliced into a 4x2 grid of 32x32 frames
        let sheet = SpriteSheet::new(
            load_texture("assets/textures/character_sheet.png").expect("missing sheet asset"),
            4,
            2,
        );
        let mut bounce = Animation::new(8, 10.0, true);

        loop {
            clear_background(Color::from_rgb(30, 30, 46));

            // delta time drives playback: identical speed at any framerate
            bounce.update(get_frame_time());
            let frame_rect = sheet.frame_rect(bounce.current_frame());

            // main character scaled up from its 32x32 frame via `source`
            draw_texture_ex(
                &sheet.texture,
                336.0,
                100.0,
                DrawTextureParams {
                    source: Some(frame_rect),
                    dest_size: Some(Vec2::splat(128.0)),
                    ..Default::default()
                },
            );

            // contact sheet: all 8 frames at native size
            for f in 0..8 {
                draw_texture_ex(
                    &sheet.texture,
                    80.0 + f as f32 * 80.0,
                    400.0,
                    DrawTextureParams {
                        source: Some(sheet.frame_rect(f)),
                        ..Default::default()
                    },
                );
            }

            next_frame().await;
        }
    });
}
