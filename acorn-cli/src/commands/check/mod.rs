use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::{cli::Check, get_all_files, get_files_from_git, print_values_as_table, to_string, Label};
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;
use tracing::{error, warn};

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: &Option<PathBuf>,
    commit: &Option<String>,
    _ignore: &Option<String>,
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
    let validation_issues = if skip.contains(&Check::Validation) {
        warn!("=> {} Schema validation", Label::skip());
        0
    } else {
        let issues = ResearchActivity::check(paths.clone());
        if issues > 0 {
            if *exit_on_first_error {
                error!("=> {} Found {} schema issues", Label::fail(), issues);
                std::process::exit(exitcode::DATAERR);
            }
            issues
        } else {
            0
        }
    };
    let analysis_issues = if skip.contains(&Check::Analysis) {
        warn!("=> {} Prose analysis", Label::skip());
        0
    } else {
        let issues = ResearchActivity::analyze(paths);
        if issues > 0 {
            if *exit_on_first_error {
                error!("=> {} Found {} prose issues", Label::fail(), issues);
                std::process::exit(exitcode::DATAERR);
            } else {
                issues
            }
        } else {
            0
        }
    };
    let readability_issues = if skip.contains(&Check::Readability) {
        warn!("=> {} Readability analysis", Label::skip());
        0
    } else {
        // TODO: Output readability analysis
        0
    };
    let total: usize = analysis_issues + validation_issues + readability_issues;
    if total > 0 {
        let headers = vec!["Issue Type", "Issue Count"];
        let rows = vec![
            to_string(vec!["Schema Validation", &validation_issues.to_string()]),
            to_string(vec!["Prose Analysis", &analysis_issues.to_string()]),
            to_string(vec!["Readability", "---"]),
        ];
        println!("\n");
        print_values_as_table(&Label::fmt_fail("Issues Overview"), headers, rows);
        std::process::exit(exitcode::DATAERR);
    }
    Ok(())
}
