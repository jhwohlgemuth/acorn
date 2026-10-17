//! Local, explicitly reviewed Power Automate JSONL intake commands.
use acorn::io::api::powerautomate::local::{self, ApplyStatus, StageSummary};
use color_eyre::eyre::{eyre, Result};
use std::path::Path;

/// Apply one proposal after a human reviews its stored source and proposed index.
pub fn apply(config: &Path, submission_id: &str) -> Result<()> {
    local::apply(config, submission_id).map(|status| match status {
        | ApplyStatus::Applied => println!("Applied {submission_id}"),
        | ApplyStatus::AlreadyApplied => println!("Already applied {submission_id}"),
    })
}
/// Stage valid local submissions as durable proposal files.
pub fn stage(config: &Path, inbox: &Path) -> Result<()> {
    StageSummary::stage(config, inbox).and_then(|summary| {
        summary.staged.iter().for_each(|id| println!("Staged {id}"));
        summary.existing.iter().for_each(|id| println!("Already staged {id}"));
        summary
            .rejected
            .iter()
            .for_each(|rejected| eprintln!("Line {} rejected: {}", rejected.line, rejected.reason));
        println!("{summary}");
        match summary.rejected.len() {
            | 0 => Ok(()),
            | count => Err(eyre!("{count} local intake line(s) rejected")),
        }
    })
}

#[cfg(test)]
mod tests;
