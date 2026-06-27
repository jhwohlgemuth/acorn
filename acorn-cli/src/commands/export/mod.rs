use crate::cli::arguments::FileFormat;
use crate::cli::parse::infer_standard_from_content;
use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use acorn::analyzer::{Check, IntoChecks, Standard};
use acorn::io::bagit::{Bag, BagInfo, Save};
use acorn::io::current_date;
use acorn::io::{create_progress_bar, finish_progress_bar, folder_size, read_file, with_progress, write_file, FromPath, InputOutput, ProgressType};
use acorn::io::{ApiFuture, ApiResult};
use acorn::prelude::{exit, remove_dir_all, Arc, HashSet, Mutex, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{format_bytes, regex_join, suffix, Label, MimeType, StringConversion};
use acorn::{fail, skip};
use bon::Builder;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use core::iter::once;
use owo_colors::OwoColorize;
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
#[derive(Builder, Debug, Default)]
#[builder(start_fn = init)]
struct ExportRequest {
    output: Option<PathBuf>,
    path: Option<PathBuf>,
    #[builder(default)]
    format: FileFormat,
    reference: Option<PathBuf>,
    combine: bool,
    #[builder(default)]
    dry_run: bool,
    #[builder(default)]
    strict: bool,
    #[builder(default)]
    progress_type: ProgressType,
    #[builder(default)]
    options: CommandOptions,
}
struct CffExportHandler;
struct StructuredExportHandler;
struct BagExportHandler;
struct PdfExportHandler;
struct PowerPointExportHandler;
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
                    | Ok(paths) => match pdf::initialize_page().await {
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
    dry_run: bool,
    strict: bool,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> ApiResult<()> {
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
        .threads(threads)
        .build();
    preflight!(&options);
    let request = ExportRequest::init()
        .maybe_output(output.clone())
        .maybe_path(path.clone())
        .format(format.clone())
        .maybe_reference(reference.clone())
        .combine(*combine)
        .dry_run(dry_run)
        .strict(strict)
        .progress_type(progress_type)
        .options(options)
        .build();
    let exporter = exporter_for(&request.format);
    if (from.is_some() || to.is_some()) && format.is_structured() {
        exporter.crosswalk(request, *from, *to).await
    } else {
        exporter.export(request).await
    }
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
                    fail!("Generate {} — {}", format, why);
                    exit(exitcode::DATAERR);
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
) -> ApiResult<()> {
    match to {
        | Some(target) => {
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
                    let collected_checks: Arc<Mutex<Vec<Check>>> = Arc::new(Mutex::new(Vec::new()));
                    let progress = with_progress(
                        items,
                        |item| format!("Generating {format} for {}", item.display()),
                        |item| {
                            let ext = ext.clone();
                            let checks = Arc::clone(&collected_checks);
                            async move {
                                let output = output_path(&item, &ext);
                                match read_file(item.clone()) {
                                    | Ok(content) => {
                                        let mime = MimeType::from_path(&item);
                                        let source = match from {
                                            | Some(value) => Ok(value),
                                            | None => infer_standard_from_content(&item, &content, &mime),
                                        };
                                        let processed = source.and_then(|resolved| {
                                            let target_mime = MimeType::from(format);
                                            if strict || dry_run {
                                                resolved
                                                    .crosswalk_with_warnings(&content, mime, target, target_mime)
                                                    .map_err(|e| eyre!("Crosswalk failed — {e}"))
                                            } else {
                                                resolved
                                                    .crosswalk(&content, mime, target, target_mime)
                                                    .map(|c| (c, Vec::new()))
                                                    .map_err(|e| eyre!("Crosswalk failed — {e}"))
                                            }
                                        });
                                        match processed {
                                            | Ok((content, warnings)) => {
                                                for w in &warnings {
                                                    info!("  {} crosswalk warning: {w}", Label::run());
                                                }
                                                if !warnings.is_empty() {
                                                    if let Ok(mut guard) = checks.lock() {
                                                        let uri = item.to_string_lossy().to_string();
                                                        guard.extend(warnings.to_checks(Some(uri)));
                                                    }
                                                }
                                                if dry_run {
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
                                            | Err(why) => Err(why),
                                        }
                                    }
                                    | Err(why) => Err(why),
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
                            if !all_warnings.is_empty() {
                                info!("=> Crosswalk warnings ({} total):", all_warnings.len().cyan().bold());
                                for check in &all_warnings {
                                    info!("  {check}");
                                }
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
                                fail!("Strict mode — {} crosswalk warning(s) found", all_warnings.len());
                                exit(exitcode::DATAERR);
                            }
                            Ok(())
                        }
                        | Err(why) => {
                            fail!("Generate {} crosswalk — {}", format, why);
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
