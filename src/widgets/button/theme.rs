use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub(super) text: Color,
    pub(super) background: Color,
    pub(super) highlight: Color,
    pub(super) shadow: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            text: Color::Black,
            background: Color::Gray,
            highlight: Color::White,
            shadow: Color::DarkGray,
        }
    }
}
