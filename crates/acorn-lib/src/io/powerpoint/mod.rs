//! PowerPoint OOXML utilities and fallible interpolation.
use crate::io::export::ExportContext;
use crate::io::{read_file, write_file, ApiResult};
use color_eyre::eyre::eyre;
use ooxml::TextParagraph;
pub use ooxml::{prettify_xml, read_xml_rel};
use std::path::PathBuf;

pub mod interpolation;
pub mod ooxml;

/// Interpolate projected research activity values into a PowerPoint template part.
pub fn interpolate_values(path: PathBuf, context: &ExportContext) -> ApiResult<()> {
    read_file(&path)
        .map_err(|why| eyre!("Failed to read PowerPoint part at {} — {why}", path.display()))
        .and_then(|content| {
            interpolation::interpolate_xml(&content, &context.replacements)
                .map_err(|why| eyre!("Failed to interpolate PowerPoint part at {} — {why}", path.display()))
        })
        .and_then(|content| write_file(&path, content).map_err(|why| eyre!("Failed to write PowerPoint part at {} — {why}", path.display())))
}
/// Parse an OOXML paragraph object.
pub fn parse_ooxml_paragraph(content: &str) -> Result<TextParagraph, quick_xml::DeError> {
    quick_xml::de::from_str::<TextParagraph>(content)
}

#[cfg(test)]
mod tests;
