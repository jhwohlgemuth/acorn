//! Schema I/O adapter.
//!
//! Hosts std-only path/file behavior for schema types so that [`crate::schema`]
//! modules stay portable. This module later moves into `acorn-host`.
pub mod migrate;
use self::migrate::Migration;
use crate::error::ApiResult;
use crate::io::document::{DocumentFormat, SourceDocument};
use crate::io::{image_paths, jsonc_parse_value, parent, read_file, write_file, FromPath, InputOutput};
use crate::prelude::alloc::{vec, Vec};
use crate::prelude::std::{Error, Path, PathBuf};
use crate::schema::pid::raid::Metadata;
use crate::schema::pid::PersistentIdentifierConvert;
use crate::schema::research_activity::{ResearchActivity, ResearchActivityMetadata};
use crate::schema::standard::cff::Cff;
use crate::schema::standard::text::Text;
use crate::schema::standard::{datacite, dcat, huwise, invenio, text};
use crate::schema::{ImageObject, MediaObject, OneOrMany};
use crate::util::{file_extension, Label, LinkedData, MarkdownSupport, MimeType, StringConversion, Unstructured};
use color_eyre::eyre::eyre;
use owo_colors::OwoColorize;
use tracing::{debug, error};

impl InputOutput for Cff {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Cff> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Cff | MimeType::Yaml => Cff::read_yaml(source),
            | MimeType::Json => Cff::read_json(source),
            | _ => Err(eyre!("Unsupported CFF data file extension")),
        }
    }
    fn read_cff(path: impl Into<PathBuf>) -> ApiResult<Cff> {
        Cff::read_yaml(path.into())
    }
    fn read_json(path: PathBuf) -> ApiResult<Cff> {
        read_file(path.clone()).and_then(|content| {
            eserde::json::from_str::<Cff>(&content).map_err(|errors| {
                let details: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                    .collect();
                eyre!("{}", details.join("\n"))
            })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Cff> {
        read_file(path.clone()).and_then(|content| serde_norway::from_str(&content).map_err(|why| eyre!("Failed to parse YAML CFF — {why}")))
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Cff => self.write_cff(output),
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported CFF data file extension for writing")),
        }
    }
    fn write_cff(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("cff");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize CFF — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON CFF — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML CFF — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl InputOutput for datacite::Record {
    fn read(path: impl Into<PathBuf>) -> ApiResult<datacite::Record> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Json => datacite::Record::read_json(source),
            | MimeType::Yaml => datacite::Record::read_yaml(source),
            | _ => Err(eyre!("Unsupported DataCite data file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<datacite::Record> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum JsonInput {
            One(Box<datacite::Record>),
            Many(Vec<datacite::Record>),
        }
        read_file(path).and_then(|content| {
            serde_json::from_str::<JsonInput>(&content)
                .map_err(|why| eyre!("Failed to parse JSON DataCite record — {why}"))
                .and_then(|value| match value {
                    | JsonInput::One(record) => Ok(*record),
                    | JsonInput::Many(records) => match records.len() {
                        | 1 => records
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one DataCite record but found none")),
                        | len => Err(eyre!("Expected one DataCite record but found {len}")),
                    },
                })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<datacite::Record> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum YamlInput {
            One(Box<datacite::Record>),
            Many(Vec<datacite::Record>),
        }
        read_file(path).and_then(|content| {
            serde_norway::from_str::<YamlInput>(&content)
                .map_err(|why| eyre!("Failed to parse YAML DataCite record — {why}"))
                .and_then(|value| match value {
                    | YamlInput::One(record) => Ok(*record),
                    | YamlInput::Many(records) => match records.len() {
                        | 1 => records
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one DataCite record but found none")),
                        | len => Err(eyre!("Expected one DataCite record but found {len}")),
                    },
                })
        })
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported DataCite data file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON DataCite record — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML DataCite record — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl InputOutput for dcat::Dataset {
    fn read(path: impl Into<PathBuf>) -> ApiResult<dcat::Dataset> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Json => dcat::Dataset::read_json(source),
            | MimeType::Yaml => dcat::Dataset::read_yaml(source),
            | _ => Err(eyre!("Unsupported DCAT data file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<dcat::Dataset> {
        read_file(path).and_then(|content| {
            serde_json::from_str::<OneOrMany<dcat::Dataset>>(&content)
                .map_err(|why| eyre!("Failed to parse JSON DCAT dataset — {why}"))
                .and_then(|value| match value {
                    | OneOrMany::One(dataset) => Ok(dataset),
                    | OneOrMany::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one DCAT dataset but found none")),
                        | len => Err(eyre!("Expected one DCAT dataset but found {len}")),
                    },
                })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<dcat::Dataset> {
        read_file(path).and_then(|content| {
            serde_norway::from_str::<OneOrMany<dcat::Dataset>>(&content)
                .map_err(|why| eyre!("Failed to parse YAML DCAT dataset — {why}"))
                .and_then(|value| match value {
                    | OneOrMany::One(dataset) => Ok(dataset),
                    | OneOrMany::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one DCAT dataset but found none")),
                        | len => Err(eyre!("Expected one DCAT dataset but found {len}")),
                    },
                })
        })
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported DCAT data file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON DCAT dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML DCAT dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl InputOutput for huwise::Dataset {
    fn read(path: impl Into<PathBuf>) -> ApiResult<huwise::Dataset> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Json => huwise::Dataset::read_json(source),
            | MimeType::Yaml => huwise::Dataset::read_yaml(source),
            | _ => Err(eyre!("Unsupported HuWise data file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<huwise::Dataset> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum JsonInput {
            One(Box<huwise::Dataset>),
            Many(Vec<huwise::Dataset>),
        }
        read_file(path).and_then(|content| {
            serde_json::from_str::<JsonInput>(&content)
                .map_err(|why| eyre!("Failed to parse JSON HuWise dataset — {why}"))
                .and_then(|value| match value {
                    | JsonInput::One(dataset) => Ok(*dataset),
                    | JsonInput::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one HuWise dataset but found none")),
                        | len => Err(eyre!("Expected one HuWise dataset but found {len}")),
                    },
                })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<huwise::Dataset> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum YamlInput {
            One(Box<huwise::Dataset>),
            Many(Vec<huwise::Dataset>),
        }
        read_file(path).and_then(|content| {
            serde_norway::from_str::<YamlInput>(&content)
                .map_err(|why| eyre!("Failed to parse YAML HuWise dataset — {why}"))
                .and_then(|value| match value {
                    | YamlInput::One(dataset) => Ok(*dataset),
                    | YamlInput::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one HuWise dataset but found none")),
                        | len => Err(eyre!("Expected one HuWise dataset but found {len}")),
                    },
                })
        })
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported HuWise data file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON HuWise dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML HuWise dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl InputOutput for invenio::Record {
    fn read(path: impl Into<PathBuf>) -> ApiResult<invenio::Record> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Json => invenio::Record::read_json(source),
            | MimeType::Yaml => invenio::Record::read_yaml(source),
            | _ => Err(eyre!("Unsupported Invenio data file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<invenio::Record> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum JsonInput {
            One(Box<invenio::Record>),
            Many(Vec<invenio::Record>),
        }
        read_file(path).and_then(|content| {
            serde_json::from_str::<JsonInput>(&content)
                .map_err(|why| eyre!("Failed to parse JSON Invenio record — {why}"))
                .and_then(|value| match value {
                    | JsonInput::One(record) => Ok(*record),
                    | JsonInput::Many(records) => match records.len() {
                        | 1 => records
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one Invenio record but found none")),
                        | len => Err(eyre!("Expected one Invenio record but found {len}")),
                    },
                })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<invenio::Record> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum YamlInput {
            One(Box<invenio::Record>),
            Many(Vec<invenio::Record>),
        }
        read_file(path).and_then(|content| {
            serde_norway::from_str::<YamlInput>(&content)
                .map_err(|why| eyre!("Failed to parse YAML Invenio record — {why}"))
                .and_then(|value| match value {
                    | YamlInput::One(record) => Ok(*record),
                    | YamlInput::Many(records) => match records.len() {
                        | 1 => records
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one Invenio record but found none")),
                        | len => Err(eyre!("Expected one Invenio record but found {len}")),
                    },
                })
        })
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported Invenio data file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON Invenio record — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML Invenio record — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl From<ResearchActivity> for Metadata {
    fn from(activity: ResearchActivity) -> Self {
        let default_start_date = crate::io::current_date();
        let default_end_year = default_start_date.chars().take(4).collect::<String>();
        let raid_id = activity.meta.raid.as_ref().and_then(|raid| raid.first()).cloned().unwrap_or_default();
        let ror_id = activity.meta.ror.as_ref().and_then(|rors| rors.first()).cloned().unwrap_or_default();
        let contributor_id = activity.contact.identifier.clone().filter(|id| id.is_orcid()).unwrap_or_default();
        let date = crate::schema::Date {
            start_date: Some(default_start_date.clone()),
            end_date: None,
        };
        let contributor_position = crate::schema::pid::raid::ContributorPosition::init()
            .id(crate::schema::pid::raid::PositionType::PrincipalInvestigator)
            .date(date.clone())
            .build();
        let contributor = crate::schema::pid::raid::Contributor {
            id: Some(contributor_id),
            schema_uri: Some(crate::schema::namespaces::DEFAULT_ORCID_SCHEMA_URI.to_string()),
            status: Some("active".to_string()),
            status_message: None,
            position: vec![contributor_position],
            leader: true,
            contact: true,
            email: Some(activity.contact.email.clone()),
            role: None,
            uuid: None,
        };
        let org_role = crate::schema::pid::raid::OrganizationRole {
            id: crate::schema::pid::raid::OrganizationRoleType::LeadResearchOrganization,
            schema_uri: None,
            date: date.clone(),
        };
        let organization = crate::schema::pid::raid::Organization {
            id: ror_id.clone(),
            schema_uri: Some(crate::schema::namespaces::DEFAULT_ROR_SCHEMA_URI.to_string()),
            role: vec![org_role],
        };
        let title = crate::schema::pid::raid::Title {
            text: Some(activity.title.clone()),
            title_type: Some(
                crate::schema::pid::raid::TitleIdentifier::init()
                    .id(crate::schema::pid::raid::TitleType::Primary)
                    .build(),
            ),
            language: None,
            start_date: Some(default_start_date.clone()),
            end_date: Some(default_end_year.clone()),
        };
        let description = crate::schema::pid::raid::Description {
            text: Some(activity.sections.mission.clone()),
            description_type: Some(crate::schema::pid::raid::DescriptionIdentifier {
                id: crate::schema::pid::raid::DescriptionType::Primary,
                schema_uri: "https://vocabulary.raid.org/description.type.schema/318".to_string(),
            }),
            language: None,
        };
        let identifier = crate::schema::pid::raid::MetadataIdentifier {
            id: Some(raid_id),
            schema_uri: Some("https://raid.org/".to_string()),
            owner: Some(crate::schema::pid::raid::Owner {
                id: ror_id.clone(),
                schema_uri: Some(crate::schema::namespaces::DEFAULT_ROR_SCHEMA_URI.to_string()),
                service_point: 0,
            }),
            raid_agency_url: Some("https://raid.org/".to_string()),
            registration_agency: Some(crate::schema::pid::raid::RegistrationAgency {
                id: ror_id,
                schema_uri: Some(crate::schema::namespaces::DEFAULT_ROR_SCHEMA_URI.to_string()),
            }),
            license: Some(OneOrMany::One("CC0-1.0".to_string())),
            version: Some(1),
        };
        let access = crate::schema::pid::raid::Access {
            access_type: Some(crate::schema::pid::raid::AccessIdentifier {
                id: crate::schema::pid::raid::AccessType::OpenAccess,
                schema_uri: "https://vocabularies.coar-repositories.org/access_rights/".to_string(),
            }),
            embargo_expiry: None,
            statement: None,
        };
        Metadata {
            access: Some(access),
            contributor: Some(vec![contributor]),
            date: None,
            description: Some(vec![description]),
            metadata: None,
            title: Some(vec![title]),
            identifier: Some(identifier),
            alternate_identifier: None,
            alternate_url: None,
            organization: Some(vec![organization]),
            related_object: None,
            related_raid: None,
            spatial_coverage: None,
            subject: None,
            traditional_knowledge_label: None,
        }
    }
}
impl TryFrom<PathBuf> for Metadata {
    type Error = Error;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        match read_file(path) {
            | Ok(data) => match eserde::json::from_str::<Self>(&data) {
                | Ok(value) => Ok(value),
                | Err(errors) => {
                    let details: Vec<String> = errors
                        .iter()
                        .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                        .collect();
                    Err(Error::other(details.join("\n")))
                }
            },
            | Err(why) => {
                let msg = format!("Read RAiD metadata file - {why}");
                error!("=> {} {}", Label::fail(), msg);
                Err(Error::other(msg))
            }
        }
    }
}
impl InputOutput for ResearchActivity {
    fn parse(content: &str) -> Option<Self> {
        jsonc_parse_value(content)
            .ok()
            .and_then(|value| serde_json::from_value(value).ok())
            .or_else(|| serde_norway::from_str(content).ok())
    }
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into().clone();
        let result = match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source.clone()),
            | MimeType::Jsonc => Self::read_jsonc(source.clone()),
            | MimeType::Markdown => Self::read_markdown(source.clone()),
            | MimeType::Yaml => Self::read_yaml(source.clone()),
            | _ => Err(eyre!("Unsupported research activity data file extension")),
        };
        if let Ok(data) = &result {
            debug!(path = source.to_string_lossy().to_string(), "=> {}", Label::using());
            debug!("=> {} Research activity data = {:#?}", Label::using(), data.dimmed().cyan());
        }
        result
    }
    fn format_with(self, context: Option<PathBuf>) -> Self {
        ResearchActivity::format_with(self, context)
    }
    fn serialize_as(&self, mime: &MimeType) -> ApiResult<String> {
        ResearchActivity::serialize_as(self, mime)
    }
    fn linked_content(&self) -> ApiResult<Option<String>> {
        serde_json::to_string_pretty(&self.with_context())
            .map(Some)
            .map_err(|why| eyre!("Failed to serialize linked research activity data — {why}"))
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_file(path.clone()).and_then(|content| {
            serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .as_ref()
                .into_iter()
                .for_each(Self::print_migration_warnings);
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
                Self::print_migration_warnings(&value);
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
    fn read_markdown(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| Self::from_markdown(&content).map_err(|why| eyre!("{why}")))
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_file(path.clone()).and_then(|content| {
            serde_norway::from_str::<serde_json::Value>(&content)
                .ok()
                .as_ref()
                .into_iter()
                .for_each(Self::print_migration_warnings);
            serde_norway::from_str(&content).map_err(|why| eyre!("Failed to parse YAML RAD — {why}"))
        })
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
impl ResearchActivityMetadata {
    /// Fix, resolve, and augment research activity metadata using optional filesystem context.
    pub fn format_with(self, context: Option<PathBuf>) -> Self {
        let path_parent = match context {
            | Some(path) => parent(path),
            | None => PathBuf::from("."),
        };
        debug!(path = path_parent.to_absolute_path(), "=> {} Parent directory", Label::using());
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
impl InputOutput for Text {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Markdown | MimeType::Text => Self::read_text(source),
            | MimeType::Json => Self::read_json(source),
            | MimeType::Yaml => Self::read_yaml(source),
            | _ => Err(eyre!("Unsupported plain text file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn read_markdown(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn serialize_as(&self, mime: &MimeType) -> ApiResult<String> {
        match mime {
            | MimeType::Json | MimeType::Markdown | MimeType::Text | MimeType::Yaml => Ok(self.content.clone()),
            | _ => Err(eyre!("Unsupported plain text workflow serialization mime type: {mime:?}")),
        }
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Markdown | MimeType::Text => self.write_text(output),
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported plain text file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
}
impl Text {
    fn read_text(path: impl Into<PathBuf>) -> ApiResult<Self> {
        read_file(path.into()).map(|content| Self { content })
    }
    fn write_text(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_file(path, self.content().to_string())
    }
}
impl TryFrom<&Path> for Text {
    type Error = color_eyre::Report;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Self::read(path.to_path_buf())
    }
}
impl InputOutput for text::Document {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match DocumentFormat::try_from(&MimeType::from(source.display().to_string())) {
            | Ok(_) => Self::read_document(source),
            | Err(_) => match file_extension(source.display().to_string()).as_deref() {
                | Some("md") | Some("markdown") | Some("txt") | Some("json") | Some("yml") | Some("yaml") => Self::read_text(source),
                | _ => Err(eyre!("Unsupported document file extension")),
            },
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn read_markdown(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        Self::read_text(path)
    }
    fn serialize_as(&self, mime: &MimeType) -> ApiResult<String> {
        match mime {
            | MimeType::Json | MimeType::Markdown | MimeType::Text | MimeType::Yaml => Ok(self.content.clone()),
            | _ => Err(eyre!("Unsupported document workflow serialization mime type: {mime:?}")),
        }
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match file_extension(output.display().to_string()).as_deref() {
            | Some("md") | Some("markdown") | Some("txt") | Some("json") | Some("yml") | Some("yaml") => self.write_text(output),
            | Some("docx") => Err(eyre!("Document writing is not implemented for DOCX")),
            | _ => Err(eyre!("Unsupported document file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_text(path)
    }
}
impl text::Document {
    fn read_document(path: impl Into<PathBuf>) -> ApiResult<Self> {
        SourceDocument::at(path).extract().map(|content| Self { content })
    }
    fn read_text(path: impl Into<PathBuf>) -> ApiResult<Self> {
        read_file(path.into()).map(|content| Self { content })
    }
    fn write_text(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_file(path, self.content().to_string())
    }
}
