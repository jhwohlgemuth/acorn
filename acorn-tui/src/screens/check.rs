use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::Screen;
use crossterm::event;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, _app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
        .split(area);
    render_top_bar(f, chunks[0]);
    render_main(f, chunks[1]);
    render_controls(f, chunks[2]);
}
fn render_top_bar(f: &mut Frame, area: Rect) {
    let tabs = vec![
        Span::styled(" All ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(" Schema ", Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(" Prose ", Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(" Links ", Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(" Readability ", Style::default().fg(Color::White)),
    ];
    let top = Paragraph::new(Line::from(tabs))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center);
    f.render_widget(top, area);
}
fn render_main(f: &mut Frame, area: Rect) {
    let placeholder = vec![
        Line::from(Span::styled(
            " Interactive check browser is under construction",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            " Use 'acorn check' from the command line to run checks",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let paragraph = Paragraph::new(placeholder)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Check Results ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
fn render_controls(f: &mut Frame, area: Rect) {
    let controls = Line::from(vec![
        Span::styled(" [Esc] Back  ", Style::default().fg(Color::DarkGray)),
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
        | Event::Back => app.navigate_to(Screen::Dashboard),
        | _ => {}
    }
}
