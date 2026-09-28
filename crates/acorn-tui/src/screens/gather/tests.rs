use super::*;
use acorn::io::{database::Row, standard_project_folder};
use ratatui::{backend::TestBackend, Terminal};
use std::{env::temp_dir, fs::remove_file};
#[test]
fn test_gather_discovers_identifiers_from_text() {
    let output = gather("doi:10.1234/example https://ror.org/01qz5mb56", &disabled_database());
    assert_eq!(output.checks.len(), 0);
    assert_eq!(output.discoveries.len(), 2);
}
#[test]
fn test_gather_persists_discoveries() {
    let path = standard_project_folder("gather-tui-test", Some(temp_dir())).with_extension("db");
    let options = TuiOptions {
        database_path: Some(path.clone()),
        ..TuiOptions::default()
    };
    let output = gather("doi:10.1234/persisted", &options);
    let rows = IdentifierRow::default().select_all(Some(path.clone())).unwrap_or_default();
    assert!(output.checks.is_empty());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].identifier.as_deref(), Some("10.1234/persisted"));
    let _ = remove_file(path);
}
#[test]
fn test_gather_reads_docx_input() {
    let output = gather("../../tests/fixtures/acorn.docx", &disabled_database());
    assert!(output.checks.is_empty());
    assert_eq!(output.input_count, 1);
}
#[test]
fn test_input_values_preserve_text_and_split_explicit_inputs() {
    assert_eq!(input_values("ordinary text value"), vec!["ordinary text value"]);
    assert_eq!(
        input_values("doi:10.1234/first https://ror.org/01qz5mb56"),
        vec!["doi:10.1234/first", "https://ror.org/01qz5mb56"]
    );
}
#[test]
fn test_offline_gather_rejects_remote_input() {
    let output = gather(
        "https://example.com/document.txt",
        &TuiOptions {
            offline: true,
            ..disabled_database()
        },
    );
    assert_eq!(output.checks.len(), 1);
    assert!(output.discoveries.is_empty());
}
#[test]
fn test_offline_remote_search_reports_error_without_spawning() {
    let mut app = App::with_options(
        Screen::Gather,
        TuiOptions {
            offline: true,
            ..disabled_database()
        },
    );
    app.gather.mode = GatherMode::OstiProjects;
    app.gather.input = "ACORN".to_string();
    start_remote(&mut app, false);
    assert!(app.gather.remote_error.as_deref().is_some_and(|value| value.contains("offline")));
    assert!(app.gather_remote.is_none());
}
#[test]
fn test_render_supports_compact_terminal() {
    let backend = TestBackend::new(60, 16);
    let mut terminal = Terminal::new(backend).expect("test backend should initialize");
    let mut app = App::with_options(Screen::Gather, disabled_database());
    assert!(terminal.draw(|frame| render(frame, &mut app)).is_ok());
}
#[test]
fn test_selection_wraps_in_both_directions() {
    assert_eq!(next_index(2, 3), 0);
    assert_eq!(previous_index(0, 3), 2);
}
#[test]
fn test_tab_cycles_remote_gather_modes() {
    let mut app = App::with_options(Screen::Gather, disabled_database());
    handle_event(&mut app, Event::Key(crossterm::event::KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
    assert_eq!(app.gather.mode, GatherMode::OstiProjects);
    assert_eq!(next_mode(GatherMode::OstiProjects), GatherMode::OstiPeople);
    assert_eq!(next_mode(GatherMode::OstiOrganizations), GatherMode::Local);
}
fn disabled_database() -> TuiOptions {
    TuiOptions {
        no_local_database: true,
        ..TuiOptions::default()
    }
}
