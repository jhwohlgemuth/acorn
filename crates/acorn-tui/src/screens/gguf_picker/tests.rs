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
