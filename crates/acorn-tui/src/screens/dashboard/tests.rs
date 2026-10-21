use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_gather_command_opens_gather_screen() {
    let mut app = App::new(Screen::Dashboard);
    execute(&mut app, SlashCommandId::Gather);
    assert_eq!(app.current_screen, Screen::Gather);
    assert!(app.dashboard.data.input.is_empty());
}
#[test]
fn test_quit_alias_still_exits_dashboard() {
    let mut app = App::new(Screen::Dashboard);
    assert_eq!(filtered_commands("/quit").first().map(|command| command.id), Some(SlashCommandId::Exit));
    execute(&mut app, SlashCommandId::Exit);
    assert!(app.should_quit);
}
#[test]
fn test_slash_filter_reports_no_unknown_commands() {
    assert!(filtered_commands("/missing").is_empty());
}
#[test]
fn test_slash_filter_updates_for_each_query_fragment() {
    assert_eq!(filtered_commands("/t")[0].id, SlashCommandId::Theme);
    assert_eq!(filtered_commands("/theme")[0].id, SlashCommandId::Theme);
    assert_eq!(filtered_commands("/doctor")[0].id, SlashCommandId::Doctor);
    assert_eq!(filtered_commands("/gather")[0].id, SlashCommandId::Gather);
    assert_eq!(filtered_commands("/harvest")[0].id, SlashCommandId::Gather);
}
#[test]
fn test_splash_renders_at_full_and_compact_terminal_sizes() {
    [Rect::new(0, 0, 100, 30), Rect::new(0, 0, 40, 10)].into_iter().for_each(|area| {
        let backend = TestBackend::new(area.width, area.height);
        let mut terminal = Terminal::new(backend).expect("test backend should initialize");
        let app = App::new(Screen::Dashboard);
        assert!(terminal.draw(|frame| render(frame, &app)).is_ok());
    });
}
#[test]
fn test_theme_command_opens_picker() {
    let mut app = App::new(Screen::Dashboard);
    execute(&mut app, SlashCommandId::Theme);
    assert_eq!(app.current_screen, Screen::ThemePicker);
    assert!(app.dashboard.data.input.is_empty());
}
#[test]
fn test_typed_characters_immediately_change_the_filter() {
    let mut app = App::new(Screen::Dashboard);
    handle_event(
        &mut app,
        Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('/'), crossterm::event::KeyModifiers::NONE)),
    );
    handle_event(
        &mut app,
        Event::Key(crossterm::event::KeyEvent::new(KeyCode::Char('c'), crossterm::event::KeyModifiers::NONE)),
    );
    assert_eq!(app.dashboard.data.input, "/c");
    assert_eq!(filtered_commands(&app.dashboard.data.input)[0].id, SlashCommandId::Check);
}
