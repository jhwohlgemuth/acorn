pub mod model;
pub mod spec;

use crate::cli::arguments::{ArchiveType, Import};
use crate::commands::download::{log_activity, print_result};
use acorn::io::config::{ApplicationConfiguration, Bucket, BucketOptions};
use acorn::io::{uri_to_path, InputOutput, TemporaryDirectory};
use acorn::util::StringConversion;
use acorn_core::util::MimeType;
use color_eyre::eyre::{eyre, Report, Result};
use futures::{future, stream, StreamExt, TryStreamExt};
use std::path::PathBuf;

/// Imports local buckets, remote repositories, and BagIt archives.
pub(crate) async fn run(
    arguments: &Import,
    database_path: &Option<PathBuf>,
    threads: usize,
    offline: bool,
    no_local_database: bool,
) -> Result<(), Report> {
    let Import {
        uri,
        config,
        filter,
        ignore,
        output,
        flatten,
        clobber,
        archive_format,
        verbose,
        ..
    } = arguments;
    let quiet = verbose.is_silent();
    let options = BucketOptions::init()
        .filter(filter.clone())
        .ignore(ignore.clone())
        .flatten(*flatten)
        .clobber(*clobber)
        .quiet(quiet)
        .threads(threads)
        .maybe_output(output.clone())
        .maybe_database_path(database_path.clone())
        .no_local_database(no_local_database)
        .offline(offline)
        .build();
    let sources = match ApplicationConfiguration::resolve(config) {
        | Some(path) => ApplicationConfiguration::read(path).map(|configuration| {
            configuration
                .buckets
                .unwrap_or_default()
                .into_iter()
                .map(InputSource::from)
                .collect::<Vec<_>>()
        }),
        | None => Ok(uri.iter().cloned().map(InputSource::from).collect::<Vec<_>>()),
    };
    match sources.and_then(|sources| match sources.is_empty() {
        | true => Err(eyre!("No import URI or configuration file found")),
        | false => Ok(sources),
    }) {
        | Ok(sources) => stream::iter(sources)
            .then(|source| source.import(*archive_format, &options))
            .try_fold(0_usize, |total, count| future::ready(Ok(total.saturating_add(count))))
            .await
            .map(|total| {
                if let Some(path) = &options.output {
                    if !options.no_local_database {
                        log_activity("import", path.to_absolute_path(), &options.database_path, true);
                    }
                }
                if !options.quiet {
                    print_result(total, options.output.as_ref());
                }
            }),
        | Err(why) => Err(why),
    }
}
enum InputSource {
    Bucket(Bucket),
    Uri(String),
}
impl From<Bucket> for InputSource {
    fn from(value: Bucket) -> Self {
        Self::Bucket(value)
    }
}
impl From<String> for InputSource {
    fn from(value: String) -> Self {
        Self::Uri(value)
    }
}
impl InputSource {
    async fn import(self, format: Option<ArchiveType>, options: &BucketOptions) -> Result<usize, Report> {
        match self {
            | Self::Bucket(bucket) => bucket.import(options, None).await,
            | Self::Uri(source) => {
                let path = uri_to_path(&source);
                let forced = format.map(MimeType::from);
                let inferred = MimeType::from(source.as_str());
                match (path.is_dir(), path.is_file(), forced.clone(), inferred) {
                    | (true, _, None, _) => match path.canonicalize() {
                        | Ok(path) => Bucket::from(path.as_path()).import(options, Some(source.as_str())).await,
                        | Err(why) => Err(why.into()),
                    },
                    | (_, true, _, _) | (_, _, Some(_), _) | (_, _, _, MimeType::Gzip | MimeType::SevenZip | MimeType::Tar | MimeType::Zip) => {
                        match TemporaryDirectory::create("acorn-import") {
                            | Ok(temporary) => match Bucket::archive(&source, forced, &temporary, options.offline).await {
                                | Ok((bucket, provenance)) => bucket.import(options, Some(&provenance)).await,
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why),
                        }
                    }
                    | (_, _, _, _) if source.contains("://") => Bucket::from(source.as_str()).import(options, Some(source.as_str())).await,
                    | _ => Err(eyre!("Import source does not exist or has an unsupported format: {source}")),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
