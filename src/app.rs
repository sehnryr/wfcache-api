use color_eyre::{eyre::Context, Result};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{layout::Position, prelude::*};

use crate::tui;
use crate::widgets;

#[derive(Debug, Default)]
pub struct App {
    // counter: u8,
    area: Rect,
    button_state: widgets::button::State,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                self.area = frame.size();
                self.render_frame(frame)
            })?;
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

        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            Event::Mouse(mouse) => self
                .handle_mouse_event(mouse)
                .wrap_err_with(|| format!("handling mouse event failed:\n{mouse:#?}")),
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

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Moved => {}
            MouseEventKind::Down(MouseButton::Left) => {
                let [_, _, _, export_button_area] = self.compute_layout(self.area);
                if export_button_area.contains(Position::new(mouse.column, mouse.row)) {
                    if self.button_state == widgets::button::State::Active {
                        self.button_state = widgets::button::State::Normal;
                    } else {
                        self.button_state = widgets::button::State::Active;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn compute_layout(&self, area: Rect) -> [Rect; 4] {
        let vertical_layout = Layout::vertical([Constraint::Min(10), Constraint::Length(5)]);
        let [content_area, extract_area] = vertical_layout.areas(area);

        let content_layout = Layout::horizontal([Constraint::Length(30), Constraint::Min(0)]);
        let [explorer_area, info_area] = content_layout.areas(content_area);

        let export_layout = Layout::horizontal([Constraint::Length(15), Constraint::Min(0)]);
        let [export_button_area, _] = export_layout.areas(extract_area.inner(&Margin::new(2, 1)));

        [explorer_area, info_area, extract_area, export_button_area]
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [exporer_area, info_area, extract_area, export_button_area] = self.compute_layout(area);

        let button_label = if self.button_state == widgets::button::State::Active {
            "Cancel"
        } else {
            "Extract"
        };

        widgets::explorer::Explorer::new().render(exporer_area, buf);
        widgets::info::Info::new().render(info_area, buf);
        widgets::extract::Extract::new().render(extract_area, buf);
        widgets::button::Button::new(button_label)
            .state(self.button_state)
            .render(export_button_area, buf);
    }
}
