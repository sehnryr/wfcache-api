use std::io::Result;

use crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use super::button::Button;

impl Button<'_> {
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
                self.state.hover = self
                    .area
                    .contains(Position::new(mouse_event.column, mouse_event.row));
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self
                    .area
                    .contains(Position::new(mouse_event.column, mouse_event.row))
                {
                    self.toggle();
                }
            }
            _ => {}
        }
        Ok(())
    }
}
