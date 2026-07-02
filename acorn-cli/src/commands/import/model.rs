use crate::cli::Void;
use acorn::io::database::{schema::Table, Database, Operations, PersistStatus};
use acorn::prelude::PathBuf;
use acorn::util::Label;

/// Import model metadata into the local database.
pub async fn run(database_path: &Option<PathBuf>) -> Void {
    match Database::<Table>::from_path(database_path.clone()).populate(Table::Models).await? {
        | PersistStatus::Downloaded(count) => println!("=> {} Imported {count} model records", Label::pass()),
        | PersistStatus::AlreadyExists => println!("=> {} Model records already exist in the local database", Label::pass()),
    }
    Ok(())
}
