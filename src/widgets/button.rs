use ratatui::prelude::*;

#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: Line<'a>,
    theme: Theme,
    state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Normal,
    Active,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
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

impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Self {
            label: label.into(),
            theme: Theme::default(),
            state: State::Normal,
        }
    }

    pub const fn state(mut self, state: State) -> Self {
        self.state = state;
        self
    }

    const fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            State::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            State::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
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
        buf.set_line(
            area.x + (area.width.saturating_sub(self.label.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &self.label,
            area.width,
        );
    }
}

#[cfg(test)]
mod tests {
    use ratatui::assert_buffer_eq;

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
        let info = Button::new("Cancel").state(State::Active);
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
