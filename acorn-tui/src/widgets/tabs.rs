#![allow(dead_code)]
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render a horizontal tab bar with the given labels and selected index
pub fn render_tabs(f: &mut Frame, area: Rect, labels: &[&str], selected: usize) {
    let mut spans = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" │ "));
        }
        let style = if i == selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(*label, style));
    }
    let paragraph = Paragraph::new(ratatui::text::Line::from(spans))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
