use crate::cli::arguments::FileFormat;
use crate::cli::{resolve_paths, CommandOptions};
use crate::commands::preflight;
use acorn::io::bagit::{Bag, BagInfo, Save};
use acorn::io::current_date;
use acorn::io::{create_progress_bar, finish_progress_bar, folder_size, with_progress, FromPath, InputOutput, ProgressType};
use acorn::prelude::{exit, remove_dir_all, Arc, HashSet, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{format_bytes, regex_join, suffix, Label, MimeType, StringConversion};
use acorn::{fail, skip};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result};
use core::future::Future;
use core::iter::once;
use core::pin::Pin;
use owo_colors::OwoColorize;
#[cfg(feature = "pdf")]
use playwright::Playwright;
use tracing::{debug, info};

#[cfg(feature = "pdf")]
mod pdf;
mod powerpoint;

type ExportFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Report>> + 'a>>;
trait ExportHandler {
    fn export<'a>(&'a self, request: ExportRequest<'a>) -> ExportFuture<'a>;
}
struct ExportRequest<'a> {
    output: &'a Option<PathBuf>,
    path: &'a Option<PathBuf>,
    format: &'a FileFormat,
    reference: &'a Option<PathBuf>,
    combine: bool,
    progress_type: ProgressType,
    options: CommandOptions,
}
struct CffExportHandler;
struct StructuredExportHandler;
struct BagExportHandler;
struct PdfExportHandler;
struct PowerPointExportHandler;
impl ExportHandler for CffExportHandler {
    fn export<'a>(&'a self, request: ExportRequest<'a>) -> ExportFuture<'a> {
        Box::pin(async move {
            export_as_format(
                request.path,
                request.options,
                request.format,
                "cff",
                |p, ext| p.with_file_name("CITATION").with_extension(ext),
                request.progress_type,
            )
            .await
        })
    }
}
impl ExportHandler for StructuredExportHandler {
    fn export<'a>(&'a self, request: ExportRequest<'a>) -> ExportFuture<'a> {
        Box::pin(async move {
            let extension = request.format.to_string();
            export_as_format(
                request.path,
                request.options,
                request.format,
                &extension,
                |p, ext| p.with_extension(ext),
                request.progress_type,
            )
            .await
        })
    }
}
impl ExportHandler for BagExportHandler {
    fn export<'a>(&'a self, request: ExportRequest<'a>) -> ExportFuture<'a> {
        Box::pin(async move {
            match (request.path.clone(), request.output.clone()) {
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
    fn export<'a>(&'a self, request: ExportRequest<'a>) -> ExportFuture<'a> {
        Box::pin(async move {
            #[cfg(feature = "pdf")]
            {
                let items = resolve_paths(request.path, &request.options).await;
                let playwright = Playwright::initialize()
                    .await
                    .map_err(|why| eyre!("Failed to initialize Playwright — {why}"))?;
                playwright
                    .install_chromium()
                    .map_err(|why| eyre!("Failed to install Chromium for Playwright — {why}"))?;
                let browser = playwright
                    .chromium()
                    .launcher()
                    .headless(true)
                    .launch()
                    .await
                    .map_err(|why| eyre!("Failed to launch Chromium — {why}"))?;
                let context = browser
                    .context_builder()
                    .build()
                    .await
                    .map_err(|why| eyre!("Failed to create browser context — {why}"))?;
                let page = context.new_page().await.map_err(|why| eyre!("Failed to create browser page — {why}"))?;
                let page = &page;
                let progress = with_progress(
                    items,
                    |path| format!("Generating PDF for {}", path.file_name_with_parent()),
                    |path| async move {
                        let pdf_options = CommandOptions::init().path(path.clone()).maybe_output(request.output.clone()).build();
                        pdf::create(page, pdf_options).await;
                        Ok(())
                    },
                    |count| format!("{}Done! Exported {} PDF{}", Label::CHECKMARK, count.green(), suffix(count)),
                    Some(1),
                    request.progress_type,
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
            #[cfg(not(feature = "pdf"))]
            {
                Err(eyre!("PDF export requires the 'pdf' feature. Rebuild with --features pdf"))
            }
        })
    }
}
impl ExportHandler for PowerPointExportHandler {
    fn export<'a>(&'a self, request: ExportRequest<'a>) -> ExportFuture<'a> {
        Box::pin(async move {
            let paths = resolve_paths(request.path, &request.options).await;
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
    combine: &bool,
    merge_request: &bool,
    raw: &bool,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
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
    let exporter = exporter_for(format);
    exporter
        .export(ExportRequest {
            output,
            path,
            format,
            reference,
            combine: *combine,
            progress_type,
            options,
        })
        .await
}
async fn export_as_format(
    path: &Option<PathBuf>,
    options: CommandOptions,
    format: &FileFormat,
    extension: &str,
    output_path: fn(&PathBuf, &str) -> PathBuf,
    progress_type: ProgressType,
) -> Result<(), Report> {
    let threads = options.threads;
    let from_options = resolve_paths(path, &options).await;
    let (skipped, paths): (Vec<_>, Vec<_>) = from_options
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
