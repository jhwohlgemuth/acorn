use self::osti::{DoeLab, OrganizationRole as OstiOrganizationRole, SearchView as OstiSearchView};
use super::parse::{
    parse_check_category, parse_check_format, parse_enrichment_provider, parse_exec_format, parse_exec_hook, parse_executor, parse_mcpb_platform,
    parse_readability, parse_ssh_remote, parse_standard, parse_watch_duration,
};
use super::DownloadCommands;
use acorn::analyzer::host::{context, discovery::CitationFormat};
use acorn::io::config::application::Hook;
use acorn::io::swhid::CalculationKind;
use acorn::io::{Executor, Platform, Remote};
use acorn::util::constants::app::{
    DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT, DEFAULT_HUGGINGFACE_SEARCH_LIMIT, DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD, DEFAULT_NEEDLE_MAX_ROUNDS,
};
use acorn::util::constants::env::{CHROME_PATH, MINIMUM_DOWNLOAD_COUNT, SEARCH_LIMIT};
use acorn_analysis::{readability::ReadabilityType, CheckCategory};
use acorn_core::{time::Milliseconds, util::MimeType, validation::ValidationError, Noul};
use acorn_schema::agent::Quantization;
use acorn_schema::enrichment::Provider;
use acorn_schema::hardware::memory::Memory;
use acorn_schema::standard::Standard;
use acorn_schema::validation::{Validate, ValidationReport};
use acorn_schema::vocabulary::VocabularyFuzzyMode;
use bon::Builder;
use clap::ValueEnum;
use clap::{Args, ValueHint};
use clap_verbosity_flag::Verbosity;
use core::net::SocketAddr;
use derive_more::Display;
use jiff::SignedDuration;
use std::path::PathBuf;
use strum::EnumIs;

/// Bot command arguments.
pub(crate) mod bot {
    use super::{parse_executor, DetachArgs, Executor, PathBuf, RemoteTarget, ValueEnum, Verbosity};
    use clap::Args;

    const DEFAULT_HOST: &str = "localhost";
    const DEFAULT_PORT: u16 = 3000;
    /// GitLab event delivery source used by the bot.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, strum::Display, strum::EnumIs, ValueEnum)]
    #[strum(serialize_all = "kebab-case")]
    pub(crate) enum EventSource {
        /// Poll GitLab's Events API.
        #[default]
        Poll,
        /// Receive authenticated GitLab webhooks.
        Webhook,
        /// Receive webhooks and retain polling as best-effort reconciliation.
        Hybrid,
    }
    /// Arguments shared by bot commands
    #[derive(Clone, Debug, Args)]
    pub(crate) struct Common {
        /// GitLab project identifier to poll for note events
        #[arg(value_name = "PROJECT_ID", env = "CI_PROJECT_ID", help_heading = "ARGS")]
        pub(crate) identifier: String,
        /// Address and port for the bot HTTP server to bind to
        #[arg(short, long, value_name = "ADDRESS", help_heading = "OPTIONS")]
        pub(crate) bind: Option<String>,
        /// Port for the bot HTTP server to bind to
        #[arg(short, long, value_name = "PORT", help_heading = "OPTIONS")]
        pub(crate) port: Option<u16>,
        /// Initial ISO 8601 timestamp to start polling from
        #[arg(long, value_name = "TIMESTAMP", help_heading = "OPTIONS")]
        pub(crate) after: Option<String>,
        /// Polling interval in seconds
        #[arg(long, default_value_t = 10, value_name = "SECONDS", help_heading = "OPTIONS")]
        pub(crate) poll_interval: u64,
        /// Container runtime (docker, podman, apptainer)
        #[arg(short = 'R', long, value_name = "RUNTIME", value_parser = parse_executor, help_heading = "OPTIONS")]
        pub(crate) runtime: Option<Executor>,
        /// Event delivery source
        #[arg(long, default_value_t, value_enum, help_heading = "OPTIONS")]
        pub(crate) event_source: EventSource,
        /// Externally reachable HTTPS base URL for webhook delivery
        #[arg(long, value_name = "URL", help_heading = "OPTIONS")]
        pub(crate) public_url: Option<String>,
        /// Register the project webhook before starting
        #[arg(long, requires = "public_url", help_heading = "FLAGS")]
        pub(crate) register_webhook: bool,
        /// Application configuration containing deployment-owned workflow policy
        #[arg(long, value_name = "PATH", value_hint = clap::ValueHint::FilePath, help_heading = "OPTIONS")]
        pub(crate) config: Option<PathBuf>,
    }
    /// Arguments for `acorn create bot`
    #[derive(Clone, Debug, Args)]
    #[command(long_about = None)]
    pub(crate) struct Create {
        #[command(flatten)]
        pub(crate) common: Common,
        /// Container name (defaults to `acorn-bot-{identifier}`)
        #[arg(short, long, value_name = "NAME", help_heading = "OPTIONS")]
        pub(crate) name: Option<String>,
        /// Acorn Docker image to use for the bot container
        #[arg(short, long, default_value = "acorn:latest", value_name = "IMAGE", help_heading = "OPTIONS")]
        pub(crate) image: Option<String>,
        /// GitLab domain override (e.g., code.ornl.gov)
        #[arg(short = 'D', long, env = "CI_SERVER_HOST", value_name = "DOMAIN", help_heading = "OPTIONS")]
        pub(crate) domain: Option<String>,
        /// Persistent named volume for bot state
        #[arg(long, value_name = "VOLUME", help_heading = "OPTIONS")]
        pub(crate) volume: Option<String>,
        #[command(flatten)]
        pub(crate) target: RemoteTarget,
        #[command(flatten)]
        pub(crate) detach: DetachArgs,
        #[command(flatten)]
        pub(crate) verbose: Verbosity,
    }

