use crate::app::{App, GatherDiscovery, GatherState};
use crate::{Screen, TuiOptions};
use acorn::analyzer::{Check, CheckCategory};
use acorn::check_err;
use acorn::io::database::schema::{IdentifierRow, Table};
use acorn::io::database::{Database, Operations};
use acorn::io::document::SourceDocument;
use acorn::io::{files_all, uri_to_path};
use acorn::prelude::PathBuf;
use acorn::schema::pid::Identifier;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use jiff::Timestamp;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

#[derive(Clone, Debug)]
struct GatherOutput {
    checks: Vec<Check>,
    discoveries: Vec<GatherDiscovery>,
    input_count: usize,
}
pub fn render(frame: &mut Frame, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(3),
        ])
        .split(frame.area());
    render_title(frame, rows[0], app);
    render_input(frame, rows[1], app);
    render_summary(frame, rows[2], app);
    render_output(frame, rows[3], app);
    render_controls(frame, rows[4], app);
}
fn render_title(frame: &mut Frame, area: Rect, app: &App) {
    let mode = if app.options.offline { "offline" } else { "online" };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ACORN Gather ", Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(format!("· {mode}"), Style::default().fg(app.theme.text_muted)),
        ]))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(app.theme.accent)))
        .alignment(Alignment::Center),
        area,
    );
}
fn render_input(frame: &mut Frame, area: Rect, app: &App) {
    let value = match app.gather.input.is_empty() {
        | true => Span::styled("Path, directory, URL, PID, or text", Style::default().fg(app.theme.text_muted)),
        | false => Span::styled(app.gather.input.as_str(), Style::default().fg(app.theme.text)),
    };
    frame.render_widget(
        Paragraph::new(Line::from(value)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input ")
                .border_style(Style::default().fg(app.theme.border)),
        ),
        area,
    );
    let offset = app.gather.input.chars().count().min(usize::from(area.width.saturating_sub(4))) as u16;
    frame.set_cursor_position((area.x.saturating_add(2).saturating_add(offset), area.y.saturating_add(1)));
}
fn render_summary(frame: &mut Frame, area: Rect, app: &App) {
    let failures = app.gather.checks.iter().filter(|check| check.is_failure()).count();
    let persistence = if app.options.no_local_database { "disabled" } else { "enabled" };
    let summary = format!(
        " Inputs: {}  ·  Discoveries: {}  ·  Failures: {}  ·  Database: {} ",
        app.gather.input_count,
        app.gather.discoveries.len(),
        failures,
        persistence,
    );
    frame.render_widget(
        Paragraph::new(summary)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(app.theme.border)))
            .alignment(Alignment::Center),
        area,
    );
}
fn render_output(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    let discoveries = match app.gather.discoveries.is_empty() {
        | true => vec![ListItem::new(Span::styled(
            " No identifiers gathered",
            Style::default().fg(app.theme.text_muted),
        ))],
        | false => app
            .gather
            .discoveries
            .iter()
            .map(|discovery| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!(" {:<8}", discovery.identifier_type), Style::default().fg(app.theme.accent)),
                        Span::styled(discovery.identifier.clone(), Style::default().fg(app.theme.text)),
                    ]),
                    Line::from(Span::styled(
                        format!("          {} ({})", discovery.source, discovery.source_format),
                        Style::default().fg(app.theme.text_muted),
                    )),
                ])
            })
            .collect(),
    };
    let selection = (!app.gather.discoveries.is_empty()).then_some(app.gather.selected_index.min(app.gather.discoveries.len().saturating_sub(1)));
    let mut state = ListState::default().with_selected(selection);
    frame.render_stateful_widget(
        List::new(discoveries)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Discoveries ")
                    .border_style(Style::default().fg(app.theme.border)),
            )
            .highlight_style(Style::default().bg(app.theme.selection_bg).fg(app.theme.selection_fg))
            .highlight_symbol("› "),
        columns[0],
        &mut state,
    );
    let checks = match app.gather.checks.is_empty() {
        | true => vec![Line::from(Span::styled(" No failures", Style::default().fg(app.theme.success)))],
        | false => app
            .gather
            .checks
            .iter()
            .map(|check| {
                Line::from(vec![
                    Span::styled(format!(" {} ", check.severity), Style::default().fg(app.theme.error)),
                    Span::styled(check.message.clone(), Style::default().fg(app.theme.text)),
                ])
            })
            .collect(),
    };
    frame.render_widget(
        Paragraph::new(checks).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Checks ")
                .border_style(Style::default().fg(app.theme.border)),
        ),
        columns[1],
    );
}
fn render_controls(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(
        Paragraph::new(" [Enter] Gather  ·  [↑↓] Select  ·  [Ctrl+L] Clear  ·  [Esc] Back  ·  [Ctrl+C] Quit ")
            .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(app.theme.border)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(app.theme.text_muted)),
        area,
    );
}
pub fn handle_event(app: &mut App, event: Event) {
    match event {
        | Event::Key(key) if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat => match key.code {
            | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
            | KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.gather = GatherState::new();
            }
            | KeyCode::Char(character) => app.gather.input.push(character),
            | KeyCode::Backspace => {
                app.gather.input.pop();
            }
            | KeyCode::Down => {
                app.gather.selected_index = next_index(app.gather.selected_index, app.gather.discoveries.len());
            }
            | KeyCode::Enter => {
                let output = gather(&app.gather.input, &app.options);
                app.gather.checks = output.checks;
                app.gather.discoveries = output.discoveries;
                app.gather.input_count = output.input_count;
                app.gather.selected_index = 0;
            }
            | KeyCode::Esc => app.navigate_to(Screen::Dashboard),
            | KeyCode::Up => {
                app.gather.selected_index = previous_index(app.gather.selected_index, app.gather.discoveries.len());
            }
            | _ => {}
        },
        | _ => {}
    }
}
fn gather(input: &str, options: &TuiOptions) -> GatherOutput {
    let loaded = load_input(input.trim(), options.offline);
    let input_count = loaded.len();
    let checks = loaded
        .iter()
        .filter_map(|value| value.as_ref().err().map(|check| check.as_ref().clone()))
        .collect::<Vec<_>>();
    let sources = loaded.into_iter().filter_map(core::result::Result::ok).collect::<Vec<_>>();
    let discoveries = sources
        .iter()
        .flat_map(|source| {
            Identifier::find_all(&source.content)
                .into_iter()
                .map(|identifier| GatherDiscovery {
                    identifier: identifier.value,
                    identifier_type: identifier.kind.as_str().to_string(),
                    source: source.source.clone(),
                    source_format: source.format.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let persistence_checks = persist(&discoveries, options);
    GatherOutput {
        checks: checks.into_iter().chain(persistence_checks).collect(),
        discoveries,
        input_count,
    }
}
fn load_input(input: &str, offline: bool) -> Vec<core::result::Result<SourceDocument, Box<Check>>> {
    input_values(input).iter().flat_map(|value| load_value(value, offline)).collect()
}
fn input_values(input: &str) -> Vec<String> {
    let parsed = shell_words::split(input).unwrap_or_default();
    match parsed.len() > 1 && parsed.iter().all(|value| is_explicit_input(value)) {
        | true => parsed,
        | false => vec![input.to_string()],
    }
}
fn is_explicit_input(input: &str) -> bool {
    uri_to_path(input).exists() || input.starts_with("http://") || input.starts_with("https://") || !Identifier::find_all(input).is_empty()
}
fn load_value(input: &str, offline: bool) -> Vec<core::result::Result<SourceDocument, Box<Check>>> {
    let path = uri_to_path(input);
    let remote = input.starts_with("http://") || input.starts_with("https://");
    let persistent = !Identifier::find_all(input).is_empty();
    match (input.is_empty(), path.exists(), remote, persistent, offline) {
        | (true, _, _, _, _) => vec![Err(Box::new(failure("Input cannot be empty", "<input>")))],
        | (false, true, _, _, _) => files_all(path, None).into_iter().filter(|path| path.is_file()).map(load_path).collect(),
        | (false, false, _, true, _) => vec![Ok(text_source(input, "pid"))],
        | (false, false, true, false, true) => vec![Err(Box::new(failure("Remote input is unavailable in offline mode", input)))],
        | (false, false, true, false, false) => vec![load_remote(input)],
        | (false, false, false, false, _) => vec![Ok(text_source(input, "text"))],
    }
}
fn load_path(path: PathBuf) -> core::result::Result<SourceDocument, Box<Check>> {
    SourceDocument::from_path(path.clone()).map_err(|why| Box::new(failure(&why.to_string(), &path.display().to_string())))
}
fn load_remote(input: &str) -> core::result::Result<SourceDocument, Box<Check>> {
    reqwest::blocking::get(input)
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(|response| response.bytes())
        .map_err(|why| Box::new(failure(&why.to_string(), input)))
        .and_then(|bytes| SourceDocument::from_bytes(&bytes, input).map_err(|why| Box::new(failure(&why.to_string(), input))))
}
fn persist(discoveries: &[GatherDiscovery], options: &TuiOptions) -> Vec<Check> {
    match options.no_local_database {
        | true => Vec::new(),
        | false => {
            let database = Database::<Table>::from_path(options.database_path.clone());
            database
                .migrate_table(Table::Discoveries)
                .map(|_| {
                    let discovered_at = Timestamp::now();
                    discoveries
                        .iter()
                        .filter_map(|discovery| {
                            database
                                .insert(
                                    IdentifierRow::init()
                                        .discovered_at(discovered_at)
                                        .identifier(discovery.identifier.clone())
                                        .identifier_type(discovery.identifier_type.clone())
                                        .resolution_status("not-requested")
                                        .source(discovery.source.clone())
                                        .source_format(discovery.source_format.clone())
                                        .build(),
                                )
                                .err()
                                .map(|why| failure(&why.to_string(), &discovery.source))
                        })
                        .collect()
                })
                .unwrap_or_else(|why| vec![failure(&why.to_string(), "<database>")])
        }
    }
}
fn failure(message: &str, source: &str) -> Check {
    check_err!(CheckCategory::Schema, message: message.to_string(), uri: source.to_string())
}
fn next_index(index: usize, length: usize) -> usize {
    match length {
        | 0 => 0,
        | _ => index.saturating_add(1) % length,
    }
}
fn previous_index(index: usize, length: usize) -> usize {
    match (index, length) {
        | (_, 0) => 0,
        | (0, _) => length.saturating_sub(1),
        | _ => index.saturating_sub(1),
    }
}
fn text_source(input: &str, format: &str) -> SourceDocument {
    let source = match format {
        | "pid" => input.to_string(),
        | _ => "<text>".to_string(),
    };
    SourceDocument::init().content(input).format(format).source(source).build()
}
#[cfg(test)]
mod tests {
    use super::*;
    use acorn::io::database::Row;
    use acorn::io::standard_project_folder;
    use acorn::prelude::{remove_file, temp_dir};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
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
    fn disabled_database() -> TuiOptions {
        TuiOptions {
            no_local_database: true,
            ..TuiOptions::default()
        }
    }
}
