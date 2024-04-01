use std::rc::Rc;

use derivative::Derivative;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::toc::{FileNode, Node, NodeKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Widget, WidgetRef};

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Info<'a> {
    #[derivative(Debug = "ignore")]
    h_cache: Rc<&'a CachePairReader>,
    node: Option<Node>,
}

impl<'a> Info<'a> {
    pub fn new(h_cache: Rc<&'a CachePairReader>) -> Self {
        Self {
            h_cache,
            node: None,
        }
    }

    pub fn set_node(&mut self, node: &Node) {
        self.node = Some(node.clone());
    }
}

impl WidgetRef for Info<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let content = if let Some(node) = &self.node {
            let node_name = format!("Name: {}", node.name());
            let node_path = format!("Path: {}", node.path().to_string_lossy());
            let mut content = vec![
                Line::from(vec![node_name.into()]),
                Line::from(vec![node_path.into()]),
            ];

            if node.kind() == NodeKind::File {
                let node_cache_offset = format!("Cache offset: {}", node.cache_offset());
                let node_timestamp = format!("Timestamp: {}", node.timestamp());
                let node_compressed_length = format!("Compressed length: {}", node.comp_len());
                let node_length = format!("Length: {}", node.len());

                content.extend(vec![
                    Line::from(vec![node_cache_offset.into()]),
                    Line::from(vec![node_timestamp.into()]),
                    Line::from(vec![node_compressed_length.into()]),
                    Line::from(vec![node_length.into()]),
                ]);
            }

            content
        } else {
            Vec::new()
        };

        let block = Block::default()
            .title(" Info ")
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1));

        Paragraph::new(content).block(block).render(area, buf);
    }
}
