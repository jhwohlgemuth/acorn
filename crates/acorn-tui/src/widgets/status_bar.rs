#![allow(dead_code)]
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render a bottom status bar with keybinding hints
pub fn render_status_bar(f: &mut Frame, area: Rect, hints: &[(char, &str)]) {
    let mut spans = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(format!("[{key}] {desc}"), Style::default().fg(Color::DarkGray)));
    }
    let paragraph = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
