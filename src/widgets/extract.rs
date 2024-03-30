use ratatui::prelude::*;
use ratatui::widgets::{block::*, Borders};

pub struct Extract {}

impl Extract {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Extract {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = Title::from(Line::from(vec![" Quit ".into(), "<Q> ".bold()]));

        let extract_block = Block::default()
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL);
        extract_block.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::assert_buffer_eq;

    use super::*;

    #[test]
    fn render() {
        let extract = Extract::new();
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