    /// Arguments for `acorn serve bot`
    #[derive(Clone, Debug, Args)]
    #[command(long_about = None)]
    pub(crate) struct Serve {
        #[command(flatten)]
        pub(crate) common: Common,
        #[command(flatten)]
        pub(crate) detach: DetachArgs,
        #[command(flatten)]
        pub(crate) target: RemoteTarget,
    }
    impl Common {
        /// Resolve the bot bind address from optional host and port arguments.
        pub(crate) fn bind_address(&self) -> String {
            let host = self.bind.as_deref().map(bind_host).unwrap_or(DEFAULT_HOST);
            let port = self.port.or_else(|| self.bind.as_deref().and_then(bind_port)).unwrap_or(DEFAULT_PORT);
            format!("{host}:{port}")
        }
        /// Resolve a bind address only when the user configured bind-related options.
        pub(crate) fn bind_address_if_configured(&self) -> Option<String> {
            (self.bind.is_some() || self.port.is_some()).then(|| self.bind_address())
        }
    }
    fn bind_host(bind: &str) -> &str {
        bind.rsplit_once(':')
            .and_then(|(host, port)| port.parse::<u16>().ok().map(|_| host))
            .unwrap_or(bind)
    }
    fn bind_port(bind: &str) -> Option<u16> {
        bind.rsplit_once(':').and_then(|(_, port)| port.parse().ok())
    }
}
pub(crate) mod osti;
/// ACP agent selected for local process inference.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum AcpAgentKind {
    /// Run OpenCode's native ACP server.
    #[default]
    #[value(name = "opencode")]
    OpenCode,
    /// Run the Codex ACP adapter.
    Codex,
}
/// Archive container format used for BagIt import and export.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, ValueEnum)]
pub(crate) enum ArchiveType {
    /// 7-Zip archive.
    #[value(name = "7z")]
    #[display("7z")]
    SevenZip,
    /// Uncompressed TAR archive.
    #[display("tar")]
    Tar,
    /// Gzip-compressed TAR archive.
    #[value(name = "tar.gz")]
    #[display("tar.gz")]
    TarGzip,
    /// ZIP archive.
    #[display("zip")]
    Zip,
}
/// Check and suggestion output format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, strum::EnumIs, ValueEnum)]
pub(crate) enum CheckOutputFormat {
    /// Unified diff previews for proposed document changes.
    Diff,
    /// Human-readable checks and fix summaries.
    #[default]
    Human,
    /// Machine-readable checks and fix proposals.
    Json,
    /// RFC 6902 preview envelope for proposed document changes.
    JsonPatch,
}
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
#[derive(Clone, Copy, Debug, Default, PartialEq, strum::Display, ValueEnum)]
#[strum(serialize_all = "lowercase")]
pub enum Diagnostic {
    #[default]
    All,
    System,
    Memory,
    Network,
    Gpu,
    Software,
}
/// Output format for semantic or similarity comparisons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, strum::EnumIs, ValueEnum)]
pub(crate) enum DiffOutputFormat {
    /// Unified semantic diff.
    #[default]
    #[value(alias = "unified")]
    Diff,
    /// Structured similarity result.
    Json,
    /// RFC 6902 JSON Patch.
    JsonPatch,
    /// Human-readable similarity result.
    Text,
}
/// Target export file formats available when exporting research activity data
#[derive(Clone, Debug, Default, Display, EnumIs, ValueEnum)]
pub enum FileFormat {
    #[default]
    #[display("PDF")]
    Pdf,
    #[display("BagIt")]
    Bag,
    #[display("CFF")]
    Cff,
    #[display("DOCX")]
    Docx,
    #[display("JSON")]
    Json,
    #[value(name = "json-ld", alias = "jsonld")]
    #[display("JSON-LD")]
    JsonLd,
    #[value(name = "jsonl", alias = "ndjson")]
    #[display("JSONL")]
    JsonLines,
    #[display("MD")]
    Markdown,
    #[display("PPTX")]
    Powerpoint,
    #[value(name = "rdf-xml", alias = "rdfxml", alias = "rdf")]
    #[display("RDF/XML")]
    RdfXml,
    #[display("Turtle")]
    Turtle,
    #[display("YAML")]
    Yaml,
    #[display("ZON")]
    Zon,
}
/// Controlled-vocabulary similarity profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum FuzzyMode {
    /// Require high-confidence fuzzy matches.
    #[default]
    Strict,
    /// Disable fuzzy scoring; exact, alias, identifier, and normalized matches remain enabled.
    Off,
    /// Accept lower-confidence fuzzy matches while retaining ambiguity safeguards.
    Relaxed,
}
/// Structured output format for gathered identifiers.
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum GatherOutputFormat {
    /// Human-readable console table.
    #[display("console")]
    Console,
    /// JavaScript Object Notation.
    #[default]
    #[display("json")]
    Json,
    /// GitHub-Flavored Markdown.
    #[display("markdown")]
    Markdown,
    /// YAML.
    #[display("yaml")]
    Yaml,
}
/// Inference backend selected for prompt execution and suggestions.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum InferenceBackendKind {
    /// Run a local Agent Client Protocol process.
    #[default]
    Acp,
    /// Call an OpenAI-compatible HTTP endpoint.
    Openai,
}
/// OpenAI-compatible endpoint request shape.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum InferenceEndpointMode {
    /// Use `POST /v1/chat/completions`.
    ChatCompletions,
    /// Use `POST /v1/responses`.
    #[default]
    Responses,
}
/// Stable command output format for inference-enabled commands.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum InferenceOutputFormat {
    /// Human-readable text.
    #[default]
    Human,
    /// Machine-readable JSON.
    Json,
}
/// Structured output format for raw/dry-run results (JSON or YAML)
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum OutputFormat {
    #[default]
    #[display("JSON")]
    Json,
    #[display("YAML")]
    Yaml,
}
/// Serialization format used for the hydrated prompt data block.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum PromptDataFormatArg {
    /// Pretty-printed JSON.
    Json,
    /// Canonical ZON.
    #[default]
    Zon,
}
/// Pairwise similarity method.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum SimilarityMethod {
    /// Piecewise-linear similarity over absolute numeric difference.
    AbsoluteDifference,
    /// German-language Cologne phonetic encoding equality.
    ColognePhonetic,
    /// Cosine similarity over comma- or whitespace-delimited finite vectors.
    Cosine,
    /// Unrestricted unit-cost Damerau-Levenshtein similarity.
    DamerauLevenshtein,
    /// Exact textual equality.
    Exact,
    /// Thresholded fuzzy Jaccard over unique token matches.
    ExtendedJaccard,
    /// Equal-length scalar Hamming similarity.
    Hamming,
    /// Deduplicated Jaccard set similarity.
    Jaccard,
    /// Conventional Jaro similarity.
    Jaro,
    /// Conventional Jaro-Winkler similarity.
    JaroWinkler,
    /// Unit-cost Levenshtein similarity.
    Levenshtein,
    /// English-language original Metaphone encoding equality.
    Metaphone,
    /// Directional Monge-Elkan token similarity.
    MongeElkan,
    /// Restricted optimal string alignment similarity.
    #[value(alias = "osa")]
    OptimalStringAlignment,
    /// Deduplicated overlap coefficient.
    Overlap,
    /// Piecewise-linear similarity over symmetric percentage difference.
    PercentageDifference,
    /// Exact equality over a leading scalar count.
    Prefix,
    /// Soft TF-IDF over an explicit two-document corpus.
    SoftTfidf,
    /// Deduplicated Sorensen-Dice set similarity.
    #[value(alias = "dice")]
    SorensenDice,
    /// English-language American Soundex encoding equality.
    Soundex,
    /// Exact equality over a trailing scalar count.
    Suffix,
    /// TF-IDF cosine over an explicit two-document corpus.
    TfidfCosine,
    /// Weighted Levenshtein with the documented default edit costs.
    WeightedLevenshtein,
}
/// Text preprocessing for similarity comparisons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, strum::Display, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum SimilarityPreprocessing {
    /// Apply full, locale-independent Unicode case folding.
    CaseFold,
    /// Apply Rust Unicode lowercase mapping.
    Lowercase,
    /// Apply Unicode canonical normalization form C.
    Nfc,
    /// Apply Unicode compatibility normalization form KC.
    Nfkc,
    /// Preserve the original scalar sequence.
    #[default]
    Raw,
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
/// SWHID object source.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum SwhidKind {
    /// Infer a local file or directory.
    #[default]
    Auto,
    /// Local file content.
    Content,
    /// Local directory traversal.
    Directory,
    /// Git blob selected by REFERENCE.
    GitBlob,
    /// Git tree selected by REFERENCE.
    GitTree,
    /// Git commit selected by REFERENCE.
    Revision,
    /// Annotated Git tag named by REFERENCE.
    Release,
    /// Local Git branches and tags.
    Snapshot,
}
/// Local inference configuration target for model synchronization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIs, ValueEnum)]
pub(crate) enum SyncTarget {
    /// Synchronize all eligible targets
    #[default]
    All,
    /// Synchronize OpenCode only
    Opencode,
    /// Synchronize llama-swap only
    LlamaSwap,
    /// Synchronize VS Code only
    Vscode,
    /// Synchronize Goose CLI only
    Goose,
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
    /// Standalone, since-filtered research activity logbook updates
    #[display("updates")]
    Updates,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub struct Check {
    /// Local path, HTTP(S) URI, or explicit OCI artifact reference to check; piped standard input is used when omitted
    #[arg(required = false, value_name = "LOCATION", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
    pub(crate) path: Option<PathBuf>,
    /// Check text copied to the system clipboard
    #[arg(long, conflicts_with_all = ["path", "branch", "commit", "filter", "ignore", "merge_request"], value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) paste: bool,
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
    /// Path to application configuration containing Rhai quality gates
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Disable website checks that require internet connection
    #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) disable_website_checks: bool,
    /// Include all check results, including suggestions and info messages
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) all: bool,
    /// Exit on first error
    #[arg(short, long = "exit-on-first-error", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) exit_on_first_error: bool,
    /// Continue watch mode when the prose analyzer cannot be synchronized
    ///
    /// Use `--skip prose` to skip prose analyzer synchronization and analysis entirely
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) ignore_sync_failure: bool,
    /// Set the interval used by watch polling and analyzer sync retries
    #[arg(short = 'p', long, value_parser = parse_watch_duration, value_name = "DURATION", help_heading = "OPTIONS")]
    pub(crate) poll_interval: Option<SignedDuration>,
    /// Regular expression pattern(s) applied to absolute paths of files to include during checks
    ///
    /// Only files matching at least one pattern will be processed
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) filter: Vec<String>,
    /// Force the file format used for standard input or clipboard text
    ///
    /// Supported formats: cff, json, jsonc, jsonld, markdown, rdfxml, text, turtle, yaml, zon
    #[arg(long, value_name = "FORMAT", value_parser = parse_check_format, help_heading = "OPTIONS", conflicts_with_all = ["path", "branch", "commit", "filter", "ignore", "merge_request"])]
    pub(crate) format: Option<MimeType>,
    /// Controlled-vocabulary fuzzy matching profile used by prose checks
    #[arg(long, default_value = "relaxed", value_enum, value_name = "MODE", help_heading = "OPTIONS")]
    pub(crate) fuzzy: FuzzyMode,
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
    /// Skip one or more available check categories
    #[arg(short = 'S', long, value_name = "LIST", value_delimiter = ',', value_parser = parse_check_category, help_heading = "OPTIONS")]
    pub(crate) skip: Vec<CheckCategory>,
    /// Skip checksum verification when downloading files (e.g., downloading Vale to perform static analysis of prose)
    /// > CAUTION: Use at your own risk
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) skip_verify_checksum: bool,
    /// Metadata standard to validate against
    ///
    /// Supported standards: acre, rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, oci, raid, text (`docx` and `pdf` alias `document`)
    #[arg(short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Option<Standard>,
    /// Readability test to use
    #[arg(default_value = "fkgl", short, long = "readability-test", value_name = "METRIC", value_parser = parse_readability, help_heading = "OPTIONS")]
    pub(crate) readability_metric: ReadabilityType,
    /// Use compact output format instead of detailed output
    #[arg(short = 't', long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) terse: bool,
    /// Ask the configured inference backend for bounded, non-executable fix proposals
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) suggest_fixes: bool,
    /// Check and suggestion output format
    #[arg(long, default_value = "human", value_enum, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) output: CheckOutputFormat,
    #[command(flatten)]
    pub(crate) inference: InferenceArgs,
    /// Existing Vale installation directory containing the executable and `.vale.ini`
    #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) vale_directory: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
