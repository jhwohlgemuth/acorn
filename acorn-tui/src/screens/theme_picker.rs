use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::theme::Theme;
use crate::widgets::detail::render_detail;
use crate::widgets::list::{list_item, render_list};
use crate::Screen;
use acorn::util::Label;
use crossterm::event;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[0]);
    let items = app
        .theme_picker
        .data
        .names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            list_item(
                name,
                index == app.theme_picker.selected,
                app.theme.text,
                app.theme.selection_fg,
                app.theme.selection_bg,
            )
        })
        .collect::<Vec<_>>();
    render_list(frame, panels[0], &items, "Themes", app.theme.border, 0);
    let preview = app
        .theme_picker
        .data
        .names
        .get(app.theme_picker.selected)
        .and_then(|name| Theme::named(name))
        .unwrap_or_else(|| app.theme.clone());
    let details = vec![
        Line::from(Span::styled(" Theme preview", Style::default().fg(preview.text))),
        Line::from(Span::styled(
            format!(" {}PASS  Operation succeeded", Label::CHECKMARK),
            Style::default().fg(preview.success),
        )),
        Line::from(Span::styled(" ! CAUTION  Review recommended", Style::default().fg(preview.warning))),
        Line::from(Span::styled(" ✗ FAIL  Operation failed", Style::default().fg(preview.error))),
        Line::from(Span::styled(
            " > SELECTED  Current choice",
            Style::default().fg(preview.selection_fg).bg(preview.selection_bg),
        )),
    ];
    render_detail(frame, panels[1], "Preview", details, preview.accent, preview.border);
    let controls = Line::from(vec![
        Span::styled(" [Up/Down or j/k] Navigate  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[Enter] Apply  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[Esc] Cancel", Style::default().fg(app.theme.text_muted)),
    ]);
    frame.render_widget(
        Paragraph::new(controls)
            .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(app.theme.border)))
            .alignment(Alignment::Center),
        chunks[1],
    );
}

pub fn handle_event(app: &mut App, event: event::Event) {
    let event = match event {
        | event::Event::Key(key) => to_internal_event(key),
        | _ => None,
    };
    match event {
        | Some(Event::Up) => app.theme_picker.selected = app.theme_picker.selected.saturating_sub(1),
        | Some(Event::Down) => {
            app.theme_picker.selected = app
                .theme_picker
                .selected
                .saturating_add(1)
                .min(app.theme_picker.data.names.len().saturating_sub(1));
        }
        | Some(Event::Enter) => {
            if let Some(name) = app.theme_picker.data.names.get(app.theme_picker.selected).copied() {
                app.set_theme(name);
            }
            app.navigate_to(Screen::Dashboard);
        }
        | Some(Event::Back | Event::Quit) => app.navigate_to(Screen::Dashboard),
        | _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::handle_event;
    use crate::app::App;
    use crate::Screen;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_enter_applies_selected_theme() {
        let mut app = App::new(Screen::ThemePicker);
        handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
        assert_eq!(app.theme.name, "one-dark");
        assert_eq!(app.current_screen, Screen::Dashboard);
    }
    #[test]
    fn test_escape_returns_to_dashboard_without_changing_theme() {
        let mut app = App::new(Screen::ThemePicker);
        let original = app.theme.name;
        handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert_eq!(app.theme.name, original);
        assert_eq!(app.current_screen, Screen::Dashboard);
    }
}
