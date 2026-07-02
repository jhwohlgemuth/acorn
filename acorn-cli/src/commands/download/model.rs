use crate::cli::Void;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;

/// Download model weights for use with acorn research harnesses and local inference
pub async fn run(
    _model: &[String],
    _filter: &[String],
    _ignore: &[String],
    _output: &Option<PathBuf>,
    _database_path: &Option<PathBuf>,
    _verbose: &Verbosity,
) -> Void {
    Ok(())
}