/// Arguments for `acorn create mcp`.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct CreateMcp {
    /// MCPB destination
    #[arg(long, default_value = "acorn.mcpb", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) output: PathBuf,
    /// Executable to package; defaults to the running ACORN executable
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) binary: Option<PathBuf>,
    /// Target platform; required when packaging another executable
    #[arg(long, value_parser = parse_mcpb_platform, requires = "binary", help_heading = "OPTIONS")]
    pub(crate) platform: Option<Platform>,
    /// Authoring Kitfile to update after creating the bundle
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) kitfile: Option<PathBuf>,
    /// Replace an existing regular bundle
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) clobber: bool,
    /// Validate and print the plan without writing files
    #[arg(long, visible_alias = "what-if", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
}
/// Detached execution flag shared by all remote-capable commands.
#[derive(Clone, Debug, Args)]
pub(crate) struct DetachArgs {
    /// Run as detached container/service instead of foreground/in-process
    #[arg(short = 'd', long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) detach: bool,
}
/// Arguments for `acorn diff`.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Diff {
    /// Original file, or left literal with --values.
    #[arg(value_name = "BEFORE", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) before: PathBuf,
    /// Revised file, or right literal with --values.
    #[arg(value_name = "AFTER", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) after: PathBuf,
    /// Positive numeric difference cutoff.
    #[arg(long, requires = "method", value_name = "FLOAT", help_heading = "OPTIONS")]
    pub(crate) cutoff: Option<f64>,
    /// Comparison output format.
    #[arg(long, value_enum, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) format: Option<DiffOutputFormat>,
    /// Positive prefix or suffix scalar count.
    #[arg(long, requires = "method", value_name = "N", help_heading = "OPTIONS")]
    pub(crate) length: Option<usize>,
    /// Maximum generated or supplied collection items.
    #[arg(long, requires = "method", value_name = "N", help_heading = "OPTIONS")]
    pub(crate) max_collection_items: Option<usize>,
    /// Maximum Unicode scalar values in either textual operand.
    #[arg(long, requires = "method", value_name = "N", help_heading = "OPTIONS")]
    pub(crate) max_input_scalars: Option<usize>,
    /// Maximum charged comparison work units.
    #[arg(long, requires = "method", value_name = "N", help_heading = "OPTIONS")]
    pub(crate) max_work_units: Option<usize>,
    /// Pairwise similarity method; switches from semantic diff mode.
    #[arg(long, value_enum, value_name = "METHOD", help_heading = "OPTIONS")]
    pub(crate) method: Option<SimilarityMethod>,
    /// Use unpadded character n-grams instead of words for set methods.
    #[arg(long, requires = "method", value_name = "N", help_heading = "OPTIONS")]
    pub(crate) ngram_width: Option<usize>,
    /// Text preprocessing; defaults to raw.
    #[arg(
        long = "preprocess",
        requires = "method",
        default_value_t,
        value_enum,
        value_name = "MODE",
        help_heading = "OPTIONS"
    )]
    pub(crate) preprocessing: SimilarityPreprocessing,
    /// Metadata standard override.
    #[arg(short, long, conflicts_with = "method", value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Option<Standard>,
    /// Token-match threshold from zero to one.
    #[arg(long, requires = "method", value_name = "0..1", help_heading = "OPTIONS")]
    pub(crate) threshold: Option<Noul>,
    /// Treat both operands as literal values instead of file paths.
    #[arg(long, requires = "method", help_heading = "FLAGS")]
    pub(crate) values: bool,
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
    /// Path to configuration file (JSON, JSONC, or YAML)
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
    /// Save downloaded files directly beneath the output directory
    #[arg(short = 'f', long, help_heading = "FLAGS")]
    pub(crate) flatten: bool,
    /// Replace existing files or directories at selected output paths
    #[arg(short = 'C', long, help_heading = "FLAGS")]
    pub(crate) clobber: bool,
    /// Force archive handling and bypass content inference
    #[arg(long, value_enum, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) archive_format: Option<ArchiveType>,
    /// Reject deprecated research activity fields during database ingestion
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) strict: bool,
    /// Rewrite deprecated research activity fields before database ingestion
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) canonicalize: bool,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(
    long_about = "Download model weights for use with ACORN research harnesses and local inference.\n\
        \n\
        By default, when downloading from a Hugging Face repository, ACORN selects GGUF model\n\
        files and prefers the Q4_K_M quantization. Use --filter to override this behavior.\n\
        \n\
        The --filter flag behaves as an include rule: only sources matching at least one pattern\n\
        will be downloaded. The --ignore flag behaves as an exclude rule: sources matching any\n\
        ignore pattern are excluded even if they match a filter pattern. Both flags accept\n\
        regular expression patterns (not Hugging Face glob patterns). Simple patterns are\n\
        automatically optimized to glob matching for Hugging Face repositories.\n\
        \n\
        ACORN downloads model weights directly via HTTP. Python, pip, or the huggingface-cli\n\
        (hf) tool are not required.\n\
        \n\
        Users of Transformers or PyTorch typically need the full repository or a broader filter\n\
        that includes config.json, tokenizer files, tokenizer_config.json, and optionally\n\
        custom code files. The default GGUF-only filter is designed for local GGUF inference.",
    verbatim_doc_comment
)]
pub(crate) struct DownloadModel {
    #[command(flatten)]
    pub(crate) source: ModelSourceArgs,
    #[command(flatten)]
    pub(crate) resolution: ModelResolutionArgs,
    #[command(flatten)]
    pub(crate) synchronization: ModelSyncArgs,
    /// Regular expression pattern(s) used to include model weight sources while downloading
    ///
    /// Only sources matching at least one pattern will be downloaded
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) filter: Vec<String>,
    /// Regular expression pattern(s) to ignore model weight sources while downloading
    ///
    /// Sources matching any ignore pattern are excluded even if they match a filter pattern
    #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) ignore: Vec<String>,
    /// Ordered, exact GGUF quantizations to consider
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) quantization: Vec<Quantization>,
    /// Maximum GPU memory available for model weights
    #[arg(long, value_name = "MEMORY", help_heading = "OPTIONS")]
    pub(crate) gpu_memory: Option<Memory>,
    /// Local directory where downloaded model weights should be saved
    ///
    /// Defaults to $HOME/.models when omitted
    #[arg(short = 'o', long = "local-dir", visible_alias = "output", value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) local_dir: Option<PathBuf>,
    /// Copy local model files into the model directory instead of referencing in place
    #[arg(long, conflicts_with = "symlink", help_heading = "OPTIONS")]
    pub(crate) copy: bool,
    /// Create symlinks to local model files in the model directory instead of referencing in place
    #[arg(long, conflicts_with = "copy", help_heading = "OPTIONS")]
    pub(crate) symlink: bool,
    /// Only download models with these user-facing names
    #[arg(long, value_name = "NAME", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) whitelist: Vec<String>,
    /// URI or local path to a model whitelist file
    ///
    /// Accepts a plain-text list of names or a JSON/YAML list of names or model details
    #[arg(long, value_name = "URI_OR_PATH", help_heading = "OPTIONS")]
    pub(crate) whitelist_file: Option<String>,
    /// Skip SHA-256 checksum verification when checksum sidecar metadata is available
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) skip_verify_checksum: bool,
    /// Output structured JSON (or YAML with --format yaml) of resolved models
    #[arg(long, conflicts_with = "sync", help_heading = "FLAGS")]
    pub(crate) raw: bool,
    /// Output format for --raw results
    #[arg(long, value_name = "FORMAT", default_value = "json", requires = "raw", help_heading = "OPTIONS")]
    pub(crate) format: OutputFormat,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ExecAutomation {
    /// Trusted local Rhai automation script.
    #[arg(value_name = "SCRIPT", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) script: PathBuf,
    /// Arguments passed to the script.
    #[arg(value_name = "ARGUMENT", trailing_var_arg = true, allow_hyphen_values = true, help_heading = "ARGS")]
    pub(crate) arguments: Vec<String>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ExecCheck {
    /// Rhai script path
    #[arg(value_name = "SCRIPT", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) script: PathBuf,
    /// Input file to evaluate against
    #[arg(value_name = "INPUT", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) input: PathBuf,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ExecCheckSafe {
    /// Rhai script path
    #[arg(value_name = "SCRIPT", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) script: PathBuf,
    /// Output format (json or markdown)
    #[arg(long, value_name = "FORMAT", value_parser = parse_exec_format, help_heading = "OPTIONS")]
    pub(crate) format: Option<MimeType>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ExecHookCheck {
    /// Rhai script path.
    #[arg(value_name = "SCRIPT", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) script: PathBuf,
    /// Deployment manifest name or explicit manifest path.
    #[arg(long, value_name = "MANIFEST", help_heading = "OPTIONS")]
    pub(crate) manifest: String,
    /// Workflow hook to evaluate.
    #[arg(long, value_name = "HOOK", value_parser = parse_exec_hook, help_heading = "OPTIONS")]
    pub(crate) hook: Hook,
    /// Input file to evaluate against.
    #[arg(long, value_name = "FILE", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) input: PathBuf,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ExecManifestCheck {
    /// Script manifest path.
    #[arg(value_name = "MANIFEST", value_hint = ValueHint::FilePath, help_heading = "ARGS")]
    pub(crate) manifest: PathBuf,
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
    #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) branch: Option<String>,
    /// Chrome or Chromium executable to use for PDF export
    #[arg(long, env = CHROME_PATH, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) chrome_path: Option<PathBuf>,
    /// Combine all export artifacts into a single artifact
    #[arg(long = "combine", value_name = "BOOL", help_heading = "FLAGS", conflicts_with = "schema")]
    pub(crate) combine: bool,
    /// Export files that were changed in a given Git commit
    #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) commit: Option<String>,
    /// Path to application configuration containing crosswalk import defaults
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS", conflicts_with_all = ["schema", "tools"])]
    pub(crate) config: Option<PathBuf>,
    /// Run export without making changes. Will print what would happen.
    #[arg(
        long = "dry-run",
        visible_alias = "what-if",
        value_name = "BOOL",
        help_heading = "FLAGS",
        conflicts_with = "schema"
    )]
    pub(crate) dry_run: bool,
    /// Export target file format
    ///
    /// When used with --schema, specifies the output format (json or yaml)
    #[arg(
        default_value = "pdf",
        default_value_if("schema", "true", "json"),
        short,
        long,
        value_name = "FORMAT",
        help_heading = "OPTIONS"
    )]
    pub(crate) format: FileFormat,
    /// BagIt archive container format.
    #[arg(long, value_enum, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) archive_format: Option<ArchiveType>,
    /// Regular expression pattern(s) applied to absolute paths of files to include during export
    ///
    /// Only files matching at least one pattern will be processed
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) filter: Vec<String>,
    /// Show the full date and time in updates PDFs instead of month and day.
    #[arg(long, help_heading = "FLAGS", requires = "since")]
    pub(crate) full_timestamp: bool,
    /// Regular expression pattern(s) applied to absolute paths of files to exclude from export process
    ///
    /// Only applies to `--path` values that point to a directory
    ///
    /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
    ///
    /// Example: --ignore "[/]valid.json$,[/]draft.json$"
    #[arg(
        short,
        long,
        value_name = "LIST",
        value_delimiter = ',',
        help_heading = "OPTIONS",
        conflicts_with = "schema"
    )]
    pub(crate) ignore: Vec<String>,
    /// Show ASPECT attribute labels in exported charts
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS", conflicts_with = "schema")]
    pub(crate) show_aspect_labels: bool,
    /// Show ASPECT numeric scores in exported charts
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS", conflicts_with = "schema")]
    pub(crate) show_aspect_scores: bool,
    /// Export files that were changed in a given merge request (Gitlab) or pull request (GitHub)
    #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS", conflicts_with = "schema")]
    pub(crate) merge_request: bool,
    /// Path to output directory where the export artifacts should be saved
    #[arg(default_value = "./export", short, long, value_name = "DIRECTORY", value_hint = ValueHint::DirPath, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) output: Option<PathBuf>,
    /// Print JSON or YAML schema for the selected standard to stdout
    ///
    /// Use --format to specify output format (default: json)
    #[arg(long = "schema", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) schema: bool,
    /// Export the canonical Needle-compatible ACORN tool catalog as tools.json
    #[arg(
        long = "tools",
        value_name = "BOOL",
        help_heading = "FLAGS",
        conflicts_with_all = [
            "schema", "branch", "combine", "commit", "config", "archive_format", "filter", "ignore", "show_aspect_labels",
            "show_aspect_scores", "merge_request", "skip", "reference", "strict", "from", "to", "crosswalk_contact", "local_identifier"
        ]
    )]
    pub(crate) tools: bool,
    /// Suppress output from external commands while preserving ACORN output — useful for piping
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) raw: bool,
    /// Skip one or more available export check categories
    #[arg(short = 'S', long, value_name = "LIST", value_delimiter = ',', value_parser = parse_check_category, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) skip: Vec<CheckCategory>,
    /// Path to reference file to be used when exporting to PowerPoint
    ///
    /// Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) reference: Option<PathBuf>,
    /// Metadata standard to validate against
    ///
    /// Supported standards: acre, rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    /// Fail if conversion loses data (only for crosswalk exports)
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS", conflicts_with = "schema")]
    pub(crate) strict: bool,
    /// Artifact target to export
    #[arg(
        long,
        default_value = "fact-sheet",
        value_name = "TARGET",
        help_heading = "OPTIONS",
        conflicts_with = "schema"
    )]
    pub(crate) target: Target,
    /// Inclusive RFC 3339 timestamp for updates exports
    #[arg(
        long,
        value_name = "TIMESTAMP",
        required_if_eq("target", "updates"),
        help_heading = "OPTIONS",
        conflicts_with = "schema"
    )]
    pub(crate) since: Option<String>,
    /// Metadata standard to use for source data inference (default: auto-detect)
    ///
    /// Supported standards: acre, rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)
    #[arg(long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) from: Option<Standard>,
    /// JSON or YAML contact required for Dublin Core or RAiD import; overrides options.crosswalk.contact
    #[arg(
        long,
        value_name = "PATH",
        value_hint = ValueHint::FilePath,
        help_heading = "OPTIONS",
        requires = "to",
        conflicts_with = "schema"
    )]
    pub(crate) crosswalk_contact: Option<PathBuf>,
    /// Local RAD import identifier; overrides options.crosswalk.localIdentifier
    #[arg(long, value_name = "IDENTIFIER", help_heading = "OPTIONS", requires = "to", conflicts_with = "schema")]
    pub(crate) local_identifier: Option<String>,
    /// Target metadata standard for crosswalk conversion
    ///
    /// Supported standards: acre, rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)
    #[arg(long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS", conflicts_with = "schema")]
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
    /// Path to application configuration containing Rhai quality gates
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Run format without making changes to target file(s). Will print a diff of changes.
    #[arg(short, long = "dry-run", visible_alias = "what-if", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Enrich metadata after formatting it.
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) enrich: bool,
    /// Controlled-vocabulary fuzzy matching profile.
    #[arg(long, value_enum, default_value_t, value_name = "MODE", help_heading = "OPTIONS")]
    pub(crate) fuzzy: FuzzyMode,
    /// Disable color in dry-run output
    #[arg(long = "no-color", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) no_color: bool,
    /// Enrichment provider(s); may be repeated or comma-delimited.
    #[arg(long, requires = "enrich", value_delimiter = ',', value_parser = parse_enrichment_provider, value_name = "PROVIDER", help_heading = "OPTIONS")]
    pub(crate) provider: Vec<Provider>,
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
    /// Supported standards: acre, rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Gather {
    /// Files, directories, URLs, or PID values; piped standard input is included automatically.
    #[arg(value_name = "INPUT", value_hint = ValueHint::AnyPath)]
    pub(crate) input: Vec<String>,
    /// Controlled-vocabulary fuzzy matching profile.
    #[arg(long, value_enum, default_value_t, value_name = "MODE", help_heading = "OPTIONS")]
    pub(crate) fuzzy: FuzzyMode,
    /// Set the interval used by watch polling
    #[arg(short = 'p', long, value_parser = parse_watch_duration, value_name = "DURATION", help_heading = "OPTIONS")]
    pub(crate) poll_interval: Option<SignedDuration>,
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
    /// Maximum directory descendant depth to search; direct children have depth 1
    #[arg(long, value_name = "DEPTH", help_heading = "OPTIONS")]
    pub(crate) max_depth: Option<usize>,
    /// Process files that were changed in a given merge request (GitLab) or pull request (GitHub)
    #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) merge_request: bool,
    /// Write the report to a file instead of stdout.
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) output: Option<PathBuf>,
    /// Search DOE CODE and return projects, people, or organizations.
    #[arg(long, value_enum, conflicts_with_all = ["filter", "ignore", "max_depth", "merge_request", "standard"], help_heading = "OPTIONS")]
    pub(crate) osti: Option<OstiSearchView>,
    /// Restrict remote results to a credited organization.
    #[arg(long, requires = "osti", value_name = "TERM", help_heading = "OPTIONS")]
    pub(crate) organization: Option<String>,
    /// Organization relationship to match.
    #[arg(long, requires = "organization", value_enum, help_heading = "OPTIONS")]
    pub(crate) organization_role: Option<OstiOrganizationRole>,
    /// Restrict results to a DOE laboratory and add its known ROR to persisted project candidates.
    #[arg(long, requires = "osti", conflicts_with_all = ["organization", "organization_role"], value_enum, ignore_case = true, help_heading = "OPTIONS")]
    pub(crate) lab: Option<DoeLab>,
    /// Number of upstream projects requested per page.
    #[arg(long, requires = "osti", default_value_t = 20, value_name = "COUNT", help_heading = "OPTIONS")]
    pub(crate) limit: usize,
    /// Zero-based upstream project offset.
    #[arg(long, requires = "osti", default_value_t = 0, value_name = "COUNT", help_heading = "OPTIONS")]
    pub(crate) offset: usize,
    /// Retrieve every matching upstream project page.
    #[arg(long, requires = "osti", help_heading = "FLAGS")]
    pub(crate) all: bool,
    /// Structured report format; defaults to console for terminals and JSON for pipes or files.
    #[arg(long, value_enum, help_heading = "OPTIONS")]
    pub(crate) format: Option<GatherOutputFormat>,
    /// Print identifiers as plain text, or enriched RAD candidates as JSON lines with `--enrich`.
    #[arg(long, conflicts_with = "format", help_heading = "FLAGS")]
    pub(crate) raw: bool,
    /// Enrich gathered research activity data with all applicable providers.
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) enrich: bool,
    /// Use compact check output and suppress the discovery table.
    #[arg(short = 't', long, conflicts_with_all = ["format", "output", "raw"], help_heading = "FLAGS")]
    pub(crate) terse: bool,
    /// Resolve supported identifiers through their metadata providers.
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) resolve: bool,
    /// Citation format used for resolved DOI lines.
    #[arg(long, value_name = "FORMAT", requires_all = ["raw", "resolve"], help_heading = "OPTIONS")]
    pub(crate) citation_format: Option<CitationFormat>,
    /// Metadata standard to validate against
    #[arg(short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Option<Standard>,
    /// Literal text to inspect; may be repeated.
    #[arg(long, value_name = "TEXT", help_heading = "OPTIONS")]
    pub(crate) text: Vec<String>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Import {
    #[command(subcommand)]
    pub(crate) command: Option<super::ImportCommands>,
    /// Local directories, repository URIs, or BagIt archive URIs.
    #[arg(value_name = "URI", required = false, value_delimiter = ',', value_hint = ValueHint::AnyPath, help_heading = "ARGS")]
    pub(crate) uri: Vec<String>,
    /// Path to configuration file (JSON, JSONC, or YAML).
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, conflicts_with = "uri", help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Regular expression patterns used to include files.
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) filter: Vec<String>,
    /// Regular expression patterns used to exclude files.
    #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) ignore: Vec<String>,
    /// Directory where imported content is saved.
    #[arg(default_value = "./content", short, long, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) output: Option<PathBuf>,
    /// Save imported files directly beneath the output directory.
    #[arg(short = 'f', long, help_heading = "FLAGS")]
    pub(crate) flatten: bool,
    /// Replace existing files or directories at selected output paths.
    #[arg(short = 'C', long, help_heading = "FLAGS")]
    pub(crate) clobber: bool,
    /// Force archive handling and bypass content inference.
    #[arg(long, value_enum, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) archive_format: Option<ArchiveType>,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ImportModel {
    #[command(flatten)]
    pub(crate) source: ModelSourceArgs,
    #[command(flatten)]
    pub(crate) resolution: ModelResolutionArgs,
    #[command(flatten)]
    pub(crate) synchronization: ModelSyncArgs,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
