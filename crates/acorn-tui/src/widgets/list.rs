#![allow(dead_code)]
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render a scrollable list of items
pub fn render_list(f: &mut Frame, area: Rect, items: &[Line], title: &str, border_color: Color, offset: u16) {
    let paragraph = Paragraph::new(items.to_vec())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {title} "))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(border_color)),
        )
        .alignment(Alignment::Left)
        .scroll((offset, 0));
    f.render_widget(paragraph, area);
}
/// Create a styled line for a list item
pub fn list_item(text: &str, is_selected: bool, color: Color, selection_fg: Color, selection_bg: Color) -> Line<'static> {
    let prefix = if is_selected { ">" } else { " " };
    let style = if is_selected {
        Style::default().fg(selection_fg).bg(selection_bg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };
    Line::from(Span::styled(format!(" {prefix} {text}"), style))
}
