use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Span, Text};
use ratatui::widgets::{List, ListState, WidgetRef};

use super::explorer::{File, FileExplorer};
use super::theme::Theme;

pub struct Renderer<'a>(pub(crate) &'a FileExplorer);

impl WidgetRef for Renderer<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut state = ListState::default().with_selected(Some(self.0.selected_idx()));

        let highlight_style = if self.0.current().is_dir() {
            self.0.theme().highlight_dir_style
        } else {
            self.0.theme().highlight_item_style
        };

        let mut list = List::new(self.0.files().iter().map(|file| file.text(self.0.theme())))
            .style(self.0.theme().style)
            .highlight_spacing(self.0.theme().highlight_spacing.clone())
            .highlight_style(highlight_style);

        if let Some(symbol) = self.0.theme().highlight_symbol.as_deref() {
            list = list.highlight_symbol(symbol);
        }

        if let Some(block) = self.0.theme().block.as_ref() {
            let mut block = block.clone();

            for title_top in self.0.theme().title_top(self.0) {
                block = block.title_top(title_top)
            }
            for title_bottom in self.0.theme().title_bottom(self.0) {
                block = block.title_bottom(title_bottom)
            }

            list = list.block(block);
        }

        ratatui::widgets::StatefulWidgetRef::render_ref(&list, area, buf, &mut state)
    }
}

impl File {
    fn text(&self, theme: &Theme) -> Text<'_> {
        let style = if self.is_dir() {
            *theme.dir_style()
        } else {
            *theme.item_style()
        };
        Span::styled(self.name().to_owned(), style).into()
    }
}
