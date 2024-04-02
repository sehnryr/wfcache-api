use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Widget, WidgetRef};

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

    pub fn set_active(&mut self, active: bool) {
        self.button_widget.set_active(active);
    }
}

impl WidgetRef for Button<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.button_widget.render(area, buf);
    }
}
