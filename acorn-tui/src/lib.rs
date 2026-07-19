mod app;
mod event;
pub mod export;
mod screens;
mod theme;
mod widgets;

use app::App;
pub use app::{Candidate, GgufPickerData, State};
use color_eyre::eyre::{Ok, Result};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;

/// Entry screen for the TUI
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    /// Dashboard hub
    Dashboard,
    /// Interactive diagnostic browser
    Doctor,
    /// Interactive check browser
    Check,
    /// Interactive GGUF repository picker
    GgufPicker,
    /// Interactive terminal theme picker
    ThemePicker,
}
/// Run the TUI with a given start screen
pub fn run_tui(initial_screen: Screen) -> Result<()> {
    run_tui_with(initial_screen, None).map(|_| ())
}
/// Run the TUI with optional GGUF picker state and return the selected repository ID.
pub fn run_tui_with(initial_screen: Screen, initial_state: Option<State<GgufPickerData>>) -> Result<Option<String>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut app = App::new(initial_screen);
    if let Some(state) = initial_state {
        app.set_gguf_picker(state);
    }
    let result = app.run(&mut terminal);
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result?;
    Ok(app.take_gguf_picker_result())
}
