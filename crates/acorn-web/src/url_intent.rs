//! Bounded URL entry intent, independent of browser location access.
use serde::Deserialize;

const URL_LIMIT: usize = 8 * 1024;
/// One validated URL entry action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Intent {
    /// No supported query parameter was present.
    None,
    /// Load inline code into the editor.
    Code(String),
    /// Load a bundled template.
    Template(String),
    /// Load remote content from an HTTPS URL.
    Url(String),
    /// Reject the link with a user-facing message.
    Invalid(&'static str),
}
/// Supported, decoded URL query values.
#[derive(Clone, Debug, Deserialize)]
pub struct Parameters {
    /// Inline code values.
    pub code: Vec<String>,
    /// Template identifiers.
    pub template: Vec<String>,
    /// Remote content URLs.
    pub url: Vec<String>,
}
/// URL and decoded query values supplied by the browser entry channel.
#[derive(Clone, Debug, Deserialize)]
pub struct UrlEntry {
    /// Complete browser URL, used to enforce the link size limit.
    pub href: String,
    /// Query parameters supported by the editor entry flow.
    pub parameters: Parameters,
    /// Whether a supported parameter contains malformed percent encoding.
    pub malformed: bool,
}
impl UrlEntry {
    /// Validate and classify one URL entry.
    pub fn intent(&self) -> Intent {
        let values = [&self.parameters.code, &self.parameters.template, &self.parameters.url];
        let selected = values.iter().filter(|values| !values.is_empty()).count();
        if selected == 0 {
            Intent::None
        } else if self.href.len() > URL_LIMIT {
            Intent::Invalid("This link is too long. Current content is unchanged.")
        } else if self.malformed {
            Intent::Invalid("This link has malformed encoding. Current content is unchanged.")
        } else if values.iter().any(|values| values.len() > 1) || selected > 1 {
            Intent::Invalid("Choose only one code, template, or URL parameter.")
        } else if let Some(value) = self.parameters.code.first() {
            if value.is_empty() {
                Intent::Invalid("The code parameter is empty.")
            } else if value.len() > crate::operations::Tool::Standard.limit() {
                Intent::Invalid("The code parameter exceeds the editor limit.")
            } else {
                Intent::Code(value.clone())
            }
        } else if let Some(value) = self.parameters.template.first() {
            if value.is_empty() {
                Intent::Invalid("The template parameter is empty.")
            } else {
                Intent::Template(value.clone())
            }
        } else if let Some(value) = self.parameters.url.first() {
            if value.is_empty() {
                Intent::Invalid("The URL parameter is empty.")
            } else if !value.starts_with("https://") {
                Intent::Invalid("The URL parameter must use HTTPS.")
            } else {
                Intent::Url(value.clone())
            }
        } else {
            Intent::None
        }
    }
}
