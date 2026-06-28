use super::parse::{parse_check_category, parse_readability, parse_standard};
use acorn::analyzer::{self, Standard};
use acorn::prelude::PathBuf;
use acorn::util::MimeType;
use clap::ValueEnum;
use clap::{Args, ValueHint};
use clap_verbosity_flag::Verbosity;
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
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub struct CheckArguments {
    /// Path to check
    #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
    pub(crate) path: Option<PathBuf>,
    /// Check files that were changed in a given Git branch
    #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
    pub(crate) branch: Option<String>,
    /// Check files that were changed in a given Git commit or commit range
    ///
    ///     Check files that were changed in the last 10 commits
    ///     &> acorn check --commit HEAD~10..HEAD
    ///
    /// See <https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection> for more information of commit ranges
    #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
    pub(crate) commit: Option<String>,
    /// Disable website checks that require internet connection
    #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) disable_website_checks: bool,
    /// Include all check results, including suggestions and info messages
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) all: bool,
    /// Exit on first error
    #[arg(short, long = "exit-on-first-error", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) exit_on_first_error: bool,
    /// Regular expression pattern(s) applied to absolute paths of files to include during checks
    ///
    /// Only files matching at least one pattern will be processed
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) filter: Vec<String>,
    /// Regular expression pattern(s) applied to absolute paths of files to exclude from checking
    ///
    /// Only applies to `--path` values that point to a directory
    ///
    /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
    ///
    /// Example: --ignore "[/]valid.json$,[/]draft.json$"
    #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) ignore: Vec<String>,
    /// Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
    #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) merge_request: bool,
    /// Prevent `acorn check` from exiting with a non-zero exit code when errors are found
    ///
    /// Useful for certain scenarios such as debugging
    ///
    /// > CAUTION: Use at your own risk
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) no_fail: bool,
    /// Suppress output from external commands (e.g., Vale) while preserving ACORN output — useful for piping
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) raw: bool,
    /// Skip one or more available quality check categories
    #[arg(short = 'S', long, value_name = "LIST", value_delimiter = ',', value_parser = parse_check_category, help_heading = "OPTIONS")]
    pub(crate) skip: Vec<analyzer::CheckCategory>,
    /// Skip checksum verification when downloading files (e.g., downloading Vale to perform static analysis of prose)
    /// > CAUTION: Use at your own risk
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) skip_verify_checksum: bool,
    /// Metadata standard to validate against
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    /// Readability test to use
    #[arg(default_value = "fkgl", short, long = "readability-test", value_name = "METRIC", value_parser = parse_readability, help_heading = "OPTIONS")]
    pub(crate) readability_metric: acorn::analyzer::readability::ReadabilityType,
    /// Use compact output format instead of detailed output
    #[arg(short = 't', long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) terse: bool,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ExportArguments {
    /// Path of input files to be exported
    #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
    pub(crate) path: Option<PathBuf>,
    /// Export files that were changed in a given Git branch
    #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
    pub(crate) branch: Option<String>,
    /// Combine all export artifacts into a single artifact
    #[arg(long = "combine", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) combine: bool,
    /// Export files that were changed in a given Git commit
    #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
    pub(crate) commit: Option<String>,
    /// Run export without making changes. Will print what would happen.
    #[arg(long = "dry-run", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Export target file format
    #[arg(default_value = "pdf", short, long, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) format: FileFormat,
    /// Regular expression pattern(s) applied to absolute paths of files to include during export
    ///
    /// Only files matching at least one pattern will be processed
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) filter: Vec<String>,
    /// Regular expression pattern(s) applied to absolute paths of files to exclude from export process
    ///
    /// Only applies to `--path` values that point to a directory
    ///
    /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
    ///
    /// Example: --ignore "[/]valid.json$,[/]draft.json$"
    #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) ignore: Vec<String>,
    /// Export files that were changed in a given merge request (Gitlab) or pull request (GitHub)
    #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) merge_request: bool,
    /// Path to output directory where the export artifacts should be saved
    #[arg(default_value = "./export", short, long, value_name = "DIRECTORY", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) output: Option<PathBuf>,
    /// Suppress output from external commands while preserving ACORN output — useful for piping
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) raw: bool,
    /// Path to reference file to be used when exporting to PowerPoint
    ///
    /// Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) reference: Option<PathBuf>,
    /// Metadata standard to validate against
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    /// Fail if conversion loses data (only for crosswalk exports)
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) strict: bool,
    /// Metadata standard to use for source data inference (default: auto-detect)
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
    #[arg(long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) from: Option<Standard>,
    /// Target metadata standard for crosswalk conversion
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
    #[arg(long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) to: Option<Standard>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
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
        MimeType::from(&format)
    }
}
impl From<&FileFormat> for MimeType {
    fn from(format: &FileFormat) -> Self {
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
