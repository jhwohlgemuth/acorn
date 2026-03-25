use crate::cli::arguments::FileFormat;
use crate::cli::{paths_from_options, CommandLineOptions};
use acorn::io::bagit::{Bag, BagInfo, Save};
use acorn::io::{folder_size, with_progress, with_progress_bar, FromPath, InputOutput};
use acorn::prelude::{exit, remove_dir_all, HashSet, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{current_date, format_bytes, suffix, Label, MimeType};
use acorn::{fail, skip};
use color_eyre::eyre::{Report, Result};
use core::iter::once;
use indicatif::{ProgressBar, ProgressStyle};
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
    format: &FileFormat,
    reference: &Option<PathBuf>,
    combine: &bool,
    merge_request: &bool,
    offline: &bool,
    threads: usize,
) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
        unimplemented!("Offline mode is not implemented yet");
    }
    let options = CommandLineOptions::init()
        .maybe_branch(branch.clone())
        .maybe_commit(commit.clone())
        .merge_request(*merge_request)
        .build();
    let extension = format.to_string();
    match format {
        | FileFormat::Cff => {
            export_convert(
                path,
                options,
                format,
                &extension,
                |p, ext| p.with_file_name("CITATION").with_extension(ext),
                threads,
            )
            .await
        }
        | FileFormat::Json | FileFormat::Markdown | FileFormat::Yaml => {
            export_convert(path, options, format, &extension, |p, ext| p.with_extension(ext), threads).await
        }
        | FileFormat::Bag => {
            let base = path.clone().unwrap().display().to_string();
            let date = current_date();
            let size = format_bytes(folder_size(base.clone()));
            let info = BagInfo::init().date(date).size(size).build();
            debug!("=> {} {:#?}", Label::using(), info);
            let bag = Bag::init().base_directory(base).info(info).build();
            let destination = output.clone().unwrap();
            bag.save(destination.clone())
                .and_then(|_| Bag::verify(destination.clone()))
                .and_then(|_| remove_dir_all(destination).map_err(|why| why.into()))
        }
        | FileFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                use acorn::util::StringConversion;

                let items = paths_from_options(path, &Some(options)).await;
                let playwright = Playwright::initialize().await.unwrap();
                playwright.install_chromium().unwrap();
                let chromium = playwright.chromium();
                let browser = chromium.launcher().headless(true).launch().await.unwrap();
                let context = browser.context_builder().build().await.unwrap();
                let page = context.new_page().await.unwrap();
                let page = &page;
                let progress = with_progress_bar(
                    items,
                    |path| format!("Generating PDF for {}", path.file_name_with_parent()),
                    |path| async move {
                        let pdf_options = CommandLineOptions::init().path(path.clone()).maybe_output(output.clone()).build();
                        pdf::create(page, pdf_options).await;
                        Ok(())
                    },
                    |count| format!("{}Done! Exported {} PDF{}", Label::CHECKMARK, count.green(), suffix(count)),
                    Some(1),
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
            let paths = paths_from_options(path, &Some(options.clone())).await;
            let options = CommandLineOptions::init()
                .maybe_path(path.clone())
                .maybe_output(output.clone())
                .maybe_reference(reference.clone())
                .build();
            let count = if *combine { 1 } else { paths.len() as u64 };
            let progress = ProgressBar::new(count);
            progress.set_style(ProgressStyle::with_template(Label::PROGRESS_BAR_TEMPLATE).unwrap());
            if *combine {
                progress.set_message("Generating combined PowerPoint artifact");
                let _ = powerpoint::create(paths, Some(options));
                progress.inc(1);
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported combined PowerPoint file", Label::pass());
            } else {
                for path in &paths {
                    progress.set_message(format!("Generating PowerPoint for {}", path.display()));
                    let _ = powerpoint::create(once(path.to_path_buf()), Some(options.clone()));
                    progress.inc(1);
                }
                progress.finish_with_message(format!("{}Done", Label::CHECKMARK));
                info!("=> {} Exported {} PowerPoint file{}", Label::pass(), count.green(), suffix(count));
            }
            Ok(())
        }
    }
}
async fn export_convert(
    path: &Option<PathBuf>,
    options: CommandLineOptions,
    format: &FileFormat,
    extension: &str,
    output_path: fn(&PathBuf, &str) -> PathBuf,
    threads: usize,
) -> Result<(), Report> {
    let from_options = paths_from_options(path, &Some(options)).await;
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
