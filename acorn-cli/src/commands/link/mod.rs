use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;

pub fn run(
    _path: &Option<PathBuf>,
    _branch: &Option<String>,
    _commit: &Option<String>,
    _ignore: &Option<String>,
    _dry_run: bool,
    _merge_request: bool,
) -> Result<(), Report> {
    Ok(())
}
