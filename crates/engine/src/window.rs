use crate::color::Color;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub background: Color,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "pet2d".into(),
            width: 800,
            height: 600,
            background: Color::BLACK,
        }
    }
}
