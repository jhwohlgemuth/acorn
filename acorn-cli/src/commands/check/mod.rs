use acorn_lib::analyzer::readability::ReadabilityType;
use acorn_lib::analyzer::Check;
use acorn_lib::schema::ResearchActivity;
use acorn_lib::util::cli::{paths_from_options, CheckCategory, Options};
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
    skip: &[CheckCategory],
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
    let conventions_issues = if skip.contains(&CheckCategory::Conventions) {
        warn!("=> {} Conventions verification", Label::skip());
        0
    } else {
        0
    };
    let schema_issues = if skip.contains(&CheckCategory::Schema) {
        warn!("=> {} Schema validation", Label::skip());
        0
    } else {
        let issues = ResearchActivity::check(paths.clone(), *offline || *disable_website_checks);
        let count = issues
            .into_iter()
            .inspect(|issue| issue.clone().print())
            .filter(|Check { success, .. }| !success)
            .map(|issue| issue.issue_count())
            .sum();
        if count > 0 && options.clone().exit_on_first_error {
            error!("=> {} Found {} schema validation errors", Label::fail(), count);
            std::process::exit(exitcode::DATAERR);
        }
        count
    };
    let prose_issues = if skip.contains(&CheckCategory::Prose) {
        warn!("=> {} Prose analysis", Label::skip());
        0
    } else {
        let issues = ResearchActivity::analyze_prose(paths.clone(), *offline);
        let count = issues
            .into_iter()
            .inspect(|issue| issue.clone().print())
            .filter(|Check { success, .. }| !success)
            .map(|issue| issue.issue_count())
            .sum();
        if count > 0 && options.exit_on_first_error {
            error!("=> {} Found {} prose issues", Label::fail(), count);
            std::process::exit(exitcode::DATAERR);
        }
        count
    };
    let readability_issues = if skip.contains(&CheckCategory::Readability) {
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
        let count = issues
            .into_iter()
            .inspect(|issue| issue.clone().print())
            .filter(|Check { success, .. }| !success)
            .map(|issue| issue.issue_count())
            .sum();
        if count > 0 && options.clone().exit_on_first_error {
            error!("=> {} Found {} readability issues", Label::fail(), count);
            std::process::exit(exitcode::DATAERR);
        }
        count
    };
    if schema_issues + prose_issues + readability_issues > 0 {
        let headers = vec!["Issue Type", "Issue Count"];
        let rows = vec![
            to_string(vec!["Schema Validation", &schema_issues.to_string()]),
            to_string(vec!["Prose Analysis", &prose_issues.to_string()]),
            to_string(vec!["Conventions", &conventions_issues.to_string()]),
            to_string(vec!["Readability", &readability_issues.to_string()]),
        ];
        println!("\n");
        print_values_as_table(&Label::fmt_fail("Issues Overview"), headers, rows);
        std::process::exit(exitcode::DATAERR);
    }
    Ok(())
}
