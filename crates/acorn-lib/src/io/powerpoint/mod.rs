//! ## PowerPoint utilities
//!
//! Here you'll find functions for working with [`OOXML`] and creating PowerPoint files.
//!
//! [`OOXML`]: https://en.wikipedia.org/wiki/Office_Open_XML
use crate::io::export::ExportContext;
use crate::io::{read_file, write_file};
use acorn_core::util::{interpolation::ReplacementValue, StringInterpolation};
use acorn_host::terminal::Label;
use fancy_regex::Regex;
use quick_xml::escape::escape;
use std::path::PathBuf;
use tracing::{debug, error};

pub mod ooxml;
pub use ooxml::{prettify_xml, read_xml_rel};
use ooxml::{TextParagraph, TextParagraphProperties, TextRun, TextString};

/// Add enhanced string interpolation functionality
pub trait PowerpointInterpolation<T>
where
    T: AsRef<str> + ToString,
{
    /// Replace placeholder instances with a PowerPoint bullet list
    fn replace_placeholder_with_bullets<I: IntoIterator<Item = String>>(&self, placeholder: &str, values: I) -> String;
}
impl<T: AsRef<str>> PowerpointInterpolation<T> for T
where
    T: ToString,
{
    fn replace_placeholder_with_bullets<I: IntoIterator<Item = String>>(&self, placeholder: &str, values: I) -> String {
        let paragraphs = match Regex::new(r#"<a:p>(?:(?!<a:p>|</a:p>)[\s\S])*</a:p>"#) {
            | Ok(re) => re
                .find_iter(self.as_ref())
                .flat_map(|m| m.ok())
                .map(|m| m.as_str().to_string())
                .collect::<Vec<String>>(),
            | Err(_) => Vec::new(),
        };
        let selected = match paragraphs
            .into_iter()
            .find(|x| match Regex::new(&format!(r"{{{{\s*{placeholder}\s*}}}}")) {
                | Ok(re) => re.is_match(x).unwrap_or_default(),
                | Err(_) => false,
            }) {
            | Some(value) => value,
            | None => "".to_string(),
        };
        match parse_ooxml_paragraph(&selected) {
            | Ok(paragraph) => {
                let TextParagraph {
                    text_paragraph_properties,
                    text_run,
                    end_paragraph_run_properties,
                    ..
                } = paragraph.clone();
                let bullet_properties = match text_paragraph_properties.first() {
                    | Some(value) => value,
                    | None => &TextParagraphProperties::init().build(),
                };
                let text_properties = match text_run.first() {
                    | Some(first) => match first.text_run_properties.first() {
                        | Some(value) => Some(value),
                        | None => None,
                    },
                    | None => None,
                };
                match text_properties {
                    | Some(text_properties) => {
                        let content: Vec<String> = values
                            .into_iter()
                            .map(|value| {
                                let run = TextRun::init()
                                    .text_run_properties(vec![text_properties.clone()])
                                    .text(TextString { value })
                                    .build();
                                let custom = TextParagraph::init()
                                    .text_paragraph_properties(vec![bullet_properties.clone()])
                                    .text_run(vec![run])
                                    .maybe_end_paragraph_run_properties(end_paragraph_run_properties.clone())
                                    .build();
                                match quick_xml::se::to_string(&custom) {
                                    | Ok(xml) => prettify_xml(&xml),
                                    | Err(why) => {
                                        debug!("=> {} Serialize OOXML Paragraph - {why}", Label::fail());
                                        String::new()
                                    }
                                }
                            })
                            .collect();
                        let formatted = content.join("");
                        self.as_ref().replace(&selected, &formatted)
                    }
                    | None => self.to_string(),
                }
            }
            | Err(why) => {
                debug!(selected, "=> {} Parse OOXML Paragraph - {why}", Label::fail());
                self.to_string()
            }
        }
    }
}
fn escape_xml_text(value: &str) -> String {
    escape(value).into_owned()
}
/// Interpolate projected research activity values into a PowerPoint template part.
pub fn interpolate_values(path: PathBuf, context: &ExportContext) {
    let updated = match read_file(&path) {
        | Ok(content) => {
            let scalars = context.replacements.iter().fold(content, |acc, (name, value)| match value {
                | ReplacementValue::Absent => acc.replace_placeholder_with_string(name, ""),
                | ReplacementValue::Scalar(value) => acc.replace_placeholder_with_string(name, &escape_xml_text(value)),
                | ReplacementValue::List(_) => acc,
            });
            let result = context.replacements.iter().fold(scalars, |acc, (name, value)| match value {
                | ReplacementValue::List(values) => acc.replace_placeholder_with_bullets(name, values.iter().cloned()),
                | ReplacementValue::Absent | ReplacementValue::Scalar(_) => acc,
            });
            Some(result)
        }
        | Err(why) => {
            error!("=> {} Cannot read file for interpolation - {why}", Label::fail());
            None
        }
    };
    if let Some(content) = updated {
        match write_file(path.clone(), content) {
            | Ok(_) => {}
            | Err(why) => {
                error!("=> {} Cannot write file after interpolation - {why}", Label::fail());
            }
        }
    }
}
/// Parse OOXML paragraph object
pub fn parse_ooxml_paragraph(content: &str) -> Result<TextParagraph, quick_xml::DeError> {
    let parsed = quick_xml::de::from_str::<TextParagraph>(content);
    debug!("=> {} OOXMLParagraph = {:#?}", Label::using(), parsed);
    parsed
}
#[cfg(test)]
mod tests;
