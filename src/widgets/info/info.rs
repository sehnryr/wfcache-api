use std::sync::Arc;

use derivative::Derivative;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::package::{Package, PackageType};
use lotus_lib::toc::{DirectoryNode, FileNode, Node, NodeKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Widget, WidgetRef, Wrap};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Info {
    #[derivative(Debug = "ignore")]
    package: Arc<Package<CachePairReader>>,

    h_node: Node,
    f_node: Option<Node>,
    b_node: Option<Node>,
}

impl Info {
    pub fn new(package: Arc<Package<CachePairReader>>) -> Self {
        let h_cache = package.borrow(PackageType::H).unwrap();
        let h_node = h_cache.get_directory_node("/").unwrap();

        Self {
            package,
            h_node,
            f_node: None,
            b_node: None,
        }
    }

    pub fn set_node(&mut self, node: &Node) {
        let node_path = node.path();
        if self.h_node.path() == node_path {
            return;
        }

        self.h_node = node.clone();

        if node.kind() == NodeKind::File {
            if let Some(f_cache) = self.package.borrow(PackageType::F) {
                self.f_node = f_cache.get_file_node(&node_path);
            }
            if let Some(b_cache) = self.package.borrow(PackageType::B) {
                self.b_node = b_cache.get_file_node(&node_path);
            }
        } else {
            self.f_node = None;
            self.b_node = None;
        }
    }
}

impl WidgetRef for Info {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let node_name = format!("Name: {}", self.h_node.name());
        let node_path = format!("Path: {}", self.h_node.path().to_string_lossy());
        let mut content = vec![
            Line::from(vec![node_name.into()]),
            Line::from(vec![node_path.into()]),
        ];

        let cache_style = Style::new().fg(Color::LightBlue).underlined();

        if self.h_node.kind() == NodeKind::File {
            content.extend(vec![
                Line::from(""),
                Line::from(Span::styled("H Cache      ", cache_style)),
            ]);
            content.extend(cache_info(&self.h_node));
        } else {
            let children = self.h_node.children();
            let (file_count, dir_count) = children.iter().fold((0, 0), |(f, d), node| {
                if node.kind() == NodeKind::File {
                    (f + 1, d)
                } else {
                    (f, d + 1)
                }
            });

            let file_count = format!("File count: {}", file_count);
            let dir_count = format!("Dir count:  {}", dir_count);

            content.extend(vec![
                Line::from(""),
                Line::from(file_count),
                Line::from(dir_count),
            ]);
        }

        if let Some(f_node) = &self.f_node {
            content.extend(vec![
                Line::from(""),
                Line::from(Span::styled("F Cache      ", cache_style)),
            ]);
            content.extend(cache_info(f_node));
        }

        if let Some(b_node) = &self.b_node {
            content.extend(vec![
                Line::from(""),
                Line::from(Span::styled("B Cache      ", cache_style)),
            ]);
            content.extend(cache_info(b_node));
        }

        let block = Block::default()
            .title(" Info ")
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1));

        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

#[inline]
fn cache_info(node: &Node) -> Vec<Line> {
    let cache_offset = format!("Cache offset: {}", node.cache_offset());
    let timestamp = format!("Timestamp:    {}", node.timestamp());
    let compressed_length = if node.comp_len() < 1000 {
        format!("Comp Length:  {} B", node.comp_len())
    } else {
        format!(
            "Comp Length:  {} B ({})",
            node.comp_len(),
            show_bytes(node.comp_len())
        )
    };
    let length = if node.len() < 1000 {
        format!("Length:       {} B", node.len())
    } else {
        format!(
            "Length:       {} B ({})",
            node.len(),
            show_bytes(node.len())
        )
    };

    vec![
        Line::from(cache_offset),
        Line::from(timestamp),
        Line::from(compressed_length),
        Line::from(length),
    ]
}

#[inline]
fn show_bytes(bytes: i32) -> String {
    if bytes < 1000 {
        format!("{} B", bytes)
    } else if bytes < 1000_000 {
        format!("{:.2} KB", bytes as f64 / 1000.0)
    } else if bytes < 1000_000_000 {
        format!("{:.2} MB", bytes as f64 / 1000_000.0)
    } else {
        format!("{:.2} GB", bytes as f64 / 1000_000_000.0)
    }
}
