use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::Widget;

use super::theme::Theme;

#[derive(Debug, Clone, Copy)]
pub struct Button<'a> {
    pub(super) area: Rect,
    pub(super) label: &'a str,
    theme: Theme,
    pub(super) active: bool,
    pub(super) hover: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            area: Rect::default(),
            label,
            theme: Theme::default(),
            active: false,
            hover: false,
        }
    }

    pub const fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn area(&mut self, area: Rect) {
        self.area = area;
    }

    #[cfg(test)]
    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub const fn is_active(&self) -> bool {
        self.active
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub(super) fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        let mut background_color = theme.background;

        if self.hover {
            background_color = theme.highlight;
        }

        if self.active {
            (background_color, theme.text, theme.highlight, theme.shadow)
        } else {
            (background_color, theme.text, theme.shadow, theme.highlight)
        }
    }

    pub fn render_widget(&self, buf: &mut Buffer) {
        self.render(self.area, buf);
    }
}
