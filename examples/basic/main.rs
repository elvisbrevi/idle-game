use engine::{Color, WindowConfig, clear_background, run};

fn main() {
    let config = WindowConfig::default();
    run(config, || {
        // Clears to a custom color; the engine repaints it on every frame
        // and after resizes. next_frame() arrives in a later ticket.
        clear_background(Color::from_rgb(30, 30, 46));
    });
}
