//! Schema I/O adapter
//!
//! Hosts std-only path/file behavior for schema types so that [`acorn_schema`]
//! modules stay portable. This module later moves into `acorn-host`.
pub mod migrate;
use self::migrate::Migration;
use crate::{
    error::ApiResult,
    io::{
        database::{
            schema::{ModelRow, Table},
            Database, Operations, Row,
        },
        document::{DocumentFormat, SourceDocument},
        image_paths, jsonc_parse_value,
        oci::{Manifest, OciPlan},
        parent, parse_jsonc_cst, read_file, replace_file, write_file, FromPath, InputOutput, PathConversion,
    },
    util::{constants::app::APPLICATION, StringConversion},
};
use acorn_core::{
    prelude::{vec, Vec},
    util::{file_extension, LinkedData, MarkdownSupport, MimeType, Unstructured},
};
use acorn_host::{fs::SafePath, terminal::Label};
use acorn_schema::standard::zon;
use acorn_schema::{
    agent::{opencode, Weights},
    modelkit::kitfile::{Kitfile, LayerInfo, McpServer},
    pid::raid::Metadata,
    research_activity::{Logbook, LogbookEntry, ResearchActivity, ResearchActivityMetadata},
    standard::{cff::Cff, ckan, datacite, dcat, huwise, invenio, text, text::Text},
    ImageObject, MediaObject, OneOrMany,
};
use color_eyre::eyre::eyre;
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Host-side batch input and output operations for schema types.
pub trait Batch {
    /// Read one or many research activity records from a supported path.
    fn batch_read(_path: impl AsRef<Path>) -> ApiResult<OneOrMany<ResearchActivity>> {
        Err(eyre!("Batch reading is not implemented for this type"))
    }
    /// Write one or many research activity records using the target path format.
    fn batch_write(&self, _path: impl Into<PathBuf>) -> ApiResult<()> {
        Err(eyre!("Batch writing is not implemented for this type"))
    }
}
/// Host-side filesystem operations for authoring Kitfiles.
pub trait KitfileExt {
    /// Resolve the model content declared by a Kitfile against an OCI manifest.
    fn modelkit_content(&self, manifest: &Manifest) -> ApiResult<OciPlan>;
    /// Add or replace the ACORN MCP bundle entry and persist the Kitfile atomically.
    fn update(self, path: impl AsRef<Path>, bundle: impl AsRef<Path>) -> ApiResult<()>;
}
/// Native persistence operations for schema-owned model weights.
pub trait WeightsExt {
    /// Refresh file-level weights for a model while preserving repository-level sources.
    fn persist(self, model_id: &str, database_path: Option<PathBuf>) -> ApiResult<()>;
}
macro_rules! impl_logbook_input_output {
    ($type:ty, $label:literal) => {
        impl InputOutput for $type {
            fn parse(content: &str) -> Option<Self> {
                serde_json::from_str(content)
                    .ok()
                    .or_else(|| serde_norway::from_str(content).ok())
                    .or_else(|| Self::from_markdown(content).ok())
            }
            fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
                let source = path.into();
                match MimeType::from_path(&source) {
                    | MimeType::Json | MimeType::Jsonc => Self::read_json(source),
                    | MimeType::Markdown => Self::read_markdown(source),
                    | MimeType::Yaml => Self::read_yaml(source),
                    | _ => Err(eyre!("Unsupported {} file extension", $label)),
                }
            }
            fn read_json(path: PathBuf) -> ApiResult<Self> {
                read_file(path).and_then(|content| serde_json::from_str(&content).map_err(|why| eyre!("Failed to parse JSON {} — {why}", $label)))
            }
            fn read_markdown(path: PathBuf) -> ApiResult<Self> {
                read_file(path).and_then(|content| Self::from_markdown(&content).map_err(|why| eyre!("Failed to parse Markdown {} — {why}", $label)))
            }
            fn read_yaml(path: PathBuf) -> ApiResult<Self> {
                read_file(path).and_then(|content| serde_norway::from_str(&content).map_err(|why| eyre!("Failed to parse YAML {} — {why}", $label)))
            }
            fn serialize_as(&self, mime: &MimeType) -> ApiResult<String> {
                match mime {
                    | MimeType::Json | MimeType::Jsonc => {
                        serde_json::to_string_pretty(self).map_err(|why| eyre!("Failed to serialize JSON {} — {why}", $label))
                    }
                    | MimeType::Markdown => self
                        .to_front_matter()
                        .map(|frontmatter| format!("---\n{frontmatter}---\n{}", self.to_markdown()))
                        .map_err(|why| eyre!("Failed to serialize Markdown {} — {why}", $label)),
                    | MimeType::Yaml => serde_norway::to_string(self).map_err(|why| eyre!("Failed to serialize YAML {} — {why}", $label)),
                    | _ => Err(eyre!("Unsupported {} serialization mime type: {mime:?}", $label)),
                }
            }
            fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
                let output = path.into();
                match MimeType::from_path(&output) {
                    | MimeType::Json | MimeType::Jsonc => self.write_json(output),
                    | MimeType::Markdown => self.write_markdown(output),
                    | MimeType::Yaml => self.write_yaml(output),
                    | _ => Err(eyre!("Unsupported {} file extension for writing", $label)),
                }
            }
            fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
                serde_json::to_string_pretty(self)
                    .map_err(|why| eyre!("Failed to serialize JSON {} — {why}", $label))
                    .and_then(|content| write_file(path.into(), content))
            }
            fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
                self.serialize_as(&MimeType::Markdown)
                    .and_then(|content| write_file(path.into(), content))
            }
            fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
                serde_norway::to_string(self)
                    .map_err(|why| eyre!("Failed to serialize YAML {} — {why}", $label))
                    .and_then(|content| write_file(path.into(), content))
            }
        }
    };
}
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
impl InputOutput for ckan::Package {
    fn parse(content: &str) -> Option<Self> {
        ckan::Package::parse(content, MimeType::Json).ok().and_then(|packages| match packages {
            | OneOrMany::One(package) => Some(package),
            | OneOrMany::Many(mut packages) if packages.len() == 1 => packages.pop(),
            | OneOrMany::Many(_) => None,
        })
    }
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source),
            | MimeType::Yaml => Self::read_yaml(source),
            | _ => Err(eyre!("Unsupported CKAN package file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_ckan_package(path, MimeType::Json)
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_ckan_package(path, MimeType::Yaml)
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from_path(&output) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported CKAN package file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON CKAN package — {why}"))
            .and_then(|content| write_file(path.into().with_extension("json"), content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML CKAN package — {why}"))
            .and_then(|content| write_file(path.into().with_extension("yaml"), content))
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
fn read_ckan_package(path: PathBuf, mime: MimeType) -> ApiResult<ckan::Package> {
    read_file(path).and_then(|content| {
        ckan::Package::parse(&content, mime)
            .map_err(|why| eyre!("Failed to parse CKAN package — {why}"))
            .and_then(|packages| match packages {
                | OneOrMany::One(package) => Ok(package),
                | OneOrMany::Many(mut packages) if packages.len() == 1 => {
                    packages.pop().ok_or_else(|| eyre!("Expected one CKAN package but found none"))
                }
                | OneOrMany::Many(packages) => Err(eyre!("Expected one CKAN package but found {}", packages.len())),
            })
    })
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
impl InputOutput for Kitfile {
    fn parse(content: &str) -> Option<Self> {
        Self::from_yaml(content).ok()
    }
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let path = path.into();
        match MimeType::from_path(&path) {
            | MimeType::Json => Self::read_json(path),
            | MimeType::Yaml => Self::read_yaml(path),
            | _ => Err(eyre!("Unsupported Kitfile extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| Self::from_resolved_json(content.as_bytes()).map_err(|why| eyre!(why)))
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| Self::from_yaml(&content).map_err(|why| eyre!(why)))
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let path = path.into();
        match MimeType::from_path(&path) {
            | MimeType::Json => self.write_json(path),
            | MimeType::Yaml => self.write_yaml(path),
            | _ => Err(eyre!("Unsupported Kitfile extension")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.to_resolved_json()
            .map_err(|why| eyre!(why))
            .and_then(|content| write_file(path, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.to_yaml().map_err(|why| eyre!(why)).and_then(|content| write_file(path, content))
    }
}
impl KitfileExt for Kitfile {
    fn modelkit_content(&self, manifest: &Manifest) -> ApiResult<OciPlan> {
        OciPlan::from_kitfile(self, manifest)
    }
    fn update(mut self, path: impl AsRef<Path>, bundle: impl AsRef<Path>) -> ApiResult<()> {
        let path = path.as_ref();
        let directory = path.parent().unwrap_or_else(|| Path::new("."));
        directory
            .canonicalize()
            .map_err(|why| eyre!("Failed to resolve Kitfile directory — {why}"))
            .and_then(|directory| {
                bundle
                    .as_ref()
                    .canonicalize()
                    .map_err(|why| eyre!("Failed to resolve MCPB output — {why}"))
                    .map(|bundle| (directory, bundle))
            })
            .and_then(|(directory, bundle)| {
                SafePath::new(bundle.to_relative(&directory)).map_err(Into::into).and_then(|relative| {
                    let entry = McpServer {
                        name: APPLICATION.to_string(),
                        path: relative.as_path().to_string_lossy().replace('\\', "/"),
                        description: "ACORN Model Context Protocol server".to_string(),
                        layer: LayerInfo::default(),
                    };
                    match self.mcp_servers.iter_mut().find(|candidate| candidate.name == APPLICATION) {
                        | Some(candidate) => *candidate = entry,
                        | None => self.mcp_servers.push(entry),
                    }
                    self.mcp_servers.sort_by(|left, right| left.name.cmp(&right.name));
                    self.to_yaml().map_err(|why| eyre!(why))
                })
            })
            .and_then(|content| replace_file(path, content))
    }
}
impl_logbook_input_output!(Logbook, "logbook");
impl_logbook_input_output!(LogbookEntry, "logbook entry");
impl InputOutput for Metadata {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let path = path.into();
        match MimeType::from_path(&path) {
            | MimeType::Json | MimeType::Jsonc => Self::read_json(path),
            | MimeType::Yaml => Self::read_yaml(path),
            | _ => Err(eyre!("Unsupported RAiD metadata file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| {
            eserde::json::from_str::<Self>(&content).map_err(|errors| {
                eyre!(errors
                    .iter()
                    .map(|error| format!("{}: {}", error.path().map_or("root".into(), |path| path.to_string()), error.message()))
                    .collect::<Vec<_>>()
                    .join("\n"))
            })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| serde_norway::from_str(&content).map_err(|why| eyre!(why)))
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        self.write_json(path)
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!(why))
            .and_then(|content| write_file(path, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        serde_norway::to_string(self)
            .map_err(|why| eyre!(why))
            .and_then(|content| write_file(path, content))
    }
}
impl Batch for OneOrMany<ResearchActivity> {
    fn batch_write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        self.serialize(MimeType::from_path(&output))
            .map_err(|why| eyre!("Serialize research activity data — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl InputOutput for opencode::Config {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source.clone()),
            | MimeType::Jsonc => Self::read_jsonc(source.clone()),
            | _ => Err(eyre!("Unsupported OpenCode configuration file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_file(path.clone())
            .and_then(|content| serde_json::from_str(&content).map_err(|why| eyre!("Failed to read JSON config `{}` — {}", path.display(), why)))
    }
    fn read_jsonc(path: PathBuf) -> ApiResult<Self> {
        read_file(path.clone()).and_then(|content| {
            parse_jsonc_cst::<Self>(&content)
                .map(|(config, _)| config)
                .map_err(|why| eyre!("Failed to read JSONC config `{}` — {}", path.display(), why))
        })
    }
    fn read_yaml(_path: PathBuf) -> ApiResult<Self> {
        Err(eyre!("YAML format is not supported for OpenCode configuration"))
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        match MimeType::from_path(&target) {
            | MimeType::Json | MimeType::Jsonc => self.write_json(&target),
            | _ => Err(eyre!("Unsupported OpenCode configuration file extension")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize OpenCode config — {why}"))
            .and_then(|content| write_file(path.into(), content))
    }
    fn write_yaml(&self, _path: impl Into<PathBuf>) -> ApiResult<()> {
        Err(eyre!("YAML format is not supported for OpenCode configuration"))
    }
}
impl Batch for ResearchActivity {
    fn batch_read(path: impl AsRef<Path>) -> ApiResult<OneOrMany<ResearchActivity>> {
        let source = path.as_ref();
        let mime = MimeType::from_path(source);
        match mime {
            | MimeType::Json | MimeType::JsonLines | MimeType::Yaml | MimeType::Zon => read_file(source)
                .map_err(|why| eyre!("Read research activity data — {why}"))
                .and_then(|content| OneOrMany::parse(&content, mime).map_err(|why| eyre!("Read research activity data — {why}"))),
            | _ => ResearchActivity::read(source)
                .map(OneOrMany::One)
                .map_err(|why| eyre!("Read research activity data — {why}")),
        }
    }
}
impl InputOutput for ResearchActivity {
    fn parse(content: &str) -> Option<Self> {
        jsonc_parse_value(content)
            .ok()
            .and_then(|value| serde_json::from_value(value).ok())
            .or_else(|| serde_norway::from_str(content).ok())
            .or_else(|| zon::decode(content).ok())
    }
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into().clone();
        let result = match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source.clone()),
            | MimeType::Jsonc => Self::read_jsonc(source.clone()),
            | MimeType::Markdown => Self::read_markdown(source.clone()),
            | MimeType::Yaml => Self::read_yaml(source.clone()),
            | MimeType::Zon => Self::read_zon(source.clone()),
            | _ => Err(eyre!("Unsupported research activity data file extension")),
        };
        if let Ok(data) = &result {
            debug!(path = source.to_string_lossy().to_string(), "=> {}", Label::using());
            debug!("=> {} Research activity data = {:#?}", Label::using(), data.dimmed().cyan());
        }
        result
    }
    fn format_with(self, context: Option<PathBuf>) -> Self {
        let ResearchActivity { meta, contact, .. } = self.clone();
        ResearchActivity {
            meta: format_metadata(meta, context.clone()),
            contact: contact.format(),
            ..self
        }
        .format()
    }
    fn serialize_as(&self, mime: &MimeType) -> ApiResult<String> {
        Ok(ResearchActivity::serialize_as(self, mime)?)
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
    fn read_zon(path: PathBuf) -> ApiResult<Self> {
        read_file(path).and_then(|content| {
            zon::decode_value(&content)
                .map_err(|why| eyre!("Failed to parse ZON RAD — {why}"))
                .and_then(|value| {
                    Self::print_migration_warnings(&value);
                    serde_json::to_string(&value)
                        .map_err(|why| eyre!("Failed to convert ZON RAD to JSON — {why}"))
                        .and_then(|json| {
                            eserde::json::from_str::<Self>(&json).map_err(|errors| {
                                let details: Vec<String> = errors
                                    .iter()
                                    .map(|error| format!("{}: {}", error.path().map_or("root".into(), |path| path.to_string()), error.message()))
                                    .collect();
                                eyre!("{}", details.join("\n"))
                            })
                        })
                })
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
            | MimeType::Zon => self.write_zon(output),
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
    fn write_zon(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        zon::encode(self)
            .map_err(|why| eyre!("Failed to serialize ZON RAD — {why}"))
            .and_then(|content| write_file(path.into(), content))
    }
}
impl InputOutput for Text {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Markdown | MimeType::Text => read_text(source),
            | MimeType::Json => Self::read_json(source),
            | MimeType::Yaml => Self::read_yaml(source),
            | _ => Err(eyre!("Unsupported plain text file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_text(path)
    }
    fn read_markdown(path: PathBuf) -> ApiResult<Self> {
        read_text(path)
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_text(path)
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
            | MimeType::Markdown | MimeType::Text => write_text(self, output),
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported plain text file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_text(self, path)
    }
    fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_text(self, path)
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_text(self, path)
    }
}
impl InputOutput for text::Document {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match DocumentFormat::try_from(&MimeType::from(source.display().to_string())) {
            | Ok(_) => read_document(source),
            | Err(_) => match file_extension(source.display().to_string()).as_deref() {
                | Some("md") | Some("markdown") | Some("txt") | Some("json") | Some("yml") | Some("yaml") => read_document_text(source),
                | _ => Err(eyre!("Unsupported document file extension")),
            },
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        read_document_text(path)
    }
    fn read_markdown(path: PathBuf) -> ApiResult<Self> {
        read_document_text(path)
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        read_document_text(path)
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
            | Some("md") | Some("markdown") | Some("txt") | Some("json") | Some("yml") | Some("yaml") => write_document_text(self, output),
            | Some("docx") => Err(eyre!("Document writing is not implemented for DOCX")),
            | _ => Err(eyre!("Unsupported document file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_document_text(self, path)
    }
    fn write_markdown(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_document_text(self, path)
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        write_document_text(self, path)
    }
}
impl WeightsExt for Weights {
    fn persist(self, model_id: &str, database_path: Option<PathBuf>) -> ApiResult<()> {
        ModelRow::init()
            .model_id(model_id.to_string())
            .build()
            .select(database_path.clone(), |row| row.model_id.as_deref() == Some(model_id))
            .and_then(|lookup| match lookup {
                | Some(row) => {
                    let existing = row
                        .weights
                        .as_deref()
                        .and_then(|value| serde_json::from_str::<Weights>(value).ok())
                        .unwrap_or_default();
                    Weights(
                        existing
                            .0
                            .into_iter()
                            .filter(|weight| !(weight.quantization.is_some() && weight.url.contains("/resolve/")))
                            .chain(self.0)
                            .collect(),
                    )
                    .serialize()
                    .map_err(color_eyre::Report::from)
                    .and_then(|weights| {
                        ModelRow {
                            weights: Some(weights),
                            ..row
                        }
                        .update_weights(database_path)
                        .map(|_| ())
                    })
                }
                | None => self.serialize().map_err(color_eyre::Report::from).and_then(|weights| {
                    Database::<Table>::from_path(database_path)
                        .insert(ModelRow::init().model_id(model_id.to_string()).weights(weights).build())
                        .map(|_| ())
                }),
            })
    }
}
fn format_metadata(metadata: ResearchActivityMetadata, context: Option<PathBuf>) -> ResearchActivityMetadata {
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
            let first_graphic = match metadata.media.clone() {
                | Some(values) if !values.is_empty() => {
                    let caption = metadata.clone().first_image_caption();
                    let image_data = ImageObject::init().caption(caption.to_string()).content_url(value.clone()).build();
                    MediaObject::Image(image_data)
                }
                | Some(_) | None => {
                    let image_data = ImageObject::init().caption("".to_string()).content_url(value.clone()).build();
                    MediaObject::Image(image_data)
                }
            };
            let rest = match metadata.clone().media {
                | Some(values) if !values.is_empty() => values.into_iter().skip(1).collect::<Vec<_>>(),
                | Some(_) | None => vec![],
            };
            Some([vec![first_graphic], rest].concat().into())
        }
        | None => metadata.media.clone(),
    };
    ResearchActivityMetadata { media, ..metadata }.format()
}
fn read_document(path: impl Into<PathBuf>) -> ApiResult<text::Document> {
    SourceDocument::at(path).extract().map(|content| text::Document { content })
}
fn read_document_text(path: impl Into<PathBuf>) -> ApiResult<text::Document> {
    read_file(path.into()).map(|content| text::Document { content })
}
fn read_text(path: impl Into<PathBuf>) -> ApiResult<Text> {
    read_file(path.into()).map(|content| Text { content })
}
fn write_document_text(value: &text::Document, path: impl Into<PathBuf>) -> ApiResult<()> {
    write_file(path, value.content().to_string())
}
fn write_text(value: &Text, path: impl Into<PathBuf>) -> ApiResult<()> {
    write_file(path, value.content().to_string())
}

#[cfg(test)]
mod tests;
