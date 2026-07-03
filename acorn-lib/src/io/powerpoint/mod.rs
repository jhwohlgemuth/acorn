//! ## PowerPoint utilities
//!
//! Here you'll find functions for working with [`OOXML`] and creating PowerPoint files.
//!
//! [`OOXML`]: https://en.wikipedia.org/wiki/Office_Open_XML
use crate::io::{read_file, write_file};
use crate::prelude::PathBuf;
use crate::schema::research_activity::ResearchActivity;
use crate::schema::{Notes, Other};
use crate::util::{Label, StringInterpolation};
use fancy_regex::Regex;
use quick_xml::escape::escape;
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
/// Interpolate research activity data into PowerPoint template
pub fn interpolate_values(path: PathBuf, data: &ResearchActivity) {
    let updated = match read_file(path.clone()) {
        | Ok(content) => {
            let caption = data.meta.clone().first_image_caption();
            let presentation_notes = match data.notes.clone() {
                | Some(value) => match value {
                    | Other::Formatted(Notes { presentation, .. }) => match presentation {
                        | Some(value) => value,
                        | None => "".to_string(),
                    },
                    | Other::Unformatted(notes) => notes,
                },
                | None => "".to_string(),
            };
            let managers = match data.notes.clone() {
                | Some(value) => match value {
                    | Other::Formatted(Notes { managers, .. }) => match managers {
                        | Some(value) => value,
                        | None => vec![],
                    },
                    | _ => vec![],
                },
                | None => vec![],
            };
            let programs = match data.notes.clone() {
                | Some(value) => match value {
                    | Other::Formatted(Notes { programs, .. }) => match programs {
                        | Some(value) => value,
                        | None => vec![],
                    },
                    | _ => vec![],
                },
                | None => vec![],
            };
            #[derive(Clone)]
            enum Replacement {
                String(&'static str, String),
                Bullets(&'static str, Vec<String>),
            }
            let replacements = vec![
                Replacement::String("caption", caption),
                Replacement::String("challenge", data.sections.challenge.clone()),
                // TODO: Use CiteAs API to get in Chicago format
                Replacement::String(
                    "citation",
                    data.meta.doi.as_ref().and_then(|values| values.first().cloned()).unwrap_or_default(),
                ),
                Replacement::String("email", data.contact.email.clone()),
                Replacement::String("first", data.contact.given_name.clone()),
                Replacement::String("focus", data.sections.research.focus.clone()),
                Replacement::String("last", data.contact.family_name.clone()),
                Replacement::String("managers", managers.join(" and ")),
                Replacement::String("mission", data.sections.mission.clone()),
                Replacement::String("notes", presentation_notes),
                Replacement::String("partners", data.meta.partners.clone().unwrap_or_default().join(", ")),
                Replacement::String(
                    "portability",
                    data.aspect.clone().unwrap_or_default().portability.unwrap_or_default().to_string(),
                ),
                Replacement::String("programs", programs.join(" and ")),
                Replacement::String("subtitle", data.subtitle.clone().unwrap_or_default()),
                Replacement::String("title", data.title.clone()),
                Replacement::Bullets("achievement", data.sections.achievement.clone().unwrap_or_default()),
                Replacement::Bullets("areas", data.sections.research.areas.clone()),
                Replacement::Bullets("impact", data.sections.impact.clone()),
                Replacement::Bullets("technical", data.sections.approach.clone()),
            ];
            let result = replacements.iter().fold(content.clone(), |acc, replacement| match replacement {
                | Replacement::String(name, value) => acc.replace_placeholder_with_string(name, &escape_xml_text(value)),
                | Replacement::Bullets(name, values) => acc.replace_placeholder_with_bullets(name, values.clone()),
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
