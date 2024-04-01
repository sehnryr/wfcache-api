use std::{io::Result, path::PathBuf};

use ratatui::widgets::WidgetRef;

use crate::input::KeyInput;

use super::theme::Theme;
use super::widget::Renderer;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Explorer {
    cwd: PathBuf,
    files: Vec<File>,
    selected: usize,
    theme: Theme,
}

impl Explorer {
    pub fn new() -> Result<Explorer> {
        let cwd = std::env::current_dir()?;

        let mut file_explorer = Self {
            cwd,
            files: vec![],
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
                    self.selected = self.files.len() - 1;
                } else {
                    self.selected -= 1;
                }
            }
            KeyInput::Down => {
                if self.selected == self.files.len() - 1 {
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
                if self.files[self.selected].path.is_dir() {
                    self.cwd = self.files.swap_remove(self.selected).path;
                    self.get_and_set_files()?;
                    self.selected = 0
                }
            }
            _ => {}
        }

        Ok(())
    }

    #[inline]
    pub fn current(&self) -> &File {
        &self.files[self.selected]
    }

    #[inline]
    pub const fn files(&self) -> &Vec<File> {
        &self.files
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
        let (mut dirs, mut none_dirs): (Vec<_>, Vec<_>) = std::fs::read_dir(&self.cwd)?
            .filter_map(|entry| {
                entry.ok().map(|e| {
                    let path = e.path();
                    let is_dir = path.is_dir();
                    let name = if is_dir {
                        format!("{}/", e.file_name().to_string_lossy())
                    } else {
                        e.file_name().to_string_lossy().into_owned()
                    };

                    File { name, path, is_dir }
                })
            })
            .partition(|file| file.is_dir);

        dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));
        none_dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));

        if let Some(parent) = self.cwd.parent() {
            let mut files = Vec::with_capacity(1 + dirs.len() + none_dirs.len());

            files.push(File {
                name: "../".to_owned(),
                path: parent.to_path_buf(),
                is_dir: true,
            });

            files.extend(dirs);
            files.extend(none_dirs);

            self.files = files
        } else {
            let mut files = Vec::with_capacity(dirs.len() + none_dirs.len());

            files.extend(dirs);
            files.extend(none_dirs);

            self.files = files;
        };

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct File {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

impl File {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub const fn is_dir(&self) -> bool {
        self.is_dir
    }
}