/// Options for one bounded Needle inference request.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Infer {
    #[command(subcommand)]
    pub(crate) command: Option<super::InferCommands>,
    /// Prompt used to select and invoke ACORN tools
    #[arg(required = false, value_name = "PROMPT")]
    pub(crate) prompt: Option<String>,
    /// ModelKit directory, file:// URI, HTTP(S) manifest URI, or digest-pinned oci:// reference
    #[arg(long, value_name = "LOCATION", help_heading = "OPTIONS")]
    pub(crate) custom_location: Option<String>,
    /// Minimum calibrated confidence required before executing a tool
    #[arg(long, default_value_t = DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD, value_name = "0..1", help_heading = "OPTIONS")]
    pub(crate) threshold: Noul,
    /// Maximum number of Needle tool-selection rounds
    #[arg(long, default_value_t = DEFAULT_NEEDLE_MAX_ROUNDS, value_name = "N", help_heading = "OPTIONS")]
    pub(crate) max_rounds: usize,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
/// Options shared by inference-enabled CLI commands.
#[derive(Builder, Clone, Debug, Args)]
#[builder(start_fn = init)]
pub(crate) struct InferenceArgs {
    /// Inference backend; CLI overrides `ACORN_INFERENCE_BACKEND`, then defaults to `acp`
    #[arg(
        long,
        env = "ACORN_INFERENCE_BACKEND",
        default_value = "acp",
        value_enum,
        value_name = "BACKEND",
        help_heading = "INFERENCE"
    )]
    #[builder(default)]
    pub(crate) backend: InferenceBackendKind,
    /// ACP agent identifier; defaults to `opencode`
    #[arg(long, value_enum, value_name = "AGENT", help_heading = "INFERENCE")]
    pub(crate) agent: Option<AcpAgentKind>,
    /// Explicit ACP agent executable
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "INFERENCE")]
    pub(crate) agent_executable: Option<PathBuf>,
    /// ACP session working directory
    #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "INFERENCE")]
    pub(crate) working_directory: Option<PathBuf>,
    /// Model selector; required for OpenAI-compatible inference
    #[arg(long, value_name = "MODEL", help_heading = "INFERENCE")]
    pub(crate) model: Option<String>,
    /// OpenAI-compatible base URL; overrides `OPENAI_SERVER_HOST`
    #[arg(long, value_name = "URL", help_heading = "INFERENCE")]
    pub(crate) base_url: Option<String>,
    /// OpenAI-compatible endpoint shape
    #[arg(long, value_enum, value_name = "MODE", help_heading = "INFERENCE")]
    pub(crate) endpoint_mode: Option<InferenceEndpointMode>,
    /// Maximum inference duration
    #[arg(long, default_value = "30s", value_parser = parse_watch_duration, value_name = "DURATION", help_heading = "INFERENCE")]
    #[builder(default = SignedDuration::from_secs(30))]
    pub(crate) timeout: SignedDuration,
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
    /// Path to application configuration containing Rhai quality gates
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Run link without making changes to target file(s). Will print a diff of changes.
    #[arg(short, long = "dry-run", visible_alias = "what-if", value_name = "BOOL", help_heading = "FLAGS")]
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
/// Fallback and search behavior shared by model commands.
#[derive(Clone, Debug, Args)]
pub(crate) struct ModelResolutionArgs {
    /// Disable automatic GGUF fallback repository discovery.
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) no_fallback: bool,
    /// Open an interactive picker when fallback finds multiple repositories.
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) interactive: bool,
    /// Maximum number of Hugging Face repositories to search.
    #[arg(long, value_name = "LIMIT", env = SEARCH_LIMIT, default_value_t = DEFAULT_HUGGINGFACE_SEARCH_LIMIT, help_heading = "OPTIONS")]
    pub(crate) search_limit: usize,
    /// Minimum downloads required for a GGUF fallback repository.
    #[arg(long = "minimum-popularity", value_name = "COUNT", env = MINIMUM_DOWNLOAD_COUNT, default_value_t = DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT, help_heading = "OPTIONS")]
    pub(crate) minimum_download_count: u64,
}
/// Model selectors and configuration shared by model commands.
#[derive(Clone, Debug, Args)]
pub(crate) struct ModelSourceArgs {
    /// Model ID/name(s), local path(s), direct URL(s), or explicit oci:// artifact reference(s).
    #[arg(value_name = "MODEL", required = false, value_delimiter = ',', help_heading = "ARGS")]
    pub(crate) model: Vec<String>,
    /// URI or local path to a model list file.
    #[arg(long, value_name = "URI_OR_PATH", help_heading = "OPTIONS")]
    pub(crate) model_file: Option<String>,
    /// Path to an application configuration file listing models.
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
}
/// Synchronization behavior shared by model commands.
#[derive(Clone, Debug, Args)]
pub(crate) struct ModelSyncArgs {
    /// Synchronize resolved models with configured inference applications.
    #[arg(long, value_enum, value_name = "TARGET", num_args = 0..=1, default_missing_value = "all", help_heading = "OPTIONS")]
    pub(crate) sync: Option<SyncTarget>,
    /// Assume synchronized models exist under the models directory.
    #[arg(long, requires = "sync", help_heading = "FLAGS")]
    pub(crate) force: bool,
    /// Preview resolution and synchronization without writes.
    #[arg(long = "dry-run", visible_alias = "what-if", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
}
/// Options for `acorn infer run`.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct PromptRun {
    /// Known embedded prompt asset name
    #[arg(value_name = "ASSET")]
    pub(crate) asset: String,
    /// JSON, YAML, or ZON input files, directories, or glob patterns; use `-` for standard input
    #[arg(value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "ARGS")]
    pub(crate) paths: Vec<PathBuf>,
    /// JSON object; omit the value to read standard input or use `--paste`
    #[arg(long, value_name = "JSON", num_args = 0..=1, help_heading = "OPTIONS")]
    pub(crate) data: Option<Option<String>>,
    /// Override one top-level scalar prompt-data value
    #[arg(long = "set", value_name = "KEY=VALUE", action = clap::ArgAction::Append, help_heading = "OPTIONS")]
    pub(crate) set: Vec<String>,
    /// Serialization format for the prompt input-data block
    #[arg(long, default_value = "zon", value_enum, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) format: PromptDataFormatArg,
    /// Preview the fully hydrated prompt without backend checks or I/O
    #[arg(long, visible_alias = "what-if", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Read bare `--data` from the system clipboard
    #[arg(long, requires = "data", help_heading = "FLAGS")]
    pub(crate) paste: bool,
    /// Command output format
    #[arg(long, default_value = "human", value_enum, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) output: InferenceOutputFormat,
    #[command(flatten)]
    pub(crate) inference: InferenceArgs,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
