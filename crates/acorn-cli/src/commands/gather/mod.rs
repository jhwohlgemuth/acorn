use crate::{
    cli::WatchOptions,
    commands::watch::{self, Prepared},
    io::{is_stdin_piped, read_stdin},
};
use acorn::{
    analyzer::host::{
        context::OutputFormat,
        discovery::{finalize_report, DiscoveryExtension, GatherOutputPolicy, Options, Records, RemoteSearchResponse, Report},
    },
    io::{
        config::FilterSet,
        document::{SourceDocument, SourceDocuments},
        enrichment::Enrich,
        files_all_with_max_depth, files_from_gitlab_merge_request, remove_fields, ApiResult, Fingerprint, PathExt,
    },
};
use acorn_analysis::{check_err, CheckBatch, CheckCategory};
use acorn_core::Location;
use acorn_host::fs::file_uri_to_path;
use color_eyre::eyre::eyre;
use core::iter::once;
use futures::future::join_all;
use std::{
    fs::remove_file,
    path::{Path, PathBuf},
};

trait Process {
    async fn process(self, load_checks: CheckBatch, failures: Vec<String>, options: &Options, resolved: Option<Records>) -> ApiResult<()>;
}
enum Input {
    Documents {
        values: Vec<ApiResult<SourceDocument>>,
        resolved: Option<Records>,
    },
    Remote(Vec<RemoteSearchResponse>),
}
impl Process for SourceDocuments {
    async fn process(self, load_checks: CheckBatch, failures: Vec<String>, options: &Options, resolved: Option<Records>) -> ApiResult<()> {
        let output_policy = GatherOutputPolicy::from(options);
        let DiscoveryExtension {
            citation_format,
            database_path,
            enrich,
            fuzzy,
            no_local_database,
            resolve,
            ..
        } = &options.extension;
        let inputs = self.0.len();
        let enrichment_checks = failures
            .iter()
            .map(|why| check_err!(CheckCategory::Link, message: why.clone()))
            .collect::<CheckBatch>();
        let sources: Vec<SourceDocument> = self.into();
        let discoveries = match (resolved, *enrich, *resolve) {
            | (Some(discoveries), false, _) => discoveries,
            | (_, _, true) => Records::from(sources).resolve().await,
            | _ => Records::from(sources),
        };
        let (candidate_results, persistence_checks) = if *no_local_database {
            (Vec::new(), CheckBatch::empty())
        } else {
            let history_checks = discoveries.persist(database_path);
            let (candidate_results, candidate_checks) = discoveries.persist_candidates(*fuzzy, database_path);
            (candidate_results, history_checks.combine(candidate_checks))
        };
        let (analysis_checks, temporary_paths) = discoveries.analyze(options).await;
        let checks = load_checks
            .combine(enrichment_checks)
            .combine(analysis_checks)
            .combine(persistence_checks);
        temporary_paths.iter().for_each(|path| {
            let _ = remove_file(path);
        });
        let report = Report::new(checks.as_slice(), discoveries, inputs)
            .with_candidates(candidate_results)
            .with_citation_format(*citation_format);
        finalize_report(checks.as_slice(), &report, output_policy, !failures.is_empty())
    }
}
async fn execute(input: Input, options: &Options) -> ApiResult<()> {
    match (input, options.extension.remote.as_ref()) {
        | (Input::Documents { values, resolved }, _) => process(values, resolved, options).await,
        | (Input::Remote(responses), Some(remote)) => remote.process(responses, options),
        | (Input::Remote(_), None) => Err(eyre!("Missing remote gather request")),
    }
}
fn expand(inputs: &[String], filters: &FilterSet, max_depth: Option<usize>) -> Vec<String> {
    inputs
        .iter()
        .flat_map(|value| {
            let path = file_uri_to_path(value).unwrap_or_else(|_| PathBuf::from(value));
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
fn is_excluded(value: &str, options: &Options) -> bool {
    let path = Path::new(value);
    let output = options.output.as_ref().is_some_and(|output| output.same_as(path));
    let database = options.extension.database_path.as_ref().is_some_and(|database| {
        let wal = PathBuf::from(format!("{}-wal", database.display()));
        let shm = PathBuf::from(format!("{}-shm", database.display()));
        database.same_as(path) || wal.as_path().same_as(path) || shm.as_path().same_as(path)
    });
    output || database
}
async fn prepare(options: &Options) -> ApiResult<Prepared<Input>> {
    match options.extension.remote.as_ref() {
        | Some(remote) => remote
            .fetch(options.extension.resolve, options.common.offline)
            .await
            .and_then(|responses| {
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
async fn prepare_documents(options: &Options) -> ApiResult<Prepared<Input>> {
    let common = &options.common;
    let DiscoveryExtension {
        enrich,
        input,
        max_depth,
        merge_request,
        resolve,
        text,
        watching,
        ..
    } = &options.extension;
    let offline = common.offline;
    let stdin_watched = *watching && (input.iter().any(|value| value == "-") || is_stdin_piped());
    let enrich_offline = offline && *enrich;
    let merge_request_offline = offline && *merge_request;
    let resolve_offline = offline && *resolve;
    match (stdin_watched, enrich_offline, resolve_offline, merge_request_offline) {
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
                    let inputs = match (*merge_request, input.is_empty(), has_content) {
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
                    match FilterSet::try_from((common.ignore.as_slice(), common.filter.as_slice())) {
                        | Ok(filters) => {
                            let expanded = expand(&inputs, &filters, *max_depth)
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
async fn process(values: Vec<ApiResult<SourceDocument>>, resolved: Option<Records>, options: &Options) -> ApiResult<()> {
    match options.extension.enrich && options.common.offline {
        | true => Err(eyre!("--enrich cannot be used with --offline")),
        | false => process_available(values, resolved, options).await,
    }
}
async fn process_available(values: Vec<ApiResult<SourceDocument>>, resolved: Option<Records>, options: &Options) -> ApiResult<()> {
    let (enrich, format) = (options.extension.enrich, options.extension.format);
    let load_checks = values
        .iter()
        .filter_map(|value| {
            value
                .as_ref()
                .err()
                .map(|why| check_err!(CheckCategory::Schema, message: why.to_string()))
        })
        .collect::<CheckBatch>();
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
pub async fn run(options: Options, watch: Option<WatchOptions>) -> ApiResult<()> {
    match watch {
        | Some(watch) => {
            let has_remote_input = options
                .extension
                .input
                .iter()
                .map(|value| Location::from(value.as_str()).scheme())
                .any(|scheme| scheme.is_http() || scheme.is_https());
            let network = options.extension.enrich
                || options.extension.remote.is_some()
                || options.extension.merge_request
                || options.extension.resolve
                || has_remote_input;
            let roots = options.extension.roots();
            let watch = WatchOptions { network, ..watch };
            watch::run("gather", &roots, watch, || prepare(&options), |input| execute(input, &options)).await
        }
        | None => match prepare(&options).await {
            | Ok(prepared) => execute(prepared.value, &options).await,
            | Err(why) => Err(why),
        },
    }
}

#[cfg(test)]
mod tests;
