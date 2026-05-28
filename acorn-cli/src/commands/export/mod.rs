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
use color_eyre::eyre::{Report, Result};
use core::iter::once;
use owo_colors::OwoColorize;
#[cfg(feature = "pdf")]
use playwright::Playwright;
use tracing::{debug, info};

#[cfg(feature = "pdf")]
mod pdf;
mod powerpoint;

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
    let extension = format.to_string();
    match format {
        | FileFormat::Cff => {
            export_convert(
                path,
                options,
                format,
                &extension,
                |p, ext| p.with_file_name("CITATION").with_extension(ext),
                progress_type,
            )
            .await
        }
        | FileFormat::Json | FileFormat::Markdown | FileFormat::Yaml => {
            export_convert(path, options, format, &extension, |p, ext| p.with_extension(ext), progress_type).await
        }
        | FileFormat::Bag => {
            #[allow(clippy::unwrap_used)]
            let base = path.clone().unwrap().display().to_string();
            let date = current_date();
            let size = format_bytes(folder_size(base.clone()));
            let info = BagInfo::init().date(date).size(size).build();
            debug!("=> {} {:#?}", Label::using(), info);
            let bag = Bag::init().base_directory(base).info(info).build();
            #[allow(clippy::unwrap_used)]
            let destination = output.clone().unwrap();
            bag.save(destination.clone())
                .and_then(|_| Bag::verify(destination.clone()))
                .and_then(|_| remove_dir_all(destination).map_err(|why| why.into()))
        }
        | FileFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                use color_eyre::eyre::eyre;
                let items = resolve_paths(path, &options).await;
                let playwright = match Playwright::initialize().await {
                    | Ok(playwright) => playwright,
                    | Err(why) => return Err(eyre!("Failed to initialize Playwright — {why}")),
                };
                match playwright.install_chromium() {
                    | Ok(_) => (),
                    | Err(why) => return Err(eyre!("Failed to install Chromium for Playwright — {why}")),
                };
                let chromium = playwright.chromium();
                let browser = match chromium.launcher().headless(true).launch().await {
                    | Ok(browser) => browser,
                    | Err(why) => return Err(eyre!("Failed to launch Chromium — {why}")),
                };
                let context = match browser.context_builder().build().await {
                    | Ok(context) => context,
                    | Err(why) => return Err(eyre!("Failed to create browser context — {why}")),
                };
                let page = match context.new_page().await {
                    | Ok(page) => page,
                    | Err(why) => return Err(eyre!("Failed to create browser page — {why}")),
                };
                let page = &page;
                let progress = with_progress(
                    items,
                    |path| format!("Generating PDF for {}", path.file_name_with_parent()),
                    |path| async move {
                        let pdf_options = CommandOptions::init().path(path.clone()).maybe_output(output.clone()).build();
                        pdf::create(page, pdf_options).await;
                        Ok(())
                    },
                    |count| format!("{}Done! Exported {} PDF{}", Label::CHECKMARK, count.green(), suffix(count)),
                    Some(1),
                    progress_type,
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
                use color_eyre::eyre::eyre;
                Err(eyre!("PDF export requires the 'pdf' feature. Rebuild with --features pdf"))
            }
        }
        | FileFormat::Powerpoint => {
            let paths = resolve_paths(path, &options).await;
            let _options = CommandOptions::init()
                .maybe_path(path.clone())
                .maybe_output(output.clone())
                .maybe_reference(reference.clone())
                .build();
            let create_options = Arc::new(_options);
            if *combine {
                let progress = create_progress_bar(
                    1,
                    match progress_type {
                        | ProgressType::Silent => ProgressType::Silent,
                        | _ => ProgressType::Spinner,
                    },
                );
                progress.set_message("Generating combined PowerPoint artifact");
                let _ = powerpoint::create(paths, Some(Arc::clone(&create_options)));
                progress.inc(1);
                finish_progress_bar(&progress, format!("{}Done! Exported combined PowerPoint file", Label::CHECKMARK));
                Ok(())
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
                    progress_type,
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
    }
}
async fn export_convert(
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

#[cfg(test)]
mod tests;
