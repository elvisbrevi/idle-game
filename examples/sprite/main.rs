//! Sprite example: loads a transparent PNG and draws it three ways — plain,
//! rotated, and cropped from a source rect with a tint.
//!
//! Run from the repo root: `cargo run -p engine --example sprite`
use engine::{
    Color, DrawTextureParams, Rect, WindowConfig, clear_background, draw_texture, draw_texture_ex,
    load_texture, run,
};

fn main() {
    let config = WindowConfig {
        title: "pet2d sprite".into(),
        ..WindowConfig::default()
    };
    run(config, || {
        // the game closure runs once today; queued draws persist per frame
        // until next_frame() lands
        clear_background(Color::from_rgb(30, 30, 46));
        let cat = load_texture("assets/textures/sprite.png");

        // plain, at native pixel size
        draw_texture(&cat, 80.0, 100.0);

        // scaled up and rotated around its center
        draw_texture_ex(
            &cat,
            260.0,
            100.0,
            DrawTextureParams {
                dest_size: Some(engine::Vec2::new(128.0, 128.0)),
                rotation: std::f32::consts::PI / 6.0,
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
                dest_size: Some(engine::Vec2::new(108.0, 96.0)),
                color: Color::new(0.6, 0.8, 1.0, 1.0),
                flip_x: true,
                ..Default::default()
            },
        );
    });
}
