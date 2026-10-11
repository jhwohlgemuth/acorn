use super::write_formatted_content;
use crate::cli::{resolve_paths, CommandOptions};
use acorn::io::{config::application::ApplicationConfiguration, FromPath, InputOutput};
use acorn_core::util::MimeType;
use acorn_schema::{enrichment::Provider, validation::Validate};
use color_eyre::eyre::{eyre, Report, Result};
use dprint_core::configuration::NewLineKind;
use dprint_plugin_json::{
    configuration::{ConfigurationBuilder, EofNewLineKind, TrailingCommaKind},
    format_text as format_json,
};
use pretty_yaml::{config::FormatOptions, format_text as format_yaml};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

const FILE_NAMES: [&str; 4] = [".acorn.json", ".acorn.jsonc", ".acorn.yaml", ".acorn.yml"];

trait Format {
    fn format(&self, path: &Path, source: &str) -> Result<String, Report>;
}
impl Format for ApplicationConfiguration {
    fn format(&self, path: &Path, source: &str) -> Result<String, Report> {
        self.validate()
            .map_err(|why| eyre!("Validate ACRE configuration — {why}"))
            .and_then(|()| match MimeType::from_path(path) {
                | MimeType::Json | MimeType::Jsonc => {
                    let mut builder = ConfigurationBuilder::new();
                    let configuration = builder
                        .use_tabs(false)
                        .indent_width(2)
                        .new_line_kind(NewLineKind::LineFeed)
                        .eof_new_line(EofNewLineKind::Always)
                        .package_json_apply_conventions(false)
                        .trailing_commas(TrailingCommaKind::Maintain)
                        .build();
                    format_json(path, source, &configuration)
                        .map(|content| content.unwrap_or_else(|| source.to_string()))
                        .map_err(|why| eyre!("Format ACRE JSON/JSONC — {why}"))
                }
                | MimeType::Yaml => format_yaml(source, &FormatOptions::default())
                    .map(|content| match content.ends_with('\n') {
                        | true => content,
                        | false => format!("{content}\n"),
                    })
                    .map_err(|why| eyre!("Format ACRE YAML — {why}")),
                | _ => Err(eyre!("ACRE formatting supports JSON, JSONC, YAML, and YML files")),
            })
    }
}
fn format_file(path: &Path, options: &CommandOptions) -> Result<(), Report> {
    ApplicationConfiguration::read(path.to_path_buf())
        .map_err(|why| eyre!("Read ACRE configuration — {why}"))
        .and_then(|configuration| {
            read_to_string(path)
                .map_err(|why| eyre!("Read ACRE configuration — {why}"))
                .map(|source| (configuration, source))
        })
        .and_then(|(configuration, source)| configuration.format(path, &source).map(|formatted| (source, formatted)))
        .and_then(|(source, formatted)| write_formatted_content(path, options, &source, &formatted).map(|_| ()))
}
async fn format_paths(path: &Option<PathBuf>, options: &CommandOptions) -> Result<(), Report> {
    let explicit_file = path.as_ref().is_some_and(|candidate| candidate.is_file());
    resolve_paths(path, options).await.and_then(|paths| {
        paths
            .par_iter()
            .filter(|candidate| {
                explicit_file
                    || candidate
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| FILE_NAMES.contains(&name))
            })
            .try_for_each(|candidate| format_file(candidate, options))
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
