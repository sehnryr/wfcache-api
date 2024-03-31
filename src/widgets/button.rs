use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Widget;

#[derive(Debug, Clone, Copy)]
struct ButtonLabel<'a>(&'a str);

#[derive(Default, Debug, Clone, Copy)]
struct ButtonState {
    active: bool,
    hover: bool,
}

#[derive(Debug, Clone, Copy)]
struct ButtonTheme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            text: Color::Black,
            background: Color::Gray,
            highlight: Color::White,
            shadow: Color::DarkGray,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Button<'a> {
    label: ButtonLabel<'a>,
    theme: ButtonTheme,
    state: ButtonState,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label: ButtonLabel(label),
            theme: ButtonTheme::default(),
            state: ButtonState::default(),
        }
    }

    pub fn set_label(&mut self, label: &'a str) {
        self.label.0 = label;
    }

    pub fn is_active(&self) -> bool {
        self.state.active
    }

    pub fn toggle(&mut self) {
        self.state.active = !self.state.active;
    }

    pub fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        let mut background_color = theme.background;

        if self.state.hover {
            background_color = theme.highlight;
        }

        if self.state.active {
            (background_color, theme.text, theme.highlight, theme.shadow)
        } else {
            (background_color, theme.text, theme.shadow, theme.highlight)
        }
    }
}

impl<'a> Widget for Button<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (background, text, shadow, highlight) = self.colors();
        buf.set_style(area, Style::new().bg(background).fg(text));

        // render top line if there's enough space
        if area.height > 2 {
            buf.set_string(
                area.x,
                area.y,
                "▔".repeat(area.width as usize),
                Style::new().fg(highlight).bg(background),
            );
        }
        // render bottom line if there's enough space
        if area.height > 1 {
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                "▁".repeat(area.width as usize),
                Style::new().fg(shadow).bg(background),
            );
        }
        // render label centered
        let label = Line::from(self.label.0);
        buf.set_line(
            area.x + (area.width.saturating_sub(label.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &label,
            area.width,
        );
    }
}

#[cfg(test)]
mod tests {
    use ratatui::assert_buffer_eq;
    use ratatui::style::Color;

    use super::*;

    const HIGHLIGHT_STYLE: Style = Style::new().fg(Color::White).bg(Color::Gray);
    const TEXT_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Gray);
    const SHADOW_STYLE: Style = Style::new().fg(Color::DarkGray).bg(Color::Gray);

    #[test]
    fn render_default() {
        let info = Button::new("Extract");
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 3));

        info.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔",
            "    Extract    ",
            "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
        ]);
        expected.set_style(Rect::new(0, 0, 15, 1), HIGHLIGHT_STYLE);
        expected.set_style(Rect::new(0, 1, 15, 1), TEXT_STYLE);
        expected.set_style(Rect::new(0, 2, 15, 1), SHADOW_STYLE);

        assert_buffer_eq!(buf, expected);
    }

    #[test]
    fn render_active() {
        let info = {
            let mut button = Button::new("Cancel");
            button.state.active = true;
            button
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 3));

        info.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔",
            "    Cancel     ",
            "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
        ]);
        expected.set_style(Rect::new(0, 0, 15, 1), SHADOW_STYLE);
        expected.set_style(Rect::new(0, 1, 15, 1), TEXT_STYLE);
        expected.set_style(Rect::new(0, 2, 15, 1), HIGHLIGHT_STYLE);

        assert_buffer_eq!(buf, expected);
    }
}
