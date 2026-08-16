use engine::{run, Color, WindowConfig};

fn main() {
    let config = WindowConfig {
        background: Color::from_rgb(30, 30, 46),
        ..Default::default()
    };
    run(config, || {
        // Game loop placeholder — window opens with solid background
        // Ticket 02 adds next_frame() + clear_background()
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
