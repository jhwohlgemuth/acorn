use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::{cli::Check, get_all_files, Label};
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;
use tracing::{error, warn};

#[allow(clippy::too_many_arguments)]
pub fn run(path: &Option<PathBuf>, skip: &[Check]) -> Result<(), Report> {
    if let Some(path) = path {
        let paths = get_all_files(path.into(), None);
        if skip.contains(&Check::Validation) {
            warn!("=> {} Schema validation", Label::skip());
        } else {
            let total: usize = ResearchActivity::check(paths.clone());
            if total > 0 {
                error!("=> {} Found {} schema errors in total", Label::fail(), total);
            }
        }
        if skip.contains(&Check::Analysis) {
            warn!("=> {} Prose analysis", Label::skip());
        } else {
            // TODO: Use JSON output to create customized and consolidated error count and output
            ResearchActivity::analyze(paths);
        }
    }
    Ok(())
}
