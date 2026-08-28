use crate::cli::WatchOptions;
use crate::commands::watch::{self, Prepared};
use crate::io::{is_stdin_piped, is_stdout_piped, read_stdin};
use acorn::analyzer::discovery::{Options, OutputFailure, Records, RemoteSearchResponse, Report};
use acorn::analyzer::{Check, CheckCategory, CheckOptions, OutputFormat, Render};
use acorn::check_err;
use acorn::io::config::FilterSet;
use acorn::io::document::{SourceDocument, SourceDocuments};
use acorn::io::enrichment::Enrich;
use acorn::io::{files_all_with_max_depth, files_from_gitlab_merge_request, remove_fields, uri_to_path, write_file, ApiResult, Fingerprint, PathExt};
use acorn::prelude::{remove_file, Path, PathBuf};
use acorn::Location;
use color_eyre::eyre::eyre;
use core::iter::once;
use futures::future::join_all;

trait Process {
    async fn process(self, load_checks: Vec<Check>, failures: Vec<String>, options: Options<'_>, resolved: Option<Records>) -> ApiResult<()>;
}
enum Input {
    Documents {
        values: Vec<ApiResult<SourceDocument>>,
        resolved: Option<Records>,
    },
    Remote(Vec<RemoteSearchResponse>),
}
impl Process for SourceDocuments {
    async fn process(self, load_checks: Vec<Check>, failures: Vec<String>, options: Options<'_>, resolved: Option<Records>) -> ApiResult<()> {
        let Options {
            citation_format,
            database_path,
            enrich,
            format,
            no_local_database,
            output,
            quiet,
            resolve,
            terse,
            verbosity,
            ..
        } = options;
        let inputs = self.0.len();
        let enrichment_checks = failures
            .iter()
            .map(|why| check_err!(CheckCategory::Link, message: why.clone()))
            .collect::<Vec<_>>();
        let sources: Vec<SourceDocument> = self.into();
        let discoveries = match (resolved, enrich, resolve) {
            | (Some(discoveries), false, _) => discoveries,
            | (_, _, true) => Records::from(sources).resolve().await,
            | _ => Records::from(sources),
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
            .chain(enrichment_checks)
            .chain(analysis_checks)
            .chain(persistence_checks)
            .collect::<Vec<_>>();
        temporary_paths.iter().for_each(|path| {
            let _ = remove_file(path);
        });
        let report = Report::new(&checks, discoveries, inputs)
            .with_candidates(candidate_results)
            .with_citation_format(citation_format);
        let format = format.unwrap_or_else(|| match (terse, output.is_none() && !is_stdout_piped()) {
            | (true, _) | (false, true) => OutputFormat::Console,
            | (false, false) => OutputFormat::Json,
        });
        let check_options = CheckOptions::init()
            .filter_by_verbosity(true)
            .quiet(quiet)
            .terse(terse)
            .format(format)
            .maybe_verbosity(verbosity)
            .build();
        checks.render(&check_options);
        let output_write_result = match output {
            | _ if terse && format == OutputFormat::Console => Ok(()),
            | Some(path) => report.serialize(format).and_then(|serialized| write_file(path.clone(), serialized)),
            | None if quiet => Ok(()),
            | None => report.serialize(format).map(|serialized| {
                if !serialized.is_empty() {
                    println!("{serialized}");
                }
            }),
        };
        output_write_result.and_then(|_| match checks.iter().any(Check::is_failure) || !failures.is_empty() {
            | true => Err(eyre!(OutputFailure::new(
                format == OutputFormat::Raw && verbosity.is_none_or(|level| level <= 1)
            ))),
            | false => Ok(()),
        })
    }
}
async fn execute(input: Input, options: Options<'_>) -> ApiResult<()> {
    match (input, options.remote) {
        | (Input::Documents { values, resolved }, _) => process(values, resolved, options).await,
        | (Input::Remote(responses), Some(remote)) => remote.process(responses, options),
        | (Input::Remote(_), None) => Err(eyre!("Missing remote gather request")),
    }
}
fn expand(inputs: &[String], filters: &FilterSet, max_depth: Option<usize>) -> Vec<String> {
    inputs
        .iter()
        .flat_map(|value| {
            let path = uri_to_path(value);
            match path.exists() {
                | true => files_all_with_max_depth(path, None::<Vec<String>>, max_depth)
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
fn is_excluded(value: &str, options: Options<'_>) -> bool {
    let path = Path::new(value);
    let output = options.output.as_ref().is_some_and(|output| output.same_as(path));
    let database = options.database_path.as_ref().is_some_and(|database| {
        let wal = PathBuf::from(format!("{}-wal", database.display()));
        let shm = PathBuf::from(format!("{}-shm", database.display()));
        database.same_as(path) || wal.as_path().same_as(path) || shm.as_path().same_as(path)
    });
    output || database
}
async fn prepare(options: Options<'_>) -> ApiResult<Prepared<Input>> {
    match options.remote {
        | Some(remote) => remote.fetch(options.resolve, options.offline).await.and_then(|responses| {
            responses
                .iter()
                .flat_map(|response| {
                    let RemoteSearchResponse {
                        provider,
                        total,
                        offset,
                        has_more,
                        matches,
                        ..
                    } = response;
                    let metadata = serde_json::to_vec(&(total, offset, has_more)).map(|content| (format!("{provider}:metadata"), content));
                    once(metadata).chain(
                        matches
                            .iter()
                            .map(move |value| serde_json::to_vec(value).map(|content| (format!("{provider}:{}", value.identifier), content))),
                    )
                })
                .collect::<serde_json::Result<Vec<_>>>()
                .map(|entries| Prepared::new(Input::Remote(responses), Fingerprint::from_entries(entries)))
                .map_err(Into::into)
        }),
        | None => prepare_documents(options).await,
    }
}
async fn prepare_documents(options: Options<'_>) -> ApiResult<Prepared<Input>> {
    let Options {
        filter,
        ignore,
        input,
        max_depth,
        merge_request,
        offline,
        resolve,
        text,
        watching,
        ..
    } = options;
    let stdin_watched = watching && (input.iter().any(|value| value == "-") || is_stdin_piped());
    match (stdin_watched, offline && options.enrich, offline && resolve, offline && merge_request) {
        | (true, _, _, _) => Err(eyre!("Standard input cannot be used with --watch")),
        | (_, true, _, _) => Err(eyre!("--enrich cannot be used with --offline")),
        | (_, _, true, _) => Err(eyre!("--resolve cannot be used with --offline")),
        | (_, _, _, true) => Err(eyre!("--merge-request cannot be used with --offline")),
        | (false, false, false, false) => {
            let stdin = read_stdin().filter(|content| !content.trim().is_empty());
            match (input.iter().any(|value| value == "-"), stdin) {
                | (true, None) => Err(eyre!("No content received from standard input")),
                | (_, stdin) => {
                    let has_content = !text.is_empty() || stdin.is_some();
                    let inputs = match (merge_request, input.is_empty(), has_content) {
                        | (true, _, _) => files_from_gitlab_merge_request(None::<Vec<String>>)
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
                            let expanded = expand(&inputs, &filters, max_depth)
                                .into_iter()
                                .filter(|value| !is_excluded(value, options))
                                .collect::<Vec<_>>();
                            let loaded = join_all(expanded.iter().map(|value| SourceDocument::load(value.as_str(), offline))).await;
                            let values = items.into_iter().chain(loaded).collect::<Vec<_>>();
                            let entries = values
                                .iter()
                                .enumerate()
                                .map(|(index, value)| match value {
                                    | Ok(document) => (
                                        format!("{}:{}:{}", document.source, document.format, index),
                                        document.content.as_bytes().to_vec(),
                                    ),
                                    | Err(why) => (format!("error:{index}"), why.to_string().into_bytes()),
                                })
                                .collect::<Vec<_>>();
                            let resolved = match resolve {
                                | true => {
                                    let sources = values.iter().filter_map(|value| value.as_ref().ok().cloned()).collect::<Vec<_>>();
                                    Some(Records::from(sources).resolve().await)
                                }
                                | false => None,
                            };
                            match resolved.as_ref() {
                                | Some(records) => serde_json::to_value(records)
                                    .map(|value| remove_fields(value, &["observed_at"]))
                                    .and_then(|value| serde_json::to_vec(&value))
                                    .map(|content| entries.into_iter().chain([("<resolved>".to_string(), content)]).collect())
                                    .map_err(color_eyre::Report::from),
                                | None => Ok(entries),
                            }
                            .map(|entries| Prepared::new(Input::Documents { values, resolved }, Fingerprint::from_entries(entries)))
                        }
                        | Err(why) => Err(why),
                    }
                }
            }
        }
    }
}
async fn process(values: Vec<ApiResult<SourceDocument>>, resolved: Option<Records>, options: Options<'_>) -> ApiResult<()> {
    match options.enrich && options.offline {
        | true => Err(eyre!("--enrich cannot be used with --offline")),
        | false => process_available(values, resolved, options).await,
    }
}
async fn process_available(values: Vec<ApiResult<SourceDocument>>, resolved: Option<Records>, options: Options<'_>) -> ApiResult<()> {
    let Options { enrich, format, .. } = options;
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
    let (sources, enriched, failures) = SourceDocuments::from(sources).enrich(enrich).await;
    match enrich && format == Some(OutputFormat::Raw) && !enriched.is_empty() {
        | true => {
            enriched.iter().for_each(|value| println!("{value}"));
            match failures.is_empty() {
                | true => Ok(()),
                | false => Err(eyre!("Enrichment completed with {} provider failure(s)", failures.len())),
            }
        }
        | false => sources.process(load_checks, failures, options, resolved).await,
    }
}
pub async fn run(options: Options<'_>, watch: Option<WatchOptions>) -> ApiResult<()> {
    match watch {
        | Some(watch) => {
            let has_remote_input = options
                .input
                .iter()
                .map(|value| Location::from(value.as_str()).scheme())
                .any(|scheme| scheme.is_http() || scheme.is_https());
            let network = options.enrich || options.remote.is_some() || options.merge_request || options.resolve || has_remote_input;
            let roots = options.roots();
            let watch = WatchOptions { network, ..watch };
            watch::run("gather", &roots, watch, || prepare(options), |input| execute(input, options)).await
        }
        | None => match prepare(options).await {
            | Ok(prepared) => execute(prepared.value, options).await,
            | Err(why) => Err(why),
        },
    }
}

#[cfg(test)]
mod tests;
