//! Read-only artifact destinations shared by export commands and native previews.
use crate::io::{ApiResult, InputOutput};
use acorn_core::util::MimeType;
use acorn_host::fs::SafePath;
use acorn_schema::research_activity::ResearchActivity;
use color_eyre::eyre::eyre;
use std::path::{Path, PathBuf};

/// Resolve the direct BagIt command's archive format and final destination.
///
/// Path authority remains with the caller: direct commands accept absolute destinations, while routed callers validate the returned path with their artifact policy.
pub fn bag_destination(requested: &Path, archive_format: Option<MimeType>) -> ApiResult<(PathBuf, MimeType)> {
    let inferred = match MimeType::from(requested.to_string_lossy().as_ref()) {
        | format @ (MimeType::Gzip | MimeType::SevenZip | MimeType::Tar | MimeType::Zip) => Some(format),
        | _ => None,
    };
    match archive_format.is_some() && inferred.is_some() && archive_format != inferred {
        | true => Err(eyre!("--archive-format conflicts with the output archive suffix")),
        | false => {
            let format = archive_format.or(inferred.clone()).unwrap_or(MimeType::Zip);
            let destination = if inferred.is_some() {
                requested.to_path_buf()
            } else {
                requested.with_extension(format.clone().file_type())
            };
            Ok((destination, format))
        }
    }
}
/// Resolve a fact-sheet PDF filename from the source activity identifier.
pub fn pdf_destination(source: &Path, output_dir: &Path) -> ApiResult<PathBuf> {
    ResearchActivity::read(source.to_path_buf()).and_then(|activity| {
        SafePath::new(format!("{}.pdf", activity.meta.identifier))
            .map_err(Into::<color_eyre::Report>::into)
            .and_then(|path| path.materialize_under(output_dir).map_err(Into::into))
    })
}
