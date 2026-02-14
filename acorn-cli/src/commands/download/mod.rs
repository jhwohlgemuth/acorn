use crate::config::{ApplicationConfiguration, Bucket};
use acorn::io::InputOutput;
use acorn::prelude::PathBuf;
use acorn::util::{suffix, Label};
use color_eyre::eyre::{Report, Result};
use owo_colors::OwoColorize;
use std::process::exit;
use tracing::error;

pub fn run(config: &Option<PathBuf>, url: &Option<String>, output: &Option<PathBuf>, offline: &bool) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
    }
    if let Some(path) = config {
        match ApplicationConfiguration::read(path.clone()) {
            | Ok(content) => {
                let mut total: usize = 0;
                if let Some(buckets) = content.buckets {
                    buckets.into_iter().for_each(|bucket| {
                        if let Some(path) = output.clone() {
                            total += if *offline || bucket.clone().code_repository.is_local() {
                                bucket.copy_files(path)
                            } else {
                                bucket.download_files(path)
                            };
                        }
                    });
                }
                println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
            }
            | Err(why) => {
                error!("=> {} Download data - {why}", Label::fail());
                exit(exitcode::DATAERR);
            }
        };
    } else if let Some(value) = url {
        let mut total: usize = 0;
        let bucket = Bucket::from(value.as_ref());
        if let Some(path) = output.clone() {
            total += if *offline || bucket.clone().code_repository.is_local() {
                bucket.copy_files(path)
            } else {
                bucket.download_files(path)
            };
            println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
        }
    }
    Ok(())
}