/// Remote target arguments shared by container-producing create/serve commands.
#[derive(Clone, Debug, Args)]
pub(crate) struct RemoteTarget {
    /// SSH URI or alias of the remote host (e.g. ssh://user@host, my-alias).
    ///
    /// The local client and SSH configuration are used. Published ports and volumes belong to the remote host.
    #[arg(long, value_name = "URI|ALIAS", value_parser = parse_ssh_remote, conflicts_with = "context", help_heading = "OPTIONS")]
    pub(crate) host: Option<Remote>,
    /// Docker context / Podman connection name (from `docker context ls` / `podman system connection ls`)
    #[arg(long, value_name = "NAME", conflicts_with = "host", help_heading = "OPTIONS")]
    pub(crate) context: Option<String>,
}
/// Arguments for scheduled artifact creation and publication.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Report {
    /// Application configuration containing PowerAutomate project mappings.
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Exact merged default-branch commit used as report input.
    #[arg(long, env = "CI_COMMIT_SHA", value_name = "SHA", help_heading = "OPTIONS")]
    pub(crate) commit: Option<String>,
    /// Write one local PDF containing the title, selected projects, and final legend.
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) combine: bool,
    /// Explicit inclusive lower bound for local or recovery runs.
    #[arg(long, value_name = "TIMESTAMP", help_heading = "OPTIONS")]
    pub(crate) from: Option<String>,
    /// Show the full date and time in report PDFs instead of month and day.
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) full_timestamp: bool,
    /// Fixed exclusive reporting upper bound shared by every project.
    #[arg(long, env = "ACORN_REPORT_THROUGH", value_name = "TIMESTAMP", help_heading = "OPTIONS")]
    pub(crate) through: String,
    /// Repository checkout root containing configured project paths.
    #[arg(long, default_value = ".", value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) root: PathBuf,
    /// Directory for generated reports and failure diagnostics.
    #[arg(short, long, default_value = ".acorn/reports", value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) output: PathBuf,
    /// Restrict reporting to selected configured project identifiers.
    #[arg(long = "project", value_name = "ID", value_delimiter = ',', help_heading = "OPTIONS")]
    pub(crate) projects: Vec<String>,
    /// GitLab pipeline identifier recorded in the manifest.
    #[arg(long, env = "CI_PIPELINE_ID", value_name = "ID", help_heading = "OPTIONS")]
    pub(crate) pipeline_id: Option<String>,
    /// GitLab job identifier recorded in the manifest.
    #[arg(long, env = "CI_JOB_ID", value_name = "ID", help_heading = "OPTIONS")]
    pub(crate) job_id: Option<String>,
    /// Path to Chromium executable used for PDF rendering.
    #[arg(long, env = CHROME_PATH, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) chrome_path: Option<PathBuf>,
    /// Generate local artifacts and manifests without GitLab publication or callbacks.
    #[arg(long = "dry-run", visible_alias = "what-if", help_heading = "FLAGS")]
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
    /// Use only command-line arguments without discovering an application configuration
    #[arg(long, conflicts_with = "config", help_heading = "FLAGS")]
    pub(crate) no_config: bool,
    /// Runner description (alternative to providing a configuration file)
    #[arg(long, value_name = "DESCRIPTION", help_heading = "OPTIONS")]
    pub(crate) description: Option<String>,
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
    /// Container runtime override (docker, podman, apptainer). Defaults to the executor's runtime.
    #[arg(short = 'R', long, value_name = "RUNTIME", value_parser = parse_executor, help_heading = "OPTIONS")]
    pub(crate) runtime: Option<Executor>,
    #[command(flatten)]
    pub(crate) target: RemoteTarget,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
