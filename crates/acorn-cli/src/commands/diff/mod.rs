use crate::{
    cli::{arguments, arguments::DiffOutputFormat, Void},
    io::write_stdout,
};
use acorn::analyzer::diff::{DiffFormat, DiffInput, SemanticDiff};
use acorn_core::util::MimeType;
use acorn_host::fs::read_text;
use color_eyre::eyre::{eyre, WrapErr};
use std::path::Path;

trait WriteSemanticDiff {
    fn write(self) -> Void;
}
impl WriteSemanticDiff for SemanticDiff {
    fn write(self) -> Void {
        match self {
            | SemanticDiff::Unified(output) => {
                write_stdout(output);
                Ok(())
            }
            | SemanticDiff::JsonPatch(patch) => serde_json::to_string_pretty(&patch)
                .map(|output| write_stdout(format!("{output}\n")))
                .map_err(|error| eyre!("Serialize JSON Patch — {error}")),
        }
    }
}
fn read_regular_file(path: &Path) -> Result<String, color_eyre::Report> {
    path.is_file()
        .then_some(())
        .ok_or_else(|| eyre!("Diff input is not a regular file: {}", path.display()))
        .and_then(|()| read_text(path).wrap_err_with(|| format!("Read UTF-8 diff input {}", path.display())))
}
pub(crate) fn run(arguments: &arguments::Diff) -> Void {
    read_regular_file(&arguments.before).and_then(|before| {
        read_regular_file(&arguments.after).and_then(|after| {
            let before_media = MimeType::from_path_or_content(&arguments.before, &before);
            let after_media = MimeType::from_path_or_content(&arguments.after, &after);
            let before_label = format!("a/{}", arguments.before.display());
            let after_label = format!("b/{}", arguments.after.display());
            let before_input = DiffInput {
                content: &before,
                label: &before_label,
                media_type: before_media,
            };
            let after_input = DiffInput {
                content: &after,
                label: &after_label,
                media_type: after_media,
            };
            let format = match arguments.format {
                | DiffOutputFormat::Diff => DiffFormat::Unified,
                | DiffOutputFormat::JsonPatch => DiffFormat::JsonPatch,
            };
            SemanticDiff::compare(&before_input, &after_input, format, arguments.standard)
                .map_err(|error| eyre!(error))
                .and_then(WriteSemanticDiff::write)
        })
    })
}

#[cfg(test)]
mod tests;
