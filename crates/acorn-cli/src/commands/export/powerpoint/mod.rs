//! PowerPoint export command adapter over the shared reference-deck renderer.
use crate::{assets::draft_placeholder_bytes, cli::CommandOptions};
use acorn::io::{export::powerpoint, ApiResult};
use acorn_core::prelude::Arc;
use color_eyre::eyre::eyre;
use std::path::PathBuf;

pub(crate) use powerpoint::{add_slide, validate_ooxml_files};

/// Create a PowerPoint presentation from research activity data.
pub fn create(
    paths: impl IntoIterator<Item = PathBuf>,
    options: Option<Arc<CommandOptions>>,
    aspect_charts: Arc<Vec<Option<Vec<u8>>>>,
) -> ApiResult<PathBuf> {
    let options = options.unwrap_or_default();
    options
        .path
        .as_deref()
        .ok_or_else(|| eyre!("Missing path for PowerPoint export — a path or reference must be provided"))
        .and_then(|source| {
            options
                .output
                .as_deref()
                .ok_or_else(|| eyre!("Missing output directory"))
                .and_then(|output| {
                    draft_placeholder_bytes()
                        .and_then(|fallback| powerpoint::render(paths, source, options.reference.as_deref(), output, &aspect_charts, &fallback))
                })
        })
}

#[cfg(test)]
mod tests;
