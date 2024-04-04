use std::path::PathBuf;
use std::sync::Arc;

use color_eyre::eyre::ContextCompat;
use color_eyre::{eyre::Context, Result};
use lotus_lib::cache_pair::{CachePair, CachePairReader};
#[cfg(test)]
use lotus_lib::package::Package;
use lotus_lib::package::PackageCollection;
use lotus_lib::package::PackageType;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Widget;
use ratatui::Frame;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::action::Action;
use crate::tui::Tui;
use crate::widgets;

pub struct App {
    exit: bool,
    action_rx: UnboundedReceiver<Action>,
    action_tx: UnboundedSender<Action>,
    #[cfg(test)]
    package: Arc<Package<CachePairReader>>,

    explorer_widget: widgets::Explorer,
    info_widget: widgets::Info,
    extract_widget: widgets::Extract,
}

impl App {
    pub fn try_init(
        cache_windows_directory: PathBuf,
        package_name: String,
        output_directory: PathBuf,
    ) -> Result<Self> {
        let mut collection =
            PackageCollection::<CachePairReader>::new(cache_windows_directory, true)
                .wrap_err("Failed to initialize package collection")?;

        let package = collection
            .borrow_mut(&package_name)
            .wrap_err(format!("Package {} not found", &package_name))?;

        package
            .borrow_mut(PackageType::H)
            .map(|cache| cache.read_toc().unwrap());
        package
            .borrow_mut(PackageType::F)
            .map(|cache| cache.read_toc().unwrap());
        package
            .borrow_mut(PackageType::B)
            .map(|cache| cache.read_toc().unwrap());

        let package = Arc::new(collection.take(&package_name).unwrap());

        let explorer_widget = widgets::Explorer::new(package.clone());
        let info_widget = widgets::Info::new(package.clone());
        let extract_widget = widgets::Extract::new(package.clone(), &output_directory);

        let (action_tx, action_rx) = unbounded_channel();
        Ok(Self {
            exit: false,
            action_rx,
            action_tx,
            #[cfg(test)]
            package,
            explorer_widget,
            info_widget,
            extract_widget,
        })
    }

    #[cfg(test)]
    fn h_cache(&self) -> &CachePairReader {
        self.package.borrow(PackageType::H).unwrap()
    }

    #[cfg(test)]
    fn f_cache(&self) -> Option<&CachePairReader> {
        self.package.borrow(PackageType::F)
    }

    #[cfg(test)]
    fn b_cache(&self) -> Option<&CachePairReader> {
        self.package.borrow(PackageType::B)
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        while !self.exit {
            let event = terminal.next().await?;
            let action = Action::from(&event);
            if action != Action::None {
                self.action_tx.send(action)?;
            }

            while let Ok(action) = self.action_rx.try_recv() {
                self.handle(&action)?;

                if let Action::Render = action {
                    terminal.draw(|frame| self.render_frame(frame))?;
                }
            }
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    fn handle(&mut self, action: &Action) -> Result<()> {
        // handle file explorer events
        self.explorer_widget.handle(&action);

        // handle extract widget events
        self.extract_widget
            .handle(&action)
            .wrap_err("extract widget handle failed")?;

        match action {
            Action::Quit => self.exit = true,
            Action::NavigateDown
            | Action::NavigateUp
            | Action::NavigateIn
            | Action::NavigateOut => {
                // Update the info widget with the current node only on navigation
                self.info_widget.set_node(self.explorer_widget.current());

                // Update the extract widget with the current node only on navigation
                self.extract_widget.set_node(self.explorer_widget.current());
            }
            _ => {}
        }
        Ok(())
    }

    fn compute_layout(&self, area: Rect) -> (Rect, Rect, Rect) {
        let vertical_layout = Layout::vertical([Constraint::Min(10), Constraint::Length(5)]);
        let [content_area, extract_area] = vertical_layout.areas(area);

        let content_layout = Layout::horizontal([Constraint::Length(50), Constraint::Min(0)]);
        let [explorer_area, info_area] = content_layout.areas(content_area);

        (explorer_area, info_area, extract_area)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (explorer_area, info_area, extract_area) = self.compute_layout(area);

        self.explorer_widget.render(explorer_area, buf);
        self.info_widget.render(info_area, buf);
        self.extract_widget.render(extract_area, buf);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const HOME_DIR: &str = env!("HOME"); // TODO: Windows support
    const CACHE_WINDOWS_DIRECTORY: &str = ".steam/steam/steamapps/common/Warframe/Cache.Windows";
    const PACKAGE_NAME: &str = "Misc";
    const OUTPUT_DIRECTORY: &str = "Downloads/wfcache-extract";

    #[test]
    fn test_cache_windows_directory() {
        let cache_windows_directory = PathBuf::from(HOME_DIR).join(CACHE_WINDOWS_DIRECTORY);
        assert!(cache_windows_directory.is_dir());
    }

    #[tokio::test]
    async fn test_init() {
        let cache_windows_directory = PathBuf::from(HOME_DIR).join(CACHE_WINDOWS_DIRECTORY);
        let package_name = PACKAGE_NAME.to_string();
        let output_directory = PathBuf::from(HOME_DIR).join(OUTPUT_DIRECTORY);

        let app = App::try_init(cache_windows_directory, package_name, output_directory).unwrap();

        // Misc package has H, F, and B caches
        assert!(!app.h_cache().files().is_empty());
        assert!(!app.f_cache().unwrap().files().is_empty());
        assert!(!app.b_cache().unwrap().files().is_empty());
    }
}
