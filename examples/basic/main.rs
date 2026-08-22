//! Basic example: opens a window and clears it every frame.
//!
//! Run from the repo root: `cargo run -p engine --example basic`
use engine::{Color, WindowConfig, clear_background, next_frame, run};

fn main() {
    let config = WindowConfig::default();
    run(config, || async {
        loop {
            // the engine fills each presented frame with this color behind
            // everything drawn on top
            clear_background(Color::from_rgb(30, 30, 46));
            next_frame().await;
        }
    });
}
