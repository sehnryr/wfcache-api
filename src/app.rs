use std::path::PathBuf;
use std::rc::Rc;

use color_eyre::eyre::ContextCompat;
use color_eyre::{eyre::Context, Result};
use crossterm::event;
use lotus_lib::cache_pair::{CachePair, CachePairReader};
use lotus_lib::package::Package;
use lotus_lib::package::PackageTrioType;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Widget;
use ratatui::Frame;

use crate::input::KeyInput;
use crate::tui;
use crate::widgets;

pub struct App<'a> {
    exit: bool,
    output_directory: PathBuf,

    h_cache: Rc<&'a CachePairReader>,
    f_cache: Option<Rc<&'a CachePairReader>>,
    b_cache: Option<Rc<&'a CachePairReader>>,

    explorer_widget: widgets::Explorer<'a>,
    info_widget: widgets::Info<'a>,
    extract_widget: widgets::Extract<'a>,
}

impl<'a> App<'a> {
    pub fn try_init(
        package: &'a Package<CachePairReader>,
        output_directory: PathBuf,
    ) -> Result<App<'a>> {
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

        let h_cache = Rc::new(h_cache);
        let f_cache = f_cache.map(|cache| Rc::new(cache));
        let b_cache = b_cache.map(|cache| Rc::new(cache));

        let explorer_widget =
            widgets::Explorer::new(h_cache.clone()).wrap_err("Explorer widget failed")?;
        let info_widget = widgets::Info::new(h_cache.clone(), f_cache.clone(), b_cache.clone())
            .wrap_err("Info widget failed")?;
        let extract_widget = widgets::Extract::new();

        Ok(App {
            exit: false,
            output_directory,
            h_cache,
            f_cache,
            b_cache,
            explorer_widget,
            info_widget,
            extract_widget,
        })
    }

    pub fn output_directory(&self) -> &PathBuf {
        &self.output_directory
    }

    pub fn h_cache(&self) -> Rc<&CachePairReader> {
        self.h_cache.clone()
    }

    pub fn f_cache(&self) -> Option<Rc<&CachePairReader>> {
        self.f_cache.clone()
    }

    pub fn b_cache(&self) -> Option<Rc<&CachePairReader>> {
        self.b_cache.clone()
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
        self.handle_key_input(&event).wrap_err("app handle failed")
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_key_input<I: Into<KeyInput>>(&mut self, input: I) -> Result<()> {
        match input.into() {
            KeyInput::Quit => self.exit(),
            KeyInput::Down | KeyInput::Up | KeyInput::Right | KeyInput::Left => {
                // Update the info widget with the current node only on navigation
                self.info_widget.set_node(self.explorer_widget.current())?;
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

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (explorer_area, info_area, extract_area) = self.compute_layout(area);

        self.explorer_widget.render(explorer_area, buf);
        self.info_widget.render(info_area, buf);
        self.extract_widget.render(extract_area, buf);
    }
}

#[cfg(test)]
mod test {
    use lotus_lib::package::PackageCollection;

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

        let collection = PackageCollection::<CachePairReader>::new(cache_windows_directory, true);
        let package = collection.get_package(&package_name).unwrap();

        let app = App::try_init(package, output_directory).unwrap();

        // Misc package has H, F, and B caches
        assert!(!app.h_cache().files().is_empty());
        assert!(!app.f_cache().unwrap().files().is_empty());
        assert!(!app.b_cache().unwrap().files().is_empty());
    }
}
