use ratatui::style::Color;

use super::theme::Theme;

#[derive(Debug, Clone, Copy)]
pub(super) struct ButtonLabel<'a>(pub(super) &'a str);

#[derive(Default, Debug, Clone, Copy)]
pub(super) struct ButtonState {
    pub(super) active: bool,
    pub(super) hover: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Button<'a> {
    pub(super) label: ButtonLabel<'a>,
    theme: Theme,
    pub(super) state: ButtonState,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label: ButtonLabel(label),
            theme: Theme::default(),
            state: ButtonState::default(),
        }
    }

    pub fn set_label(&mut self, label: &'a str) {
        self.label.0 = label;
    }

    pub fn is_active(&self) -> bool {
        self.state.active
    }

    pub fn toggle(&mut self) {
        self.state.active = !self.state.active;
    }

    pub fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        let mut background_color = theme.background;

        if self.state.hover {
            background_color = theme.highlight;
        }

        if self.state.active {
            (background_color, theme.text, theme.highlight, theme.shadow)
        } else {
            (background_color, theme.text, theme.shadow, theme.highlight)
        }
    }
}
