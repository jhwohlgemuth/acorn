use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::{cli::Check, get_all_files, get_files_from_git, Label};
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;
use tracing::{error, warn};

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: &Option<PathBuf>,
    commit: &Option<String>,
    ignore: &Option<String>,
    skip: &[Check],
    exit_on_first_error: &bool,
) -> Result<(), Report> {
    let extension = Some("json");
    let paths = match commit {
        | Some(hash) => get_files_from_git(hash, extension),
        | None => {
            let value = match path {
                | Some(x) => x.clone(),
                | None => PathBuf::from("."),
            };
            get_all_files(value, extension)
        }
    };
    let mut total: usize = 0;
    let mut success: bool = true;
    if skip.contains(&Check::Validation) {
        warn!("=> {} Schema validation", Label::skip());
    } else {
        total = ResearchActivity::check(paths.clone());
        if total > 0 {
            error!("=> {} Found {} schema errors in total", Label::fail(), total);
            if *exit_on_first_error {
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
    if skip.contains(&Check::Analysis) {
        warn!("=> {} Prose analysis", Label::skip());
    } else {
        // TODO: Use JSON output to create customized and consolidated error count and output
        success = ResearchActivity::analyze(paths);
    }
    if skip.contains(&Check::Readability) {
        warn!("=> {} Readability analysis", Label::skip());
    } else {
        // TODO: Output readability analysis
    }
    if total > 0 || !success {
        std::process::exit(exitcode::DATAERR);
    }
    Ok(())
}
