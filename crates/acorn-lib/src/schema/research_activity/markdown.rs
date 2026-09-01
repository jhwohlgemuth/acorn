#[cfg(feature = "std")]
use super::Research;
use super::{ResearchActivity, ResearchActivityContext, ResearchActivityMetadata, Sections as ResearchSections};
#[cfg(feature = "std")]
use crate::error::ApiResult;
use crate::prelude::alloc::{format, String, ToString};
#[cfg(feature = "std")]
use crate::prelude::alloc::{HashMap, Vec};
use crate::schema::research_activity::aspect::AspectFramework;
use crate::schema::{ContactPoint, ContactPointContext, Other};
use crate::util::constants::app::RAD_MARKDOWN_SCHEMA;
#[cfg(feature = "std")]
use crate::util::constants::app::{RAD_MARKDOWN_OPTIONAL_SECTIONS, RAD_MARKDOWN_REQUIRED_SECTIONS};
use crate::util::{frontmatter_and_body, MarkdownSupport};
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug)]
struct Body {
    title: String,
    subtitle: Option<String>,
    sections: ResearchSections,
    contact: ContactPoint,
}
#[cfg(feature = "std")]
#[derive(Debug)]
struct BodySections {
    title: String,
    subtitle: Option<String>,
    values: HashMap<String, String>,
}
#[cfg(feature = "std")]
#[derive(Debug)]
struct Contact {
    job_title: String,
    given_name: String,
    family_name: String,
    identifier: Option<String>,
    email: String,
    telephone: String,
    url: String,
    organization: String,
    affiliation: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ContactMetadata {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    context: Option<ContactPointContext>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    contact_point_type: Option<String>,
}
#[derive(Clone, Debug)]
pub(super) struct Document {
    frontmatter: Header,
    body: Body,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Header {
    schema: String,
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    context: Option<ResearchActivityContext>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    research_activity_type: Option<String>,
    meta: ResearchActivityMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect: Option<AspectFramework>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<Other>,
    #[serde(default, skip_serializing_if = "ContactMetadata::is_empty")]
    contact: ContactMetadata,
}
#[cfg(feature = "std")]
impl BodySections {
    fn into_body(self, frontmatter: &Header) -> ApiResult<Body> {
        let achievement = self.optional_list("Achievement");
        let capabilities = self.optional_list("Capabilities");
        let mission = self.required_text("Mission");
        let challenge = self.required_text("Challenge");
        let approach = self.required_list("Approach");
        let impact = self.required_list("Impact");
        let focus = self.required_text("Focus");
        let areas = self.required_list("Areas");
        let contact = self.contact(frontmatter);
        achievement
            .and_then(|achievement| capabilities.map(|capabilities| (achievement, capabilities)))
            .and_then(|(achievement, capabilities)| mission.map(|mission| (achievement, capabilities, mission)))
            .and_then(|(achievement, capabilities, mission)| challenge.map(|challenge| (achievement, capabilities, mission, challenge)))
            .and_then(|(achievement, capabilities, mission, challenge)| {
                approach.map(|approach| (achievement, capabilities, mission, challenge, approach))
            })
            .and_then(|(achievement, capabilities, mission, challenge, approach)| {
                impact.map(|impact| (achievement, capabilities, mission, challenge, approach, impact))
            })
            .and_then(|(achievement, capabilities, mission, challenge, approach, impact)| {
                focus.map(|focus| (achievement, capabilities, mission, challenge, approach, impact, focus))
            })
            .and_then(|(achievement, capabilities, mission, challenge, approach, impact, focus)| {
                areas.map(|areas| (achievement, capabilities, mission, challenge, approach, impact, focus, areas))
            })
            .and_then(|(achievement, capabilities, mission, challenge, approach, impact, focus, areas)| {
                contact.map(|contact| Body {
                    title: self.title,
                    subtitle: self.subtitle,
                    sections: ResearchSections {
                        mission,
                        challenge,
                        approach,
                        impact,
                        achievement,
                        capabilities,
                        research: Research { focus, areas },
                    },
                    contact,
                })
            })
    }
    fn required_text(&self, name: &str) -> ApiResult<String> {
        self.values
            .get(name)
            .map(|value| value.trim().decode_markdown_text())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| eyre!("Markdown RAD section '{name}' cannot be empty"))
    }
    fn required_list(&self, name: &str) -> ApiResult<Vec<String>> {
        self.values
            .get(name)
            .ok_or_else(|| eyre!("Markdown RAD requires section '## {name}'"))
            .and_then(|value| Self::list(name, value))
            .and_then(|values| {
                (!values.is_empty())
                    .then_some(values)
                    .ok_or_else(|| eyre!("Markdown RAD section '{name}' requires a list"))
            })
    }
    fn optional_list(&self, name: &str) -> ApiResult<Option<Vec<String>>> {
        self.values.get(name).map_or(Ok(None), |value| {
            Self::list(name, value).and_then(|values| {
                (!values.is_empty())
                    .then_some(Some(values))
                    .ok_or_else(|| eyre!("Markdown RAD section '{name}' requires a list"))
            })
        })
    }
    fn list(name: &str, content: &str) -> ApiResult<Vec<String>> {
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                line.strip_prefix("- ")
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(MarkdownSupport::decode_markdown_text)
                    .ok_or_else(|| eyre!("Markdown RAD section '{name}' must contain only '- ' list items"))
            })
            .collect()
    }
    fn contact(&self, frontmatter: &Header) -> ApiResult<ContactPoint> {
        self.values
            .get("Contact")
            .ok_or_else(|| eyre!("Markdown RAD requires section '## Contact'"))
            .and_then(|value| Contact::parse(value))
            .map(|value| value.into_contact(frontmatter.contact.clone()))
    }
}
#[cfg(feature = "std")]
impl TryFrom<&str> for BodySections {
    type Error = color_eyre::Report;

