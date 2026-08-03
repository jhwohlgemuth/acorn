use super::parse::{parse_check_category, parse_executor, parse_readability, parse_ssh_remote, parse_standard};
use super::DownloadCommands;
use acorn::analyzer::{self, Standard};
use acorn::io::{Executor, Remote};
use acorn::prelude::PathBuf;
use acorn::schema::agent::Quantization;
use acorn::schema::hardware::memory::Memory;
use acorn::util::constants::app::{DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT, DEFAULT_HUGGINGFACE_SEARCH_LIMIT};
use acorn::util::constants::env::{CHROME_PATH, MINIMUM_DOWNLOAD_COUNT, SEARCH_LIMIT};
use acorn::util::MimeType;
use clap::ValueEnum;
use clap::{Args, ValueHint};
use clap_verbosity_flag::Verbosity;
use derive_more::Display;
use strum::EnumIs;
/// Remote Docker target arguments shared by container-producing create commands.
#[derive(Clone, Debug, Args)]
pub(crate) struct RemoteTarget {
    /// SSH endpoint of the remote Docker daemon
    ///
    /// The local Docker client and SSH configuration are used. Published ports and volumes belong to the remote host.
    #[arg(long, value_name = "URI", value_parser = parse_ssh_remote, help_heading = "OPTIONS")]
    pub(crate) remote: Option<Remote>,
}
/// Bot command arguments.
pub(crate) mod bot {
    use super::{parse_executor, Executor, RemoteTarget, ValueEnum, Verbosity};
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
        #[arg(short = 'd', long, env = "CI_SERVER_HOST", value_name = "DOMAIN", help_heading = "OPTIONS")]
        pub(crate) domain: Option<String>,
        /// Persistent named volume for bot state
        #[arg(long, value_name = "VOLUME", help_heading = "OPTIONS")]
        pub(crate) volume: Option<String>,
        #[command(flatten)]
        pub(crate) target: RemoteTarget,
        #[command(flatten)]
        pub(crate) verbose: Verbosity,
    }
    /// Arguments for `acorn serve bot`
    #[derive(Clone, Debug, Args)]
    #[command(long_about = None)]
    pub(crate) struct Serve {
        #[command(flatten)]
        pub(crate) common: Common,
        /// Create and run the bot in a detached Docker/Podman container instead of in-process
        #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
        pub(crate) detach: bool,
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

    #[cfg(test)]
    mod tests {
        use super::{Common, EventSource};

        fn common(bind: Option<&str>, port: Option<u16>) -> Common {
            Common {
                identifier: "123".to_string(),
                bind: bind.map(String::from),
                port,
                after: None,
                poll_interval: 10,
                runtime: None,
                event_source: EventSource::Poll,
                public_url: None,
                register_webhook: false,
            }
        }

        #[test]
        fn bind_address_uses_default_host_with_port() {
            assert_eq!(common(None, Some(8080)).bind_address(), "localhost:8080");
            assert_eq!(common(Some("0.0.0.0"), Some(8080)).bind_address(), "0.0.0.0:8080");
            assert_eq!(common(Some("0.0.0.0"), None).bind_address(), "0.0.0.0:3000");
            assert_eq!(common(Some("0.0.0.0:9000"), Some(8080)).bind_address(), "0.0.0.0:8080");
            assert_eq!(common(None, None).bind_address_if_configured(), None);
        }
    }
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
/// Structured output format for raw/dry-run results (JSON or YAML)
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum OutputFormat {
    #[default]
    #[display("JSON")]
    Json,
    #[display("YAML")]
    Yaml,
}
/// Local inference configuration target for model synchronization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EnumIs, ValueEnum)]
pub(crate) enum SyncTarget {
    /// Synchronize all targets
    #[default]
    All,
    /// Synchronize OpenCode only
    Opencode,
    /// Synchronize llama-swap only
    LlamaSwap,
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
        custom code files. The default GGUF-only filter is designed for llama.cpp inference.",
    verbatim_doc_comment
)]
pub(crate) struct DownloadModel {
    /// Model ID/name(s), local path(s), or direct URL(s) to download
    #[arg(value_name = "MODEL", required = false, value_delimiter = ',', help_heading = "ARGS")]
    pub(crate) model: Vec<String>,
    /// URI or local path to a model list file
    ///
    /// Accepts a plain-text list of repository IDs or a JSON/YAML list of IDs or model details
    #[arg(long, value_name = "URI_OR_PATH", help_heading = "OPTIONS")]
    pub(crate) model_file: Option<String>,
    /// Add downloaded models to ACORN and synchronize OpenCode and/or llama-swap configuration
    #[arg(
        long,
        value_enum,
        value_name = "TARGET",
        num_args = 0..=1,
        default_missing_value = "all",
        conflicts_with = "raw",
        help_heading = "OPTIONS"
    )]
    pub(crate) sync: Option<SyncTarget>,
    /// Assume synchronized models exist under the models directory
    #[arg(long, requires = "sync", help_heading = "FLAGS")]
    pub(crate) force: bool,
    /// Path to configuration file listing models to download
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
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
    /// Path to output directory where downloaded model weights should be saved
    ///
    /// Defaults to $HOME/.models when omitted
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) output: Option<PathBuf>,
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
    /// Maximum number of Hugging Face repositories to search when discovering GGUF quantizations
    ///
    /// Applies when the requested repository contains no GGUF files and automatic fallback is enabled
    #[arg(long, value_name = "LIMIT", env = SEARCH_LIMIT, default_value_t = DEFAULT_HUGGINGFACE_SEARCH_LIMIT, help_heading = "OPTIONS")]
    pub(crate) search_limit: usize,
    /// Minimum number of downloads required for a GGUF fallback repository
    #[arg(
        long = "minimum-popularity",
        value_name = "COUNT",
        env = MINIMUM_DOWNLOAD_COUNT,
        default_value_t = DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT,
        help_heading = "OPTIONS"
    )]
    pub(crate) minimum_download_count: u64,
    /// Disable automatic GGUF quantization repository discovery when the target has no GGUF files
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) no_fallback: bool,
    /// Open an interactive picker when GGUF fallback finds multiple repositories
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) interactive: bool,
    /// Preview resolved models and fallbacks without downloading
    #[arg(long = "dry-run", visible_alias = "what-if", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Output structured JSON (or YAML with --format yaml) of resolved models
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) raw: bool,
    /// Output format for --raw results
    #[arg(long, value_name = "FORMAT", default_value = "json", requires = "raw", help_heading = "OPTIONS")]
    pub(crate) format: OutputFormat,
    #[command(flatten)]
    pub(crate) verbose: Verbosity,
}
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct ImportModel {
    /// Hugging Face model repository ID(s) to inspect
    #[arg(value_name = "MODEL", required = false, value_delimiter = ',', help_heading = "ARGS")]
    pub(crate) model: Vec<String>,
    /// URI or local path to a model list file
    ///
    /// Accepts a plain-text list of repository IDs or a JSON/YAML list of IDs or model details
    #[arg(long, value_name = "URI_OR_PATH", help_heading = "OPTIONS")]
    pub(crate) model_file: Option<String>,
    /// Add imported models to ACORN and synchronize OpenCode and/or llama-swap configuration
    #[arg(long, value_enum, value_name = "TARGET", num_args = 0..=1, default_missing_value = "all", help_heading = "OPTIONS")]
    pub(crate) sync: Option<SyncTarget>,
    /// Assume synchronized models exist under the models directory
    #[arg(long, requires = "sync", help_heading = "FLAGS")]
    pub(crate) force: bool,
    /// Resolve model metadata without database or configuration writes
    #[arg(long = "dry-run", visible_alias = "what-if", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Path to configuration file listing models to inspect
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Disable automatic GGUF quantization repository discovery
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) no_fallback: bool,
    /// Open an interactive picker when GGUF fallback finds multiple repositories
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) interactive: bool,
    /// Maximum number of Hugging Face repositories to search during GGUF fallback discovery
    #[arg(long, value_name = "LIMIT", env = SEARCH_LIMIT, default_value_t = DEFAULT_HUGGINGFACE_SEARCH_LIMIT, help_heading = "OPTIONS")]
    pub(crate) search_limit: usize,
    /// Minimum number of downloads required for a GGUF fallback repository
    #[arg(
        long = "minimum-popularity",
        value_name = "COUNT",
        env = MINIMUM_DOWNLOAD_COUNT,
        default_value_t = DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT,
        help_heading = "OPTIONS"
    )]
    pub(crate) minimum_download_count: u64,
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
    #[arg(long, env = CHROME_PATH, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) chrome_path: Option<PathBuf>,
    /// Combine all export artifacts into a single artifact
    #[arg(long = "combine", value_name = "BOOL", help_heading = "FLAGS", conflicts_with = "schema")]
    pub(crate) combine: bool,
    /// Export files that were changed in a given Git commit
    #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) commit: Option<String>,
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
    #[arg(default_value = "pdf", short, long, value_name = "FORMAT", help_heading = "OPTIONS")]
    pub(crate) format: FileFormat,
    /// Regular expression pattern(s) applied to absolute paths of files to include during export
    ///
    /// Only files matching at least one pattern will be processed
    #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) filter: Vec<String>,
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
    /// Suppress output from external commands while preserving ACORN output — useful for piping
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) raw: bool,
    /// Skip one or more available export check categories
    #[arg(short = 'S', long, value_name = "LIST", value_delimiter = ',', value_parser = parse_check_category, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) skip: Vec<analyzer::CheckCategory>,
    /// Path to reference file to be used when exporting to PowerPoint
    ///
    /// Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) reference: Option<PathBuf>,
    /// Metadata standard to validate against
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
    #[arg(default_value = "rads", short, long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS")]
    pub(crate) standard: Standard,
    /// Fail if conversion loses data (only for crosswalk exports)
    #[arg(long, value_name = "BOOL", help_heading = "FLAGS", conflicts_with = "schema")]
    pub(crate) strict: bool,
    /// Metadata standard to use for source data inference (default: auto-detect)
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
    #[arg(long, value_name = "STANDARD", value_parser = parse_standard, help_heading = "OPTIONS", conflicts_with = "schema")]
    pub(crate) from: Option<Standard>,
    /// Target metadata standard for crosswalk conversion
    ///
    /// Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
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
    /// Run format without making changes to target file(s). Will print a diff of changes.
    #[arg(short, long = "dry-run", visible_alias = "what-if", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) dry_run: bool,
    /// Disable color in dry-run output
    #[arg(long = "no-color", value_name = "BOOL", help_heading = "FLAGS")]
    pub(crate) no_color: bool,
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
#[derive(Clone, Debug, Args)]
#[command(
    long_about = "Synchronize local inference configuration from ACORN model entries.\n\
        \n\
        Resolves configured models to downloaded GGUF files and merges them into\n\
        llama-swap and/or OpenCode configuration.\n\
        \n\
        With neither target flag, synchronizes both configurations.\n\
        With --opencode or --llama-swap, synchronizes only the selected target.\n\
        Both flags may be used together.",
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
    /// Assume models exist under the models directory without checking the filesystem
    #[arg(long, help_heading = "FLAGS")]
    pub(crate) force: bool,
    /// Synchronize OpenCode configuration only
    #[arg(long, help_heading = "TARGETS")]
    pub(crate) opencode: bool,
    /// Synchronize llama-swap configuration only
    #[arg(long, help_heading = "TARGETS")]
    pub(crate) llama_swap: bool,
    /// Default directory where downloaded model weights live (overrides config)
    #[arg(long = "models-dir", value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
    pub(crate) models_dir: Option<PathBuf>,
    /// Path to OpenCode configuration file (overrides config)
    #[arg(long = "opencode-config", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) opencode_config: Option<PathBuf>,
    /// Path to llama-swap configuration file (overrides config)
    #[arg(long = "llama-swap-config", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) llama_swap_config: Option<PathBuf>,
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
#[derive(Clone, Debug, Args)]
#[command(long_about = None)]
pub(crate) struct Runner {
    /// Path to configuration file for creating the resource
    #[arg(short, long, default_value = "./", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
    pub(crate) config: Option<PathBuf>,
    /// Runner description (alternative to providing a configuration file)
    #[arg(short, long, value_name = "DESCRIPTION", help_heading = "OPTIONS")]
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
impl From<&FileFormat> for OutputFormat {
    fn from(format: &FileFormat) -> Self {
        match format {
            | FileFormat::Json => Self::Json,
            | FileFormat::Yaml => Self::Yaml,
            | _ => Self::default(),
        }
    }
}
