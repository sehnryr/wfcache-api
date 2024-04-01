use std::io::Result;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::{Block, Borders, Widget, WidgetRef};

use crate::input::KeyInput;

use super::Button;

#[derive(Debug, Clone)]
pub struct Extract<'a> {
    button_widget: Button<'a>,
}

impl Extract<'_> {
    pub fn new<'a>() -> Self {
        Self {
            button_widget: Button::new(),
        }
    }

    fn compute_layout(&self, area: Rect) -> (Rect, Rect) {
        let export_layout = Layout::horizontal([
            Constraint::Length(15),
            Constraint::Max(30),
            Constraint::Min(0),
        ]);
        let [export_button_area, export_progress_area, _] =
            export_layout.areas(area.inner(&Margin::new(2, 1)));
        (export_button_area, export_progress_area)
    }

    pub fn handle<I: Into<KeyInput>>(&mut self, input: I) -> Result<()> {
        self.button_widget.handle(input.into())?;
        Ok(())
    }
}

impl WidgetRef for Extract<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Extract ".into(),
            "<Space> ".light_blue(),
            "Quit ".into(),
            "<Q> ".light_blue(),
        ]);
        let instructions = Title::from(instructions)
            .alignment(Alignment::Center)
            .position(Position::Bottom);

        Block::default()
            .title(instructions)
            .borders(Borders::ALL)
            .render(area, buf);

        let (export_button_area, _) = self.compute_layout(area);

        self.button_widget.render(export_button_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::assert_buffer_eq;
    use ratatui::style::Style;

    use super::*;

    #[test]
    fn render() {
        let extract = Extract::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 2));

        extract.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┌────────────────────────────────────────────────┐",
            "└─────────── Extract <Space> Quit <Q> ───────────┘",
        ]);
        let extract_style = Style::new().light_blue();
        let quit_style = Style::new().light_blue();
        expected.set_style(Rect::new(21, 1, 8, 1), extract_style);
        expected.set_style(Rect::new(34, 1, 4, 1), quit_style);

        assert_buffer_eq!(buf, expected);
    }
}
