//! Input example: steer a sprite with the arrow keys, click to tint it, and
//! read the cursor position.
//!
//! Run from the repo root: `cargo run -p engine --example input`
use engine::{
    Color, KeyCode, MouseButton, WindowConfig, clear_background, draw_texture, get_frame_time,
    is_key_down, is_mouse_button_down, load_texture, mouse_position, next_frame, run,
};

fn main() {
    let config = WindowConfig {
        title: "pet2d input".into(),
        ..WindowConfig::default()
    };
    run(config, || async {
        let cat = load_texture("assets/textures/sprite.png").expect("missing sprite asset");
        let mut x = 368.0;
        let mut y = 268.0;

        loop {
            // delta time keeps movement identical at any frame rate
            let speed = 200.0 * get_frame_time();
            if is_key_down(KeyCode::ArrowLeft) {
                x -= speed;
            }
            if is_key_down(KeyCode::ArrowRight) {
                x += speed;
            }
            if is_key_down(KeyCode::ArrowUp) {
                y -= speed;
            }
            if is_key_down(KeyCode::ArrowDown) {
                y += speed;
            }

            clear_background(Color::from_rgb(30, 30, 46));
            draw_texture(&cat, x, y);

            // click anywhere: the cursor position prints alongside the press
            if is_mouse_button_down(MouseButton::Left) {
                let (mx, my) = mouse_position();
                println!("left button down at ({mx}, {my})");
            }

            next_frame().await;
        }
    });
}