    fn try_from(content: &str) -> Result<Self, Self::Error> {
        let lines = content.lines().collect::<Vec<_>>();
        let titles = lines.iter().enumerate().filter(|(_, line)| line.starts_with("# ")).collect::<Vec<_>>();
        match titles.as_slice() {
            | [(title_index, title)] => {
                let headings = lines
                    .iter()
                    .enumerate()
                    .filter_map(|(index, line)| line.strip_prefix("## ").map(|name| (index, name.trim().to_string())))
                    .collect::<Vec<_>>();
                let preface_end = headings.first().map(|(index, _)| *index).unwrap_or(lines.len());
                let subtitle = lines
                    .iter()
                    .skip(title_index.saturating_add(1))
                    .take(preface_end.saturating_sub(title_index.saturating_add(1)))
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .map(|line| {
                        line.strip_prefix("> ")
                            .map(ToString::to_string)
                            .ok_or_else(|| eyre!("Markdown RAD subtitle must be a blockquote immediately after the title"))
                    })
                    .collect::<ApiResult<Vec<_>>>()
                    .map(|values| (!values.is_empty()).then(|| values.join("\n").decode_markdown_text()));
                subtitle.and_then(|subtitle| {
                    let allowed = RAD_MARKDOWN_REQUIRED_SECTIONS.into_iter().chain(RAD_MARKDOWN_OPTIONAL_SECTIONS);
                    let invalid = headings
                        .iter()
                        .map(|(_, name)| name.as_str())
                        .find(|name| !allowed.clone().any(|allowed| allowed == *name));
                    invalid.map_or_else(
                        || {
                            headings
                                .iter()
                                .enumerate()
                                .try_fold(HashMap::new(), |mut values, (offset, (start, name))| {
                                    let end = headings.get(offset.saturating_add(1)).map(|(index, _)| *index).unwrap_or(lines.len());
                                    let value = lines
                                        .get(start.saturating_add(1)..end)
                                        .map(|section| section.join("\n"))
                                        .unwrap_or_default()
                                        .trim()
                                        .to_string();
                                    if values.insert(name.clone(), value).is_some() {
                                        Err(eyre!("Duplicate Markdown RAD section '## {name}'"))
                                    } else {
                                        Ok(values)
                                    }
                                })
                                .and_then(|values| {
                                    RAD_MARKDOWN_REQUIRED_SECTIONS
                                        .iter()
                                        .find(|name| !values.contains_key(**name))
                                        .map_or(Ok(values), |name| Err(eyre!("Markdown RAD requires section '## {name}'")))
                                })
                                .map(|values| Self {
                                    title: title.trim_start_matches("# ").trim().decode_markdown_text(),
                                    subtitle,
                                    values,
                                })
                        },
                        |name| Err(eyre!("Unknown Markdown RAD section '## {name}'")),
                    )
                })
            }
            | [] => Err(eyre!("Markdown RAD requires exactly one '# ' title")),
            | _ => Err(eyre!("Markdown RAD contains multiple '# ' titles")),
        }
    }
}
#[cfg(feature = "std")]
impl Contact {
    fn email(value: &str) -> String {
        value
            .strip_prefix('[')
            .and_then(|value| value.split_once("](mailto:"))
            .map(|(email, _)| email.to_string())
            .unwrap_or_else(|| value.to_string())
    }
    fn into_contact(self, metadata: ContactMetadata) -> ContactPoint {
        ContactPoint {
            context: metadata.context,
            contact_point_type: metadata.contact_point_type,
            job_title: self.job_title,
            given_name: self.given_name,
            family_name: self.family_name,
            identifier: self.identifier,
            email: self.email,
            telephone: self.telephone,
            url: self.url,
            organization: self.organization,
            affiliation: self.affiliation,
        }
    }
    fn is_known(key: &str) -> bool {
        matches!(
            key,
            "job title" | "given name" | "family name" | "identifier" | "email" | "telephone" | "url" | "organization" | "affiliation"
        )
    }
    fn required(values: &HashMap<String, String>, key: &str) -> ApiResult<String> {
        values.get(key).cloned().ok_or_else(|| eyre!("Markdown RAD contact is missing '{key}'"))
    }
    fn parse(content: &str) -> ApiResult<Self> {
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                line.strip_prefix("- ")
                    .and_then(|line| line.split_once(":"))
                    .map(|(key, value)| (key.trim().to_ascii_lowercase(), value.trim().decode_markdown_text()))
                    .ok_or_else(|| eyre!("Markdown RAD contact entries must use '- Key: value'"))
            })
            .collect::<ApiResult<Vec<_>>>()
            .and_then(|entries| {
                entries.into_iter().try_fold(HashMap::new(), |mut values, (key, value)| {
                    if !Self::is_known(&key) {
                        Err(eyre!("Unknown Markdown RAD contact field '{key}'"))
                    } else if values.insert(key.clone(), value).is_some() {
                        Err(eyre!("Duplicate Markdown RAD contact field '{key}'"))
                    } else {
                        Ok(values)
                    }
                })
            })
            .and_then(|values| {
                Self::required(&values, "job title")
                    .and_then(|job_title| Self::required(&values, "given name").map(|given_name| (job_title, given_name)))
                    .and_then(|(job_title, given_name)| {
                        Self::required(&values, "family name").map(|family_name| (job_title, given_name, family_name))
                    })
                    .and_then(|(job_title, given_name, family_name)| {
                        Self::required(&values, "email").map(|email| (job_title, given_name, family_name, Self::email(&email)))
                    })
                    .and_then(|(job_title, given_name, family_name, email)| {
                        Self::required(&values, "telephone").map(|telephone| (job_title, given_name, family_name, email, telephone))
                    })
                    .and_then(|(job_title, given_name, family_name, email, telephone)| {
                        Self::required(&values, "url").map(|url| (job_title, given_name, family_name, email, telephone, url))
                    })
                    .and_then(|(job_title, given_name, family_name, email, telephone, url)| {
                        Self::required(&values, "organization")
                            .map(|organization| (job_title, given_name, family_name, email, telephone, url, organization))
                    })
                    .map(|(job_title, given_name, family_name, email, telephone, url, organization)| Self {
                        job_title,
                        given_name,
                        family_name,
                        identifier: values.get("identifier").cloned(),
                        email,
                        telephone,
                        url,
                        organization,
                        affiliation: values.get("affiliation").cloned(),
                    })
            })
    }
}
impl ContactMetadata {
    fn is_empty(&self) -> bool {
        self.context.is_none() && self.contact_point_type.is_none()
    }
}
impl Document {
    pub(super) fn recognizes(content: &str) -> bool {
        let (frontmatter, _) = frontmatter_and_body(content.replace("\r\n", "\n"));
        frontmatter.is_some_and(|frontmatter| {
            serde_norway::from_str::<serde_json::Value>(&frontmatter)
                .ok()
                .and_then(|value| value.get("schema").and_then(serde_json::Value::as_str).map(ToString::to_string))
                .is_some_and(|schema| schema == RAD_MARKDOWN_SCHEMA)
        })
    }
    #[cfg(feature = "std")]
    fn normalize_empty_required_lists(value: serde_json::Value) -> serde_json::Value {
        let defaults = serde_json::to_value(Document::from(&ResearchActivity::default()).frontmatter).unwrap_or_default();
        Self::normalize_empty_lists(value, &defaults)
    }
    #[cfg(feature = "std")]
    fn normalize_empty_lists(value: serde_json::Value, defaults: &serde_json::Value) -> serde_json::Value {
        match (value, defaults) {
            | (serde_json::Value::Null, serde_json::Value::Array(_)) => serde_json::Value::Array(Vec::new()),
            | (serde_json::Value::Object(values), serde_json::Value::Object(defaults)) => serde_json::Value::Object(
                values
                    .into_iter()
                    .map(|(key, value)| {
                        let value = match defaults.get(&key) {
                            | Some(default) => Self::normalize_empty_lists(value, default),
                            | None => value,
                        };
                        (key, value)
                    })
                    .collect(),
            ),
            | (value, _) => value,
        }
    }
    #[cfg(feature = "std")]
    fn parse_frontmatter(content: &str) -> ApiResult<(Header, String)> {
        let normalized = content.replace("\r\n", "\n");
        let (frontmatter, body) = frontmatter_and_body(&normalized);
        frontmatter
            .ok_or_else(|| eyre!("Markdown RAD requires YAML frontmatter delimited by '---'"))
            .and_then(|frontmatter| {
                serde_norway::from_str::<serde_json::Value>(&frontmatter)
                    .map(Self::normalize_empty_required_lists)
                    .map_err(|why| why.to_string())
                    .and_then(|value| serde_json::from_value::<Header>(value).map_err(|why| why.to_string()))
                    .map_err(|why| eyre!("Failed to decode Markdown RAD frontmatter — {why}"))
                    .and_then(|parsed| {
                        (parsed.schema == RAD_MARKDOWN_SCHEMA)
                            .then_some((parsed, body))
                            .ok_or_else(|| eyre!("Unsupported Markdown RAD schema discriminator"))
                    })
            })
    }
    #[cfg(feature = "std")]
    fn parse_body(content: &str, frontmatter: &Header) -> ApiResult<Body> {
        BodySections::try_from(content).and_then(|sections| sections.into_body(frontmatter))
    }
}
impl From<&ResearchActivity> for Document {
    fn from(activity: &ResearchActivity) -> Self {
        Self {
            frontmatter: Header {
                schema: RAD_MARKDOWN_SCHEMA.to_string(),
                context: activity.context.clone(),
                research_activity_type: activity.research_activity_type.clone(),
                meta: activity.meta.clone(),
                aspect: activity.aspect.clone(),
                notes: activity.notes.clone(),
                contact: ContactMetadata {
                    context: activity.contact.context.clone(),
                    contact_point_type: activity.contact.contact_point_type.clone(),
                },
            },
            body: Body {
                title: activity.title.clone(),
                subtitle: activity.subtitle.clone(),
                sections: activity.sections.clone(),
                contact: activity.contact.clone(),
            },
        }
    }
}
impl MarkdownSupport for Document {
    fn to_markdown(&self) -> String {
        let frontmatter = self.frontmatter.to_markdown();
        let subtitle = self
            .body
            .subtitle
            .as_ref()
            .map(|value| format!("> {}\n", value.to_markdown_text()))
            .unwrap_or_default();
        let aspect = self
            .frontmatter
            .aspect
            .as_ref()
            .map(|value| format!("\n\n{}", value.to_markdown()))
            .unwrap_or_default();
        format!(
            "---\n{frontmatter}---\n# {title}\n{subtitle}{sections}\n\n{contact}{aspect}\n",
            title = self.body.title.to_markdown_text(),
            sections = self.body.sections.to_markdown(),
            contact = self.body.contact.to_markdown().trim_end(),
        )
    }
}
#[cfg(feature = "std")]
impl TryFrom<&str> for Document {
    type Error = color_eyre::Report;

    fn try_from(content: &str) -> Result<Self, Self::Error> {
        Self::parse_frontmatter(content).and_then(|(frontmatter, body)| Self::parse_body(&body, &frontmatter).map(|body| Self { frontmatter, body }))
    }
}
impl MarkdownSupport for Header {
    fn to_markdown(&self) -> String {
        self.to_front_matter().unwrap_or_default()
    }
}
#[cfg(feature = "std")]
impl TryFrom<Document> for ResearchActivity {
    type Error = color_eyre::Report;

    fn try_from(document: Document) -> Result<Self, Self::Error> {
        Ok(Self {
            context: document.frontmatter.context,
            research_activity_type: document.frontmatter.research_activity_type,
            meta: document.frontmatter.meta,
            aspect: document.frontmatter.aspect,
            title: document.body.title,
            subtitle: document.body.subtitle,
            sections: document.body.sections,
            contact: document.body.contact,
            notes: document.frontmatter.notes,
        })
    }
}