/// Arguments for `acorn serve grpc`.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ServeGrpc {
    /// Enable registered mutating operations
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) allow_mutation: bool,
    /// Address for the authenticated gRPC endpoint
    #[arg(long, default_value = "127.0.0.1:50051", value_name = "ADDRESS", help_heading = "OPTIONS")]
    pub(crate) bind: SocketAddr,
    #[command(flatten)]
    pub(crate) target: RemoteTarget,
    #[command(flatten)]
    pub(crate) detach: DetachArgs,
    /// Container runtime for detached execution
    #[arg(short = 'R', long, value_name = "RUNTIME", value_parser = parse_executor, help_heading = "OPTIONS")]
    pub(crate) runtime: Option<Executor>,
    /// PEM-encoded TLS certificate chain
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, requires = "tls_key", help_heading = "OPTIONS")]
    pub(crate) tls_cert: Option<PathBuf>,
    /// PEM-encoded TLS private key
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath, requires = "tls_cert", help_heading = "OPTIONS")]
    pub(crate) tls_key: Option<PathBuf>,
}
/// Arguments for `acorn serve mcp`.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ServeMcp {
    /// Enable persistent mutating tools
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) allow_mutation: bool,
    #[command(flatten)]
    pub(crate) target: RemoteTarget,
    #[command(flatten)]
    pub(crate) detach: DetachArgs,
    /// Container runtime for detached execution
    #[arg(short = 'R', long, value_name = "RUNTIME", value_parser = parse_executor, help_heading = "OPTIONS")]
    pub(crate) runtime: Option<Executor>,
}
/// Arguments for `acorn serve rpc`.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ServeRpc {
    /// Enable registered mutating operations
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) allow_mutation: bool,
    /// Address for the authenticated JSON-RPC HTTP endpoint
    #[arg(long, default_value = "127.0.0.1:8001", value_name = "ADDRESS", help_heading = "OPTIONS")]
    pub(crate) bind: SocketAddr,
    #[command(flatten)]
    pub(crate) target: RemoteTarget,
    #[command(flatten)]
    pub(crate) detach: DetachArgs,
    /// Container runtime for detached execution
    #[arg(short = 'R', long, value_name = "RUNTIME", value_parser = parse_executor, help_heading = "OPTIONS")]
    pub(crate) runtime: Option<Executor>,
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
    #[arg(long = "dry-run", visible_alias = "what-if", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
