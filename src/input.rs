use crossterm::event::KeyCode;

use crate::tui::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyInput {
    // Movement keys for the explorer
    Up,
    Down,
    Left,
    Right,

    // Toggle keys for the extract button
    Space,

    /// Quit the application
    Quit,

    None,
}

impl From<&Event> for KeyInput {
    fn from(value: &Event) -> Self {
        if let Event::Key(key) = value {
            return match key.code {
                KeyCode::Char('j') | KeyCode::Down => KeyInput::Down,
                KeyCode::Char('k') | KeyCode::Up => KeyInput::Up,
                KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => KeyInput::Left,
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => KeyInput::Right,
                KeyCode::Char(' ') => KeyInput::Space,
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => KeyInput::Quit,
                _ => KeyInput::None,
            };
        }

        KeyInput::None
    }
}
