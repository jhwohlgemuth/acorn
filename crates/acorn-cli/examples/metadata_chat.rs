//! Open a sample citation file in the interactive metadata chat.

use color_eyre::eyre::{eyre, Result};
use std::{fs, path::Path, process::Command};

const CITATION: &str = include_str!("metadata_chat/CITATION.cff");

fn main() -> Result<()> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let directory = tempfile::tempdir()?;
    fs::write(directory.path().join("CITATION.cff"), CITATION)?;
    let status = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(manifest)
        .args(["--bin", "acorn", "--", "--no-local-database", "ask", "-i", "/open CITATION.cff"])
        .current_dir(directory.path())
        .status()?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| eyre!("Interactive chat exited with {status}"))
}
