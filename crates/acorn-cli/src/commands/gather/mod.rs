use crate::io::{is_stdout_piped, read_stdin};
use acorn::analyzer::discovery::{Options, OutputFormat, Records, Report};
use acorn::analyzer::{Analysis, Check, CheckCategory};
use acorn::check_err;
use acorn::io::config::FilterSet;
use acorn::io::document::SourceDocument;
use acorn::io::{files_all_with_max_depth, files_from_gitlab_merge_request, uri_to_path, write_file, ApiResult};
use acorn::prelude::remove_file;
use color_eyre::eyre::eyre;
use futures::future::join_all;

pub async fn run(options: Options<'_>) -> ApiResult<()> {
    match options.remote {
        | Some(remote) => remote.run(options).await,
        | None => {
            let Options {
                filter,
                ignore,
                input,
                max_depth,
                merge_request,
                offline,
                resolve,
                text,
                ..
            } = options;
            match (offline && resolve, offline && merge_request) {
                | (true, _) => Err(eyre!("--resolve cannot be used with --offline")),
                | (_, true) => Err(eyre!("--merge-request cannot be used with --offline")),
                | (false, false) => {
                    let stdin = read_stdin().filter(|content| !content.trim().is_empty());
                    match (input.iter().any(|value| value == "-"), stdin) {
                        | (true, None) => Err(eyre!("No content received from stdin for '-'")),
                        | (_, stdin) => {
                            let content_supplied = !text.is_empty() || stdin.is_some();
                            let inputs = match (merge_request, input.is_empty(), content_supplied) {
                                | (true, _, _) => files_from_gitlab_merge_request(None)
                                    .await
                                    .into_iter()
                                    .map(|path| path.display().to_string())
                                    .collect(),
                                | (false, true, false) => vec![".".to_string()],
                                | (false, _, _) => input.iter().filter(|value| value.as_str() != "-").cloned().collect(),
                            };
                            let items = text
                                .iter()
                                .enumerate()
                                .map(|(index, content)| {
                                    Ok(SourceDocument::init()
                                        .content(content)
                                        .format("text")
                                        .source(format!("<text:{}>", index.saturating_add(1)))
                                        .build())
                                })
                                .chain(stdin.map(|content| Ok(SourceDocument::init().content(content).format("text").source("<stdin>").build())))
                                .collect::<Vec<_>>();
                            match FilterSet::compile(ignore.as_slice(), filter.as_slice()) {
                                | Ok(filters) => {
                                    let expanded = expand(&inputs, &filters, max_depth);
                                    let loaded = join_all(expanded.iter().map(|value| SourceDocument::load(value.as_str(), offline))).await;
                                    process(items.into_iter().chain(loaded).collect(), options).await
                                }
                                | Err(why) => Err(why),
                            }
                        }
                    }
                }
            }
        }
    }
}
fn expand(inputs: &[String], filters: &FilterSet, max_depth: Option<usize>) -> Vec<String> {
    inputs
        .iter()
        .flat_map(|value| {
            let path = uri_to_path(value);
            match path.exists() {
                | true => files_all_with_max_depth(path, None, max_depth)
                    .into_iter()
                    .filter(|path| path.is_file())
                    .map(|path| path.display().to_string())
                    .collect(),
                | false => vec![value.clone()],
            }
        })
        .filter(|value| filters.matches(value))
        .collect()
}
async fn process(values: Vec<ApiResult<SourceDocument>>, options: Options<'_>) -> ApiResult<()> {
    let Options {
        database_path,
        format,
        no_local_database,
        output,
        resolve,
        ..
    } = options;
    let load_checks = values
        .iter()
        .filter_map(|value| {
            value
                .as_ref()
                .err()
                .map(|why| check_err!(CheckCategory::Schema, message: why.to_string()))
        })
        .collect::<Vec<_>>();
    let sources = values.into_iter().filter_map(core::result::Result::ok).collect::<Vec<_>>();
    let inputs = sources.len();
    let discoveries = Records::from(sources);
    let discoveries = match resolve {
        | true => discoveries.resolve().await,
        | false => discoveries,
    };
    let persistence_checks = if no_local_database {
        Vec::new()
    } else {
        discoveries.persist(database_path)
    };
    let (analysis_checks, temporary_paths) = discoveries.analyze(options).await;
    let checks = load_checks
        .into_iter()
        .chain(discoveries.check_resolution())
        .chain(analysis_checks)
        .chain(persistence_checks)
        .collect::<Vec<_>>();
    temporary_paths.iter().for_each(|path| {
        let _ = remove_file(path);
    });
    let report = Report::new(&checks, discoveries, inputs);
    let format = format.unwrap_or_else(|| match output.is_none() && !is_stdout_piped() {
        | true => OutputFormat::Console,
        | false => OutputFormat::Json,
    });
    match output {
        | Some(path) => report.serialize(format).and_then(|serialized| write_file(path.clone(), serialized)),
        | None => report.serialize(format).map(|serialized| println!("{serialized}")),
    }
    .and_then(|_| match checks.iter().any(Check::is_failure) {
        | true => Err(eyre!("ACORN gather found one or more failures")),
        | false => Ok(()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use acorn::prelude::PathBuf;
    use acorn::Location;
    use assert_cmd::Command;

    #[test]
    fn test_explicit_empty_stdin_fails() {
        let output = Command::cargo_bin("acorn")
            .unwrap()
            .args(["--offline", "--no-local-database", "gather", "-"])
            .write_stdin("")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("No content received from stdin for '-'"));
    }
    #[test]
    fn test_filter_and_ignore_patterns_compose() {
        let values = vec!["keep.md".to_string(), "skip.md".to_string(), "skip.txt".to_string()];
        let filters = FilterSet::compile(&["skip".to_string()], &[r"[.]md$".to_string()]).unwrap();
        assert_eq!(expand(&values, &filters, None), vec!["keep.md".to_string()]);
    }
    #[tokio::test]
    async fn test_load_accepts_location() {
        let document = SourceDocument::load(Location::from("10.1234/example"), true).await.unwrap();
        assert_eq!(document.format, "pid");
    }
    #[test]
    fn test_max_depth_limits_directory_expansion() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/gather/depth");
        let input = vec![root.display().to_string()];
        let filters = FilterSet::compile(&[], &[]).unwrap();
        assert!(expand(&input, &filters, Some(0)).is_empty());
        assert_eq!(expand(&input, &filters, Some(1)).len(), 1);
        assert_eq!(expand(&input, &filters, Some(2)).len(), 2);
        assert_eq!(expand(&input, &filters, None).len(), 3);
    }
    #[test]
    fn test_piped_stdin_does_not_require_dash() {
        let output = Command::cargo_bin("acorn")
            .unwrap()
            .args(["--offline", "--no-local-database", "gather", "--format", "json"])
            .write_stdin("doi:10.1234/example")
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let report = String::from_utf8_lossy(&output.stdout);
        assert!(report.contains(r#""identifier": "10.1234/example""#));
        assert!(report.contains(r#""source": "<stdin>""#));
    }
    #[test]
    fn test_console_format_renders_current_gather_results() {
        let output = Command::cargo_bin("acorn")
            .unwrap()
            .args([
                "--offline",
                "--no-local-database",
                "gather",
                "--text",
                "doi:10.1234/example",
                "--format",
                "console",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let report = String::from_utf8_lossy(&output.stdout);
        assert!(report.contains("ACORN gather:"));
        assert!(report.contains("10.1234/example"));
    }
    #[test]
    fn test_osti_rejects_offline_mode_before_network_access() {
        let output = Command::cargo_bin("acorn")
            .unwrap()
            .args(["--offline", "--no-local-database", "gather", "--osti", "projects", "ACORN"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("--osti cannot be used with --offline"));
    }
    #[test]
    fn test_osti_conflicts_with_file_analysis_options() {
        let output = Command::cargo_bin("acorn")
            .unwrap()
            .args(["--no-local-database", "gather", "--osti", "projects", "ACORN", "--resolve"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
    }
}
