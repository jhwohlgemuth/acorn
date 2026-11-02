use super::{format_diff, write_formatted_content};
use crate::cli::{resolve_paths, CommandOptions};
use crate::io::interactive::Viewport;
use acorn::io::mcp::format_acre;
use acorn_schema::enrichment::Provider;
use color_eyre::eyre::{eyre, Report, Result};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

const FILE_NAMES: [&str; 4] = [".acorn.json", ".acorn.jsonc", ".acorn.yaml", ".acorn.yml"];

fn format_file(path: &Path, options: &CommandOptions) -> Result<String, Report> {
    read_to_string(path)
        .map_err(|why| eyre!("Read ACRE configuration — {why}"))
        .and_then(|source| format_acre(path, &source).map(|formatted| (source, formatted)))
        .and_then(|(source, formatted)| {
            let feedback = match source == formatted {
                | true => format!("No format changes for {}", path.display()),
                | false if options.dry_run => {
                    format!(
                        "Dry run: {}\n{}",
                        path.display(),
                        format_diff(&source, &formatted, path, !options.no_color)
                    )
                }
                | false => format!("Formatted {}", path.display()),
            };
            write_formatted_content(path, options, &source, &formatted).map(|_| feedback)
        })
}
async fn format_paths(path: &Option<PathBuf>, options: &CommandOptions) -> Result<(), Report> {
    let explicit_file = path.as_ref().is_some_and(|candidate| candidate.is_file());
    resolve_paths(path, options).await.and_then(|paths| {
        let selected = paths
            .iter()
            .filter(|candidate| {
                explicit_file
                    || candidate
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| FILE_NAMES.contains(&name))
            })
            .collect::<Vec<_>>();
        let viewport = match options.interactive {
            | true => Viewport::open().map(Some),
            | false => Ok(None),
        };
        viewport.and_then(|mut viewport| {
            viewport
                .as_mut()
                .map_or(Ok(()), |viewport| viewport.push("Formatting ACRE configuration"))
                .and_then(|()| {
                    let results = selected.par_iter().map(|path| format_file(path, options)).collect::<Vec<_>>();
                    viewport
                        .as_mut()
                        .map_or(Ok(()), |viewport| {
                            selected.iter().zip(&results).try_for_each(|(path, result)| {
                                viewport.push(match result {
                                    | Ok(feedback) => feedback.clone(),
                                    | Err(why) => format!("{}: failed — {why}", path.display()),
                                })
                            })
                        })
                        .and_then(|()| results.into_iter().collect::<Result<Vec<_>, _>>().map(|_| ()))
                })
        })
    })
}
pub(super) async fn run(
    path: &Option<PathBuf>,
    options: &CommandOptions,
    config: &Option<PathBuf>,
    enrich: bool,
    providers: &[Provider],
) -> Result<(), Report> {
    match (config.is_some(), enrich, providers.is_empty()) {
        | (true, _, _) => Err(eyre!("--config is not supported when formatting ACRE configuration")),
        | (_, true, _) => Err(eyre!("--enrich is not supported when formatting ACRE configuration")),
        | (_, _, false) => Err(eyre!("--provider is not supported when formatting ACRE configuration")),
        | (false, false, true) => format_paths(path, options).await,
    }
}
