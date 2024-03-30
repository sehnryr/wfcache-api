use ratatui::prelude::*;
use ratatui::widgets::{block::*, Borders};

pub struct Info {}

impl Info {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Info {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .title(" Info ")
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
        let info = Info::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 2));

        info.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "┌ Info ──────────────────────────────────────────┐",
            "└────────────────────────────────────────────────┘",
        ]);
        assert_buffer_eq!(buf, expected);
    }
}
