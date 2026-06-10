use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Processed key event
pub enum Event {
    /// Quit the application
    Quit,
    /// Navigate up
    Up,
    /// Navigate down
    Down,
    /// Navigate left
    Left,
    /// Navigate right
    Right,
    /// Enter/select
    Enter,
    /// Go back
    Back,
    /// Refresh
    Refresh,
    /// Export
    Export,
    /// Number key (1-9)
    Number(u8),
    /// Character key (reserved for search/filter)
    #[allow(dead_code)]
    Char(char),
}
/// Convert a crossterm KeyEvent to our internal Event
pub fn to_internal_event(key: KeyEvent) -> Option<Event> {
    if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
        return None;
    }
    match key.code {
        | KeyCode::Char('q') | KeyCode::Char('Q') => Some(Event::Quit),
        | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Event::Quit),
        | KeyCode::Up | KeyCode::Char('k') => Some(Event::Up),
        | KeyCode::Down | KeyCode::Char('j') => Some(Event::Down),
        | KeyCode::Left | KeyCode::Char('h') => Some(Event::Left),
        | KeyCode::Right | KeyCode::Char('l') => Some(Event::Right),
        | KeyCode::Enter => Some(Event::Enter),
        | KeyCode::Esc => Some(Event::Back),
        | KeyCode::Char('r') | KeyCode::Char('R') => Some(Event::Refresh),
        | KeyCode::Char('e') | KeyCode::Char('E') => Some(Event::Export),
        | KeyCode::Char('1') => Some(Event::Number(1)),
        | KeyCode::Char('2') => Some(Event::Number(2)),
        | KeyCode::Char('3') => Some(Event::Number(3)),
        | KeyCode::Char('4') => Some(Event::Number(4)),
        | KeyCode::Char('5') => Some(Event::Number(5)),
        | KeyCode::Char('6') => Some(Event::Number(6)),
        | KeyCode::Char('7') => Some(Event::Number(7)),
        | KeyCode::Char('8') => Some(Event::Number(8)),
        | KeyCode::Char('9') => Some(Event::Number(9)),
        | KeyCode::Char(c) => Some(Event::Char(c)),
        | _ => None,
    }
}
