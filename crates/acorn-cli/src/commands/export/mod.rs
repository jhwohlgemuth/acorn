use crate::{
    cli::{
        arguments::{ArchiveType, FileFormat, Target},
        parse::infer_standard_from_content,
        resolve_export_paths, resolve_paths, CommandOptions,
    },
    commands::preflight,
};
#[cfg(feature = "chart")]
use acorn::io::chart::ChartOptions;
use acorn::{
    fail,
    io::{
        adapter::Batch,
        bagit::{Bag, BagInfo, Save},
        config::application::ApplicationConfiguration,
        create_progress_bar, current_date, finish_progress_bar, folder_size, sidecar, with_progress, write_file, ApiFuture, ApiResult, FromPath,
        InputOutput, ProgressType,
    },
    skip,
    util::{print_values_as_table, StringConversion},
};
use acorn_analysis::{Check, CheckCategory, CheckSummary, IntoChecks, StandardCrosswalk};
#[cfg(not(feature = "chart"))]
use acorn_core::prelude::HashMap;
use acorn_core::{
    prelude::{Arc, HashSet},
    util::{format_bytes, regex_join, suffix, MimeType},
};
use acorn_host::{fs::read_text, terminal::Label};
use acorn_schema::{
    codec::DocumentCodec,
    modelkit::kitfile,
    pid::raid::Metadata,
    research_activity::ResearchActivity,
    standard::{
        cff, ckan,
        crosswalk::{Conversion, CrosswalkError, DublinCoreImportOptions},
        datacite, dcat,
        dublin_core::DublinCoreDocument,
        huwise, invenio, text, Standard,
    },
    ContactPoint,
};
use bon::Builder;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use core::iter::once;
use nanoid::nanoid;
use owo_colors::OwoColorize;
use schemars::{schema_for, JsonSchema};
use std::{
    fs::{create_dir_all, remove_dir_all, remove_file, rename},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tracing::{debug, info};
#[cfg(feature = "pdf")]
pub(crate) mod pdf;
mod powerpoint;
pub(crate) mod updates;

trait ExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a>;
    fn crosswalk<'a>(&'a self, request: ExportRequest, from: Option<Standard>, to: Option<Standard>) -> ApiFuture<'a> {
        Box::pin(async move {
            let _request = request;
            let _from = from;
            let _to = to;
            Err(eyre!(
                "Crosswalk export is only supported for structured output formats (json, jsonl, yaml, zon)"
            ))
        })
    }
}
struct BagExportHandler;
struct CffExportHandler;
#[derive(Clone, Debug, Default)]
struct CrosswalkImportOptions {
    contact: Option<ContactPoint>,
    identifier: Option<String>,
}
#[derive(Builder, Debug, Default)]
#[builder(start_fn = init)]
struct ExportRequest {
    output: Option<PathBuf>,
    path: Option<PathBuf>,
    #[cfg_attr(not(feature = "pdf"), allow(dead_code))]
    chrome_path: Option<PathBuf>,
    #[builder(default)]
    format: FileFormat,
    archive_format: Option<ArchiveType>,
    #[builder(default)]
    #[cfg_attr(not(feature = "chart"), allow(dead_code))]
    show_aspect_labels: bool,
    #[builder(default)]
    #[cfg_attr(not(feature = "chart"), allow(dead_code))]
    show_aspect_scores: bool,
    reference: Option<PathBuf>,
    combine: bool,
    #[builder(default)]
    dry_run: bool,
    #[builder(default)]
    strict: bool,
    #[builder(default)]
    crosswalk_import: CrosswalkImportOptions,
    #[builder(default)]
    skip: Vec<CheckCategory>,
    #[builder(default)]
    progress_type: ProgressType,
    #[builder(default)]
    options: CommandOptions,
    #[builder(default)]
    target: Target,
    since: Option<String>,
}
struct PdfExportHandler;
struct PowerPointExportHandler;
struct StructuredExportHandler;
struct UpdatesExportHandler;
impl ExportHandler for BagExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            match (request.path, request.output) {
                | (Some(path), Some(destination)) => {
                    let inferred = match MimeType::from(destination.to_string_lossy().as_ref()) {
                        | format @ (MimeType::Gzip | MimeType::SevenZip | MimeType::Tar | MimeType::Zip) => Some(format),
                        | _ => None,
                    };
                    let explicit = request.archive_format.map(MimeType::from);
                    match explicit.is_some() && inferred.is_some() && explicit != inferred {
                        | true => Err(eyre!("--archive-format conflicts with the output archive suffix")),
                        | false => {
                            let archive_format = explicit.or(inferred.clone()).unwrap_or(MimeType::Zip);
                            let archive_destination = match inferred.is_some() {
                                | true => destination,
                                | false => destination.with_extension(archive_format.clone().file_type()),
                            };
                            let staging = archive_destination
                                .parent()
                                .unwrap_or_else(|| Path::new("."))
                                .join(format!(".acorn-bag-{}", nanoid!()));
                            let base = path.display().to_string();
                            let date = current_date();
                            let size = format_bytes(folder_size(base.clone()));
                            let info = BagInfo::init().date(date).size(size).build();
                            debug!("=> {} {:#?}", Label::using(), info);
                            let bag = Bag::init().base_directory(base).info(info).build();
                            let result = bag
                                .save_as(staging.clone(), archive_format, Some(archive_destination))
                                .and_then(|_| Bag::verify(staging.clone()));
                            let cleanup = match staging.exists() {
                                | true => remove_dir_all(staging).map_err(Into::into),
                                | false => Ok(()),
                            };
                            result.and(cleanup)
                        }
                    }
                }
                | (None, _) => Err(eyre!("Missing input path for bag export")),
                | (_, None) => Err(eyre!("Missing output directory for bag export")),
            }
        })
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
impl CrosswalkImportOptions {
    fn load(config: &Option<PathBuf>, contact: Option<&Path>, identifier: Option<&str>) -> ApiResult<Self> {
        ApplicationConfiguration::load_with_path(config).and_then(|(configuration, resolved)| {
            let configured = configuration.options.as_ref().and_then(|options| options.crosswalk.as_ref());
            let root = ApplicationConfiguration::root(&resolved);
            let contact = contact.map(Path::to_path_buf).or_else(|| {
                configured.and_then(|options| {
                    options.contact.as_ref().map(|path| match path.is_absolute() {
                        | true => path.clone(),
                        | false => root.join(path),
                    })
                })
            });
            let identifier = identifier
                .map(str::to_string)
                .or_else(|| configured.and_then(|options| options.local_identifier.clone()));
            contact
                .as_deref()
                .map(Self::load_contact)
                .transpose()
                .map(|contact| Self { contact, identifier })
        })
    }
    fn load_contact(path: &Path) -> ApiResult<ContactPoint> {
        read_text(path)
            .map_err(|why| eyre!("Read crosswalk contact — {why}"))
            .and_then(|content| match MimeType::from_path(path) {
                | MimeType::Json => serde_json::from_str(&content).map_err(|why| eyre!("Parse crosswalk contact JSON — {why}")),
                | MimeType::Yaml => serde_norway::from_str(&content).map_err(|why| eyre!("Parse crosswalk contact YAML — {why}")),
                | format => Err(eyre!("Crosswalk contact must use JSON or YAML, not {format}")),
            })
    }
}
impl ExportRequest {
    fn exporter(&self) -> Box<dyn ExportHandler> {
        match (self.target, &self.format) {
            | (Target::Updates, _) | (_, FileFormat::Docx) => Box::new(UpdatesExportHandler),
            | (_, FileFormat::Cff) => Box::new(CffExportHandler),
            | (
                _,
                FileFormat::Json
                | FileFormat::JsonLd
                | FileFormat::JsonLines
                | FileFormat::Markdown
                | FileFormat::RdfXml
                | FileFormat::Turtle
                | FileFormat::Yaml
                | FileFormat::Zon,
            ) => Box::new(StructuredExportHandler),
            | (_, FileFormat::Bag) => Box::new(BagExportHandler),
            | (_, FileFormat::Pdf) => Box::new(PdfExportHandler),
            | (_, FileFormat::Powerpoint) => Box::new(PowerPointExportHandler),
        }
    }
}
impl ExportHandler for PdfExportHandler {
    fn export<'a>(&'a self, #[cfg_attr(not(feature = "pdf"), allow(unused_variables))] request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            #[cfg(feature = "pdf")]
            {
                match resolve_paths(&request.path, &request.options).await {
                    | Ok(paths) => match pdf::initialize_page(request.options.offline, request.chrome_path.as_deref()).await {
                        | Ok(session) => {
                            let page = session.page();
                            let output = request.output.clone();
                            let chart_options = ChartOptions::init()
                                .file_type("svg")
                                .show_labels(request.show_aspect_labels)
                                .show_scores(request.show_aspect_scores)
                                .build();
                            let progress = with_progress(
                                paths,
                                |path| format!("Generating PDF for {}", path.file_name_with_parent()),
                                |path| {
                                    let output = output.clone();
                                    async move {
                                        let pdf_options = CommandOptions::init().path(path.clone()).maybe_output(output).build();
                                        pdf::create(page, pdf_options, chart_options).await
                                    }
                                },
                                |count| format!("{}Done! Exported {} PDF{}", Label::CHECKMARK, count.green(), suffix(count)),
                                Some(1),
                                request.progress_type,
                            );
                            let result = progress.await;
                            session.close().await;
                            match result {
                                | Ok(_) => Ok(()),
                                | Err(why) => {
                                    fail!("Generate PDF — {why}");
                                    Err(eyre!("Generate PDF — {why}"))
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
                    let options = CommandOptions::init()
                        .maybe_path(request.path.clone())
                        .maybe_output(request.output.clone())
                        .maybe_reference(request.reference.clone())
                        .build();
                    let create_options = Arc::new(options);
                    #[cfg(feature = "chart")]
                    let chart_options = ChartOptions::init()
                        .file_type("png")
                        .show_labels(request.show_aspect_labels)
                        .show_scores(request.show_aspect_scores)
                        .build();
                    #[cfg(feature = "chart")]
                    let aspect_charts = chart_options.render_aspect_charts(&paths).map(Arc::new);
                    #[cfg(not(feature = "chart"))]
                    let aspect_charts: ApiResult<Arc<HashMap<PathBuf, Option<Vec<u8>>>>> = Ok(Arc::new(HashMap::new()));
                    match aspect_charts {
                        | Ok(aspect_charts) if request.combine => {
                            let progress = create_progress_bar(
                                1,
                                match request.progress_type {
                                    | ProgressType::Silent => ProgressType::Silent,
                                    | _ => ProgressType::Spinner,
                                },
                            );
                            progress.set_message("Generating combined PowerPoint artifact");
                            let charts = Arc::new(
                                paths
                                    .iter()
                                    .map(|path| aspect_charts.get(path).cloned().unwrap_or_default())
                                    .collect::<Vec<_>>(),
                            );
                            match powerpoint::create(paths, Some(Arc::clone(&create_options)), charts) {
                                | Ok(_) => {
                                    progress.inc(1);
                                    finish_progress_bar(&progress, format!("{}Done! Exported combined PowerPoint file", Label::CHECKMARK));
                                    Ok(())
                                }
                                | Err(why) => {
                                    fail!("Generate combined PowerPoint — {why}");
                                    Err(eyre!("Generate combined PowerPoint — {why}"))
                                }
                            }
                        }
                        | Ok(aspect_charts) => {
                            let progress = with_progress(
                                paths,
                                |path| format!("Generating PowerPoint for {}", path.file_name_with_parent()),
                                |path| {
                                    let options = Arc::clone(&create_options);
                                    let chart = aspect_charts.get(&path).cloned().unwrap_or_default();
                                    async move { powerpoint::create(once(path), Some(options), Arc::new(vec![chart])).map(|_| ()) }
                                },
                                |count| format!("{}Done! Exported {} PowerPoint file{}", Label::CHECKMARK, count.green(), suffix(count)),
                                Some(1),
                                request.progress_type,
                            );
                            match progress.await {
                                | Ok(_) => Ok(()),
                                | Err(why) => {
                                    fail!("Generate PowerPoint — {why}");
                                    Err(eyre!("Generate PowerPoint — {why}"))
                                }
                            }
                        }
                        | Err(why) => {
                            fail!("Render ASPECT charts — {why}");
                            Err(why)
                        }
                    }
                }
                | Err(why) => Err(why),
            }
        })
    }
}
impl ExportHandler for StructuredExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move {
            let extension = MimeType::from(&request.format).file_type();
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
            let extension = MimeType::from(&request.format).file_type();
            export_as_format_crosswalk(
                &request.path,
                request.options,
                &request.format,
                &extension,
                |p, ext| p.with_extension(ext),
                request.progress_type,
                from,
                to,
                request.dry_run,
                request.strict,
                request.skip,
                request.crosswalk_import,
                &request.output,
            )
            .await
        })
    }
}
impl ExportHandler for UpdatesExportHandler {
    fn export<'a>(&'a self, request: ExportRequest) -> ApiFuture<'a> {
        Box::pin(async move { updates::export(request).await })
    }
}
fn crosswalk_item(
    item: &Path,
    from: Option<Standard>,
    target: Standard,
    format: &FileFormat,
    skip_crosswalk: bool,
    import: &CrosswalkImportOptions,
) -> Result<Conversion<String>, CrosswalkError> {
    read_text(item)
        .map_err(|error| CrosswalkError::ParseFailed(error.to_string()))
        .and_then(|content| {
            let mime = MimeType::from_path(item);
            let source = match from {
                | Some(value) => Ok(value),
                | None => infer_standard_from_content(item, &content, &mime).map_err(|error| CrosswalkError::ParseFailed(error.to_string())),
            };
            source.and_then(|resolved| {
                let target_mime = MimeType::from(format);
                let conversion = match (resolved, target) {
                    | (Standard::DublinCore, Standard::ResearchActivityData) => DublinCoreDocument::decode(&content, &mime)
                        .map_err(|diagnostics| {
                            CrosswalkError::ParseFailed(
                                diagnostics
                                    .into_iter()
                                    .map(|diagnostic| diagnostic.to_string())
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            )
                        })
                        .and_then(|document| {
                            document
                                .to_research_activity(&DublinCoreImportOptions {
                                    contact: import.contact.clone(),
                                    identifier: import.identifier.clone(),
                                })
                                .and_then(|(activity, warnings)| {
                                    activity
                                        .encode(&target_mime)
                                        .map(|content| Conversion::new(content, warnings))
                                        .map_err(|error| CrosswalkError::SerializeFailed(error.to_string()))
                                })
                        }),
                    | (source, _) if skip_crosswalk => source
                        .crosswalk(&content, mime, target, target_mime)
                        .map(|converted| Conversion::new(converted, Vec::new())),
                    | (source, _) => source.crosswalk_with_warnings(&content, mime, target, target_mime),
                };
                conversion.map(|conversion| match skip_crosswalk {
                    | true => Conversion::new(conversion.into_value(), Vec::new()),
                    | false => conversion,
                })
            })
        })
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
    let resolved = match format.is_batch() {
        | true => resolve_export_paths(path, &options).await,
        | false => resolve_paths(path, &options).await,
    };
    match resolved {
        | Ok(resolved_paths) => {
            let (skipped, paths): (Vec<_>, Vec<_>) = resolved_paths.iter().partition(|path| {
                let target_mime = MimeType::from(format);
                let is_same_json_lines_path = !format.is_json_lines() || output_path(path, extension).as_path() == path.as_path();
                MimeType::from_path(path) == target_mime && is_same_json_lines_path
            });
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
                        match format.is_batch() {
                            | true => ResearchActivity::batch_read(&item).and_then(|data| data.batch_write(output)),
                            | false => match ResearchActivity::read(&item) {
                                | Ok(data) => match data.write(output) {
                                    | Ok(_) => Ok(()),
                                    | Err(_) => Err(eyre!("Failed to write output file")),
                                },
                                | Err(why) => {
                                    fail!("Read research activity data — {why}");
                                    Err(eyre!("Read research activity data — {why}"))
                                }
                            },
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
                    fail!("Generate {format} — {why}");
                    Err(eyre!("Generate {format} — {why}"))
                }
            }
        }
        | Err(why) => Err(why),
    }
}
#[allow(clippy::too_many_arguments)]
async fn export_as_format_crosswalk(
    path: &Option<PathBuf>,
    options: CommandOptions,
    format: &FileFormat,
    extension: &str,
    output_path: fn(&PathBuf, &str) -> PathBuf,
    progress_type: ProgressType,
    from: Option<Standard>,
    to: Option<Standard>,
    dry_run: bool,
    strict: bool,
    skip: Vec<CheckCategory>,
    import: CrosswalkImportOptions,
    output: &Option<PathBuf>,
) -> ApiResult<()> {
    match to {
        | Some(target) => {
            let threads = options.threads;
            let resolved_paths = resolve_export_paths(path, &options).await;
            match resolved_paths {
                | Ok(resolved_paths) => {
                    let mut seen = HashSet::new();
                    let items: Vec<_> = resolved_paths
                        .iter()
                        .filter(|&p| seen.insert(output_path(p, extension)))
                        .cloned()
                        .collect();
                    let count = items.len() as u64;
                    let ext = extension.to_owned();
                    let collected_checks: Arc<Mutex<Vec<Check>>> = Arc::new(Mutex::new(Vec::new()));
                    let output_dir = output.clone();
                    let skip_crosswalk = skip.contains(&CheckCategory::Crosswalk);
                    if strict && !skip_crosswalk {
                        let preflight = items.iter().try_fold(Vec::new(), |mut checks, item| {
                            crosswalk_item(item, from, target, format, false, &import).map(|conversion| {
                                let (_, warnings) = conversion.into_parts();
                                checks.extend(warnings.to_checks(Some(item.to_string_lossy().to_string())));
                                checks
                            })
                        });
                        match preflight {
                            | Ok(checks) if checks.is_empty() => {}
                            | Ok(checks) => {
                                let warning_count = checks.len();
                                if !progress_type.is_silent() {
                                    info!("=> Crosswalk warnings ({} total):", checks.len().cyan().bold());
                                    for check in &checks {
                                        info!("  {}: {}", check.category, check.message);
                                    }
                                    print_values_as_table::<String>(vec!["", "Count"], CheckSummary::from(checks.as_slice()).rows(), None);
                                }
                                fail!("Strict mode — {warning_count} crosswalk warning(s) found");
                                return Err(eyre!("Strict mode — {warning_count} crosswalk warning(s) found"));
                            }
                            | Err(why) => return Err(eyre!("Crosswalk failed — {why}")),
                        }
                    }
                    let progress = with_progress(
                        items,
                        |item| format!("Generating {format} for {}", item.display()),
                        |item| {
                            let ext = ext.clone();
                            let checks = Arc::clone(&collected_checks);
                            let output_dir = output_dir.clone();
                            let import = import.clone();
                            async move {
                                let output = match &output_dir {
                                    | Some(dir) => {
                                        let name = item.file_name().unwrap_or(item.as_os_str());
                                        dir.join(Path::new(name)).with_extension(&ext)
                                    }
                                    | None => output_path(&item, &ext),
                                };
                                match crosswalk_item(&item, from, target, format, skip_crosswalk, &import) {
                                    | Ok(conversion) => {
                                        let (content, warnings) = conversion.into_parts();
                                        let uri = item.to_string_lossy().to_string();
                                        if let Ok(mut guard) = checks.lock() {
                                            guard.extend(warnings.to_checks(Some(uri)));
                                        }
                                        if strict && !warnings.is_empty() {
                                            Err(eyre!(
                                                "Strict mode — {} crosswalk warning(s) found for {}",
                                                warnings.len(),
                                                item.display()
                                            ))
                                        } else if dry_run {
                                            info!(
                                                "  {} would write {} ({} warnings)",
                                                Label::fmt_skip("DRYRUN"),
                                                output.display(),
                                                warnings.len()
                                            );
                                            Ok(())
                                        } else {
                                            write_file(output, content)
                                        }
                                    }
                                    | Err(why) => Err(eyre!("Crosswalk failed — {why}")),
                                }
                            }
                        },
                        |count| {
                            if dry_run {
                                format!("{}{} file{} would be generated", Label::CHECKMARK, count.green(), suffix(count))
                            } else {
                                format!("{}Done! Generated {} {format} file{}", Label::CHECKMARK, count.green(), suffix(count))
                            }
                        },
                        Some(threads),
                        progress_type,
                    );
                    match progress.await {
                        | Ok(_) => {
                            let all_warnings = collected_checks.lock().map(|g| g.clone()).unwrap_or_default();
                            if !(all_warnings.is_empty() || progress_type.is_silent()) {
                                info!("=> Crosswalk warnings ({} total):", all_warnings.len().cyan().bold());
                                for check in &all_warnings {
                                    info!("  {}: {}", check.category, check.message);
                                }
                                print_values_as_table::<String>(vec!["", "Count"], CheckSummary::from(all_warnings.as_slice()).rows(), None);
                            }
                            if dry_run {
                                info!(
                                    "=> {} Dry-run complete for {} {format} crosswalk file{}",
                                    Label::pass(),
                                    count.green(),
                                    suffix(count)
                                );
                            } else {
                                info!("=> {} Exported {} {format} crosswalk file{}", Label::pass(), count.green(), suffix(count));
                            }
                            if strict && !all_warnings.is_empty() {
                                let count = all_warnings.len();
                                fail!("Strict mode — {count} crosswalk warning(s) found");
                                Err(eyre!("Strict mode — {count} crosswalk warning(s) found"))
                            } else {
                                Ok(())
                            }
                        }
                        | Err(why) => {
                            fail!("Generate {format} crosswalk — {why}");
                            Err(eyre!("Generate {format} crosswalk — {why}"))
                        }
                    }
                }
                | Err(why) => Err(why),
            }
        }
        | None => Err(eyre!(
            "Crosswalk export requires --to <STANDARD> (supported: rads, raid, datacite, dcmi, dcat, invenio, huwise)"
        )),
    }
}
/// Serialize the selected modeled standard as deterministic JSON or YAML schema text.
pub(crate) fn export_schema(standard: Standard, format: &FileFormat) -> ApiResult<String> {
    fn serialize<T: JsonSchema>(format: &FileFormat) -> ApiResult<String> {
        let schema = schema_for!(T);
        match format {
            | FileFormat::Json => serde_json::to_string_pretty(&schema).map_err(|why| eyre!("Serialize JSON schema — {why}")),
            | FileFormat::Yaml => serde_norway::to_string(&schema).map_err(|why| eyre!("Serialize YAML schema — {why}")),
            | _ => Err(eyre!("Schema export supports only JSON or YAML output")),
        }
    }
    match standard {
        | Standard::ApplicationConfigurationResearchEnablement => serialize::<ApplicationConfiguration>(format),
        | Standard::CitationFileFormat => serialize::<cff::Cff>(format),
        | Standard::Ckan => serialize::<ckan::Package>(format),
        | Standard::Datacite => serialize::<datacite::Record>(format),
        | Standard::Dcat => serialize::<dcat::Dataset>(format),
        | Standard::Document | Standard::Text => serialize::<text::Text>(format),
        | Standard::DublinCore => serialize::<DublinCoreDocument>(format),
        | Standard::Huwise => serialize::<huwise::Dataset>(format),
        | Standard::Invenio => serialize::<invenio::Record>(format),
        | Standard::Kitfile | Standard::ModelKit => serialize::<kitfile::Kitfile>(format),
        | Standard::Oci => Err(eyre!("OCI checking has no single generated schema")),
        | Standard::Raid => serialize::<Metadata>(format),
        | Standard::ResearchActivityData => serialize::<ResearchActivity>(format),
    }
}
#[allow(clippy::too_many_arguments)]
#[cfg_attr(not(test), allow(dead_code))]
pub async fn run(
    output: &Option<PathBuf>,
    path: &Option<PathBuf>,
    branch: &Option<String>,
    chrome_path: &Option<PathBuf>,
    commit: &Option<String>,
    filter: &[String],
    ignore: &[String],
    show_aspect_labels: bool,
    show_aspect_scores: bool,
    format: &FileFormat,
    archive_format: Option<ArchiveType>,
    reference: &Option<PathBuf>,
    from: &Option<Standard>,
    to: &Option<Standard>,
    combine: &bool,
    merge_request: &bool,
    raw: &bool,
    skip: &[CheckCategory],
    dry_run: bool,
    strict: bool,
    target: Target,
    since: Option<&str>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> ApiResult<()> {
    run_with_import_context(
        output,
        path,
        branch,
        chrome_path,
        commit,
        filter,
        ignore,
        show_aspect_labels,
        show_aspect_scores,
        format,
        archive_format,
        reference,
        from,
        to,
        &None,
        None,
        None,
        combine,
        merge_request,
        raw,
        skip,
        dry_run,
        strict,
        target,
        since,
        threads,
        verbose,
        offline,
    )
    .await
}
#[allow(clippy::too_many_arguments)]
pub async fn run_with_import_context(
    output: &Option<PathBuf>,
    path: &Option<PathBuf>,
    branch: &Option<String>,
    chrome_path: &Option<PathBuf>,
    commit: &Option<String>,
    filter: &[String],
    ignore: &[String],
    show_aspect_labels: bool,
    show_aspect_scores: bool,
    format: &FileFormat,
    archive_format: Option<ArchiveType>,
    reference: &Option<PathBuf>,
    from: &Option<Standard>,
    to: &Option<Standard>,
    config: &Option<PathBuf>,
    crosswalk_contact: Option<&Path>,
    local_identifier: Option<&str>,
    combine: &bool,
    merge_request: &bool,
    raw: &bool,
    skip: &[CheckCategory],
    dry_run: bool,
    strict: bool,
    target: Target,
    since: Option<&str>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> ApiResult<()> {
    let crosswalk_import = CrosswalkImportOptions::load(config, crosswalk_contact, local_identifier)?;
    match archive_format.is_some() && !format.is_bag() {
        | true => Err(eyre!("--archive-format is only supported for BagIt exports")),
        | false => {
            let is_silent = verbose.is_silent();
            let progress_type = match is_silent || *raw {
                | true => ProgressType::Silent,
                | false => ProgressType::Bar,
            };
            let options = CommandOptions::init()
                .maybe_branch(branch.clone())
                .maybe_commit(commit.clone())
                .maybe_filter(regex_join(filter))
                .maybe_ignore(regex_join(ignore))
                .merge_request(*merge_request)
                .offline(offline)
                .quiet(is_silent)
                .supported(true)
                .threads(threads)
                .build();
            preflight!(&options);
            match offline && *merge_request {
                | true => Err(eyre!("Export cannot resolve a remote merge request in offline mode")),
                | false => {
                    let request = ExportRequest::init()
                        .maybe_output(output.clone())
                        .maybe_path(path.clone())
                        .maybe_chrome_path(chrome_path.clone())
                        .format(format.clone())
                        .maybe_archive_format(archive_format)
                        .show_aspect_labels(show_aspect_labels)
                        .show_aspect_scores(show_aspect_scores)
                        .maybe_reference(reference.clone())
                        .combine(*combine)
                        .dry_run(dry_run)
                        .strict(strict)
                        .crosswalk_import(crosswalk_import)
                        .target(target)
                        .maybe_since(since.map(str::to_string))
                        .skip(skip.to_vec())
                        .progress_type(progress_type)
                        .options(options)
                        .build();
                    let exporter = request.exporter();
                    match (from.is_some() || to.is_some()) && format.is_structured() {
                        | true => exporter.crosswalk(request, *from, *to).await,
                        | false => exporter.export(request).await,
                    }
                }
            }
        }
    }
}
/// Export the canonical Needle-native catalog to `tools.json` atomically.
pub fn tools(output: Option<&Path>, dry_run: bool) -> ApiResult<()> {
    let directory = output.unwrap_or_else(|| Path::new("./export"));
    let destination = directory.join("tools.json");
    match (dry_run, destination.exists()) {
        | (true, _) => {
            info!("{} Would export Needle tools to {}", Label::run(), destination.display());
            Ok(())
        }
        | (false, true) => Err(eyre!("Refusing to overwrite existing Needle tool catalog — {}", destination.display())),
        | (false, false) => create_dir_all(directory)
            .map_err(|why| eyre!("Failed to create tools export directory {} — {why}", directory.display()))
            .and_then(|()| {
                let temporary = directory.join(format!(".tools-{}.json.tmp", nanoid!()));
                sidecar::needle::write_tools(temporary.clone()).and_then(|()| {
                    rename(&temporary, &destination).map_err(|why| {
                        let _ = remove_file(&temporary);
                        eyre!("Failed to publish Needle tool catalog {} — {why}", destination.display())
                    })
                })
            })
            .map(|()| info!("{} Exported Needle tools to {}", Label::pass(), destination.display())),
    }
}
#[cfg(test)]
mod tests;
