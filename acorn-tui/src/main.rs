use acorn_tui::{run_tui, Screen};

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    run_tui(Screen::Dashboard)
}
