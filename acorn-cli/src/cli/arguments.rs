use acorn::analyzer;
use acorn::util::MimeType;
use clap::ValueEnum;
use derive_more::Display;

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
    /// InvenioRDM
    #[display("invenio")]
    Invenio,
    /// Huwise
    #[display("huwise")]
    Huwise,
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
