use acorn_lib::schema::ResearchActivity;
use color_eyre::eyre::{Report, Result};

#[allow(clippy::too_many_arguments)]
pub fn run() -> Result<(), Report> {
    ResearchActivity::to_schema();
    std::process::exit(exitcode::OK);
}
