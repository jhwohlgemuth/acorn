use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

pub async fn run(
    config: &Option<PathBuf>,
    _description: &Option<String>,
    _group: &Option<u64>,
    _project: &Option<u64>,
    _tags: &Vec<String>,
    _verbose: &Verbosity,
) -> Result<(), Report> {
    dbg!(&config);
    Ok(())
}
