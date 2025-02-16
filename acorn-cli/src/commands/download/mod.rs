use acorn_lib::util::Label;
use acorn_lib::BucketsConfig;
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;
use tracing::error;

#[allow(clippy::too_many_arguments)]
pub fn run(buckets: &Option<PathBuf>, output: &Option<PathBuf>) -> Result<(), Report> {
    if let Some(path) = buckets {
        match BucketsConfig::read_json(path.clone()) {
            | Ok(content) => {
                content.buckets.into_iter().for_each(|bucket| {
                    if let Some(path) = output {
                        bucket.download_files(path.to_path_buf());
                    }
                });
            }
            | Err(err) => {
                error!("=> {} Failed to download data - {}", Label::fail(), err);
            }
        };
    }
    Ok(())
}
