use acorn_lib::util::{suffix, Label};
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
        match BucketsConfig::read(path.clone()) {
            | Some(content) => {
                let mut total: usize = 0;
                content.buckets.into_iter().for_each(|bucket| {
                    if let Some(path) = output {
                        total += bucket.download_files(path.to_path_buf());
                    }
                });
                println!("\n\n   => {} Downloaded {} item{}\n", Label::pass(), total.green(), suffix(total));
            }
            | None => {
                error!("=> {} Download data", Label::fail());
                std::process::exit(exitcode::DATAERR);
            }
        };
    }
    Ok(())
}
