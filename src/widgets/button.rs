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
pub struct Button {
    default_label: String,
    active_label: Option<String>,
    inactive_label: Option<String>,
    theme: ButtonTheme,
    active: bool,
}

impl Button {
    pub fn new(default_label: &str) -> Self {
        Self {
            default_label: default_label.to_string(),
            active_label: None,
            inactive_label: None,
            theme: ButtonTheme::default(),
            active: false,
        }
    }

    pub fn active_label(mut self, label: &str) -> Self {
        self.active_label = Some(label.to_string());
        self
    }

    pub fn inactive_label(mut self, label: &str) -> Self {
        self.inactive_label = Some(label.to_string());
        self
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    fn label(&self) -> &String {
        if self.active {
            self.active_label.as_ref().unwrap_or(&self.default_label)
        } else {
            self.inactive_label.as_ref().unwrap_or(&self.default_label)
        }
    }
}

impl WidgetRef for Button {
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
        let label = Line::from(self.label().as_str());
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

    #[test]
    fn render_small() {
        let info = Button::new("Button");
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 2));

        info.render_ref(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec!["    Button     ", "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁"]);
        expected.set_style(Rect::new(0, 0, 15, 1), TEXT_STYLE);
        expected.set_style(Rect::new(0, 1, 15, 1), SHADOW_STYLE);

        assert_buffer_eq!(buf, expected);
    }

    #[test]
    fn render_smaller() {
        let info = Button::new("Button");
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 1));

        info.render_ref(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec!["    Button     "]);
        expected.set_style(Rect::new(0, 0, 15, 1), TEXT_STYLE);

        assert_buffer_eq!(buf, expected);
    }

    #[test]
    fn render_custom_labels() {
        let mut info = Button::new("Button")
            .active_label("Active")
            .inactive_label("Inactive");
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 1));

        // render default label (inactive)
        info.render_ref(buf.area, &mut buf);

        let expected = {
            let mut expected = Buffer::with_lines(vec!["   Inactive    "]);
            expected.set_style(Rect::new(0, 0, 15, 1), TEXT_STYLE);
            expected
        };
        assert_buffer_eq!(buf, expected);

        // render active label
        buf.reset();
        info.set_active(true);
        info.render_ref(buf.area, &mut buf);

        let expected = {
            let mut expected = Buffer::with_lines(vec!["    Active     "]);
            expected.set_style(Rect::new(0, 0, 15, 1), TEXT_STYLE);
            expected
        };
        assert_buffer_eq!(buf, expected);
    }
}
