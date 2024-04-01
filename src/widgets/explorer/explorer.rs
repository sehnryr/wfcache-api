use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::rc::Rc;

use derivative::Derivative;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::toc::{DirectoryNode, Node, NodeKind};
use ratatui::widgets::WidgetRef;

use crate::input::KeyInput;

use super::theme::Theme;
use super::widget::Renderer;

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

    #[inline]
    pub const fn widget(&self) -> impl WidgetRef + '_ {
        Renderer(self)
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
