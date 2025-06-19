use acorn_lib::util::{get_suffix, Label};
use acorn_lib::BucketsConfig;
use color_eyre::eyre::{Report, Result};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use tracing::error;

#[allow(clippy::too_many_arguments)]
pub fn run(buckets: &Option<PathBuf>, output: &Option<PathBuf>, offline: &bool) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
        unimplemented!("Offline mode is not implemented yet");
    }
    if let Some(path) = buckets {
        match BucketsConfig::read_json(path.clone()) {
            | Ok(content) => {
                let mut total: usize = 0;
                content.buckets.into_iter().for_each(|bucket| {
                    if let Some(path) = output {
                        total += bucket.download_files(path.to_path_buf());
                    }
                });
                println!("\n\n   => {} Downloaded {} item{}\n", Label::pass(), total.green(), get_suffix(total));
            }
            | Err(err) => {
                error!("=> {} Failed to download data - {}", Label::fail(), err);
            }
        };
    }
    Ok(())
}
