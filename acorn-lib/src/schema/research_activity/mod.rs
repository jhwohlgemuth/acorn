//! # Research activity schema
//!
//! Module that defines the research activity schema
//!
#[cfg(feature = "std")]
use crate::fail;
#[cfg(feature = "std")]
use crate::io::ApiResult;
#[cfg(feature = "std")]
use crate::io::{image_paths, jsonc_parse_value, parent, read_file, write_file, FromPath, InputOutput};
#[cfg(feature = "std")]
use crate::prelude::PathBuf;
use crate::prelude::*;
use crate::schema::hardware::Resource;
use crate::schema::namespaces::{bibo, codemeta, dcat, dcterms, schema_org, DEFAULT_RAID_SCHEMA_URI, DEFAULT_ROR_SCHEMA_URI};
use crate::schema::pid::PersistentIdentifierConvert;
use crate::schema::standard::cff::{Agent, Cff, Identifier, IdentifierType, Person};
use crate::schema::validate::{is_doi, is_kebabcase, is_ror, is_urls};
use crate::schema::{ClassificationLevel, ContactPoint, ImageObject, Keyword, MediaObject, OrganizationType, Other, Status, Website};
use crate::util::constants::{DEFAULT_GRAPHIC_CAPTION, DEFAULT_GRAPHIC_HREF, MAX_LENGTH_SUBTITLE, MAX_LENGTH_TITLE};
use crate::util::constants::{
    MAX_COUNT_APPROACH, MAX_COUNT_CAPABILITIES, MAX_COUNT_IMPACT, MAX_COUNT_RESEARCH_AREAS, MAX_LENGTH_RESEARCH_FOCUS, MAX_LENGTH_SECTION_CHALLENGE,
    MAX_LENGTH_SECTION_MISSION,
};
use crate::util::constants::{MAX_LENGTH_APPROACH, MAX_LENGTH_CAPABILIY, MAX_LENGTH_IMPACT, MAX_LENGTH_RESEARCH_AREA};
#[cfg(feature = "std")]
use crate::util::frontmatter_and_body;
#[cfg(feature = "std")]
use crate::util::{Constant, Label};
use crate::util::{LinkedData, ToMarkdown, ToProse};
#[cfg(feature = "std")]
use crate::util::{MimeType, StringConversion};
use bon::Builder;
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use convert_case::{Case, Casing};
use core::hash::{Hash, Hasher};
use derive_more::Display;
#[cfg(feature = "std")]
use fancy_regex::Regex;
#[cfg(feature = "std")]
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
#[cfg(feature = "std")]
use owo_colors::OwoColorize;
#[cfg(feature = "std")]
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim, vec_string_trim};
use serde_with::skip_serializing_none;
#[cfg(feature = "std")]
use tracing::{debug, error, trace};
use validator::{Validate, ValidationError};

pub mod aspect;
pub mod s3a;
use aspect::AspectFramework;

