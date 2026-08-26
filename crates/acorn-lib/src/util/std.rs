//! Utilities that require operating-system or runtime services
use super::alloc::{is_uri_or_path_portable, MimeType};
pub use super::cmd::*;
pub use super::terminal::{
    print_changes, print_changes_with_color, print_values_as_table, text_diff_changes, text_diff_changes_with_color, values_as_table,
};
use crate::prelude::{OsStr, Path, String, Vec};
use nanoid::nanoid;
impl AsRef<OsStr> for MimeType {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.file_type_ref())
    }
}
impl From<anydoc::Format> for MimeType {
    fn from(value: anydoc::Format) -> Self {
        match value {
            | anydoc::Format::Csv => Self::Csv,
            | anydoc::Format::Doc => Self::Doc,
            | anydoc::Format::Docx => Self::Docx,
            | anydoc::Format::Epub => Self::Epub,
            | anydoc::Format::Excel => Self::Excel,
            | anydoc::Format::Odp => Self::Odp,
            | anydoc::Format::Ods => Self::Ods,
            | anydoc::Format::Odt => Self::Odt,
            | anydoc::Format::Pdf => Self::Pdf,
            | anydoc::Format::Ppt => Self::Ppt,
            | anydoc::Format::Pptx => Self::Powerpoint,
            | anydoc::Format::Rtf => Self::Rtf,
        }
    }
}
impl MimeType {
    /// Infer a MIME type from file content using host-backed format detection.
    ///
    /// Semantic document containers are identified before generic binary
    /// signatures so formats such as DOCX and EPUB are not reduced to ZIP.
    pub fn infer(bytes: &[u8]) -> Option<Self> {
        anydoc::Format::from_bytes(bytes).map(Self::from).or_else(|| {
            infer::get(bytes).and_then(|kind| match kind.mime_type() {
                | "application/gzip" | "application/x-gzip" => Some(Self::Gzip),
                | "application/pdf" => Some(Self::Pdf),
                | "application/x-7z-compressed" => Some(Self::SevenZip),
                | "application/zip" => Some(Self::Zip),
                | "application/x-tar" => Some(Self::Tar),
                | "image/jpeg" => Some(Self::Jpeg),
                | "image/png" => Some(Self::Png),
                | _ => None,
            })
        })
    }
}
/// Generate a random ten-character ACORN identifier.
pub fn generate_guid() -> String {
    let alphabet = [
        '-', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 't', 'w', 'x', 'y', 'z', '3', '4', '6', '7', '8', '9',
    ];
    nanoid!(10, &alphabet)
}
/// Build a predicate that matches paths with one of the allowed file extensions.
pub fn is_filetype<I, S, P>(extensions: I) -> impl Fn(&P) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    P: AsRef<Path> + ?Sized,
{
    let extensions = extensions
        .into_iter()
        .map(|extension| extension.as_ref().trim_start_matches('.').to_ascii_lowercase())
        .collect::<Vec<_>>();
    move |path| {
        let path = path.as_ref();
        let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();
        extensions
            .iter()
            .any(|allowed| filename.eq_ignore_ascii_case(allowed) || extension.eq_ignore_ascii_case(allowed))
    }
}
/// Check if a value is a URI or filesystem path.
pub fn is_uri_or_path(value: &str) -> bool {
    is_uri_or_path_portable(value) || Path::new(value).is_absolute()
}
