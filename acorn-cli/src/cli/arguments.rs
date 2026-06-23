use acorn::analyzer;
use acorn::io::{ApiResult, Executor};
use acorn::prelude::Path;
use acorn::util::MimeType;
use clap::ValueEnum;
use color_eyre::eyre::eyre;
use derive_more::Display;
use serde_json::Value;

/// Database backend used for local cache and history tables.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, ValueEnum)]
pub enum DatabaseBackend {
    /// SQLite backend
    #[display("sqlite")]
    Sqlite,
    /// DuckDB backend
    #[display("duckdb")]
    Duckdb,
}
/// Categories available when analyzing ("checking") research activity data
#[derive(Clone, Debug, Display, PartialEq, PartialOrd, ValueEnum)]
pub enum CheckCategoryArgument {
    #[display("link")]
    Link,
    #[display("prose")]
    Prose,
    #[display("quality")]
    Quality,
    #[display("readability")]
    Readability,
    #[display("schema")]
    Schema,
}
/// Categories available when performing system diagnostics before using ACORN
#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum Diagnostic {
    #[default]
    All,
    System,
    Memory,
    Network,
    Gpu,
    Software,
}
/// Executor types available when creating GitLab runners
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, ValueEnum)]
pub enum ExecutorArgument {
    #[default]
    #[display("docker")]
    Docker,
    #[display("apptainer")]
    Apptainer,
    #[display("kubernetes")]
    Kubernetes,
    #[display("podman")]
    Podman,
    #[display("sandbox")]
    Sandbox,
    #[display("shell")]
    Shell,
    #[display("ssh")]
    Ssh,
    #[display("virtual-machine")]
    VirtualMachine,
    #[display("other")]
    Other,
}
/// Target export file formats available when exporting research activity data
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum FileFormat {
    #[default]
    #[display("PDF")]
    Pdf,
    #[display("BagIt")]
    Bag,
    #[display("CFF")]
    Cff,
    #[display("JSON")]
    Json,
    #[display("MD")]
    Markdown,
    #[display("PPTX")]
    Powerpoint,
    #[display("YAML")]
    Yaml,
}
/// Readability Type
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum ReadabilityTypeArgument {
    /// Flesch-Kincaid Grade Level (FKGL)
    #[default]
    #[display("fkgl")]
    Fkgl,
    /// Automated Readability Index (ARI)
    #[display("ari")]
    Ari,
    /// Coleman-Liau Index (CLI)
    #[display("cli")]
    Cli,
    /// Flesch Reading Ease (FRES)
    #[display("fres")]
    Fres,
    /// Gunning Fog Index (GFI)
    #[display("gfi")]
    Gfi,
    /// Lix (abbreviation of Swedish läsbarhetsindex)
    #[display("lix")]
    Lix,
    /// SMOG Index (SMOG)
    #[display("smog")]
    Smog,
}
/// Target artifact aspect ratio size available when exporting research activity data using acorn
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum Size {
    /// Widescreen size (16:9)
    #[default]
    #[display("widescreen")]
    Widescreen,
    /// Standard size (4:3)
    #[display("standard")]
    Standard,
}
/// Metadata standard to use when processing research activity data
#[derive(Clone, Copy, Debug, Default, Display, ValueEnum)]
pub enum Standard {
    /// Research Activity Data Schema (RADS)
    #[default]
    #[display("rads")]
    Rads,
    /// Citation file format (CFF)
    #[display("cff")]
    Cff,
    /// DataCite
    #[display("datacite")]
    Datacite,
    /// Data Catalog (DCAT)
    #[display("dcat")]
    Dcat,
    /// Dublin Core Metadata Initiative (DCMI)
    #[display("dcmi")]
    Dcmi,
    /// DOCX-derived text
    #[display("docx")]
    Docx,
    /// InvenioRDM
    #[display("invenio")]
    Invenio,
    /// Huwise
    #[display("huwise")]
    Huwise,
    /// Research Activity Identifier metadata (RAiD)
    #[display("raid")]
    Raid,
    /// Plain text (no schema structure)
    #[display("text")]
    Text,
}
/// Target artifact types available when exporting research activity data using acorn
///
/// Used primarily by ACORN CLI
#[derive(Clone, Copy, Debug, Default, Display, ValueEnum)]
pub enum Target {
    /// US letter sized single page PDF document presenting a certain research activity data
    #[default]
    #[display("fact-sheet")]
    FactSheet,
    /// Single slide PowerPoint presentation for a certain research activity data
    #[display("highlight")]
    Highlight,
    /// Poster sized presentation format intended for large printing and presentation
    #[display("poster")]
    Poster,
}
impl From<analyzer::CheckCategory> for CheckCategoryArgument {
    fn from(value: analyzer::CheckCategory) -> Self {
        match value {
            | analyzer::CheckCategory::Link => CheckCategoryArgument::Link,
            | analyzer::CheckCategory::Prose => CheckCategoryArgument::Prose,
            | analyzer::CheckCategory::Quality => CheckCategoryArgument::Quality,
            | analyzer::CheckCategory::Readability => CheckCategoryArgument::Readability,
            | analyzer::CheckCategory::Schema => CheckCategoryArgument::Schema,
        }
    }
}
impl From<Executor> for ExecutorArgument {
    fn from(value: Executor) -> Self {
        match value {
            | Executor::Apptainer => ExecutorArgument::Apptainer,
            | Executor::Docker => ExecutorArgument::Docker,
            | Executor::Podman => ExecutorArgument::Podman,
            | Executor::Sandbox => ExecutorArgument::Sandbox,
            | Executor::Shell => ExecutorArgument::Shell,
            | Executor::Ssh => ExecutorArgument::Ssh,
            | Executor::Kubernetes => ExecutorArgument::Kubernetes,
            | Executor::VirtualMachine => ExecutorArgument::VirtualMachine,
            | Executor::Other(_) => ExecutorArgument::Other,
        }
    }
}
impl FileFormat {
    pub fn is_structured(&self) -> bool {
        match self {
            | FileFormat::Cff | FileFormat::Json | FileFormat::Yaml => true,
            | _ => false,
        }
    }
}
impl From<FileFormat> for MimeType {
    fn from(format: FileFormat) -> Self {
        match format {
            | FileFormat::Pdf => MimeType::Pdf,
            | FileFormat::Bag => MimeType::Zip,
            | FileFormat::Cff => MimeType::Cff,
            | FileFormat::Json => MimeType::Json,
            | FileFormat::Markdown => MimeType::Markdown,
            | FileFormat::Powerpoint => MimeType::Powerpoint,
            | FileFormat::Yaml => MimeType::Yaml,
        }
    }
}
impl From<FileFormat> for Standard {
    fn from(format: FileFormat) -> Self {
        match format {
            | FileFormat::Cff => Standard::Cff,
            | FileFormat::Bag | FileFormat::Json | FileFormat::Markdown | FileFormat::Pdf | FileFormat::Powerpoint | FileFormat::Yaml => {
                unimplemented!("Cannot infer standard for {format} file")
            }
        }
    }
}
impl From<Standard> for analyzer::Standard {
    fn from(value: Standard) -> Self {
        match value {
            | Standard::Rads => analyzer::Standard::ResearchActivityData,
            | Standard::Cff => analyzer::Standard::CitationFileFormat,
            | Standard::Datacite => analyzer::Standard::Datacite,
            | Standard::Dcat => analyzer::Standard::Dcat,
            | Standard::Dcmi => analyzer::Standard::DublinCore,
            | Standard::Docx => analyzer::Standard::Docx,
            | Standard::Invenio => analyzer::Standard::Invenio,
            | Standard::Huwise => analyzer::Standard::Huwise,
            | Standard::Raid => analyzer::Standard::Raid,
            | Standard::Text => analyzer::Standard::Text,
        }
    }
}
impl From<analyzer::Standard> for Standard {
    fn from(value: analyzer::Standard) -> Self {
        match value {
            | analyzer::Standard::ResearchActivityData => Standard::Rads,
            | analyzer::Standard::CitationFileFormat => Standard::Cff,
            | analyzer::Standard::Datacite => Standard::Datacite,
            | analyzer::Standard::Dcat => Standard::Dcat,
            | analyzer::Standard::DublinCore => Standard::Dcmi,
            | analyzer::Standard::Docx => Standard::Docx,
            | analyzer::Standard::Invenio => Standard::Invenio,
            | analyzer::Standard::Huwise => Standard::Huwise,
            | analyzer::Standard::Raid => Standard::Raid,
            | analyzer::Standard::Text => Standard::Text,
        }
    }
}
impl Standard {
    pub(crate) fn _crosswalk(&self, target: Standard, _content: &str, _mime: MimeType, _format: &FileFormat) -> ApiResult<String> {
        match self {
            | Standard::Datacite => self.unsupported(target),
            | Standard::Dcat => self.unsupported(target),
            | Standard::Invenio => self.unsupported(target),
            | Standard::Huwise => self.unsupported(target),
            | _ => self.unsupported(target),
        }
    }
    pub(crate) fn infer(path: &Path, content: &str, mime: &MimeType) -> ApiResult<Standard> {
        let root = match mime {
            | MimeType::Json => {
                serde_json::from_str(content).map_err(|why| eyre!("Unable to infer source schema for {} — invalid JSON ({why})", path.display()))
            }
            | MimeType::Yaml => {
                serde_norway::from_str(content).map_err(|why| eyre!("Unable to infer source schema for {} — invalid YAML ({why})", path.display()))
            }
            | _ => Err(eyre!("Unable to infer source schema for {} — unsupported file type", path.display())),
        };
        match root {
            | Ok(Value::Object(map)) => Ok(analyzer::Standard::from(&map).into()),
            | Ok(Value::Array(values)) => match values.into_iter().next() {
                | Some(Value::Object(map)) => Ok(analyzer::Standard::from(&map).into()),
                | Some(_) => Err(eyre!(
                    "Unable to infer source schema for {} — array items must be objects",
                    path.display()
                )),
                | None => Err(eyre!("Unable to infer source schema for {} — empty array", path.display())),
            },
            | Ok(_) => Err(eyre!(
                "Unable to infer source schema for {} — expected object or array of objects",
                path.display()
            )),
            | Err(why) => Err(why),
        }
    }
    pub(crate) fn unsupported(&self, target: Standard) -> ApiResult<String> {
        Err(eyre!(
            "Unsupported metadata crosswalk pair: {self} -> {target} (supported: datacite, dcat, invenio, huwise)"
        ))
    }
}
