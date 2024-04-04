use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::sync::Arc;

use derivative::Derivative;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::package::{Package, PackageType};
use lotus_lib::toc::{Node, NodeKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::{Block, Borders, Widget, WidgetRef};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::action::Action;
use crate::extract::{extract_dir, extract_file};

use super::button::Button;
use super::gauge::Gauge;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Extract {
    output_dir: PathBuf,
    #[derivative(Debug = "ignore")]
    package: Arc<Package<CachePairReader>>,
    h_node: Node,
    recursive: bool,

    button_widget: Button,
    gauge_widget: Gauge,

    extract_task: Option<JoinHandle<()>>,
    extract_tx: Option<UnboundedSender<()>>,
    progress_rx: UnboundedReceiver<(usize, usize)>,
    progress_tx: UnboundedSender<(usize, usize)>,
}

impl Extract {
    pub fn new<P>(package: Arc<Package<CachePairReader>>, output_dir: P) -> Self
    where
        P: Into<PathBuf>,
    {
        let output_dir = output_dir.into();

        let h_node = package
            .borrow(PackageType::H)
            .unwrap()
            .get_directory_node("/")
            .unwrap();

        let (progress_tx, progress_rx) = unbounded_channel();

        Self {
            output_dir,
            package,
            h_node,
            recursive: false,
            button_widget: Button::new(),
            gauge_widget: Gauge::new(),
            extract_task: None,
            extract_tx: None,
            progress_rx,
            progress_tx,
        }
    }

    pub fn set_node(&mut self, node: &Node) {
        self.h_node = node.clone();
    }

    fn compute_layout(&self, area: Rect) -> (Rect, Rect) {
        let extract_layout = Layout::horizontal([
            Constraint::Length(15),
            Constraint::Length(1),
            Constraint::Min(0),
        ]);
        let [extract_button_area, _, extract_progress_area] =
            extract_layout.areas(area.inner(&Margin::new(2, 1)));
        (extract_button_area, extract_progress_area)
    }

    pub fn handle(&mut self, action: &Action) -> Result<()> {
        match action {
            Action::ExtractToggle => self.toggle_extract()?,
            Action::Tick => self.update_progress(),
            _ => {}
        }
        Ok(())
    }

    fn toggle_extract(&mut self) -> Result<()> {
        if self.extract_task.is_none() {
            self.button_widget.set_active(true);

            let (extract_tx, mut extract_rx) = unbounded_channel();
            self.extract_tx = Some(extract_tx);

            let package = self.package.clone();
            let h_node = self.h_node.clone();
            let output_dir = self.output_dir.clone();
            let recursive = self.recursive;
            let progress_tx = self.progress_tx.clone();
            self.extract_task = Some(tokio::spawn(async move {
                extract(
                    package,
                    h_node,
                    output_dir,
                    recursive,
                    &mut extract_rx,
                    progress_tx,
                );
            }));
        } else {
            self.extract_tx
                .take()
                .unwrap()
                .send(())
                .map_err(|_| Error::new(ErrorKind::Other, "Failed to send cancel signal"))?;

            self.extract_task = None;
            self.extract_tx = None;
            self.button_widget.set_active(false);
        }

        Ok(())
    }

    fn update_progress(&mut self) {
        while let Ok((count, progress)) = self.progress_rx.try_recv() {
            self.gauge_widget.set_progress(count, progress);

            if count == progress {
                self.extract_task = None;
                self.extract_tx = None;
                self.button_widget.set_active(false);
            }
        }
    }
}

impl Drop for Extract {
    fn drop(&mut self) {
        if let Some(extract_tx) = self.extract_tx.take() {
            let _ = extract_tx.send(());
        }
    }
}

impl WidgetRef for Extract {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Extract ".into(),
            "<Space> ".light_blue(),
            "Quit ".into(),
            "<Q> ".light_blue(),
        ]);
        let instructions = Title::from(instructions)
            .alignment(Alignment::Center)
            .position(Position::Bottom);

        Block::default()
            .title(instructions)
            .borders(Borders::ALL)
            .render(area, buf);

        let (extract_button_area, extract_progress_area) = self.compute_layout(area);

        self.button_widget.render(extract_button_area, buf);
        self.gauge_widget.render(extract_progress_area, buf);
    }
}

// #[cfg(test)]
// mod tests {
//     use ratatui::assert_buffer_eq;
//     use ratatui::style::Style;

//     use super::*;

//     #[test]
//     fn render() {
//         let extract = Extract::new();
//         let mut buf = Buffer::empty(Rect::new(0, 0, 50, 2));

//         extract.render(buf.area, &mut buf);

//         let mut expected = Buffer::with_lines(vec![
//             "┌────────────────────────────────────────────────┐",
//             "└─────────── Extract <Space> Quit <Q> ───────────┘",
//         ]);
//         let extract_style = Style::new().light_blue();
//         let quit_style = Style::new().light_blue();
//         expected.set_style(Rect::new(21, 1, 8, 1), extract_style);
//         expected.set_style(Rect::new(34, 1, 4, 1), quit_style);

//         assert_buffer_eq!(buf, expected);
//     }
// }

fn extract(
    package: Arc<Package<CachePairReader>>,
    node: Node,
    output_dir: PathBuf,
    recursive: bool,
    extract_rx: &mut UnboundedReceiver<()>,
    progress_tx: UnboundedSender<(usize, usize)>,
) {
    if let NodeKind::File = node.kind() {
        return extract_file(&package, &node, &output_dir, 0, 1, progress_tx).unwrap();
    }

    extract_dir(
        &package,
        &node,
        &output_dir,
        recursive,
        extract_rx,
        progress_tx,
    )
    .unwrap();
}
