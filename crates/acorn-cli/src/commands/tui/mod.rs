use crate::cli::Void;
#[cfg(feature = "tui")]
use acorn_schema::vocabulary::VocabularyFuzzyMode;
#[cfg(not(feature = "tui"))]
use color_eyre::eyre;
use std::path::PathBuf;
#[cfg(feature = "tui")]
pub fn run(database_path: &Option<PathBuf>, no_local_database: bool, offline: bool) -> Void {
    acorn_tui::run_with_options(
        acorn_tui::Screen::Dashboard,
        None,
        acorn_tui::TuiOptions {
            database_path: database_path.clone(),
            doctor_fix: false,
            no_local_database,
            offline,
            vocabulary_fuzzy: VocabularyFuzzyMode::Relaxed,
        },
    )
    .map(|_| ())
}
#[cfg(not(feature = "tui"))]
pub fn run(_database_path: &Option<PathBuf>, _no_local_database: bool, _offline: bool) -> Void {
    Err(eyre::eyre!("TUI feature not enabled. Build with: cargo build --features tui"))
}
