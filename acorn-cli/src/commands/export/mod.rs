use crate::cli::arguments::{FileFormat, Standard};
use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use acorn::analyzer;
use acorn::io::bagit::{Bag, BagInfo, Save};
use acorn::io::current_date;
use acorn::io::{create_progress_bar, finish_progress_bar, folder_size, read_file, with_progress, write_file, FromPath, InputOutput, ProgressType};
use acorn::io::{ApiFuture, ApiResult};
use acorn::prelude::{exit, remove_dir_all, Arc, HashSet, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::schema::standard::{datacite, dcat, huwise, invenio};
use acorn::util::{format_bytes, regex_join, suffix, Label, MimeType, StringConversion};
use acorn::{fail, skip};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use core::iter::once;
use owo_colors::OwoColorize;
#[cfg(feature = "pdf")]
use playwright::{api::Page, Playwright};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info};

#[cfg(feature = "pdf")]
mod pdf;
mod powerpoint;

trait ExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a>;

    fn crosswalk<'a>(&'a self, request: ExportRequest, from: Option<Standard>, to: Option<Standard>) -> ApiFuture<'a> {
        Box::pin(async move {
            let _request = request;
            let _from = from;
            let _to = to;
            Err(eyre!("Crosswalk export is only supported for structured output formats (json, yaml)"))
        })
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Batch<T> {
    One(T),
    Many(Vec<T>),
}
struct ExportRequest {
    output: Option<PathBuf>,
    path: Option<PathBuf>,
    format: FileFormat,
    reference: Option<PathBuf>,
    combine: bool,
    progress_type: ProgressType,
    options: CommandOptions,
}
struct CffExportHandler;
struct StructuredExportHandler;
struct BagExportHandler;
struct PdfExportHandler;
struct PowerPointExportHandler;
impl<T> Batch<T> {
    fn map<U, F>(self, mut f: F) -> ApiResult<Batch<U>>
    where
        F: FnMut(T) -> ApiResult<U>,
    {
        match self {
            | Batch::One(value) => f(value).map(Batch::One),
            | Batch::Many(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| f(value).map_err(|why| eyre!("Failed to convert record at index {index} — {why}")))
                .collect::<ApiResult<Vec<_>>>()
                .map(Batch::Many),
        }
    }
}
impl ExportHandler for CffExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            export_as_format(
                &request.path,
                request.options,
                &request.format,
                "cff",
                |p, ext| p.with_file_name("CITATION").with_extension(ext),
                request.progress_type,
            )
            .await
        })
    }
}
impl ExportHandler for StructuredExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            let extension = request.format.to_string();
            export_as_format(
                &request.path,
                request.options,
                &request.format,
                &extension,
                |p, ext| p.with_extension(ext),
                request.progress_type,
            )
            .await
        })
    }

    fn crosswalk<'a>(&'a self, request: ExportRequest, from: Option<Standard>, to: Option<Standard>) -> ApiFuture<'a> {
        Box::pin(async move {
            let extension = request.format.to_string();
            export_as_crosswalk_format(
                &request.path,
                request.options,
                &request.format,
                &extension,
                |p, ext| p.with_extension(ext),
                request.progress_type,
                from,
                to,
            )
            .await
        })
    }
}
impl ExportHandler for BagExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            match (request.path, request.output) {
                | (Some(path), Some(destination)) => {
                    let base = path.display().to_string();
                    let date = current_date();
                    let size = format_bytes(folder_size(base.clone()));
                    let info = BagInfo::init().date(date).size(size).build();
                    debug!("=> {} {:#?}", Label::using(), info);
                    let bag = Bag::init().base_directory(base).info(info).build();
                    bag.save(destination.clone())
                        .and_then(|_| Bag::verify(destination.clone()))
                        .and_then(|_| remove_dir_all(destination).map_err(|why| why.into()))
                }
                | (None, _) => Err(eyre!("Missing input path for bag export")),
                | (_, None) => Err(eyre!("Missing output directory for bag export")),
            }
        })
    }
}
impl ExportHandler for PdfExportHandler {
    fn export<'a>(&'a self, _request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            #[cfg(feature = "pdf")]
            {
                match resolve_paths(&_request.path, &_request.options).await {
                    | Ok(paths) => match initialize_pdf_page().await {
                        | Ok(page) => {
                            let page = &page;
                            let output = _request.output.clone();
                            let progress = with_progress(
                                paths,
                                |path| format!("Generating PDF for {}", path.file_name_with_parent()),
                                |path| {
                                    let output = output.clone();
                                    async move {
                                        let pdf_options = CommandOptions::init().path(path.clone()).maybe_output(output).build();
                                        pdf::create(page, pdf_options).await;
                                        Ok(())
                                    }
                                },
                                |count| format!("{}Done! Exported {} PDF{}", Label::CHECKMARK, count.green(), suffix(count)),
                                Some(1),
                                _request.progress_type,
                            );
                            match progress.await {
                                | Ok(_) => {
                                    let _close = page.close(Some(false)).await;
                                    Ok(())
                                }
                                | Err(why) => {
                                    fail!("Generate PDF — {}", why);
                                    exit(exitcode::DATAERR);
                                }
                            }
                        }
                        | Err(why) => Err(why),
                    },
                    | Err(why) => Err(why),
                }
            }
            #[cfg(not(feature = "pdf"))]
            {
                Err(eyre!("PDF export requires the 'pdf' feature. Rebuild with --features pdf"))
            }
        })
    }
}
impl ExportHandler for PowerPointExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            match resolve_paths(&request.path, &request.options).await {
                | Ok(paths) => {
                    let _options = CommandOptions::init()
                        .maybe_path(request.path.clone())
                        .maybe_output(request.output.clone())
                        .maybe_reference(request.reference.clone())
                        .build();
                    let create_options = Arc::new(_options);
                    if request.combine {
                        let progress = create_progress_bar(
                            1,
                            match request.progress_type {
                                | ProgressType::Silent => ProgressType::Silent,
                                | _ => ProgressType::Spinner,
                            },
                        );
                        progress.set_message("Generating combined PowerPoint artifact");
                        match powerpoint::create(paths, Some(Arc::clone(&create_options))) {
                            | Ok(_) => {
                                progress.inc(1);
                                finish_progress_bar(&progress, format!("{}Done! Exported combined PowerPoint file", Label::CHECKMARK));
                                Ok(())
                            }
                            | Err(why) => {
                                fail!("Generate combined PowerPoint — {}", why);
                                exit(exitcode::DATAERR);
                            }
                        }
                    } else {
                        let progress = with_progress(
                            paths,
                            |path| format!("Generating PowerPoint for {}", path.file_name_with_parent()),
                            |path| {
                                let options = Arc::clone(&create_options);
                                async move { powerpoint::create(once(path), Some(options)).map(|_| ()) }
                            },
                            |count| format!("{}Done! Exported {} PowerPoint file{}", Label::CHECKMARK, count.green(), suffix(count)),
                            Some(1),
                            request.progress_type,
                        );
                        match progress.await {
                            | Ok(_) => Ok(()),
                            | Err(why) => {
                                fail!("Generate PowerPoint — {}", why);
                                exit(exitcode::DATAERR);
                            }
                        }
                    }
                }
                | Err(why) => Err(why),
            }
        })
    }
}
fn build_export_options(
    branch: &Option<String>,
    commit: &Option<String>,
    filter: &[String],
    ignore: &[String],
    merge_request: bool,
    offline: bool,
    is_silent: bool,
    threads: usize,
) -> CommandOptions {
    CommandOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .maybe_filter(regex_join(filter))
        .maybe_ignore(regex_join(ignore))
        .merge_request(merge_request)
        .offline(offline)
        .quiet(is_silent)
        .threads(threads)
        .build()
}
fn build_export_request(
    output: &Option<PathBuf>,
    path: &Option<PathBuf>,
    format: &FileFormat,
    reference: &Option<PathBuf>,
    combine: bool,
    progress_type: ProgressType,
    options: CommandOptions,
) -> ExportRequest {
    ExportRequest {
        output: output.clone(),
        path: path.clone(),
        format: format.clone(),
        reference: reference.clone(),
        combine,
        progress_type,
        options,
    }
}
fn should_crosswalk(format: &FileFormat, from: Option<Standard>, to: Option<Standard>) -> bool {
    match (from.is_some() || to.is_some(), format) {
        | (true, FileFormat::Json | FileFormat::Yaml) => true,
        | _ => false,
    }
}
async fn dispatch_export(exporter: Box<dyn ExportHandler>, request: ExportRequest, from: Option<Standard>, to: Option<Standard>) -> ApiResult<()> {
    if should_crosswalk(&request.format, from, to) {
        exporter.crosswalk(request, from, to).await
    } else {
        exporter.export(request).await
    }
}
#[allow(clippy::too_many_arguments)]
pub async fn run(
    output: &Option<PathBuf>,
    path: &Option<PathBuf>,
    branch: &Option<String>,
    commit: &Option<String>,
    filter: &[String],
    ignore: &[String],
    format: &FileFormat,
    reference: &Option<PathBuf>,
    from: &Option<Standard>,
    to: &Option<Standard>,
    combine: &bool,
    merge_request: &bool,
    raw: &bool,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> ApiResult<()> {
    let is_silent = verbose.is_silent();
    let progress_type = match is_silent || *raw {
        | true => ProgressType::Silent,
        | false => ProgressType::Bar,
    };
    let options = build_export_options(branch, commit, filter, ignore, *merge_request, offline, is_silent, threads);
    preflight!(&options);
    let request = build_export_request(output, path, format, reference, *combine, progress_type, options);
    let exporter = exporter_for(&request.format);
    dispatch_export(exporter, request, *from, *to).await
}
async fn export_as_format(
    path: &Option<PathBuf>,
    options: CommandOptions,
    format: &FileFormat,
    extension: &str,
    output_path: fn(&PathBuf, &str) -> PathBuf,
    progress_type: ProgressType,
) -> ApiResult<()> {
    let threads = options.threads;
    match resolve_paths(path, &options).await {
        | Ok(resolved_paths) => {
            let (skipped, paths): (Vec<_>, Vec<_>) = resolved_paths
                .iter()
                .partition(|path| MimeType::from_path(path) == MimeType::from(format.clone()));
            skipped.iter().for_each(|path| {
                skip!("{} is already in {format} format", path.display());
            });
            let mut seen = HashSet::new();
            let items: Vec<_> = paths.into_iter().filter(|&p| seen.insert(output_path(p, extension))).cloned().collect();
            let count = items.len() as u64;
            let ext = extension.to_owned();
            let progress = with_progress(
                items,
                |item| format!("Generating {format} for {}", item.display()),
                |item| {
                    let ext = ext.clone();
                    async move {
                        let output = output_path(&item, &ext);
                        match ResearchActivity::read(&item) {
                            | Ok(data) => match data.write(output) {
                                | Ok(_) => Ok(()),
                                | Err(_) => {
                                    exit(exitcode::UNAVAILABLE);
                                }
                            },
                            | Err(why) => {
                                fail!("Read research activity data — {}", why);
                                exit(exitcode::DATAERR);
                            }
                        }
                    }
                },
                |count| format!("{}Done! Generated {} {format} file{}", Label::CHECKMARK, count.green(), suffix(count)),
                Some(threads),
                progress_type,
            );
            match progress.await {
                | Ok(_) => {
                    info!("=> {} Exported {} {format} file{}", Label::pass(), count.green(), suffix(count));
                    Ok(())
                }
                | Err(why) => {
                    fail!("Generate {} - {}", format, why);
                    exit(exitcode::DATAERR);
                }
            }
        }
        | Err(why) => Err(why),
    }
}
#[cfg(feature = "pdf")]
fn install_chromium(playwright: Playwright) -> ApiResult<Playwright> {
    playwright
        .install_chromium()
        .map_err(|why| eyre!("Failed to install Chromium for Playwright — {why}"))
        .map(|_| playwright)
}
#[cfg(feature = "pdf")]
async fn initialize_pdf_page() -> ApiResult<Page> {
    match Playwright::initialize()
        .await
        .map_err(|why| eyre!("Failed to initialize Playwright — {why}"))
        .and_then(install_chromium)
    {
        | Ok(playwright) => match playwright.chromium().launcher().headless(true).launch().await {
            | Ok(browser) => match browser.context_builder().build().await {
                | Ok(context) => context.new_page().await.map_err(|why| eyre!("Failed to create browser page — {why}")),
                | Err(why) => Err(eyre!("Failed to create browser context — {why}")),
            },
            | Err(why) => Err(eyre!("Failed to launch Chromium — {why}")),
        },
        | Err(why) => Err(why),
    }
}
async fn export_as_crosswalk_format(
    path: &Option<PathBuf>,
    options: CommandOptions,
    format: &FileFormat,
    extension: &str,
    output_path: fn(&PathBuf, &str) -> PathBuf,
    progress_type: ProgressType,
    from: Option<Standard>,
    to: Option<Standard>,
) -> ApiResult<()> {
    match to {
        | Some(target_standard) => {
            let threads = options.threads;
            match resolve_paths(path, &options).await {
                | Ok(resolved_paths) => {
                    let mut seen = HashSet::new();
                    let items: Vec<_> = resolved_paths
                        .iter()
                        .filter(|&p| seen.insert(output_path(p, extension)))
                        .cloned()
                        .collect();
                    let count = items.len() as u64;
                    let ext = extension.to_owned();
                    let progress = with_progress(
                        items,
                        |item| format!("Generating {format} for {}", item.display()),
                        |item| {
                            let ext = ext.clone();
                            async move {
                                let output = output_path(&item, &ext);
                                match read_file(item.clone()) {
                                    | Ok(content) => {
                                        let source_mime = MimeType::from_path(item.clone());
                                        let source_standard = match from {
                                            | Some(value) => Ok(value),
                                            | None => infer_source_standard(&item, &content, source_mime.clone()),
                                        };
                                        match source_standard.and_then(|resolved_standard| {
                                            crosswalk_content(&content, source_mime, format, resolved_standard, target_standard)
                                        }) {
                                            | Ok(converted) => write_file(output, converted),
                                            | Err(why) => Err(why),
                                        }
                                    }
                                    | Err(why) => Err(why),
                                }
                            }
                        },
                        |count| format!("{}Done! Generated {} {format} file{}", Label::CHECKMARK, count.green(), suffix(count)),
                        Some(threads),
                        progress_type,
                    );
                    match progress.await {
                        | Ok(_) => {
                            info!("=> {} Exported {} {format} crosswalk file{}", Label::pass(), count.green(), suffix(count));
                            Ok(())
                        }
                        | Err(why) => {
                            fail!("Generate {} crosswalk - {}", format, why);
                            exit(exitcode::DATAERR);
                        }
                    }
                }
                | Err(why) => Err(why),
            }
        }
        | None => Err(eyre!(
            "Crosswalk export requires --to <STANDARD> (supported: datacite, dcat, invenio, huwise)"
        )),
    }
}
fn crosswalk_content(
    content: &str,
    source_mime: MimeType,
    output_format: &FileFormat,
    source_standard: Standard,
    target_standard: Standard,
) -> ApiResult<String> {
    match source_standard {
        | Standard::Datacite => match parse_batch::<datacite::Record>(content, source_mime) {
            | Ok(source) => match target_standard {
                | Standard::Datacite => serialize_batch(&source, output_format),
                | Standard::Dcat => source
                    .map(|record| Ok(dcat::Dataset::from(record)))
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Invenio => source
                    .map(|record| Ok(datacite_to_invenio_safe(record)))
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Huwise => source
                    .map(|record| Ok(datacite_to_huwise_safe(record)))
                    .and_then(|target| serialize_batch(&target, output_format)),
                | _ => unsupported_crosswalk_pair(source_standard, target_standard),
            },
            | Err(why) => Err(why),
        },
        | Standard::Dcat => match parse_batch::<dcat::Dataset>(content, source_mime) {
            | Ok(source) => match target_standard {
                | Standard::Dcat => serialize_batch(&source, output_format),
                | Standard::Datacite => source
                    .map(|dataset| Ok(datacite::Record::from(dataset)))
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Invenio => source
                    .map(|dataset| {
                        let datacite_record = datacite::Record::from(dataset);
                        Ok(datacite_to_invenio_safe(datacite_record))
                    })
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Huwise => source
                    .map(|dataset| {
                        let datacite_record = datacite::Record::from(dataset);
                        Ok(datacite_to_huwise_safe(datacite_record))
                    })
                    .and_then(|target| serialize_batch(&target, output_format)),
                | _ => unsupported_crosswalk_pair(source_standard, target_standard),
            },
            | Err(why) => Err(why),
        },
        | Standard::Invenio => match parse_batch::<invenio::Record>(content, source_mime) {
            | Ok(source) => match target_standard {
                | Standard::Invenio => serialize_batch(&source, output_format),
                | Standard::Datacite => source
                    .map(|record| Ok(invenio_to_datacite_safe(record)))
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Dcat => source
                    .map(|record| {
                        let datacite_record = invenio_to_datacite_safe(record);
                        Ok(dcat::Dataset::from(datacite_record))
                    })
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Huwise => source
                    .map(|record| {
                        let datacite_record = invenio_to_datacite_safe(record);
                        Ok(datacite_to_huwise_safe(datacite_record))
                    })
                    .and_then(|target| serialize_batch(&target, output_format)),
                | _ => unsupported_crosswalk_pair(source_standard, target_standard),
            },
            | Err(why) => Err(why),
        },
        | Standard::Huwise => match parse_batch::<huwise::Dataset>(content, source_mime) {
            | Ok(source) => match target_standard {
                | Standard::Huwise => serialize_batch(&source, output_format),
                | Standard::Datacite => source
                    .map(|dataset| Ok(datacite::Record::from(dataset)))
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Dcat => source
                    .map(|dataset| {
                        let datacite_record = datacite::Record::from(dataset);
                        Ok(dcat::Dataset::from(datacite_record))
                    })
                    .and_then(|target| serialize_batch(&target, output_format)),
                | Standard::Invenio => source
                    .map(|dataset| {
                        let datacite_record = datacite::Record::from(dataset);
                        Ok(datacite_to_invenio_safe(datacite_record))
                    })
                    .and_then(|target| serialize_batch(&target, output_format)),
                | _ => unsupported_crosswalk_pair(source_standard, target_standard),
            },
            | Err(why) => Err(why),
        },
        | _ => unsupported_crosswalk_pair(source_standard, target_standard),
    }
}
fn parse_batch<T>(content: &str, source_mime: MimeType) -> ApiResult<Batch<T>>
where
    T: DeserializeOwned,
{
    match source_mime {
        | MimeType::Json => serde_json::from_str(content).map_err(|why| eyre!("Failed to parse JSON source — {why}")),
        | MimeType::Yaml => serde_norway::from_str(content).map_err(|why| eyre!("Failed to parse YAML source — {why}")),
        | _ => Err(eyre!("Crosswalk source must be JSON or YAML")),
    }
}
fn serialize_batch<T>(value: &Batch<T>, output_format: &FileFormat) -> ApiResult<String>
where
    T: Serialize,
{
    match output_format {
        | FileFormat::Json => serde_json::to_string_pretty(value).map_err(|why| eyre!("Failed to serialize JSON output — {why}")),
        | FileFormat::Yaml => serde_norway::to_string(value).map_err(|why| eyre!("Failed to serialize YAML output — {why}")),
        | _ => Err(eyre!("Crosswalk output format must be JSON or YAML")),
    }
}
fn infer_source_standard(path: &PathBuf, content: &str, source_mime: MimeType) -> ApiResult<Standard> {
    let root = match source_mime {
        | MimeType::Json => {
            serde_json::from_str(content).map_err(|why| eyre!("Unable to infer source schema for {} — invalid JSON ({why})", path.display()))
        }
        | MimeType::Yaml => {
            serde_norway::from_str(content).map_err(|why| eyre!("Unable to infer source schema for {} — invalid YAML ({why})", path.display()))
        }
        | _ => Err(eyre!("Unable to infer source schema for {} — unsupported file type", path.display())),
    };
    match root {
        | Ok(serde_json::Value::Object(map)) => Ok(analyzer::Standard::from(&map).into()),
        | Ok(serde_json::Value::Array(values)) => match values.into_iter().next() {
            | Some(serde_json::Value::Object(map)) => Ok(analyzer::Standard::from(&map).into()),
            | Some(_) => Err(eyre!(
                "Unable to infer source schema for {} — array items must be objects",
                path.display()
            )),
            | None => Err(eyre!("Unable to infer source schema for {} — empty array", path.display())),
        },
        | Ok(_) => Err(eyre!(
            "Unable to infer source schema for {}: expected object or array of objects",
            path.display()
        )),
        | Err(why) => Err(why),
    }
}
fn unsupported_crosswalk_pair(source: Standard, target: Standard) -> ApiResult<String> {
    Err(eyre!(
        "Unsupported metadata crosswalk pair: {} -> {} (supported: datacite, dcat, invenio, huwise)",
        source,
        target
    ))
}
fn datacite_to_invenio_safe(record: datacite::Record) -> invenio::Record {
    let datacite::Record { id, attributes, .. } = record;
    let identifier = if !id.is_empty() { id } else { attributes.doi.clone() };
    let creators = attributes.creators.map(|values| {
        values
            .into_iter()
            .map(|creator| invenio::Creator {
                person_or_org: invenio::PersonOrOrganization {
                    kind: "personal".to_string(),
                    name: Some(creator.name),
                    given_name: creator.given_name,
                    family_name: creator.family_name,
                    identifiers: None,
                },
                role: None,
                affiliations: None,
            })
            .collect::<Vec<_>>()
    });
    let publication_date = attributes.publication_year.map(|year| format!("{year:04}-01-01"));
    let title = attributes
        .titles
        .as_ref()
        .and_then(|titles| titles.first().map(|value| value.title.clone()));
    let description = attributes
        .descriptions
        .as_ref()
        .and_then(|descriptions| descriptions.first().map(|value| value.description.clone()));
    let languages = attributes.language.map(|language| vec![invenio::Language { id: language }]);
    let resource_type = None;
    let subjects = attributes.subjects.map(|values| {
        values
            .into_iter()
            .map(|subject| invenio::Subject {
                id: None,
                subject: Some(subject.subject),
            })
            .collect::<Vec<_>>()
    });
    let metadata = invenio::Metadata {
        resource_type,
        title,
        publication_date,
        creators,
        additional_titles: None,
        description,
        additional_descriptions: None,
        rights: None,
        copyright: None,
        contributors: None,
        subjects,
        languages,
        dates: None,
        version: attributes.version,
        publisher: attributes.publisher.map(|publisher| publisher.name),
        identifiers: None,
        related_identifiers: None,
        sizes: None,
        formats: None,
        locations: None,
        funding: None,
        references: None,
    };
    invenio::Record {
        schema: Some("https://inveniordm.docs.cern.ch/reference/metadata/".to_string()),
        id: Some(identifier),
        pid: Some(invenio::PersistentIdentifier {
            pk: None,
            status: Some("U".to_string()),
        }),
        pids: None,
        parent: None,
        access: None,
        metadata: Some(metadata),
        custom_fields: None,
        files: None,
        tombstone: None,
        created: None,
        updated: None,
    }
}
fn datacite_to_huwise_safe(record: datacite::Record) -> huwise::Dataset {
    let datacite::Record { id, attributes, .. } = record;
    let title = attributes
        .titles
        .as_ref()
        .and_then(|titles| titles.first().map(|value| value.title.clone()));
    let creators = attributes
        .creators
        .as_ref()
        .map(|values| Value::Array(values.iter().map(|creator| Value::String(creator.name.clone())).collect::<Vec<_>>()));
    let description = attributes
        .descriptions
        .as_ref()
        .and_then(|descriptions| descriptions.first().map(|value| value.description.clone()));
    let resource_type = attributes
        .resource_types
        .and_then(|resource_types| resource_types.resource_type.map(Value::String));
    let datacite_meta = huwise::Datacite {
        identifier: Some(attributes.doi.clone()),
        title,
        alternative_title: None,
        publisher: attributes.publisher.map(|publisher| publisher.name),
        creator: creators,
        publication_year: attributes.publication_year.map(|year| year.to_string()),
        date: None,
        subject: attributes
            .subjects
            .map(|subjects| subjects.into_iter().map(|subject| subject.subject).collect::<Vec<_>>()),
        contributor: None,
        description,
        language: attributes.language,
        resource_type,
        alternate_identifier: None,
        related_identifier: None,
        size: None,
        format: None,
        version: attributes.version,
        rights: None,
        geolocation: None,
    };
    huwise::Dataset {
        dataset_id: if !id.is_empty() { id } else { attributes.doi },
        has_attachments: false,
        attachments_count: 0,
        has_records: false,
        fields: Value::Array(vec![]),
        metas: huwise::Meta {
            dcat: None,
            r#default: None,
            dublin_core: None,
            dcat_ap: None,
            custom_template: None,
            datacite: Some(datacite_meta),
        },
        features: vec![],
    }
}
fn invenio_to_datacite_safe(record: invenio::Record) -> datacite::Record {
    let fallback_identifier = record.id.clone().unwrap_or_else(|| "10.5555/acorn-generated".to_string());
    let metadata = record.metadata.clone();
    let title = metadata
        .as_ref()
        .and_then(|value| value.title.clone())
        .unwrap_or_else(|| fallback_identifier.clone());
    let publication_year = metadata
        .as_ref()
        .and_then(|value| value.publication_date.clone())
        .and_then(|date| date.split('-').next().and_then(|year| year.parse::<i32>().ok()));
    let creators = metadata.as_ref().and_then(|value| value.creators.clone()).map(|values| {
        values
            .into_iter()
            .map(|creator| {
                let person_or_org = creator.person_or_org;
                let person_name = person_or_org
                    .name
                    .clone()
                    .or_else(|| match (person_or_org.family_name.clone(), person_or_org.given_name.clone()) {
                        | (Some(family_name), Some(given_name)) => Some(format!("{family_name}, {given_name}")),
                        | (Some(family_name), None) => Some(family_name),
                        | (None, Some(given_name)) => Some(given_name),
                        | (None, None) => None,
                    })
                    .unwrap_or_else(|| "Unknown Creator".to_string());
                datacite::Creator {
                    name: person_name,
                    name_type: None,
                    given_name: person_or_org.given_name,
                    family_name: person_or_org.family_name,
                    name_identifiers: None,
                    affiliation: None,
                }
            })
            .collect::<Vec<_>>()
    });
    let descriptions = metadata.as_ref().and_then(|value| value.description.clone()).map(|description| {
        vec![datacite::Description {
            description,
            description_type: None,
            language: None,
        }]
    });
    let language = metadata
        .as_ref()
        .and_then(|value| value.languages.as_ref())
        .and_then(|values| values.first().map(|language| language.id.clone()));
    let attributes = datacite::Attributes {
        doi: fallback_identifier.clone(),
        event: None,
        titles: Some(vec![datacite::Title { title, title_type: None }]),
        creators,
        publisher: metadata
            .as_ref()
            .and_then(|value| value.publisher.clone())
            .map(|publisher| datacite::Publisher {
                name: publisher,
                publisher_identifier: None,
                publisher_identifier_scheme: None,
                scheme_uri: None,
            }),
        publication_year,
        resource_types: None,
        url: None,
        subjects: None,
        contributors: None,
        dates: None,
        language,
        alternate_identifiers: None,
        related_identifiers: None,
        sizes: None,
        formats: None,
        version: metadata.as_ref().and_then(|value| value.version.clone()),
        rights_list: None,
        descriptions,
        geo_locations: None,
        funding_references: None,
        schema_version: None,
    };
    datacite::Record {
        id: fallback_identifier,
        kind: "dois".to_string(),
        attributes,
    }
}
fn exporter_for(format: &FileFormat) -> Box<dyn ExportHandler> {
    match format {
        | FileFormat::Cff => Box::new(CffExportHandler),
        | FileFormat::Json | FileFormat::Markdown | FileFormat::Yaml => Box::new(StructuredExportHandler),
        | FileFormat::Bag => Box::new(BagExportHandler),
        | FileFormat::Pdf => Box::new(PdfExportHandler),
        | FileFormat::Powerpoint => Box::new(PowerPointExportHandler),
    }
}

#[cfg(test)]
mod tests;
