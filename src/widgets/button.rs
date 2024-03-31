use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::WidgetRef;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Button<'a> {
    default_label: &'a str,
    active_label: Option<&'a str>,
    inactive_label: Option<&'a str>,
    theme: ButtonTheme,
    active: bool,
}

impl<'a> Button<'a> {
    pub fn new(default_label: &'a str) -> Self {
        Self {
            default_label,
            active_label: None,
            inactive_label: None,
            theme: ButtonTheme::default(),
            active: false,
        }
    }

    pub fn active_label(mut self, label: &'a str) -> Self {
        self.active_label = Some(label);
        self
    }

    pub fn inactive_label(mut self, label: &'a str) -> Self {
        self.inactive_label = Some(label);
        self
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    fn label(&self) -> &str {
        if self.active {
            self.active_label.unwrap_or(self.default_label)
        } else {
            self.inactive_label.unwrap_or(self.default_label)
        }
    }
}

impl WidgetRef for Button<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let background_color = self.theme.background;
        let text_color = self.theme.text;
        let (shadow_color, highlight_color) = if self.active {
            (self.theme.highlight, self.theme.shadow)
        } else {
            (self.theme.shadow, self.theme.highlight)
        };

        buf.set_style(area, Style::new().bg(background_color).fg(text_color));

        // render top line if there's enough space
        if area.height > 2 {
            buf.set_string(
                area.x,
                area.y,
                "▔".repeat(area.width as usize),
                Style::new().fg(highlight_color).bg(background_color),
            );
        }
        // render bottom line if there's enough space
        if area.height > 1 {
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                "▁".repeat(area.width as usize),
                Style::new().fg(shadow_color).bg(background_color),
            );
        }
        // render label centered
        let label = Line::from(self.label());
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
        let info = Button::new("Button");
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 3));

        info.render_ref(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔",
            "    Button     ",
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
            let mut button = Button::new("Button");
            button.active = true;
            button
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 3));

        info.render_ref(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔",
            "    Button     ",
            "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
        ]);
        expected.set_style(Rect::new(0, 0, 15, 1), SHADOW_STYLE);
        expected.set_style(Rect::new(0, 1, 15, 1), TEXT_STYLE);
        expected.set_style(Rect::new(0, 2, 15, 1), HIGHLIGHT_STYLE);

        assert_buffer_eq!(buf, expected);
    }
}
