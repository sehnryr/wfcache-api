use crossterm::event::KeyCode;

use crate::tui::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    NavigateUp,
    NavigateDown,
    NavigateIn,
    NavigateOut,

    ExtractToggle,
    RecursiveModeToggle,

    Tick,
    Render,
    Quit,
    None,
}

impl From<&Event> for Action {
    fn from(event: &Event) -> Self {
        match event {
            Event::Quit => Action::Quit,
            Event::Tick => Action::Tick,
            Event::Render => Action::Render,
            Event::Key(key) => match key.code {
                KeyCode::Char('j') | KeyCode::Down => Action::NavigateDown,
                KeyCode::Char('k') | KeyCode::Up => Action::NavigateUp,
                KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => Action::NavigateOut,
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => Action::NavigateIn,
                KeyCode::Char(' ') => Action::ExtractToggle,
                KeyCode::Char('r') | KeyCode::Char('R') => Action::RecursiveModeToggle,
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Action::Quit,
                _ => Action::None,
            },
            _ => Action::None,
        }
    }
}
