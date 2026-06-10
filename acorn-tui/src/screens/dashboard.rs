use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::Screen;
use acorn::util::constants::app::LOGO;
use crossterm::event;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, _app: &App) {
    let logo_height = LOGO.lines().count() as u16;
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(logo_height),
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Min(2),
        ])
        .split(area);
    render_logo(f, chunks[0]);
    render_menu(f, chunks[2]);
    render_controls(f, chunks[4]);
}
fn render_logo(f: &mut Frame, area: Rect) {
    use unicode_width::UnicodeWidthStr;
    let raw_lines: Vec<&str> = LOGO.lines().collect();
    let max_width = raw_lines.iter().map(|l| l.width()).max().unwrap_or(0);
    let styled_lines: Vec<Line> = raw_lines
        .into_iter()
        .map(|line| {
            let pad = max_width.saturating_sub(line.width());
            let padded = format!("{line}{}", " ".repeat(pad));
            Line::from(Span::styled(padded, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        })
        .collect();
    let paragraph = Paragraph::new(styled_lines)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
fn render_menu(f: &mut Frame, area: Rect) {
    let menu_items = vec![
        Line::from(Span::styled(
            "  [1] Doctor   - System diagnostics & fixes",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "  [2] Check    - Interactive check browser",
            Style::default().fg(Color::Yellow),
        )),
    ];
    let paragraph = Paragraph::new(menu_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Menu ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::White)),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}
fn render_controls(f: &mut Frame, area: Rect) {
    let controls = Line::from(vec![
        Span::styled(" [1/2] Select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Q] Quit", Style::default().fg(Color::DarkGray)),
    ]);
    let paragraph = Paragraph::new(controls)
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
pub fn handle_event(app: &mut App, evt: event::Event) {
    let event = match evt {
        | event::Event::Key(key) => to_internal_event(key),
        | _ => None,
    };
    let Some(event) = event else {
        return;
    };
    match event {
        | Event::Quit => app.should_quit = true,
        | Event::Number(1) => app.navigate_to(Screen::Doctor),
        | Event::Number(2) => app.navigate_to(Screen::Check),
        | Event::Enter => {
            app.navigate_to(Screen::Doctor);
        }
        | _ => {}
    }
}
