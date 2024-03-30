use std::io::Result;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::buffer::Buffer;
use ratatui::layout::{self, Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::{Block, Borders, Widget};

use crate::widgets::button::Button;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Extract {
    active: bool,
    hover: bool,
    area: Rect,
}

impl Extract {
    pub fn new() -> Result<Self> {
        Ok(Self {
            active: false,
            hover: false,
            area: Rect::default(),
        })
    }

    pub fn area(&mut self, area: Rect) {
        self.area = area;
    }

    fn compute_layout(&self) -> [Rect; 1] {
        let export_layout = Layout::horizontal([Constraint::Length(15), Constraint::Min(0)]);
        let [export_button_area, _] = export_layout.areas(self.area.inner(&Margin::new(2, 1)));
        [export_button_area]
    }

    pub fn handle(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event.clone())?
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char(' ') => self.active = !self.active,
            _ => {}
        };
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent) -> Result<()> {
        match mouse_event.kind {
            MouseEventKind::Moved => {
                let [export_button_area] = self.compute_layout();
                self.hover = export_button_area
                    .contains(layout::Position::new(mouse_event.column, mouse_event.row));
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let [export_button_area] = self.compute_layout();
                if export_button_area
                    .contains(layout::Position::new(mouse_event.column, mouse_event.row))
                {
                    self.active = !self.active;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for Extract {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".bold()]);
        let instructions = Title::from(instructions)
            .alignment(Alignment::Center)
            .position(Position::Bottom);

        let [export_button_area] = self.compute_layout();

        Block::default()
            .title(instructions)
            .borders(Borders::ALL)
            .render(area, buf);
        Button::new(if self.active { "Cancel" } else { "Extract" })
            .active(self.active)
            .hover(self.hover)
            .render(export_button_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::assert_buffer_eq;
    use ratatui::style::Style;

    use super::*;

    #[test]
    fn render() {
        let extract = Extract::new().unwrap();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 2));

        extract.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┌────────────────────────────────────────────────┐",
            "└─────────────────── Quit <Q> ───────────────────┘",
        ]);
        let quit_style = Style::new().bold();
        expected.set_style(Rect::new(26, 1, 4, 1), quit_style);

        assert_buffer_eq!(buf, expected);
    }
}
