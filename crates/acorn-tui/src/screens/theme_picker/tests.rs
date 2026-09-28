use super::handle_event;
use crate::{app::App, Screen};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_enter_applies_selected_theme() {
    let mut app = App::new(Screen::ThemePicker);
    handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
    handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    assert_eq!(app.theme.name, "one-dark");
    assert_eq!(app.current_screen, Screen::Dashboard);
}
#[test]
fn test_escape_returns_to_dashboard_without_changing_theme() {
    let mut app = App::new(Screen::ThemePicker);
    let original = app.theme.name;
    handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
    assert_eq!(app.theme.name, original);
    assert_eq!(app.current_screen, Screen::Dashboard);
}
