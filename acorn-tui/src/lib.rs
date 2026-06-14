mod app;
mod event;
pub mod export;
mod screens;
mod theme;
mod widgets;

use app::App;
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
}
/// Run the TUI with a given start screen
pub fn run_tui(initial_screen: Screen) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut app = App::new(initial_screen);
    let result = app.run(&mut terminal);
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    if let Err(err) = result {
        eprintln!("TUI error: {err}");
    }
    Ok(())
}
