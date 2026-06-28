use crate::cli::Void;
#[cfg(not(feature = "tui"))]
use color_eyre::eyre;

pub fn run() -> Void {
    #[cfg(feature = "tui")]
    return acorn_tui::run_tui(acorn_tui::Screen::Dashboard);
    #[cfg(not(feature = "tui"))]
    {
        return Err(eyre::eyre!("TUI feature not enabled. Build with: cargo build --features tui"));
    }
}
