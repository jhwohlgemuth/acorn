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
        citation_format,
        database_path,
        format,
        no_local_database,
        output,
        quiet,
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
    let (candidate_results, persistence_checks) = if no_local_database {
        (Vec::new(), Vec::new())
    } else {
        let history_checks = discoveries.persist(database_path);
        let (candidate_results, candidate_checks) = discoveries.persist_candidates(database_path);
        (candidate_results, history_checks.into_iter().chain(candidate_checks).collect())
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
    let report = Report::new(&checks, discoveries, inputs)
        .with_candidates(candidate_results)
        .with_citation_format(citation_format);
    let format = format.unwrap_or_else(|| match output.is_none() && !is_stdout_piped() {
        | true => OutputFormat::Console,
        | false => OutputFormat::Json,
    });
    let failures = checks.iter().filter(|check| check.is_failure()).collect::<Vec<_>>();
    match format {
        | OutputFormat::Raw => failures.iter().for_each(|check| eprintln!("{check}")),
        | OutputFormat::Console => failures
            .into_iter()
            .enumerate()
            .for_each(|(index, check)| check.clone().with_index(index).report()),
        | _ => {}
    }
    let output_write_result = match output {
        | Some(path) => report.serialize(format).and_then(|serialized| write_file(path.clone(), serialized)),
        | None if quiet => Ok(()),
        | None => report.serialize(format).map(|serialized| println!("{serialized}")),
    };
    output_write_result.and_then(|_| match checks.iter().any(Check::is_failure) {
        | true => Err(eyre!("ACORN gather found one or more failures")),
        | false => Ok(()),
    })
}

#[cfg(test)]
mod tests;
