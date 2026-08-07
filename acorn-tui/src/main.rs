use acorn_tui::{run, Screen};

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    run(Screen::Dashboard)
}
