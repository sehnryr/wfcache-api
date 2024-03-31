use std::io::Result;

use crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Widget;

use super::theme::Theme;

#[derive(Debug, Clone, Copy)]
pub struct Button<'a> {
    area: Rect,
    label: &'a str,
    theme: Theme,
    active: bool,
    hover: bool,
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

    const fn colors(&self) -> (Color, Color, Color, Color) {
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

    pub fn handle(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent) -> Result<()> {
        match mouse_event.kind {
            MouseEventKind::Moved => {
                self.hover = self
                    .area
                    .contains(Position::new(mouse_event.column, mouse_event.row));
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self
                    .area
                    .contains(Position::new(mouse_event.column, mouse_event.row))
                {
                    self.active = !self.active;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn render_widget(&self, buf: &mut Buffer) {
        self.render(self.area, buf);
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
        let label = Line::from(self.label);
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
        let info = Button::new("Cancel").active(true);
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
