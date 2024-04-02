use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Gauge as GaugeWidget, Widget, WidgetRef};

#[derive(Debug, Clone)]
pub struct Gauge {
    ratio: f64,
    label: String,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            ratio: 0.0,
            label: String::from("0/0"),
        }
    }

    pub fn set_progress(&mut self, count: usize, total: usize) {
        assert!(count <= total);
        if total == 0 {
            self.ratio = 0.0;
            self.label = String::from("0/0");
            return;
        }
        self.ratio = count as f64 / total as f64;
        self.label = format!("{}/{}", count, total);
    }
}

impl WidgetRef for Gauge {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default(); // .style(Style::default().bg(Color::DarkGray));
        GaugeWidget::default()
            .block(block)
            .gauge_style(Style::default().fg(Color::Gray).bg(Color::DarkGray))
            .ratio(self.ratio)
            .label(&self.label)
            .use_unicode(true)
            .render(area, buf);
    }
}
