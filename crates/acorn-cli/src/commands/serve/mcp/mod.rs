use acorn::io::mcp;
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;

pub async fn run(offline: bool, allow_mutation: bool, database_path: Option<PathBuf>) -> Result<(), Report> {
    mcp::run(offline, allow_mutation, database_path).await
}
