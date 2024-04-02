use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::rc::Rc;

use derivative::Derivative;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::toc::{DirectoryNode, Node, NodeKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListState, WidgetRef};

use crate::action::Action;

#[derive(Clone, Derivative)]
#[derivative(Debug, PartialEq, Eq, Hash)]
pub struct Explorer<'a> {
    cwd: PathBuf,
    #[derivative(Debug = "ignore", PartialEq = "ignore", Hash = "ignore")]
    h_cache: Rc<&'a CachePairReader>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    nodes: Vec<Node>,
    selected: usize,
}

impl<'a> Explorer<'a> {
    pub fn new(h_cache: Rc<&'a CachePairReader>) -> Result<Explorer<'a>> {
        let mut file_explorer = Self {
            cwd: PathBuf::from("/"),
            h_cache,
            nodes: vec![],
            selected: 0,
        };

        file_explorer.get_and_set_files()?;

        Ok(file_explorer)
    }

    pub fn handle(&mut self, action: &Action) -> Result<()> {
        match action {
            Action::NavigateUp => {
                if self.selected == 0 {
                    self.selected = self.nodes.len() - 1;
                } else {
                    self.selected -= 1;
                }
            }
            Action::NavigateDown => {
                if self.selected == self.nodes.len() - 1 {
                    self.selected = 0;
                } else {
                    self.selected += 1;
                }
            }
            Action::NavigateOut => {
                let parent = self.cwd.parent();

                if let Some(parent) = parent {
                    self.cwd = parent.to_path_buf();
                    self.get_and_set_files()?;
                    self.selected = 0
                }
            }
            Action::NavigateIn => {
                if self.selected != 0 && self.nodes[self.selected].kind() == NodeKind::Directory {
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
    pub fn current(&self) -> &Node {
        &self.nodes[self.selected]
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
            let mut nodes = Vec::with_capacity(2 + directories.len() + files.len());

            let parent_node = current_directory.parent().ok_or(Error::new(
                ErrorKind::NotFound,
                "Parent directory not found",
            ))?;

            nodes.push(current_directory);
            nodes.push(parent_node);

            nodes.extend(directories);
            nodes.extend(files);

            self.nodes = nodes;
        } else {
            let mut nodes = Vec::with_capacity(1 + directories.len() + files.len());

            nodes.push(current_directory);

            nodes.extend(directories);
            nodes.extend(files);

            self.nodes = nodes;
        }

        Ok(())
    }
}

impl WidgetRef for Explorer<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut state = ListState::default().with_selected(Some(self.selected));

        let highlight_style: Style = {
            let style: NodeStyle = self.current().kind().into();
            style.highlight()
        };

        let nodes_text = self.nodes.iter().enumerate().map(|(index, node)| {
            if index == 0 {
                Span::styled("./", NodeStyle::Directory).into()
            } else if index == 1 && self.cwd.parent().is_some() {
                Span::styled("../", NodeStyle::Directory).into()
            } else {
                node.text()
            }
        });

        let mut list = List::new(nodes_text)
            .style(Style::default())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(highlight_style);

        let mut block = Block::default().borders(Borders::ALL);

        let current_directory_name = format!(
            "/{}",
            self.cwd
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
        );
        block = block.title_top(current_directory_name);

        list = list.block(block);

        ratatui::widgets::StatefulWidgetRef::render_ref(&list, area, buf, &mut state)
    }
}

enum NodeStyle {
    Directory,
    Item,
}

impl NodeStyle {
    #[inline]
    fn highlight(self) -> Style {
        let style: Style = self.into();
        style.bg(Color::DarkGray)
    }
}

impl Into<NodeStyle> for NodeKind {
    fn into(self) -> NodeStyle {
        match self {
            NodeKind::Directory => NodeStyle::Directory,
            NodeKind::File => NodeStyle::Item,
        }
    }
}

impl From<NodeStyle> for Style {
    fn from(style: NodeStyle) -> Self {
        match style {
            NodeStyle::Directory => Style::default().fg(Color::LightBlue),
            NodeStyle::Item => Style::default().fg(Color::White),
        }
    }
}

trait NodeExt {
    fn text(&self) -> Text<'_>;
}

impl NodeExt for Node {
    #[inline]
    fn text(&self) -> Text<'_> {
        let mut name = self.name();
        if self.kind() == NodeKind::Directory {
            name.push('/');
        }
        let style: NodeStyle = self.kind().into();
        let style: Style = style.into();
        Span::styled(name, style).into()
    }
}
