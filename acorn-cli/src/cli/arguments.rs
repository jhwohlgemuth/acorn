use super::parse::{parse_check_category, parse_executor, parse_readability, parse_standard};
use super::DownloadCommands;
use acorn::analyzer::{self, Standard};
use acorn::io::Executor;
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
pub struct Check {
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
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
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
pub(crate) struct Doctor {
    /// Auto-correct systems issues where possible
    #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) fix: bool,
    /// Start interactive TUI to selectively apply fixes
    #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) interactive: bool,
    /// Choose to output a JSON issue report to stdout
    #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) report: bool,
    /// Select one or more available diagnostic checks
    #[arg(default_value = "all", short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) check: Vec<Diagnostic>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Download {
    #[command(subcommand)]
    pub(crate) command: Option<DownloadCommands>,
    /// Bucket repository URL(s)
    #[arg(value_name = "URL", required = false, value_delimiter = ',', value_hint = ValueHint::Url, help_heading = "ARGS")]
    pub(crate) url: Vec<String>,
    /// Path to configuration file (alternative to URL argument)
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, conflicts_with = "url", help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Regular expression pattern(s) used to include files while downloading
    ///
    /// Only files matching at least one pattern will be processed
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) filter: Vec<String>,
    /// Regular expression pattern(s) to ignore while downloading
    ///
    /// Values augment built-in ignores and can be provided as a comma-separated list
    #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) ignore: Vec<String>,
    /// Path to output directory where the downloaded content should be saved
    #[arg(default_value = "./content", short, long, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) output: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct DownloadModel {
    /// Model ID/name(s) from models.dev, or direct HTTP(S) model weight URL(s)
    #[arg(value_name = "MODEL", required = false, value_delimiter = ',', value_hint = ValueHint::Url, help_heading = "ARGS")]
    pub(crate) model: Vec<String>,
    /// Regular expression pattern(s) used to include model weight sources while downloading
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) filter: Vec<String>,
    /// Regular expression pattern(s) to ignore model weight sources while downloading
    #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) ignore: Vec<String>,
    /// Path to output directory where downloaded model weights should be saved
    #[arg(default_value = "./models", short, long, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) output: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Export {
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
    /// Skip one or more available export check categories
    #[arg(short = 'S', long, value_name = "LIST", value_delimiter = ',', value_parser = parse_check_category, help_heading = "OPTIONS")]
    pub(crate) skip: Vec<analyzer::CheckCategory>,
    /// Path to reference file to be used when exporting to PowerPoint
    ///
    /// Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) reference: Option<PathBuf>,
    /// Metadata standard to validate against
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
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
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Format {
    /// Path to look for files to format
    #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
    pub(crate) path: Option<PathBuf>,
    /// Format files that were changed in a given Git branch
    #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
    pub(crate) branch: Option<String>,
    /// Format files that were changed in a given Git commit
    #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
    pub(crate) commit: Option<String>,
    /// Run format without making changes to target file(s). Will print a diff of changes.
    #[arg(short, long = "dry-run", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Regex pattern applied to absolute paths of files to include in formatting process
    ///
    /// Only files matching the pattern will be processed
    #[arg(long, value_name = "REGEX", help_heading = "OPTIONS")]
    pub(crate) filter: Option<String>,
    /// Regex pattern applied to absolute paths of files that determines whether they should be included in formatting process
    ///
    /// Only applies to path values that point to a directory
    ///
    /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
    ///
    /// Example: --ignore "[/]valid.json$"
    #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
    pub(crate) ignore: Option<String>,
    /// Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
    #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) merge_request: bool,
    /// Metadata standard to validate against
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Gather {
    /// Path to look for files to process
    #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
    pub(crate) path: Option<PathBuf>,
    /// Regex pattern applied to absolute paths of files to include in processing
    ///
    /// Only files matching the pattern will be processed
    #[arg(long, value_name = "REGEX", help_heading = "OPTIONS")]
    pub(crate) filter: Option<String>,
    /// Regex pattern applied to absolute paths of files that determines whether they should be included in processing
    ///
    /// Only applies to path values that point to a directory
    ///
    /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
    ///
    /// Example: --ignore "[/]valid.json$"
    #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
    pub(crate) ignore: Option<String>,
    /// Process files that were changed in a given merge request (GitLab) or pull request (GitHub)
    #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) merge_request: bool,
    /// Metadata standard to validate against
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Link {
    /// Path to look for files to process
    #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
    pub(crate) path: Option<PathBuf>,
    /// Process files that were changed in a given Git branch
    #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
    pub(crate) branch: Option<String>,
    /// Process files that were changed in a given Git commit
    #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
    pub(crate) commit: Option<String>,
    /// Run link without making changes to target file(s). Will print a diff of changes.
    #[arg(short, long = "dry-run", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Regex pattern applied to absolute paths of files that determines whether they should be included in processing
    ///
    /// Only applies to path values that point to a directory
    ///
    /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
    ///
    /// Example: --ignore "[/]valid.json$"
    #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
    pub(crate) ignore: Option<String>,
    /// Processes files that were changed in a given merge request (Gitlab) or pull request (GitHub)
    #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) merge_request: bool,
    /// Metadata standard to validate against
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Spec {
    /// API specification source path or HTTP(S) URL
    #[arg(default_value = "openapi.yaml", value_name = "SOURCE", value_hint = ValueHint::AnyPath, help_heading = "ARGS")]
    pub(crate) source: String,
    /// Endpoint template name to place on the generated object
    #[arg(long, value_name = "NAME", help_heading = "OPTIONS")]
    pub(crate) name: Option<String>,
    /// API domain for the generated endpoint object
    #[arg(long, value_name = "DOMAIN", help_heading = "OPTIONS")]
    pub(crate) domain: Option<String>,
    /// Optional API root path for the generated endpoint object
    #[arg(long, value_name = "ROOT", help_heading = "OPTIONS")]
    pub(crate) root: Option<String>,
    /// Optional bearer token value for the generated endpoint object
    #[arg(long = "auth-token", value_name = "TOKEN", help_heading = "OPTIONS")]
    pub(crate) auth_token: Option<String>,
    /// Write generated endpoint JSON to this file
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) output: Option<PathBuf>,
    /// Print generated endpoint JSON without writing a file
    #[arg(long = "dry-run", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Runner {
    /// Path to configuration file for creating the resource
    #[arg(short, long, default_value = "./", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Runner description (alternative to providing a configuration file)
    #[arg(short, long, value_name = "DESCRIPTION", help_heading = "OPTIONS")]
    pub(crate) description: Option<String>,
    /// GitLab runner executor type
    #[arg(short, long, value_name = "TYPE", value_parser = parse_executor, help_heading = "OPTIONS")]
    pub(crate) executor: Option<Executor>,
    /// GitLab group ID to create a runner for (alternative to providing a configuration file)
    #[arg(short, long, conflicts_with = "project", value_name = "ID", help_heading = "OPTIONS")]
    pub(crate) group: Option<u64>,
    /// GitLab repository domain override for runner creation (e.g., code.ornl.gov)
    #[arg(short = 'r', long, alias = "domain", value_name = "REPO", help_heading = "OPTIONS")]
    pub(crate) repo: Option<String>,
    /// GitLab project ID to create a runner for (alternative to providing a configuration file)
    #[arg(short, long, conflicts_with = "group", value_name = "ID", help_heading = "OPTIONS")]
    pub(crate) project: Option<u64>,
    /// Runner name (used as Docker container name)
    #[arg(short = 'n', long, value_name = "NAME", help_heading = "OPTIONS")]
    pub(crate) name: Option<String>,
    /// Hosting server location (for use during registration)
    #[arg(short, long, value_name = "SERVER", help_heading = "OPTIONS")]
    pub(crate) server: Option<String>,
    /// Tags to register the runner with (alternative to providing a configuration file)
    #[arg(
        short,
        long,
        conflicts_with = "untagged",
        value_name = "LIST",
        value_delimiter = ',',
        help_heading = "OPTIONS"
    )]
    pub(crate) tags: Vec<String>,
    /// Whether this runner can pick up jobs without matching tags (alternative to providing a configuration file)
    #[arg(short, long, conflicts_with = "tags", help_heading = "FLAGS")]
    pub(crate) untagged: bool,
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
