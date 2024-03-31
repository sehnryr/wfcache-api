use std::io::Result;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Widget, WidgetRef};

use crate::widgets::button::Button;

#[derive(Debug, Clone)]
pub struct ExtractButton<'a> {
    button_widget: Button<'a>,
}

impl ExtractButton<'_> {
    pub fn new<'a>() -> Self {
        Self {
            button_widget: Button::new("")
                .active_label("Cancel")
                .inactive_label("Extract"),
        }
    }

    fn toggle(&mut self) {
        self.button_widget.toggle();
    }

    pub fn handle(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event.clone())?
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char(' ') => self.toggle(),
            _ => {}
        };
        Ok(())
    }
}

impl WidgetRef for ExtractButton<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.button_widget.render(area, buf);
    }
}
