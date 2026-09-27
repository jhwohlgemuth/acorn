pub mod model;
pub mod spec;

use crate::{
    cli::arguments::Import,
    commands::download::{log_activity, print_result},
};
use acorn::{
    io::config::{ApplicationConfiguration, BucketExtension, BucketOptions, TransferPolicy},
    util::StringConversion,
};
use acorn_core::{
    options::{CommonRuntime, RuntimeOptions},
    util::MimeType,
};
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
        .common(CommonRuntime {
            filter: filter.clone(),
            ignore: ignore.clone(),
            offline,
            quiet,
            threads,
            ..CommonRuntime::default()
        })
        .extension(BucketExtension {
            clobber: *clobber,
            database_path: database_path.clone(),
            flatten: *flatten,
            no_local_database,
            ..BucketExtension::default()
        })
        .maybe_output(output.clone())
        .build();
    let shared = &options;
    let format = archive_format.map(MimeType::from);
    let shared_format = format.as_ref();
    let sources = ApplicationConfiguration::sources(config, uri);
    let empty = sources.as_ref().map(|sources| sources.is_empty()).unwrap_or_default();
    match empty {
        | true => Err(eyre!("No import URI or configuration file found")),
        | false => match sources {
            | Ok(sources) => stream::iter(sources)
                .then(|source| async move {
                    match source.resolve(shared_format.cloned(), offline).await {
                        | Ok(bucket) => bucket.transfer(shared, TransferPolicy::Import).await,
                        | Err(why) => Err(why),
                    }
                })
                .try_fold(0_usize, |total, count| future::ready(Ok(total.saturating_add(count))))
                .await
                .map(|total| {
                    let RuntimeOptions { common, extension, output } = &options;
                    let CommonRuntime { quiet, .. } = common;
                    let BucketExtension {
                        database_path,
                        no_local_database,
                        ..
                    } = extension;
                    if let Some(path) = output {
                        if !*no_local_database {
                            log_activity("import", path.to_absolute_path(), database_path, true);
                        }
                    }
                    if !*quiet {
                        print_result(total, output.as_ref());
                    }
                }),
            | Err(why) => Err(why),
        },
    }
}

#[cfg(test)]
mod tests;
