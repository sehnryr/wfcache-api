use std::path::PathBuf;

use color_eyre::eyre::ContextCompat;
use color_eyre::{eyre::Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use lotus_lib::cache_pair::{CachePair, CachePairReader};
use lotus_lib::package::PackageCollection;
use lotus_lib::package::PackageTrioType;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Widget;
use ratatui::Frame;

use crate::tui;
use crate::widgets;

pub struct App<'a> {
    exit: bool,
    output_directory: PathBuf,
    package_name: String,
    package_collection: PackageCollection<CachePairReader>,
    current_lotus_dir: PathBuf,
    selected_lotus_node: usize,
    explorer_widget: widgets::Explorer,
    info_widget: widgets::Info,
    extract_widget: widgets::Extract<'a>,
}

impl App<'_> {
    pub fn try_init<'a>(
        cache_windows_directory: PathBuf,
        package_name: String,
        output_directory: PathBuf,
    ) -> Result<App<'a>> {
        let package_collection =
            PackageCollection::<CachePairReader>::new(cache_windows_directory.clone(), true);
        let package = package_collection
            .get_package(&package_name)
            .wrap_err_with(|| format!("Package {} not found", package_name))?;

        let h_cache = package
            .get(&PackageTrioType::H)
            .wrap_err("H cache not found")?;
        let f_cache = package.get(&PackageTrioType::F);
        let b_cache = package.get(&PackageTrioType::B);

        h_cache.read_toc().unwrap();
        f_cache.and_then(|cache| {
            cache.read_toc().unwrap();
            Some(cache)
        });
        b_cache.and_then(|cache| {
            cache.read_toc().unwrap();
            Some(cache)
        });

        Ok(App {
            exit: false,
            output_directory,
            package_name,
            package_collection,
            current_lotus_dir: PathBuf::from("/"),
            selected_lotus_node: 0,
            explorer_widget: widgets::Explorer::new().wrap_err("Explorer widget failed")?,
            info_widget: widgets::Info::new(),
            extract_widget: widgets::Extract::new(),
        })
    }

    pub fn output_directory(&self) -> &PathBuf {
        &self.output_directory
    }

    fn get_cache(&self, package_type: &PackageTrioType) -> Option<&CachePairReader> {
        self.package_collection
            .get_package(&self.package_name)
            .unwrap()
            .get(package_type)
    }

    pub fn h_cache(&self) -> &CachePairReader {
        self.get_cache(&PackageTrioType::H).unwrap()
    }

    pub fn f_cache(&self) -> Option<&CachePairReader> {
        self.get_cache(&PackageTrioType::F)
    }

    pub fn b_cache(&self) -> Option<&CachePairReader> {
        self.get_cache(&PackageTrioType::B)
    }

    pub fn current_lotus_dir(&self) -> &PathBuf {
        &self.current_lotus_dir
    }

    pub fn selected_lotus_node(&self) -> usize {
        self.selected_lotus_node
    }
}

impl App<'_> {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        // poll for events every 16ms or approximately 60fps
        if !event::poll(std::time::Duration::from_millis(16))? {
            return Ok(());
        }

        let event = event::read()?;

        // handle file explorer events
        self.explorer_widget
            .handle(&event)
            .wrap_err("explorer widget handle failed")?;

        // handle extract widget events
        self.extract_widget
            .handle(&event)
            .wrap_err("extract widget handle failed")?;

        // handle application events
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.exit(),
            _ => {}
        }
        Ok(())
    }

    fn compute_layout(&self, area: Rect) -> (Rect, Rect, Rect) {
        let vertical_layout = Layout::vertical([Constraint::Min(10), Constraint::Length(5)]);
        let [content_area, extract_area] = vertical_layout.areas(area);

        let content_layout = Layout::horizontal([Constraint::Length(30), Constraint::Min(0)]);
        let [explorer_area, info_area] = content_layout.areas(content_area);

        (explorer_area, info_area, extract_area)
    }
}

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (explorer_area, info_area, extract_area) = self.compute_layout(area);

        self.explorer_widget.widget().render(explorer_area, buf);
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

    #[test]
    fn test_init() {
        let cache_windows_directory = PathBuf::from(HOME_DIR).join(CACHE_WINDOWS_DIRECTORY);
        let package_name = PACKAGE_NAME.to_string();
        let output_directory = PathBuf::from(HOME_DIR).join(OUTPUT_DIRECTORY);

        App::try_init(cache_windows_directory, package_name, output_directory).unwrap();
    }

    #[test]
    fn test_init_package_toc_read() {
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
