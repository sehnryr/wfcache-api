use std::sync::Arc;

use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, HighlightSpacing};

use super::explorer::FileExplorer;

type LineFactory = Arc<dyn Fn(&FileExplorer) -> Line<'static>>;

#[derive(Clone, derivative::Derivative)]
#[derivative(Debug, PartialEq, Eq, Hash)]
pub struct Theme {
    pub(super) block: Option<Block<'static>>,
    #[derivative(Debug = "ignore", PartialEq = "ignore", Hash = "ignore")]
    title_top: Vec<LineFactory>,
    pub(super) style: Style,
    item_style: Style,
    dir_style: Style,
    pub(super) highlight_spacing: HighlightSpacing,
    pub(super) highlight_item_style: Style,
    pub(super) highlight_dir_style: Style,
    pub(super) highlight_symbol: Option<String>,
}

impl Theme {
    #[inline]
    pub const fn item_style(&self) -> &Style {
        &self.item_style
    }

    #[inline]
    pub const fn dir_style(&self) -> &Style {
        &self.dir_style
    }

    #[inline]
    pub fn title_top(&self, file_explorer: &FileExplorer) -> Vec<Line> {
        self.title_top
            .iter()
            .map(|title_top| title_top(file_explorer))
            .collect()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            block: Some(Block::default().borders(Borders::ALL)),
            title_top: Vec::new(),
            style: Style::default(),
            item_style: Style::default().fg(Color::White),
            dir_style: Style::default().fg(Color::LightBlue),
            highlight_spacing: HighlightSpacing::Always,
            highlight_item_style: Style::default().fg(Color::White).bg(Color::DarkGray),
            highlight_dir_style: Style::default().fg(Color::LightBlue).bg(Color::DarkGray),
            highlight_symbol: None,
        }
    }
}
