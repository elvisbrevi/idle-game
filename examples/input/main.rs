//! Input example: steer a sprite with the arrow keys, click to tint it, and
//! read the cursor position.
//!
//! Run from the repo root: `cargo run -p engine --example input`
use engine::{
    Color, KeyCode, MouseButton, WindowConfig, clear_background, draw_texture, is_key_down,
    is_mouse_button_down, load_texture, mouse_position, run,
};

fn main() {
    let config = WindowConfig {
        title: "pet2d input".into(),
        ..WindowConfig::default()
    };
    run(config, || {
        // the game closure runs once until next_frame() lands (ticket 07);
        // this frame reads live input state — hold a key or click before
        // launching to see it take effect on this first drawn frame
        let mut x = 368.0;
        let mut y = 268.0;
        if is_key_down(KeyCode::ArrowLeft) {
            x -= 32.0;
        }
        if is_key_down(KeyCode::ArrowRight) {
            x += 32.0;
        }
        if is_key_down(KeyCode::ArrowUp) {
            y -= 32.0;
        }
        if is_key_down(KeyCode::ArrowDown) {
            y += 32.0;
        }

        clear_background(Color::from_rgb(30, 30, 46));
        let cat = load_texture("assets/textures/sprite.png");
        draw_texture(&cat, x, y);

        // click anywhere: the cursor position prints alongside the press
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            println!("left button down at ({mx}, {my})");
        }
    });
}
