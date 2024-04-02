use std::io::Result;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Widget, WidgetRef};

use crate::action::Action;
use crate::widgets::button::Button as ButtonWidget;

#[derive(Debug, Clone)]
pub struct Button<'a> {
    button_widget: ButtonWidget<'a>,
}

impl Button<'_> {
    pub fn new<'a>() -> Self {
        Self {
            button_widget: ButtonWidget::new("")
                .active_label("Cancel")
                .inactive_label("Extract"),
        }
    }

    fn toggle(&mut self) {
        self.button_widget.toggle();
    }

    pub fn handle(&mut self, action: &Action) -> Result<()> {
        match action {
            Action::ExtractToggle => self.toggle(),
            _ => {}
        }
        Ok(())
    }
}

impl WidgetRef for Button<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.button_widget.render(area, buf);
    }
}
