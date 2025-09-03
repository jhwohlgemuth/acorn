use acorn_lib::analyzer::readability::ReadabilityType;
use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::cli::{paths_from_options, Check, Options};
use acorn_lib::util::{print_values_as_table, to_string, Label};
use color_eyre::eyre::{Report, Result};
use dotenvy::dotenv;
use std::path::PathBuf;
use tracing::{error, warn};

const READABILITY_METRIC: &str = "READABILITY_METRIC";

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    ignore: &Option<String>,
    skip: &[Check],
    disable_website_checks: &bool,
    exit_on_first_error: &bool,
    merge_request: &bool,
    offline: &bool,
    readability_metric: &ReadabilityType,
) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
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
        let issues = ResearchActivity::check(paths.clone(), *offline || *disable_website_checks);
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
        let issues = ResearchActivity::analyze(paths.clone(), *offline);
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
        let metric = match dotenv() {
            | Ok(_) => match dotenvy::var(READABILITY_METRIC) {
                | Ok(metric) if !metric.is_empty() => ReadabilityType::from_string(&metric),
                | _ => *readability_metric,
            },
            | _ => *readability_metric,
        };
        let issues = ResearchActivity::calculate_readability(paths, metric);
        if issues > 0 && options.clone().exit_on_first_error {
            error!("=> {} Found {} readability issues", Label::fail(), issues);
            std::process::exit(exitcode::DATAERR);
        }
        issues
    };
    if validation_issues + analysis_issues + readability_issues > 0 {
        let headers = vec!["Issue Type", "Issue Count"];
        let rows = vec![
            to_string(vec!["Schema Validation", &validation_issues.to_string()]),
            to_string(vec!["Prose Analysis", &analysis_issues.to_string()]),
            to_string(vec!["Conventions", &conventions_issues.to_string()]),
            to_string(vec!["Readability", &readability_issues.to_string()]),
        ];
        println!("\n");
        print_values_as_table(&Label::fmt_fail("Issues Overview"), headers, rows);
        std::process::exit(exitcode::DATAERR);
    }
    Ok(())
}