type ValidationResult = core::result::Result<(), ValidationError>;
#[derive(Debug, Display)]
enum Vocabulary {
    #[display("keywords")]
    Keywords,
    #[display("partners")]
    Partners,
    #[display("sponsors")]
    Sponsors,
    #[display("technology")]
    Technology,
}
/// Overview of research focus and areas
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields)]
pub struct Research {
    /// Brief overview of the project or organization's research
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_RESEARCH_FOCUS",
        message = "Focus is too long, reduce the length below 150 characters."
    ))]
    #[builder(default = "Focus of the research".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub focus: String,
    /// Topics related to and encapsulated within the project or organization
    #[validate(length(min = 1, max = "MAX_COUNT_RESEARCH_AREAS"), custom(function = "is_attribute_areas"))]
    #[builder(default = vec!["Areas of research".to_string()])]
    #[serde(deserialize_with = "vec_string_trim")]
    pub areas: Vec<String>,
}
/// # Research Activity
/// Identifiable package of work involving organized, systematic investigation
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, eserde::Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[display("Research Activity ({title})")]
#[serde(deny_unknown_fields)]
pub struct ResearchActivity {
    /// Linked data (e.g., JSON-LD) context for research activity
    #[serde(rename = "@context")]
    #[eserde(compat)]
    pub context: Option<ResearchActivityContext>,
    /// Linked data (e.g., JSON-LD) type for research activity
    #[serde(rename = "@type")]
    pub research_activity_type: Option<String>,
    /// Associated metadata
    #[validate(nested)]
    #[builder(default)]
    pub meta: ResearchActivityMetadata,
    /// Technology ASPECT of associated research activity - describes the data, compute, and algorithms used in the associated research activity
    #[eserde(compat)]
    pub aspect: Option<AspectFramework>,
    /// Heading that identifies and describes the associated research activity
    #[validate(length(min = 4, max = "MAX_LENGTH_TITLE"))]
    #[builder(default = "Research Activity Title".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    /// Short description that augments the title of the associated research activity
    #[validate(length(max = "MAX_LENGTH_SUBTITLE", message = "Subtitle is too long, reduce the length below 75 characters."))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub subtitle: Option<String>,
    /// Prose components of associated research activity
    #[validate(nested)]
    #[builder(default)]
    #[eserde(compat)]
    pub sections: Sections,
    /// Contact point (i.e. point of contact) for research activity
    #[validate(nested)]
    #[builder(default)]
    #[eserde(compat)]
    pub contact: ContactPoint,
    /// Other information related to the associated research activity not easily captured in structured areas of the schema
    #[eserde(compat)]
    pub notes: Option<Other>,
}
/// Linked data (e.g., JSON-LD) context for research activity
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityContext {
    /// Associated metadata
    pub meta: String,
    /// Research activity title
    pub title: String,
    /// Research activity subtitle
    pub subtitle: String,
    /// Research activity sections of prose
    pub sections: String,
    /// Research activity contact point
    pub contact: String,
    /// Research activity notes
    pub notes: String,
}
/// ## Research Activity Metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, eserde::Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityMetadata {
    /// Linked data (e.g., JSON-LD) context for contact point
    #[serde(rename = "@context")]
    #[eserde(compat)]
    pub context: Option<ResearchActivityMetadataContext>,
    /// Linked data (e.g., JSON-LD) type for contact point
    #[serde(rename = "@type")]
    pub metadata_type: Option<String>,
    /// Classification level of associated research activity data
    #[eserde(compat)]
    pub classification: Option<ClassificationLevel>,
    /// Describes the active status of the associated research activity data
    ///
    /// <div class="warning">Archived content typically will be omitted from public artifacts such as <a href="https://research.ornl.gov">the ORNL research activity index</a></div>
    #[builder(default = false)]
    pub archive: bool,
    /// Describes the draft status of the associated research activity data
    ///
    /// <div class="warning">Draft content typically will be omitted from public artifacts such as <a href="https://research.ornl.gov">the ORNL research activity index</a></div>
    #[builder(default = true)]
    pub draft: bool,
    /// Describes the status of the associated research activity data
    #[builder(default = Status::Active)]
    #[serde(default)]
    #[eserde(compat)]
    pub status: Status,
    /// Identifier for associated research activity data
    /// ### Example
    /// > `my-research-project`
    ///
    /// <div class="warning">Should be <a href="https://developer.mozilla.org/en-US/docs/Glossary/Kebab_case">lower-kebab-case</a></div>
    ///
    #[validate(custom(function = "is_kebabcase"))]
    #[builder(default = "some-research-project".to_string())]
    #[serde(alias = "id", rename = "identifier", deserialize_with = "string_trim")]
    pub identifier: String,
    /// Digital Object Identifier(s) related to the associated research activity data
    ///
    /// See <https://www.doi.org/> for more information
    #[validate(custom(function = "is_attribute_doi_list"))]
    #[serde(default)]
    pub doi: Option<Vec<String>>,
    /// ISBN(s) of books related to the associated research activity data
    #[validate(custom(function = "is_attribute_books"))]
    #[serde(default)]
    pub books: Option<Vec<String>>,
    /// Patent(s) related to the associated research activity data
    #[serde(default)]
    pub patents: Option<Vec<String>>,
    /// URL(s) of internet location where associated publication(s) can be found
    #[validate(custom(function = "is_urls"))]
    #[serde(default)]
    pub publications: Option<Vec<String>>,
    /// Research Activity Identifier (RAiD)
    #[validate(custom(function = "is_attribute_doi_list"))]
    #[serde(default)]
    pub raid: Option<Vec<String>>,
    /// Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[validate(custom(function = "is_attribute_ror_list"))]
    #[serde(default)]
    pub ror: Option<Vec<String>>,
    /// Type of associated research activity data when directly associated with an organization
    #[eserde(compat)]
    pub additional_type: Option<OrganizationType>,
    /// Images, videos, and other media related to the associated research activity data
    #[validate(nested)]
    #[serde(alias = "graphics")]
    #[eserde(compat)]
    pub media: Option<Vec<MediaObject>>,
    /// Websites related to the associated research activity data
    #[validate(nested)]
    #[eserde(compat)]
    pub websites: Option<Vec<Website>>,
    /// Keywords related to the associated research activity data
    ///
    /// See [Keyword]
    #[builder(default = Vec::<String>::new())]
    pub keywords: Vec<Keyword>,
    /// Software, programmings languages, and digital resources (e.g., tools, libraries, frameworks, data) related to the associated research activity data
    /// ### Examples
    /// - Rust
    /// - Polars
    /// - gdal
    /// - matplotlib
    /// - LaTeX
    ///
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/technology.csv">Full list of technologies</a></div>
    #[builder(default = Vec::<String>::new())]
    #[serde(deserialize_with = "vec_string_trim")]
    pub technology: Vec<String>,
    /// Organization(s) responsible for funding associated research activity data
    ///
    /// Includes any office within a US cabinet-level department that has leadership appointed by the president and confirmed by the Senate, e.g., NNSA or Office of Science.
    ///
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/sponsors.csv">Full list of sponsors</a></div>
    pub sponsors: Option<Vec<String>>,
    /// Organization(s) related to the associated research activity data
    /// ### Examples
    /// - Los Alamos National Laboratory
    /// - University of Tennessee
    /// - IBM
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/partners.csv">Full list of partners</a></div>
    pub partners: Option<Vec<String>>,
    /// Related resarch activity data identifiers of related research activity data
    pub related: Option<Vec<String>>,
    /// Hardware/compute resource requirements
    /// ### Note
    /// Selected components should reflect the minimum hardware/compute resources required to perform the research activity
    ///
    /// ### Example
    /// For an AI/ML workflow that requires at least one GPU to perform training that cannot be executed on a CPU alone,
    /// the resources attribute should include GPU, but need not include CPU
    #[eserde(compat)]
    pub resources: Option<Vec<Resource>>,
}
/// Linked data (e.g., JSON-LD) context for metadata
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityMetadataContext {
    /// Classification level
    pub classification: String,
    /// Archive status
    pub archive: String,
    /// Draft status
    pub draft: String,
    /// Research activity status
    pub status: String,
    /// Local CURIE research activity identifier
    pub identifier: String,
    /// Associated Digital Object Identifiers
    pub doi: String,
    /// Research Activity Identifier
    pub raid: String,
    /// Research Organization Registry
    pub ror: String,
    /// Additional type (for organizations)
    pub additional_type: String,
    /// Images, videos, and other media
    pub media: String,
    /// Websites
    pub websites: String,
    /// Keywords
    pub keywords: String,
    /// Software, programmings languages, and digital resources used by research activity
    pub technology: String,
    /// Sponsors
    pub sponsors: String,
    /// Partners
    pub partners: String,
    /// Related research activity data
    pub related: String,
    /// Hardware/compute resource requirements
    pub resources: String,
}
/// Research activity prose components that describe the activity using natural language
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields)]
pub struct Sections {
    /// The reason for the research or research organization to exist
    /// ### Example
    /// > "Develop the first atomic bombs in the world to assist the Allied forces and bring an end to WWII"
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_MISSION",
        message = "Mission is too long, reduce the length below 250 characters."
    ))]
    #[builder(default = "Purpose of the research".to_string())]
    #[serde(alias = "introduction", deserialize_with = "string_trim")]
    pub mission: String,
    /// A problem or situation within a research field requiring scientific effort, resources, and/or innovation to overcome
    /// ### Example
    /// > "During WWII, there was a fear that Germany was researching and developing nuclear weapons, giving them a decisive advantage over Allied forces, including the United States, Great Britain, and Canada."
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_CHALLENGE",
        message = "Challenge is too long, reduce the length below 500 characters."
    ))]
    #[builder(default = "Reason for the research".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub challenge: String,
    /// The plan, resources and actions taken to perform the research in a given project or organization
    /// ### Examples
    /// - "Production across four different sites in the United States, each with a different focus, for security and safety purposes"
    /// - "Research into new fields including nuclear fission, isotope separation methods, uranium enrichment, plutonium development, and weapons design"
    /// - "Military coordination for project construction and security management as well as defense communications to national leaders"
    #[validate(
        length(min = 1, max = "MAX_COUNT_APPROACH", message = "Limit the number of approaches to 6"),
        custom(function = "is_attribute_approach")
    )]
    #[builder(default = vec!["List of actions taken to perform the research".to_string()])]
    #[serde(deserialize_with = "vec_string_trim")]
    pub approach: Vec<String>,
    /// Tangible effects the research approach has on areas outside academia, such as industry, society, the surrounding environment, or culture
    /// ### Examples
    /// - "Development of the world's first atomic weapons"
    /// - "Introduction of the nuclear age, including advancements in nuclear science, engineering and a new source of energy"
    /// - "The end of WWII, along with many ethical and moral considerations related to use of atomic weapons"
    #[validate(length(min = 1, max = "MAX_COUNT_IMPACT"), custom(function = "is_attribute_impact"))]
    #[builder(default = vec!["List of tangible proof that validates the research approach".to_string()])]
    #[serde(alias = "outcomes", deserialize_with = "vec_string_trim")]
    pub impact: Vec<String>,
    /// Notable recognition or awards given to the research team, organization, or research products
    /// ### Examples
    /// - "At least six Nobel Prizes awarded to Manhattan Project researchers in the years following the end of the project"
    /// - "Creation of the Atomic Energy Commission in 1946, later becoming the Department of Energy and Nuclear Regulatory Commission"
    #[validate(length(min = 1, max = 4, message = "Limit the number of achievements to 4"))]
    pub achievement: Option<Vec<String>>,
    /// Expertise as applied to technology in a given mission space
    /// ### Examples
    /// - "Gaseous diffusion and electromagnetic separation to create fissionable materials"
    /// - "Mechanisms for achieving supercritical mass for nuclear detonation"
    /// - "Nuclear reactor development, which paved the way for nuclear power"
    /// - "Radiochemistry for nuclear detonation analysis and advanced medical research with radioisotopes"
    /// - "Large-scale multidisciplinary scientific collaboration"
    #[validate(length(min = 1, max = "MAX_COUNT_CAPABILITIES"), custom(function = "is_attribute_capabilities"))]
    pub capabilities: Option<Vec<String>>,
    /// Overview of research focus and areas
    /// ### Example Focus
    /// > "Developing fissionable materials for nuclear reactions to develop the world's first atomic weapons"
    /// ### Example Areas
    /// - "Nuclear fission"
    /// - "Radiochemistry"
    /// - "Uranium enrichment"
    /// - "Electromagnetic separation"
    /// - "Weapon design"
    #[validate(nested)]
    #[builder(default = Research::init().build())]
    pub research: Research,
}
impl Default for ResearchActivity {
    fn default() -> Self {
        ResearchActivity::init().build()
    }
}
impl Default for ResearchActivityContext {
    fn default() -> Self {
        ResearchActivityContext::init()
            .meta(schema_org("CreativeWork"))
            .title(dcterms("title"))
            .subtitle(schema_org("alternativeHeadline"))
            .sections(schema_org("CreativeWork"))
            .contact(dcat("contactPoint"))
            .notes(schema_org("Text"))
            .build()
    }
}
impl Default for ResearchActivityMetadata {
    fn default() -> Self {
        ResearchActivityMetadata::init().build()
    }
}
impl Default for ResearchActivityMetadataContext {
    fn default() -> Self {
        ResearchActivityMetadataContext::init()
            .classification(schema_org("DefinedTerm"))
            .archive(schema_org("Boolean"))
            .draft(schema_org("Boolean"))
            .status(schema_org("DefinedTerm"))
            .identifier(codemeta("identifier"))
            .doi(bibo("doi"))
            .raid(DEFAULT_RAID_SCHEMA_URI)
            .ror(DEFAULT_ROR_SCHEMA_URI)
            .additional_type(schema_org("additionalType"))
            .media(schema_org("MediaObject"))
            .websites(schema_org("WebSite"))
            .keywords(dcat("keyword"))
            .technology(schema_org("DefinedTerm"))
            .sponsors(codemeta("sponsor"))
            .partners(schema_org("Text"))
            .related(schema_org("Text"))
            .resources(schema_org("DefinedTerm"))
            .build()
    }
}
impl Hash for ResearchActivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.identifier.hash(state);
    }
}
impl LinkedData for ResearchActivity {
    fn with_context(&self) -> Self {
        Self {
            context: Some(ResearchActivityContext::default()),
            research_activity_type: None,
            meta: self.meta.with_context(),
            contact: self.contact.with_context(),
            ..self.clone().copy()
        }
    }
}
impl LinkedData for ResearchActivityMetadata {
    fn with_context(&self) -> Self {
        Self {
            context: Some(ResearchActivityMetadataContext::default()),
            metadata_type: None,
            ..self.clone()
        }
    }
}
#[cfg(feature = "std")]
impl InputOutput for ResearchActivity {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into().clone();
        let result = match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source.clone()),
            | MimeType::Jsonc => Self::read_jsonc(source.clone()),
            | MimeType::Yaml => Self::read_yaml(source.clone()),
            | _ => Err(eyre!("Unsupported research activity data file extension")),
        };
        if let Ok(data) = &result {
            debug!(path = source.to_string_lossy().to_string(), "=> {}", Label::using());
            debug!("=> {} Research activity data = {:#?}", Label::using(), data.dimmed().cyan());
        }
        result
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_file(path.clone()).and_then(|content| {
            eserde::json::from_str::<Self>(&content).map_err(|errors| {
                let details: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                    .collect();
                eyre!("{}", details.join("\n"))
            })
        })
    }
    fn read_jsonc(path: PathBuf) -> ApiResult<Self> {
        read_file(path.clone()).and_then(|content| {
            jsonc_parse_value(&content).and_then(|value| {
                serde_json::to_string(&value)
                    .map_err(|why| eyre!("JSONC conversion error — {why}"))
                    .and_then(|json| {
                        eserde::json::from_str::<Self>(&json).map_err(|errors| {
                            let details: Vec<String> = errors
                                .iter()
                                .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                                .collect();
                            eyre!("{}", details.join("\n"))
                        })
                    })
            })
        })
    }
    fn read_markdown(path: PathBuf) -> Option<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) => value,
            | Err(_) => {
                error!(path = path.to_string_lossy().to_string(), "=> {} RAD content is not valid", Label::fail());
                "{}".to_owned()
            }
        };
        let (frontmatter, _body) = frontmatter_and_body(&content);
        let _meta: ResearchActivityMetadata = match frontmatter {
            | Some(value) => match serde_norway::from_str(&value) {
                | Ok(data) => data,
                | Err(why) => {
                    fail!("Parse RAD frontmatter - {}", why.red());
                    ResearchActivityMetadata::default()
                }
            },
            | None => ResearchActivityMetadata::default(),
        };
        None
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_file(path.clone()).and_then(|content| serde_norway::from_str(&content).map_err(|why| eyre!("Failed to parse YAML RAD — {why}")))
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        let mime = MimeType::from_path(&output);
        match mime {
            | MimeType::Cff => self.write_cff(output),
            | MimeType::Json | MimeType::Jsonc => self.write_json(output),
            | MimeType::Markdown => self.write_markdown(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported research activity data file extension for writing")),
        }
    }
    fn write_cff(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("cff");
        let data: Cff = self.into();
        serde_norway::to_string(&data)
            .map_err(|why| eyre!("Failed to serialize RAD CITATION.cff — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        serde_json::to_string_pretty(&self)
            .map_err(|why| eyre!("Failed to serialize JSON RAD — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("md");
        let content = self.to_markdown();
        write_file(output, content)
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_json::to_value(self)
            .map_err(|why| eyre!("Failed to convert RAD to value for YAML serialization — {why}"))
            .and_then(|value| serde_norway::to_string(&value).map_err(|why| eyre!("Failed to serialize YAML RAD — {why}")))
            .and_then(|content| write_file(output, content))
    }
}
impl ResearchActivity {
    /// Creates a new `ResearchActivity`
    pub fn new() -> Self {
        ResearchActivity::default()
    }
    /// Serializes a research activity into a string for a supported mime type
    #[cfg(feature = "std")]
    pub fn serialize_as(&self, mime: &MimeType) -> ApiResult<String> {
        match mime {
            | MimeType::Json | MimeType::Jsonc => serde_json::to_string_pretty(self).map_err(|why| eyre!("Failed to serialize JSON RAD — {why}")),
            | MimeType::Yaml => serde_json::to_value(self)
                .map_err(|why| eyre!("Failed to convert RAD to value for YAML serialization — {why}"))
                .and_then(|value| serde_norway::to_string(&value).map_err(|why| eyre!("Failed to serialize YAML RAD — {why}"))),
            | _ => Err(eyre!("Unsupported mime type for research activity serialization: {mime:?}")),
        }
    }
    /// Print research activity schema as JSON or YAML schema
    #[cfg(feature = "std")]
    pub fn to_schema(format: &str) {
        let schema = schema_for!(ResearchActivity);
        let output = match format.to_lowercase().as_str() {
            | "yaml" | "yml" => serde_norway::to_string(&schema).unwrap_or_default(),
            | _ => serde_json::to_string_pretty(&schema).unwrap_or_default(),
        };
        println!("{output}");
    }
    /// Creates a copy of a `ResearchActivity`
    pub fn copy(&self) -> ResearchActivity {
        let ResearchActivity {
            meta,
            title,
            subtitle,
            sections,
            contact,
            notes,
            ..
        } = self.clone();
        ResearchActivity::init()
            .meta(meta)
            .title(title)
            .maybe_subtitle(subtitle)
            .sections(sections)
            .contact(contact)
            .maybe_notes(notes)
            .build()
    }
    /// Formats research activity data
    /// ### Actions
    /// - Resolves keywords, technology, organization, partners, sponsors, and affiliation using fuzzy matching against controlled vocabularies
    /// - Formats contact telephone number
    pub fn format(self) -> ResearchActivity {
        let Self { meta, contact, .. } = self.clone();
        Self {
            meta: meta.format(),
            contact: contact.format(),
            ..self
        }
    }
    /// Formats research activity data with context of filesystem and/or remote resources
    /// ### Actions
    /// - Resolves URL of first media object (if found) and add empty caption
    /// - Resolves ORCiD identifier from contact first name, last name, and email (if found)
    #[cfg(feature = "std")]
    pub fn format_with(self, context: Option<PathBuf>) -> ResearchActivity {
        let Self { meta, contact, .. } = self.clone();
        Self {
            meta: meta.format_with(context.clone()),
            contact: contact.format_with(context.clone()),
            ..self
        }
        .format()
    }
}
impl From<&ResearchActivity> for Cff {
    fn from(rad: &ResearchActivity) -> Self {
        let sections = rad.sections.clone();
        let contact = rad.contact.clone();
        let meta = rad.meta.clone();
        let title = rad.title.clone();
        let ContactPoint {
            given_name,
            family_name,
            identifier: orcid,
            email,
            organization,
            affiliation,
            ..
        } = contact;
        let person_affiliation = affiliation.or(if organization.is_empty() { None } else { Some(organization) });
        let person = Agent::Person(Person {
            given_names: Some(given_name),
            family_names: Some(family_name),
            orcid,
            email: Some(email),
            affiliation: person_affiliation,
            address: None,
            alias: None,
            city: None,
            country: None,
            fax: None,
            name_particle: None,
            name_suffix: None,
            postal_code: None,
            region: None,
            tel: None,
            website: None,
        });
        let keywords = if meta.keywords.is_empty() { None } else { Some(meta.keywords) };
        let (doi, identifiers) = match meta.doi {
            | Some(dois) => match dois.as_slice() {
                | [single] => (Some(single.clone()), None),
                | [] => (None, None),
                | _ => (
                    None,
                    Some(
                        dois.into_iter()
                            .map(|value| Identifier {
                                description: None,
                                kind: IdentifierType::Doi,
                                value,
                            })
                            .collect(),
                    ),
                ),
            },
            | None => (None, None),
        };
        Cff {
            abstract_text: Some(sections.mission),
            authors: vec![person.clone()],
            contact: Some(vec![person]),
            keywords,
            doi,
            identifiers,
            title,
            ..Cff::default()
        }
    }
}
impl From<ResearchActivity> for Cff {
    fn from(rad: ResearchActivity) -> Self {
        Cff::from(&rad)
    }
}
impl ToMarkdown for ResearchActivity {
    fn to_markdown(&self) -> String {
        let Self {
            meta,
            title,
            subtitle,
            sections,
            ..
        } = self;
        let classification = meta.clone().classification.unwrap_or_default();
        let frontmatter = serde_norway::to_string(&meta).unwrap_or_default();
        let sections = sections.to_markdown();
        let contact = self.contact.to_markdown();
        match &subtitle {
            | Some(subtitle) => format!(
                r#"---
classification: {classification}
{frontmatter}---
# {title}
> {subtitle}
{sections}

{contact}"#
            ),
            | None => format!(
                r#"---
classification: {classification}
{frontmatter}---
# {title}
{sections}

{contact}"#
            ),
        }
    }
}
impl ToProse for ResearchActivity {
    fn to_prose(&self) -> String {
        let websites = self
            .meta
            .websites
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|website| website.to_markdown())
            .collect::<Vec<String>>()
            .join("\n");
        let sections = self.sections.to_markdown();
        match &self.subtitle {
            | Some(subtitle) => format!(
                r#"{title}
{subtitle}
{sections}
{websites}"#,
                title = self.title
            ),
            | None => format!(
                r#"{title}
{sections}
{websites}"#,
                title = self.title
            ),
        }
    }
}
impl ResearchActivityMetadata {
    pub(crate) fn first_image(self) -> Option<MediaObject> {
        match self.media {
            | Some(values) => values.into_iter().filter(|x| x.clone().is_image()).collect::<Vec<_>>().first().cloned(),
            | None => None,
        }
    }
    /// Returns the content URL of the first image in the list of media objects, or a default value if none are present.
    pub fn first_image_content_url(self) -> String {
        match self.first_image() {
            | Some(media) => match media {
                | MediaObject::Image(ImageObject { content_url, .. }) => match content_url {
                    | Some(value) if !value.is_empty() => value.clone().trim().to_string(),
                    | Some(_) | None => DEFAULT_GRAPHIC_HREF.to_string(),
                },
                | _ => DEFAULT_GRAPHIC_HREF.to_string(),
            },
            | None => DEFAULT_GRAPHIC_HREF.to_string(),
        }
    }
    /// Returns the caption of the first image in the list of media objects, or a default value if none are present.
    pub fn first_image_caption(self) -> String {
        match self.first_image() {
            | Some(MediaObject::Image(ImageObject { caption, .. })) => match caption.clone() {
                | value if !value.is_empty() => value.clone(),
                | _ => DEFAULT_GRAPHIC_CAPTION.to_string(),
            },
            | Some(_) | None => DEFAULT_GRAPHIC_CAPTION.to_string(),
        }
    }
    /// Fix, resolve, and augment research activity metadata
    pub fn format(self) -> Self {
        let keywords = Vocabulary::Keywords.resolve(Some(self.keywords.clone()));
        let technology = Vocabulary::Technology.resolve(Some(self.technology.clone()));
        let partners = match Vocabulary::Partners.resolve(self.partners.clone()) {
            | values if !values.is_empty() => Some(values),
            | _ => None,
        };
        let sponsors = match Vocabulary::Sponsors.resolve(self.sponsors.clone()) {
            | values if !values.is_empty() => Some(values),
            | _ => None,
        };
        Self {
            keywords,
            technology,
            partners,
            sponsors,
            ..self
        }
    }
    /// Fix, resolve, and augment research activity metadata with access to filesystem and/or remote resources
    #[cfg(feature = "std")]
    pub fn format_with(self, path: Option<PathBuf>) -> Self {
        let path_parent = match path {
            | Some(value) => parent(value),
            | None => PathBuf::from("."),
        };
        debug!(path = path_parent.to_absolute_string(), "=> {} Parent directory", Label::using());
        let name = match image_paths(&path_parent) {
            | value if !value.is_empty() => value.first().and_then(|v| v.file_name().map(|f| f.to_string_lossy().to_string())),
            | _ => None,
        };
        let media = match name {
            | Some(value) => {
                debug!(value, "=> {} First image", Label::using());
                let first_graphic = match self.media.clone() {
                    | Some(values) if !values.is_empty() => {
                        let caption = self.clone().first_image_caption();
                        let image_data = ImageObject::init().caption(caption.to_string()).content_url(value.clone()).build();
                        MediaObject::Image(image_data)
                    }
                    | Some(_) | None => {
                        let image_data = ImageObject::init().caption("".to_string()).content_url(value.clone()).build();
                        MediaObject::Image(image_data)
                    }
                };
                let rest = match self.clone().media {
                    | Some(values) if !values.is_empty() => values.into_iter().skip(1).collect::<Vec<_>>(),
                    | Some(_) | None => vec![],
                };
                Some([vec![first_graphic], rest].concat())
            }
            | None => self.media.clone(),
        };
        Self { media, ..self }.format()
    }
}
impl Vocabulary {
    /// Resolve values to intended values according to associated controlled vocabulary
    fn resolve(self, values: Option<Vec<String>>) -> Vec<String> {
        match values {
            | Some(items) => {
                let mut data: Vec<_> = items.into_iter().flat_map(|x| resolve_from_csv_asset(self.to_string(), x)).collect();
                data.sort();
                data.dedup();
                data
            }
            | None => vec![],
        }
    }
}
impl Default for Sections {
    fn default() -> Self {
        Sections::init().build()
    }
}
impl ToMarkdown for Research {
    fn to_markdown(&self) -> String {
        let Research { focus, areas } = self;
        format!(
            r#"
## Focus
{focus}

## Areas{}"#,
            areas.to_markdown(),
        )
    }
}
impl ToMarkdown for Sections {
    fn to_markdown(&self) -> String {
        let Sections {
            mission,
            challenge,
            approach,
            impact,
            research,
            ..
        } = self;
        format!(
            r#"
## Mission
{}

## Challenge
{}

## Approach{}

## Impact{}
{}"#,
            mission,
            challenge,
            approach.to_markdown(),
            impact.to_markdown(),
            research.to_markdown(),
        )
    }
}
#[cfg(feature = "std")]
pub(crate) fn resolve_from_csv_asset(name: String, value: String) -> Option<String> {
    let data = Constant::csv(&name);
    resolve_from_list_of_lists(value, data, name)
}
#[cfg(not(feature = "std"))]
pub(crate) fn resolve_from_csv_asset(_name: String, value: String) -> Option<String> {
    Some(value)
}
#[cfg(feature = "std")]
pub(crate) fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, data: I, name: String) -> Option<String> {
    fn match_list<I: IntoIterator<Item = String> + Clone>(value: String, values: I) -> Vec<(String, u32)> {
        let pattern = Pattern::parse(&value, CaseMatching::Ignore, Normalization::Smart);
        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        pattern.match_list(values.clone(), &mut matcher)
    }
    fn print_resolution(output: Option<String>, value: String, name: String) {
        let label = name.to_case(Case::Title);
        match output {
            | Some(resolved) => {
                if resolved.eq(&value.to_string()) {
                    trace!("=> {} {} = \"{}\"", Label::using(), label, value.clone());
                } else {
                    debug!(input = value.clone(), resolved, "=> {} {}", Label::found(), label);
                }
            }
            | None => {
                debug!(value = value.clone(), "=> {} {}", Label::not_found(), label);
            }
        };
    }
    fn sanitize(value: String) -> String {
        match Regex::new(r"[-_.,]") {
            | Ok(re) => re.replace_all(&value, "").replace("&", "and").trim().to_string(),
            | Err(err) => err.to_string(),
        }
    }
    let output = data
        .into_iter()
        .flat_map(|values| {
            let sanitized = sanitize(value.clone());
            let matched = match_list(sanitized, values.clone().into_iter().take(4));
            trace!("{} => {:?}", value.clone(), matched.clone());
            if matched.clone().is_empty() {
                None
            } else {
                match values.first() {
                    | Some(x) => {
                        if value.eq(x) {
                            Some((x.into(), 10000))
                        } else {
                            let score = matched.into_iter().map(|(_, score)| score).max();
                            match score {
                                | Some(value) if value > 0 => Some((x.to_string(), value)),
                                | Some(_) | None => None,
                            }
                        }
                    }
                    | None => None,
                }
            }
        })
        .max_by_key(|(_, score)| *score)
        .map(|(x, _)| x.to_string());
    print_resolution(output.clone(), value, name);
    output
}
#[cfg(not(feature = "std"))]
#[allow(dead_code)]
pub(crate) fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, _data: I, _name: String) -> Option<String> {
    Some(value)
}
fn validation_error_with_index(code: &'static str, message: String, index: usize) -> ValidationError {
    let mut err = ValidationError::new(code).with_message(message.into());
    err.add_param("index".into(), &index);
    err
}
fn validation_error_with_index_and_length(code: &'static str, message: String, index: usize, length: usize) -> ValidationError {
    let mut err = validation_error_with_index(code, message, index);
    err.add_param("length".into(), &length);
    err
}
/// Custom validator function for [approach](Sections::approach)
pub(crate) fn is_attribute_approach(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.approach";
    const MAX_LENGTH: usize = MAX_LENGTH_APPROACH;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(
                CODE,
                format!("Each approach statement should be less than {MAX_LENGTH} characters"),
                index,
                length,
            )
        })
        .map_or(Ok(()), Err)
}
/// Custom validator function for [research areas](Research::areas)
pub(crate) fn is_attribute_areas(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.areas";
    const MAX_LENGTH: usize = MAX_LENGTH_RESEARCH_AREA;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(CODE, format!("Each area should be less than {MAX_LENGTH} characters"), index, length)
        })
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [books](ResearchActivityMetadata::books)
pub(crate) fn is_attribute_books(value: &[String]) -> ValidationResult {
    const CODE: &str = "meta.books";
    value
        .iter()
        .position(|x| !x.is_isbn())
        .map(|index| validation_error_with_index(CODE, "Every book should be a valid ISBN".to_string(), index))
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [capabilities](Sections::capabilities)
pub(crate) fn is_attribute_capabilities(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.capabilities";
    const MAX_LENGTH: usize = MAX_LENGTH_CAPABILIY;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(
                CODE,
                format!("Each capability should be less than {MAX_LENGTH} characters"),
                index,
                length,
            )
        })
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [doi](ResearchActivityMetadata::doi)
pub(crate) fn is_attribute_doi_list(value: &[String]) -> ValidationResult {
    const CODE: &str = "meta.doi";
    value
        .iter()
        .position(|x| is_doi(x).is_err())
        .map(|index| validation_error_with_index(CODE, "Every DOI should be valid".to_string(), index))
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [ror](ResearchActivityMetadata::ror)
pub(crate) fn is_attribute_ror_list(value: &[String]) -> ValidationResult {
    const CODE: &str = "meta.ror";
    value
        .iter()
        .position(|x| is_ror(x).is_err())
        .map(|index| validation_error_with_index(CODE, "Every ROR should be valid".to_string(), index))
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [impact](Sections::impact)
pub(crate) fn is_attribute_impact(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.impact";
    const MAX_LENGTH: usize = MAX_LENGTH_IMPACT;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(
                CODE,
                format!("Each impact statement should be less than {MAX_LENGTH} characters"),
                index,
                length,
            )
        })
        .or_else(|| {
            value
                .first()
                .and_then(|first| {
                    let ends_with_period = first.trim().ends_with(".");
                    value.iter().position(|x| x.trim().ends_with(".") != ends_with_period)
                })
                .map(|index| {
                    validation_error_with_index(
                        CODE,
                        "Impact statements should be all sentences with periods or all phrases without periods".to_string(),
                        index,
                    )
                })
        })
        .or_else(|| {
            value
                .iter()
                .position(|x| {
                    x.trim().chars().find(|c| c.is_alphabetic()).is_none_or(|letter| {
                        let actual = letter.to_string();
                        actual != actual.to_case(Case::Upper)
                    })
                })
                .map(|index| validation_error_with_index(CODE, "Impact statements should begin with a capital letter".to_string(), index))
        })
        .map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests;
