#![allow(dead_code)]
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render a detail pane with key-value rows
pub fn render_detail(f: &mut Frame, area: Rect, title: &str, lines: Vec<Line>, _accent: Color, border: Color) {
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {title} "))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(border)),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}
/// Create a key-value line for detail pane
pub fn kv_line(key: &str, value: &str, key_color: Color, value_color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {key}: "), Style::default().fg(key_color)),
        Span::styled(value.to_string(), Style::default().fg(value_color)),
    ])
}
