use ratatui::prelude::*;
use ratatui::widgets::{block::*, Borders};

pub struct Explorer {}

impl Explorer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Explorer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .title(" Explorer ")
            .borders(Borders::ALL)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::assert_buffer_eq;

    use super::*;

    #[test]
    fn render() {
        let explorer = Explorer::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 2));

        explorer.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "┌ Explorer ──────────────────────────────────────┐",
            "└────────────────────────────────────────────────┘",
        ]);
        assert_buffer_eq!(buf, expected);
    }
}
