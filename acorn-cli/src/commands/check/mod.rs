use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::cli::{paths_from_options, Check, Options};
use acorn_lib::util::{print_values_as_table, to_string, Label};
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;
use tracing::{error, warn};

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &Option<String>,
    skip: &[Check],
    exit_on_first_error: &bool,
    merge_request: &bool,
    offline: &bool,
) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
        unimplemented!("Offline mode is not implemented yet");
    }
    let options = Options::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_ignore(ignore.clone())
        .exit_on_first_error(*exit_on_first_error)
        .merge_request(*merge_request)
        .build();
    let paths = paths_from_options(path, &Some(options.clone()));
    let conventions_issues = if skip.contains(&Check::Conventions) {
        warn!("=> {} Conventions verification", Label::skip());
        0
    } else {
        0
    };
    let validation_issues = if skip.contains(&Check::Validation) {
        warn!("=> {} Schema validation", Label::skip());
        0
    } else {
        let issues = ResearchActivity::check(paths.clone());
        if issues > 0 && options.clone().exit_on_first_error {
            error!("=> {} Found {} schema validation errors", Label::fail(), issues);
            std::process::exit(exitcode::DATAERR);
        }
        issues
    };
    let analysis_issues = if skip.contains(&Check::Analysis) {
        warn!("=> {} Prose analysis", Label::skip());
        0
    } else {
        let issues = ResearchActivity::analyze(paths);
        if issues > 0 && options.exit_on_first_error {
            error!("=> {} Found {} prose issues", Label::fail(), issues);
            std::process::exit(exitcode::DATAERR);
        }
        issues
    };
    let readability_issues = if skip.contains(&Check::Readability) {
        warn!("=> {} Readability analysis", Label::skip());
        0
    } else {
        // TODO: Output readability analysis
        0
    };
    if analysis_issues + validation_issues + readability_issues > 0 {
        let headers = vec!["Issue Type", "Issue Count"];
        let rows = vec![
            to_string(vec!["Schema Validation", &validation_issues.to_string()]),
            to_string(vec!["Prose Analysis", &analysis_issues.to_string()]),
            to_string(vec!["Conventions", &conventions_issues.to_string()]),
            to_string(vec!["Readability", "---"]),
        ];
        println!("\n");
        print_values_as_table(&Label::fmt_fail("Issues Overview"), headers, rows);
        std::process::exit(exitcode::DATAERR);
    }
    Ok(())
}
