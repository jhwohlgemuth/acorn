use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::Screen;
use acorn::util::constants::app::LOGO;
use crossterm::event;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

pub fn render(f: &mut Frame, _app: &App) {
    let logo_width = LOGO.lines().map(|l| l.width()).max().unwrap_or(0) as u16;
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
    render_logo(f, chunks[0], _app);
    let menu_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Min(logo_width), Constraint::Fill(1)])
        .split(chunks[2])[1];
    render_menu(f, menu_area, _app);
    render_controls(f, chunks[4], _app);
}
fn split_acorn_line(line: &str) -> (&str, &str) {
    let split = line
        .char_indices()
        .find(|(_, c)| matches!(*c, '█' | '▒' | '▓' | '▌' | '▐' | '▀' | '▄' | '░'))
        .map(|(pos, _)| pos)
        .unwrap_or(line.len());
    line.split_at(split)
}
fn render_logo(f: &mut Frame, area: Rect, app: &App) {
    use unicode_width::UnicodeWidthStr;
    let raw_lines: Vec<&str> = LOGO.lines().collect();
    let max_width = raw_lines.iter().map(|l| l.width()).max().unwrap_or(0);
    let accent = app.theme.accent;
    let muted = app.theme.text_muted;
    let styled_lines: Vec<Line> = raw_lines
        .into_iter()
        .map(|line| {
            let (braille_part, text_part) = split_acorn_line(line);
            let braille_width = braille_part.width();
            let text_width = text_part.width();
            let pad = max_width.saturating_sub(braille_width + text_width);
            let mut spans = vec![Span::styled(
                braille_part.to_string(),
                Style::default().fg(muted).add_modifier(Modifier::BOLD),
            )];
            if !text_part.is_empty() {
                spans.push(Span::styled(
                    text_part.to_string(),
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ));
            }
            if pad > 0 {
                spans.push(Span::raw(" ".repeat(pad)));
            }
            Line::from(spans)
        })
        .collect();
    let paragraph = Paragraph::new(styled_lines)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
fn render_menu(f: &mut Frame, area: Rect, app: &App) {
    let menu_items = vec![
        Line::from(Span::styled(
            "  [1] Doctor   - System diagnostics & fixes",
            Style::default().fg(app.theme.accent),
        )),
        Line::from(Span::styled(
            "  [2] Check    - Interactive check browser",
            Style::default().fg(app.theme.text_muted),
        )),
    ];
    let paragraph = Paragraph::new(menu_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Menu ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(app.theme.border)),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}
fn render_controls(f: &mut Frame, area: Rect, app: &App) {
    let controls = Line::from(vec![
        Span::styled(" [1/2] Select  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[Q] Quit  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[T] Theme", Style::default().fg(app.theme.text_muted)),
    ]);
    let paragraph = Paragraph::new(controls)
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(app.theme.border)))
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