/// Calculate or verify a Software Hash Identifier.
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Swhid {
    /// Local file, directory, or Git repository
    #[arg(default_value = ".", value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "ARGS")]
    pub(crate) path: PathBuf,
    /// Object source to calculate
    #[arg(long, value_enum, default_value_t, help_heading = "OPTIONS")]
    pub(crate) kind: SwhidKind,
    /// Git revision expression or annotated tag name
    #[arg(short, long, value_name = "REFERENCE", help_heading = "OPTIONS")]
    pub(crate) reference: Option<String>,
    /// Verify the calculated core object against this SWHID
    #[arg(long, value_name = "SWHID", help_heading = "OPTIONS")]
    pub(crate) verify: Option<String>,
    /// Print only the calculated identifier
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) raw: bool,
}
#[derive(Clone, Debug, Args)]
#[command(
    long_about = "Synchronize local inference configuration from ACORN model entries.\n\
        \n\
        Resolves configured models to downloaded GGUF files and merges them into\n\
        llama-swap, OpenCode, VS Code, and/or Goose configuration.\n\
        \n\
        With no target flags, synchronizes configurations for installed applications.\n\
        Target flags include one or more applications, subject to command availability.\n\
        --force bypasses command detection only for explicitly included targets.
        --assume-models bypasses model directory and GGUF file verification.",
    verbatim_doc_comment
)]
pub(crate) struct Sync {
    /// Path to ACORN configuration file (JSON, JSONC, or YAML)
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// URI or local path to a model list file
    ///
    /// Accepts a plain-text list of repository IDs or a JSON/YAML list of IDs or model details
    #[arg(long, value_name = "URI_OR_PATH", help_heading = "OPTIONS")]
    pub(crate) model_file: Option<String>,
    /// Bypass command detection for explicitly included targets
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) force: bool,
    /// Assume configured models exist without checking directories or GGUF files
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) assume_models: bool,
    /// Disable resolution through previously downloaded GGUF fallback repositories
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) no_fallback: bool,
    /// Include OpenCode configuration synchronization
    #[arg(long, help_heading = "TARGETS")]
    pub(crate) opencode: bool,
    /// Include llama-swap configuration synchronization
    #[arg(long, help_heading = "TARGETS")]
    pub(crate) llama_swap: bool,
    /// Include VS Code custom endpoint configuration synchronization
    #[arg(long, help_heading = "TARGETS")]
    pub(crate) vscode: bool,
    /// Include Goose CLI configuration synchronization
    #[arg(long, help_heading = "TARGETS")]
    pub(crate) goose: bool,
    /// Configure the ACORN MCP server in selected compatible clients
    #[arg(long, help_heading = "TARGETS")]
    pub(crate) mcp: bool,
    /// Default directory where downloaded model weights live (overrides config)
    #[arg(long = "models-dir", value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) models_dir: Option<PathBuf>,
    /// Path to OpenCode configuration file (overrides config)
    #[arg(long = "opencode-config", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) opencode_config: Option<PathBuf>,
    /// Path to llama-swap configuration file (overrides config)
    #[arg(long = "llama-swap-config", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) llama_swap_config: Option<PathBuf>,
    /// Path to VS Code `chatLanguageModels.json` (overrides config)
    #[arg(long = "vscode-config", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) vscode_config: Option<PathBuf>,
    /// Path to VS Code workspace `mcp.json` (overrides default)
    #[arg(long = "vscode-mcp-config", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) vscode_mcp_config: Option<PathBuf>,
    /// Path to Goose CLI `config.yaml` (overrides config)
    #[arg(long = "goose-config", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) goose_config: Option<PathBuf>,
    /// Preview changes without writing any files
    #[arg(long = "dry-run", visible_alias = "what-if", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Disable color in dry-run output
    #[arg(long = "no-color", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) no_color: bool,
    /// Remove stale managed models from target configurations
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) prune: bool,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
impl From<SwhidKind> for CalculationKind {
    fn from(value: SwhidKind) -> Self {
        match value {
            | SwhidKind::Auto => Self::Auto,
            | SwhidKind::Content => Self::Content,
            | SwhidKind::Directory => Self::Directory,
            | SwhidKind::GitBlob => Self::GitBlob,
            | SwhidKind::GitTree => Self::GitTree,
            | SwhidKind::Revision => Self::Revision,
            | SwhidKind::Release => Self::Release,
            | SwhidKind::Snapshot => Self::Snapshot,
        }
    }
}
impl CheckOutputFormat {
    pub(crate) fn is_patch(self) -> bool {
        self.is_diff() || self.is_json_patch()
    }
}
impl From<&GatherOutputFormat> for context::OutputFormat {
    fn from(format: &GatherOutputFormat) -> Self {
        match format {
            | GatherOutputFormat::Console => Self::Text,
            | GatherOutputFormat::Json => Self::Json,
            | GatherOutputFormat::Markdown => Self::Markdown,
            | GatherOutputFormat::Yaml => Self::Yaml,
        }
    }
}
impl Download {
    /// Bucket-only flag that the selected subcommand does not support, if any.
    ///
    /// The model and needle subcommands transfer weights and engines rather than buckets, so bucket transfer options are rejected for them.
    /// Flags are reported in a fixed order so a combination of unsupported flags always names the same one.
    pub(crate) fn unsupported_bucket_flag(&self) -> Option<&'static str> {
        let flags = [
            ("--flatten", self.flatten),
            ("--clobber", self.clobber),
            ("--strict", self.strict),
            ("--canonicalize", self.canonicalize),
            ("--archive-format", self.archive_format.is_some()),
        ];
        match &self.command {
            | Some(DownloadCommands::Model(_) | DownloadCommands::Needle) => flags.into_iter().find(|(_, selected)| *selected).map(|(flag, _)| flag),
            | _ => None,
        }
    }
}
impl FileFormat {
    /// Return whether the format supports batches of structured records.
    pub fn is_batch(&self) -> bool {
        self.is_json() || self.is_json_lines() || self.is_yaml() || self.is_zon()
    }
    /// Return whether the format is structured data.
    pub fn is_structured(&self) -> bool {
        self.is_cff() || self.is_json_ld() || self.is_rdf_xml() || self.is_turtle() || self.is_batch()
    }
}
impl Default for InferenceArgs {
    fn default() -> Self {
        Self::new(InferenceBackendKind::default())
    }
}
impl InferenceArgs {
    pub(crate) fn new(backend: InferenceBackendKind) -> Self {
        Self::init().backend(backend).build()
    }
}
impl Validate for InferenceArgs {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if Milliseconds::try_from(self.timeout).is_err() {
            report.add(
                "timeout",
                ValidationError::new("range").with_message("Inference timeout must be non-negative and fit in milliseconds"),
            );
        }
        match self.backend {
            | InferenceBackendKind::Acp => {
                if self.base_url.is_some() {
                    report.add(
                        "baseUrl",
                        ValidationError::new("backend").with_message("--base-url is only valid with --backend openai"),
                    );
                }
                if self.endpoint_mode.is_some() {
                    report.add(
                        "endpointMode",
                        ValidationError::new("backend").with_message("--endpoint-mode is only valid with --backend openai"),
                    );
                }
            }
            | InferenceBackendKind::Openai => {
                [
                    ("agent", "--agent", self.agent.is_some()),
                    ("agentExecutable", "--agent-executable", self.agent_executable.is_some()),
                    ("workingDirectory", "--working-directory", self.working_directory.is_some()),
                ]
                .into_iter()
                .filter(|(_, _, present)| *present)
                .for_each(|(path, flag, _)| {
                    report.add(
                        path,
                        ValidationError::new("backend").with_message(format!("{flag} is only valid with --backend acp")),
                    );
                });
                if self.model.as_ref().is_none_or(|model| model.trim().is_empty()) {
                    report.add(
                        "model",
                        ValidationError::new("required").with_message("--model is required with --backend openai"),
                    );
                }
            }
        }
        report.finish()
    }
}
impl From<&FileFormat> for MimeType {
    fn from(format: &FileFormat) -> Self {
        match format {
            | FileFormat::Pdf => MimeType::Pdf,
            | FileFormat::Bag => MimeType::Zip,
            | FileFormat::Cff => MimeType::Cff,
            | FileFormat::Docx => MimeType::Docx,
            | FileFormat::Json => MimeType::Json,
            | FileFormat::JsonLd => MimeType::LdJson,
            | FileFormat::JsonLines => MimeType::JsonLines,
            | FileFormat::Markdown => MimeType::Markdown,
            | FileFormat::Powerpoint => MimeType::Powerpoint,
            | FileFormat::RdfXml => MimeType::RdfXml,
            | FileFormat::Turtle => MimeType::Turtle,
            | FileFormat::Yaml => MimeType::Yaml,
            | FileFormat::Zon => MimeType::Zon,
        }
    }
}
impl From<ArchiveType> for MimeType {
    fn from(value: ArchiveType) -> Self {
        match value {
            | ArchiveType::SevenZip => Self::SevenZip,
            | ArchiveType::Tar => Self::Tar,
            | ArchiveType::TarGzip => Self::Gzip,
            | ArchiveType::Zip => Self::Zip,
        }
    }
}
impl From<FileFormat> for MimeType {
    fn from(format: FileFormat) -> Self {
        MimeType::from(&format)
    }
}
impl From<&FileFormat> for OutputFormat {
    fn from(format: &FileFormat) -> Self {
        match format {
            | FileFormat::Json => Self::Json,
            | FileFormat::Yaml => Self::Yaml,
            | _ => Self::default(),
        }
    }
}
impl From<FuzzyMode> for VocabularyFuzzyMode {
    fn from(value: FuzzyMode) -> Self {
        match value {
            | FuzzyMode::Off => Self::Off,
            | FuzzyMode::Relaxed => Self::Relaxed,
            | FuzzyMode::Strict => Self::Strict,
        }
    }
}
