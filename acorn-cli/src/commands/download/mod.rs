use acorn::io::config::{ApplicationConfiguration, Bucket};
use acorn::io::database::print_activity;
use acorn::io::database::schema::{ActivityRow, Row, Table};
use acorn::io::database::Database;
use acorn::io::InputOutput;
use acorn::prelude::PathBuf;
use acorn::util::{suffix, Label, StringConversion};
use color_eyre::eyre::{Report, Result};
use owo_colors::OwoColorize;
use std::process::exit;
use tracing::error;

pub async fn run(
    config: &Option<PathBuf>,
    url: &Option<String>,
    output: &Option<PathBuf>,
    offline: &bool,
    database_path: &Option<PathBuf>,
) -> Result<(), Report> {
    if *offline {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
    }
    let database = Database::<Table>::from_path(database_path.clone());
    if let Some(path) = config {
        match ApplicationConfiguration::read(path.clone()) {
            | Ok(content) => {
                let mut total: usize = 0;
                if let Some(buckets) = content.buckets {
                    for bucket in buckets {
                        if let Some(path) = output.clone() {
                            total += if *offline || bucket.clone().code_repository.is_local() {
                                bucket.copy_files(path)
                            } else {
                                bucket.download_files(path).await
                            };
                        }
                    }
                }
                println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
                if let Some(path) = output {
                    let activity = ActivityRow::init()
                        .command("download".to_string())
                        .user_path(path.to_absolute_string())
                        .success(true)
                        .build();
                    database.with_connection(|conn| activity.clone().insert(conn).map(|_| ()))?;
                }
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
                bucket.copy_files(path.clone())
            } else {
                bucket.download_files(path.clone()).await
            };
            println!("\n\n   => {} Processed {} item{}\n", Label::pass(), total.green(), suffix(total));
            let activity = ActivityRow::init()
                .command("download".to_string())
                .user_path(path.to_absolute_string())
                .success(true)
                .build();
            database.with_connection(|conn| activity.clone().insert(conn).map(|_| ()))?;
        }
    }
    print_activity(ActivityRow::default(), database_path.as_ref());
    Ok(())
}
