use std::io::Result;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Widget, WidgetRef};

use crate::widgets::button::Button;

const ACTIVE_LABEL: &'static str = "Cancel";
const INACTIVE_LABEL: &'static str = "Extract";

#[derive(Debug, Clone)]
pub struct ExtractButton<'a> {
    button_widget: Button<'a>,
}

impl ExtractButton<'_> {
    pub fn new<'a>() -> Self {
        #[cfg(not(test))]
        // to avoid the label overlapping the instructions
        return Self {
            button_widget: Button::new(INACTIVE_LABEL),
        };
        #[cfg(test)]
        return Self {
            button_widget: Button::new(""),
        };
    }

    fn toggle(&mut self) {
        self.button_widget.toggle();
        self.button_widget
            .set_label(if self.button_widget.is_active() {
                ACTIVE_LABEL
            } else {
                INACTIVE_LABEL
            });
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
