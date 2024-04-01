use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::rc::Rc;

use derivative::Derivative;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::toc::{DirectoryNode, Node, NodeKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Span, Text};
use ratatui::widgets::{List, ListState, WidgetRef};

use crate::input::KeyInput;

use super::theme::Theme;

#[derive(Clone, Derivative)]
#[derivative(Debug, PartialEq, Eq, Hash)]
pub struct Explorer<'a> {
    cwd: PathBuf,
    #[derivative(Debug = "ignore", PartialEq = "ignore", Hash = "ignore")]
    h_cache: Rc<&'a CachePairReader>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    nodes: Vec<Node>,
    selected: usize,
    theme: Theme,
}

impl<'a> Explorer<'a> {
    pub fn new(h_cache: Rc<&'a CachePairReader>) -> Result<Explorer<'a>> {
        let mut file_explorer = Self {
            cwd: PathBuf::from("/"),
            h_cache,
            nodes: vec![],
            selected: 0,
            theme: Theme::default(),
        };

        file_explorer.get_and_set_files()?;

        Ok(file_explorer)
    }

    pub fn handle<I: Into<KeyInput>>(&mut self, input: I) -> Result<()> {
        match input.into() {
            KeyInput::Up => {
                if self.selected == 0 {
                    self.selected = self.nodes.len() - 1;
                } else {
                    self.selected -= 1;
                }
            }
            KeyInput::Down => {
                if self.selected == self.nodes.len() - 1 {
                    self.selected = 0;
                } else {
                    self.selected += 1;
                }
            }
            KeyInput::Left => {
                let parent = self.cwd.parent();

                if let Some(parent) = parent {
                    self.cwd = parent.to_path_buf();
                    self.get_and_set_files()?;
                    self.selected = 0
                }
            }
            KeyInput::Right => {
                if self.nodes[self.selected].kind() == NodeKind::Directory {
                    self.cwd = self.nodes.swap_remove(self.selected).path();
                    self.get_and_set_files()?;
                    self.selected = 0
                }
            }
            _ => {}
        }

        Ok(())
    }

    #[inline]
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    #[inline]
    pub fn current(&self) -> &Node {
        &self.nodes[self.selected]
    }

    #[inline]
    pub const fn files(&self) -> &Vec<Node> {
        &self.nodes
    }

    #[inline]
    pub const fn selected_idx(&self) -> usize {
        self.selected
    }

    #[inline]
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    fn get_and_set_files(&mut self) -> Result<()> {
        let current_directory = self
            .h_cache
            .get_directory_node(&self.cwd)
            .ok_or(Error::new(ErrorKind::NotFound, "Directory not found"))?;

        let mut directories = Vec::new();
        let mut files = Vec::new();

        current_directory.children().iter().for_each(|node| {
            if node.kind() == NodeKind::Directory {
                directories.push(node.clone());
            } else {
                files.push(node.clone());
            }
        });

        directories.sort_by(|a, b| a.name().cmp(&b.name()));
        files.sort_by(|a, b| a.name().cmp(&b.name()));

        if let Some(_parent) = self.cwd.parent() {
            let mut nodes = Vec::with_capacity(1 + directories.len() + files.len());

            let parent_node = current_directory.parent().ok_or(Error::new(
                ErrorKind::NotFound,
                "Parent directory not found",
            ))?;

            nodes.push(parent_node);

            nodes.extend(directories);
            nodes.extend(files);

            self.nodes = nodes;
        } else {
            let mut nodes = Vec::with_capacity(directories.len() + files.len());

            nodes.extend(directories);
            nodes.extend(files);

            self.nodes = nodes;
        }

        Ok(())
    }
}

impl WidgetRef for Explorer<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut state = ListState::default().with_selected(Some(self.selected_idx()));

        let highlight_style = if self.current().kind() == NodeKind::Directory {
            self.theme().highlight_dir_style
        } else {
            self.theme().highlight_item_style
        };

        let mut list = List::new(
            self.files()
                .iter()
                .map(|file| file.text(self.cwd(), self.theme())),
        )
        .style(self.theme().style)
        .highlight_spacing(self.theme().highlight_spacing.clone())
        .highlight_style(highlight_style);

        if let Some(symbol) = self.theme().highlight_symbol.as_deref() {
            list = list.highlight_symbol(symbol);
        }

        if let Some(block) = self.theme().block.as_ref() {
            let mut block = block.clone();

            for title_top in self.theme().title_top(self) {
                block = block.title_top(title_top)
            }

            list = list.block(block);
        }

        ratatui::widgets::StatefulWidgetRef::render_ref(&list, area, buf, &mut state)
    }
}

trait NodeExt {
    fn text(&self, cwd: &PathBuf, theme: &Theme) -> Text<'_>;
}

impl NodeExt for Node {
    #[inline]
    fn text(&self, cwd: &PathBuf, theme: &Theme) -> Text<'_> {
        let mut name = self.name();
        let root_path = PathBuf::from("");
        let parent_name = cwd
            .parent()
            .unwrap_or(&root_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name.as_str() == parent_name {
            name = "..".to_string();
        } else if self.kind() == NodeKind::Directory {
            name.push('/');
        }
        let style = if self.kind() == NodeKind::Directory {
            *theme.dir_style()
        } else {
            *theme.item_style()
        };
        Span::styled(name, style).into()
    }
}
