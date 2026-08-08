use crate::app::App;
use crate::Screen;
use acorn::util::constants::app::COMPACT_LOGO;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph};
use ratatui::Frame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    Check,
    Doctor,
    Gather,
    Harvest,
    Quit,
    Theme,
}
impl Command {
    const ALL: [Self; 6] = [Self::Check, Self::Doctor, Self::Gather, Self::Harvest, Self::Quit, Self::Theme];
    fn name(self) -> &'static str {
        match self {
            | Self::Check => "/check",
            | Self::Doctor => "/doctor",
            | Self::Gather => "/gather",
            | Self::Harvest => "/harvest",
            | Self::Quit => "/quit",
            | Self::Theme => "/theme",
        }
    }
    fn description(self) -> &'static str {
        match self {
            | Self::Check => "Browse research activity checks",
            | Self::Doctor => "Inspect system diagnostics",
            | Self::Gather => "Discover persistent identifiers",
            | Self::Harvest => "Discover persistent identifiers",
            | Self::Quit => "Exit ACORN",
            | Self::Theme => "Choose an accessible terminal palette",
        }
    }
}
fn filtered_commands(input: &str) -> Vec<Command> {
    let query = input.strip_prefix('/').unwrap_or(input).trim().to_ascii_lowercase();
    Command::ALL
        .into_iter()
        .filter(|command| command.name().trim_start_matches('/').starts_with(&query))
        .collect()
}
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(height.min(area.height)), Constraint::Fill(1)])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(width.min(area.width)), Constraint::Fill(1)])
        .split(vertical[1])[1]
}
pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(app.theme.bg)), area);
    let content = centered(area, 82, 18);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(content);
    let logo = COMPACT_LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD))))
        .collect::<Vec<_>>();
    f.render_widget(Paragraph::new(logo).alignment(Alignment::Center), rows[0]);
    f.render_widget(
        Paragraph::new("Accessible Content Optimization for Research Needs")
            .style(Style::default().fg(app.theme.text_muted))
            .alignment(Alignment::Center),
        rows[1],
    );
    render_composer(f, rows[2], app);
    f.render_widget(
        Paragraph::new("Type / for commands  ·  ↑↓ select  ·  enter run  ·  esc clear")
            .style(Style::default().fg(app.theme.text_muted))
            .alignment(Alignment::Center),
        rows[4],
    );
    if app.dashboard.data.input.starts_with('/') {
        render_dropdown(f, rows[2], app);
    }
}
fn render_composer(f: &mut Frame, area: Rect, app: &App) {
    let elapsed = app.dashboard.data.started_at.elapsed().as_secs();
    let title = format!(
        " acorn-tui v{} · {} · {} ",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        app.theme.name
    );
    let footer = format!(" {} · {:02}:{:02} ", app.dashboard.data.working_directory, elapsed / 60, elapsed % 60);
    let input = if app.dashboard.data.input.is_empty() {
        Span::styled("Ask ACORN or type / for commands", Style::default().fg(app.theme.text_muted))
    } else {
        Span::styled(app.dashboard.data.input.as_str(), Style::default().fg(app.theme.text))
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border))
        .title(Line::from(title).centered())
        .title_bottom(Line::from(footer).centered())
        .padding(Padding::horizontal(2));
    f.render_widget(Paragraph::new(Line::from(input)).block(block), area);
    let cursor_offset = app.dashboard.data.input.chars().count().min(usize::from(area.width.saturating_sub(6))) as u16;
    f.set_cursor_position((area.x.saturating_add(3).saturating_add(cursor_offset), area.y.saturating_add(2)));
}
fn render_dropdown(f: &mut Frame, composer: Rect, app: &App) {
    let commands = filtered_commands(&app.dashboard.data.input);
    let height = u16::try_from(commands.len()).unwrap_or(u16::MAX).saturating_add(2).min(7);
    let y = composer.y.saturating_add(composer.height);
    let available = f.area().bottom().saturating_sub(y);
    let area = Rect::new(composer.x, y, composer.width, height.min(available));
    if area.height < 2 {
        return;
    }
    let items = if commands.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No matching commands",
            Style::default().fg(app.theme.text_muted),
        )))]
    } else {
        commands
            .iter()
            .map(|command| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<18}", command.name()), Style::default().fg(app.theme.accent)),
                    Span::styled(command.description(), Style::default().fg(app.theme.text_muted)),
                ]))
            })
            .collect()
    };
    let mut state =
        ListState::default().with_selected((!commands.is_empty()).then_some(app.dashboard.selected.min(commands.len().saturating_sub(1))));
    f.render_widget(Clear, area);
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(app.theme.border)))
            .highlight_style(
                Style::default()
                    .bg(app.theme.selection_bg)
                    .fg(app.theme.selection_fg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("› "),
        area,
        &mut state,
    );
}
fn execute(app: &mut App, command: Command) {
    match command {
        | Command::Check => app.navigate_to(Screen::Check),
        | Command::Doctor => app.navigate_to(Screen::Doctor),
        | Command::Gather | Command::Harvest => app.navigate_to(Screen::Gather),
        | Command::Quit => app.should_quit = true,
        | Command::Theme => app.navigate_to(Screen::ThemePicker),
    }
    app.dashboard.data.input.clear();
    app.dashboard.selected = 0;
}
pub fn handle_event(app: &mut App, event: Event) {
    let Event::Key(key) = event else { return };
    if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
        return;
    }
    match key.code {
        | KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => app.should_quit = true,
        | KeyCode::Char(character) => {
            app.dashboard.data.input.push(character);
            app.dashboard.selected = 0;
        }
        | KeyCode::Backspace => {
            app.dashboard.data.input.pop();
            app.dashboard.selected = 0;
        }
        | KeyCode::Esc => {
            app.dashboard.data.input.clear();
            app.dashboard.selected = 0;
        }
        | KeyCode::Up if app.dashboard.data.input.starts_with('/') => {
            let len = filtered_commands(&app.dashboard.data.input).len();
            app.dashboard.selected = app.dashboard.selected.checked_sub(1).unwrap_or_else(|| len.saturating_sub(1));
        }
        | KeyCode::Down if app.dashboard.data.input.starts_with('/') => {
            let len = filtered_commands(&app.dashboard.data.input).len();
            app.dashboard.selected = if len == 0 { 0 } else { (app.dashboard.selected + 1) % len };
        }
        | KeyCode::Enter if app.dashboard.data.input.starts_with('/') => {
            if let Some(command) = filtered_commands(&app.dashboard.data.input).get(app.dashboard.selected).copied() {
                execute(app, command);
            }
        }
        | _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_gather_command_opens_gather_screen() {
        let mut app = App::new(Screen::Dashboard);
        execute(&mut app, Command::Gather);
        assert_eq!(app.current_screen, Screen::Gather);
        assert!(app.dashboard.data.input.is_empty());
    }
    #[test]
    fn test_slash_filter_reports_no_unknown_commands() {
        assert!(filtered_commands("/missing").is_empty());
    }
    #[test]
    fn test_slash_filter_updates_for_each_query_fragment() {
        assert_eq!(filtered_commands("/t"), vec![Command::Theme]);
        assert_eq!(filtered_commands("/theme"), vec![Command::Theme]);
        assert_eq!(filtered_commands("/doctor"), vec![Command::Doctor]);
        assert_eq!(filtered_commands("/gather"), vec![Command::Gather]);
        assert_eq!(filtered_commands("/harvest"), vec![Command::Harvest]);
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
        execute(&mut app, Command::Theme);
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
        assert_eq!(filtered_commands(&app.dashboard.data.input), vec![Command::Check]);
    }
}
