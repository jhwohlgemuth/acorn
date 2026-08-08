use crate::app::App;
use crate::event::{to_internal_event, Event};
use crate::widgets::detail::{kv_line, render_detail};
use crate::widgets::list::{list_item, render_list};
use crossterm::event;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[0]);
    let Some(state) = app.gguf_picker.as_ref() else {
        return;
    };
    let items = state
        .data
        .candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let likes = candidate.likes.map_or_else(|| "-".to_string(), |value| value.to_string());
            list_item(
                format!("{}  {} downloads  {} likes", candidate.id, candidate.downloads, likes).as_str(),
                index == state.selected,
                app.theme.accent,
                app.theme.selection_fg,
                app.theme.selection_bg,
            )
        })
        .collect::<Vec<_>>();
    let visible_items = panels[0].height.saturating_sub(2);
    let offset = u16::try_from(state.selected.saturating_sub(usize::from(visible_items.saturating_sub(1)))).unwrap_or(u16::MAX);
    render_list(
        frame,
        panels[0],
        &items,
        format!(" GGUF repositories for {} ", state.data.base_model).as_str(),
        app.theme.border,
        offset,
    );
    let detail = state.data.candidates.get(state.selected).map_or_else(
        || vec![Line::from(" No candidates available")],
        |candidate| {
            let quantizations = if candidate.quantizations.is_empty() {
                "Unknown".to_string()
            } else {
                candidate.quantizations.join(", ")
            };
            vec![
                kv_line("Repository", candidate.id.as_str(), app.theme.text_muted, app.theme.text),
                kv_line(
                    "Downloads",
                    candidate.downloads.to_string().as_str(),
                    app.theme.text_muted,
                    app.theme.text,
                ),
                kv_line(
                    "Likes",
                    candidate.likes.map_or_else(|| "Unknown".to_string(), |value| value.to_string()).as_str(),
                    app.theme.text_muted,
                    app.theme.text,
                ),
                kv_line("Quantizations", quantizations.as_str(), app.theme.text_muted, app.theme.text),
            ]
        },
    );
    render_detail(frame, panels[1], " Selected repository ", detail, app.theme.accent, app.theme.border);
    let controls = Line::from(vec![
        Span::styled(" [Up/Down or j/k] Navigate  ", Style::default().fg(app.theme.text_muted)),
        Span::styled("[Enter] Download  ", Style::default().fg(app.theme.text_muted)),
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
    if let (Some(event), Some(state)) = (event, app.gguf_picker.as_mut()) {
        match event {
            | Event::Up => state.selected = state.selected.saturating_sub(1),
            | Event::Down => state.selected = state.selected.saturating_add(1).min(state.data.candidates.len().saturating_sub(1)),
            | Event::Enter => {
                state.data.result = state.data.candidates.get(state.selected).map(|candidate| candidate.id.clone());
                app.should_quit = true;
            }
            | Event::Back | Event::Quit => app.should_quit = true,
            | _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::handle_event;
    use crate::app::{App, Candidate, GgufPickerData, State};
    use crate::Screen;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    fn app() -> App {
        let mut app = App::new(Screen::GgufPicker);
        app.set_gguf_picker(State::new(GgufPickerData {
            candidates: vec![
                Candidate {
                    id: "one/model".to_string(),
                    downloads: 20,
                    likes: Some(2),
                    quantizations: vec!["Q4_K_M".to_string()],
                },
                Candidate {
                    id: "two/model".to_string(),
                    downloads: 10,
                    likes: Some(1),
                    quantizations: vec!["Q5_K_M".to_string()],
                },
            ],
            base_model: "base/model".to_string(),
            result: None,
        }));
        app
    }
    #[test]
    fn test_enter_returns_selected_repository() {
        let mut app = app();
        handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
        assert!(app.should_quit);
        assert_eq!(app.take_gguf_picker_result().as_deref(), Some("two/model"));
    }
    #[test]
    fn test_escape_cancels_without_result() {
        let mut app = app();
        handle_event(&mut app, Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(app.should_quit);
        assert_eq!(app.take_gguf_picker_result(), None);
    }
    #[test]
    fn test_library_candidate_converts_to_picker_candidate() {
        let candidate = Candidate::from(acorn::io::api::huggingface::Candidate {
            id: "owner/model".to_string(),
            downloads: 42,
            likes: Some(7),
            quantizations: vec!["Q4_K_M".to_string()],
        });
        assert_eq!(candidate.id, "owner/model");
        assert_eq!(candidate.downloads, 42);
    }
}
