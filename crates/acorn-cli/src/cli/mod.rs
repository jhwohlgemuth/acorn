use acorn::io::{
    files_all, files_from_git_branch, files_from_git_commit, files_from_gitlab_merge_request, filter_ignored, filter_ignored_with_root, ApiResult,
    FromPath, Source, SourceAction,
};
use acorn::util::constants::app::DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT;
use acorn::util::constants::env::{CACHE_TTL, DATABASE_BACKEND, DATABASE_PATH, NO_LOCAL_DATABASE};
use acorn_core::util::{regex_inverse, regex_join, MimeType};
use acorn_schema::research_activity::ResearchActivity;
use arguments::DatabaseBackend;
use bon::Builder;
use clap::builder::{
    styling::{Ansi256Color, AnsiColor},
    Styles,
};
use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre;
use jiff::SignedDuration;
use parse::parse_watch_duration;
use std::{fs::read_to_string, path::PathBuf};
pub mod arguments;
pub mod parse;
const PRIMARY_COLOR: u8 = 135;
const STYLES: Styles = Styles::styled()
    .header(Ansi256Color(PRIMARY_COLOR).on_default().bold())
    .usage(Ansi256Color(PRIMARY_COLOR).on_default().bold())
    .literal(AnsiColor::White.on_default())
    .placeholder(Ansi256Color(183).on_default());
const SUPPORTED_EXTENSIONS: [MimeType; 6] = [
    MimeType::Json,
    MimeType::Jsonc,
    MimeType::Markdown,
    MimeType::Pdf,
    MimeType::Yaml,
    MimeType::Zon,
];
/// Return type for main function
pub type Void = eyre::Result<(), eyre::Report>;
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum Commands {
    /// Perform static analysis on research activity data and apply standardized best practices
    ///
    ///     Check a file
    ///     $> acorn check /path/to/file.json
    ///
    ///     Check all supported files in a folder
    ///     $> acorn check /path/to/folder
    ///
    ///     Skip static analysis
    ///     $> acorn check /path/to/folder --skip schema
    ///
    ///     Skip schema validation
    ///     $> acorn check /path/to/folder --skip prose,schema
    ///
    ///     Check file(s) that were changed in a given Git commit
    ///     $> acorn check --commit <commit hash>
    ///
    ///     Check file(s) that were changed in latest Git commit
    ///     $> acorn check --commit HEAD
    ///
    ///     Check text provided through standard input
    ///     $> echo 'Text to check' | acorn check
    ///
    ///     Check text copied to the clipboard
    ///     $> acorn check --paste
    ///
    ///     Check YAML copied to the clipboard as RAD
    ///     $> acorn check --paste --format yaml --standard rads
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    #[command(visible_alias = "lint")]
    Check(Box<arguments::Check>),
    /// Create research resources like GitLab runners, MCP servers, etc.
    ///
    ///     $> acorn create runner --config ./acorn.json
    ///
    ///     $> acorn create runner --group 12345
    ///
    ///     Generate a secret and copy it to the system clipboard
    ///     $> acorn create secret --paste
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Create {
        #[command(subcommand)]
        command: Option<CreateCommands>,
    },
    /// Diagnose and correct system requirements for using acorn
    ///
    ///     Print system diagnostics and identify issues
    ///     $> acorn doctor
    ///
    ///     Auto-correct systems issues
    ///     $> acorn doctor --fix
    ///
    ///     Start interactive TUI to selectively apply fixes
    ///     $> acorn doctor --fix --interactive
    ///
    ///     Generate JSON report for use when filing bug reports
    ///     $> acorn doctor --report
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Doctor(Box<arguments::Doctor>),
    /// Download research activity data from buckets or model weights for local inference
    ///
    ///     Download research activity data from a bucket repository URL
    ///     $> acorn download https://github.com/user/repo
    ///
    ///     Download research activity data from a list of buckets
    ///     $> acorn download --config /path/to/.acorn.json
    ///
    ///     Download research activity data to a specific output directory
    ///     $> acorn download --config /path/to/.acorn.yml --output /path/to/output
    ///
    ///     Replace selected files that already exist in the output directory
    ///     $> acorn download https://github.com/user/repo --output ./content --clobber
    ///
    ///     Download using default configuration in current directory (first match):
    ///     .acorn.json, .acorn.jsonc, .acorn.yaml, .acorn.yml
    ///     $> acorn download
    ///
    ///     Download default GGUF (Q4_K_M) model from a Hugging Face repository
    ///     $> acorn download model meta-llama/Llama-3.1-8B
    ///
    ///     Download a specific quantization
    ///     $> acorn download model meta-llama/Llama-3.1-8B --filter "Q8_0.*\\.gguf$"
    ///
    ///     Exclude low-quality quantizations
    ///     $> acorn download model meta-llama/Llama-3.1-8B --ignore "Q2_|Q3_|imatrix"
    ///
    ///     Download models from a config file
    ///     $> acorn download model --config .acorn.json
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    #[command(visible_alias = "fetch")]
    Download(Box<arguments::Download>),
    /// Export research activity data to a specific target
    ///
    ///     $> acorn export /path/to/data
    ///
    ///     $> acorn export /path/to/data/data.json -vv
    ///
    ///     $> acorn export /path/to/data --target highlight --format powerpoint
    ///
    ///     $> acorn export /path/to/data -t poster -o /path/to/output
    ///
    ///     Export folder as a BagIt archive to /path/to/bag.zip
    ///     $> acorn export /path/to/data --format bag --output /path/to/bag
    ///
    ///     Export an ordered JSON Lines metadata batch
    ///     $> acorn export /path/to/data.ndjson --format jsonl --from datacite --to dcat
    ///
    ///     Export file(s) that were changed in a given Git commit
    ///     $> acorn export --commit <commit hash>
    ///
    ///     Export file(s) that were changed in latest Git commit
    ///     $> acorn export --commit HEAD
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Export(Box<arguments::Export>),
    /// Run bounded inference through Needle or a configured prompt backend
    ///
    ///     Use the pinned official runner
    ///     $> acorn infer "Validate this research activity"
    ///
    ///     Read the prompt from standard input
    ///     $> echo "Validate this research activity" | acorn infer
    ///
    ///     Use a local or private-registry ModelKit
    ///     $> acorn infer "Validate this" --custom-location oci://harbor.example/acorn/needle@sha256:...
    ///
    ///     Preview a hydrated prompt without starting a backend
    ///     $> acorn infer run summarize activity.json --dry-run
    ///
    ///     Run through OpenCode ACP (the default backend)
    ///     $> acorn infer run summarize activity.json
    ///
    ///     Run through an OpenAI-compatible endpoint
    ///     $> acorn infer run summarize --backend openai --model qwen3 --base-url http://localhost:8080
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS", visible_alias = "ask")]
    Infer(Box<arguments::Infer>),
    /// Formats supported metadata standards in place (inherently includes some elements of `acorn check`)
    ///
    ///     $> acorn format /path/to/file.json
    ///
    ///     $> acorn format /path/to/folder --dry-run
    ///
    ///     Format file(s) that were changed in a given Git commit
    ///     $> acorn format --commit <commit hash>
    ///
    ///     Format file(s) that were changed in latest Git commit
    ///     $> acorn format --commit HEAD
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    #[command(visible_alias = "fmt")]
    Format(Box<arguments::Format>),
    /// Gather persistent identifiers from documents, URLs, or text
    ///
    ///     $> acorn gather report.docx --text "doi:10.1234/example"
    ///
    /// Directory inputs are searched recursively. Use `--max-depth 1` to search only direct children.
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    #[command(visible_alias = "harvest")]
    Gather(Box<arguments::Gather>),
    /// Import external metadata into ACORN
    ///
    ///     Generate an endpoint object from a local API spec
    ///     $> acorn import spec ./openapi.yaml --name example::api --domain api.example.com
    ///
    ///     Preview generated endpoint JSON without writing
    ///     $> acorn import spec https://example.com/openapi.yaml --name example::api --domain api.example.com --dry-run
    ///
    ///     Import model metadata into the local database
    ///     $> acorn import model
    #[clap(verbatim_doc_comment)]
    #[command(long_about = None, visible_alias = "load")]
    Import(Box<arguments::Import>),
    /// Add linked data context to research activity data
    ///
    ///     $> acorn link /path/to/data/
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Link(Box<arguments::Link>),
    /// Generate bounded artifacts and publish immutable GitLab packages
    ///
    ///     $> acorn report --through 2026-10-01T00:00:00Z --commit "$CI_COMMIT_SHA"
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Report(Box<arguments::Report>),
    /// Calculate or verify a Software Hash Identifier (SWHID)
    ///
    ///     Hash a local file or directory
    ///     $> acorn swhid ./src
    ///
    ///     Calculate the current Git revision
    ///     $> acorn swhid . --kind revision --reference HEAD
    ///
    ///     Verify a local object
    ///     $> acorn swhid ./src --verify swh:1:dir:0123456789abcdef0123456789abcdef01234567
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Swhid(Box<arguments::Swhid>),
    /// Print the embedded ACORN skill path and copy agent instructions to clipboard
    ///
    ///     $> acorn skill
    ///
    #[clap(verbatim_doc_comment)]
    Skill,
    /// Start a server (bot, MCP, etc.)
    ///
    ///     Start a GitLab bot server
    ///     $> acorn serve bot <project-id>
    ///
    ///     Start an MCP server
    ///     $> acorn serve mcp
    ///
    #[clap(verbatim_doc_comment)]
    Serve {
        #[command(subcommand)]
        command: Option<ServeCommands>,
    },
    /// Synchronize local inference configuration from ACORN model entries
    ///
    ///     Synchronize configurations for installed applications
    ///     $> acorn sync
    ///
    ///     Include OpenCode configuration
    ///     $> acorn sync --opencode
    ///
    ///     Include llama-swap configuration
    ///     $> acorn sync --llama-swap
    ///
    ///     Include VS Code configuration
    ///     $> acorn sync --vscode
    ///
    ///     Include Goose CLI configuration
    ///     $> acorn sync --goose
    ///
    ///     Preview changes without writing files
    ///     $> acorn sync --dry-run
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Sync(Box<arguments::Sync>),
    /// Launch the interactive terminal user interface (TUI)
    ///
    ///     $> acorn tui
    ///
    #[clap(verbatim_doc_comment)]
    Tui,
    /// Execute constrained Rhai extensions
    ///
    ///     Evaluate a script against an input file
    ///     $> acorn exec check ./script.rhai ./input.json
    ///
    ///     Perform an evaluation-free static check
    ///     $> acorn exec check-safe ./script.rhai
    ///
    /// ABI version: `// @acorn-exec-v0.2.0`
    ///
    /// Hook evaluations contain `hook`, `path`, `index`, `input`, and `proposed`; scripts return a map containing a `findings` array.
    /// Hook source is limited to 512 KiB; evaluation is bounded and does not permit dynamic evaluation or imports.
    ///
    /// `automation` runs trusted local scripts with filesystem and process access. It is separate from manifest-authorized workflow hooks.
    ///
    #[clap(verbatim_doc_comment)]
    Exec {
        #[command(subcommand)]
        command: Option<ExecCommands>,
    },
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum CreateCommands {
    /// Model context provider (MCP) server
    Mcp(Box<arguments::CreateMcp>),
    /// GitLab runner
    Runner(Box<arguments::Runner>),
    /// Bot server container
    Bot(Box<arguments::bot::Create>),
    /// Cryptographically secure 256-bit secret
    Secret {
        /// Copy the generated value to the system clipboard
        #[arg(long, visible_alias = "clipboard")]
        paste: bool,
    },
}
#[derive(Clone, Debug, Subcommand)]
#[command(long_about = None)]
pub enum DownloadCommands {
    /// Install the pinned Chromium build used by PDF export
    #[command(hide = true)]
    Chromium,
    /// Download model weights for use with ACORN research harnesses and local inference
    Model(Box<arguments::DownloadModel>),
    /// Download and verify the pinned Needle 3 engine, model, and license for this platform
    Needle,
    /// Install the pinned Vale build used by prose analysis
    #[command(hide = true)]
    Vale,
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum ExecCommands {
    /// Run a trusted local Rhai automation script.
    Automation(Box<arguments::ExecAutomation>),
    /// Evaluate a script against supplied input and render findings
    Check(Box<arguments::ExecCheck>),
    /// Check Rhai source without evaluating it
    CheckSafe(Box<arguments::ExecCheckSafe>),
    /// Validate or evaluate a manifest-authorized workflow hook.
    Hook {
        #[command(subcommand)]
        command: Option<ExecHookCommands>,
    },
    /// Validate executable-hook deployment manifests.
    Manifest {
        #[command(subcommand)]
        command: Option<ExecManifestCommands>,
    },
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum ExecHookCommands {
    /// Evaluate one hook without applying its proposals.
    Check(Box<arguments::ExecHookCheck>),
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum ExecManifestCommands {
    /// Parse and validate a script manifest.
    Check(Box<arguments::ExecManifestCheck>),
}
#[derive(Clone, Debug, Subcommand)]
#[command(long_about = None)]
pub enum ImportCommands {
    /// Import model metadata into the local database
    Model(Box<arguments::ImportModel>),
    /// Import endpoint resources from an API specification
    #[command(name = "spec", visible_alias = "openapi")]
    Spec(Box<arguments::Spec>),
}
#[derive(Clone, Debug, Subcommand)]
#[command(long_about = None)]
pub enum InferCommands {
    /// Hydrate and run one known embedded prompt asset
    Run(Box<arguments::PromptRun>),
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum ServeCommands {
    /// Webhook/API bot server that polls GitLab for merge request note events
    Bot(Box<arguments::bot::Serve>),
    /// Authenticated Protobuf/gRPC server
    Grpc(arguments::ServeGrpc),
    /// Model context protocol (MCP) server
    Mcp(arguments::ServeMcp),
    /// Authenticated JSON-RPC HTTP server
    Rpc(arguments::ServeRpc),
}
/// Filesystem notification strategy used by watch mode.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum WatchMode {
    /// Prefer native notifications and use polling when required.
    #[default]
    Auto,
    /// Require native filesystem notifications.
    Native,
    /// Poll local and network sources.
    Poll,
}
/// "Plant an ACORN and grow your science"
///
///⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣄⣀⠀⠀⠀
///⠀⠀⠀⠀⠀⠀⠀⣀⠀⢴⣶⠀⢶⣦⠀⢄⣀⠀⠠⢾⣿⠿⠿⠿⠿⢦⠀
///⠀⠀⠀⠀⠀⠀⠺⠿⠇⢸⣿⣇⠘⣿⣆⠘⣿⡆⠠⣄⡀⠀⠀⠀⠀⠀⠀
///⠀⠀⠀⠀⢀⣴⣶⣶⣤⣄⡉⠛⠀⢹⣿⡄⢹⣿⡀⢻⣧⠀⡀⠀⠀⠀⠀
///⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡈⠓⠀⣿⣧⠈⢿⡆⠸⡄⠀⠀⠀   █████████     █████████     ███████    ███████████   ██████   █████
///⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣈⠙⢆⠘⣿⡀⢻⠀⠀   ███▒▒▒▒▒███   ███▒▒▒▒▒███  ███▒▒▒▒▒███ ▒▒███▒▒▒▒▒███ ▒▒██████ ▒▒███
///⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠹⣧⠈⠀⠀  ▒███    ▒███  ███     ▒▒▒  ███     ▒▒███ ▒███    ▒███  ▒███▒███ ▒███
///⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠈⠃⠀⠀⠀ ▒███████████ ▒███         ▒███      ▒███ ▒██████████   ▒███▒▒███▒███
///⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀ ▒███▒▒▒▒▒███ ▒███         ▒███      ▒███ ▒███▒▒▒▒▒███  ▒███ ▒▒██████⠀
///⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀ ▒███    ▒███ ▒▒███     ███▒▒███     ███  ▒███    ▒███  ▒███  ▒▒█████
///⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀ █████   █████ ▒▒█████████  ▒▒▒███████▒   █████   █████ █████  ▒▒█████⠀
///⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀▒▒▒▒▒   ▒▒▒▒▒   ▒▒▒▒▒▒▒▒▒     ▒▒▒▒▒▒▒    ▒▒▒▒▒   ▒▒▒▒▒ ▒▒▒▒▒    ▒▒▒▒▒
///⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠉⠀⠀⠀⠀⠀⠀⠀
/// ⠀ ⣿⣿⠿⠿⠿⠿⠿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀~ Accessible Content Optimization for Research Needs ~⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
///
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about,
    verbatim_doc_comment,
    styles(STYLES),
    next_help_heading = "FLAGS",
    subcommand_help_heading = "COMMANDS"
)]
pub struct Arguments {
    /// Select database backend (`sqlite` or `duckdb`) for local cache/history storage
    #[arg(
        long = "database-backend",
        value_name = "BACKEND",
        env = DATABASE_BACKEND,
        help_heading = "OPTIONS"
    )]
    pub database_backend: Option<DatabaseBackend>,
    /// Override local database path (defaults to cache directory when omitted)
    #[arg(
        short = 'P',
        long = "database-path",
        value_name = "PATH",
        value_hint = ValueHint::FilePath,
        conflicts_with = "no_local_database",
        env = DATABASE_PATH,
        help_heading = "OPTIONS"
    )]
    pub database_path: Option<PathBuf>,
    /// Disable local database initialization at startup
    #[arg(
        short = 'N',
        long = "no-local-database",
        value_name = "BOOL",
        conflicts_with = "database_path",
        env = NO_LOCAL_DATABASE,
        help_heading = "FLAGS"
    )]
    pub no_local_database: bool,
    /// Launch interactive terminal user interface (TUI)
    #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
    pub interactive: bool,
    /// Prevent communication with the internet - intended for disconnected local environments
    ///
    /// Note: Use of --offline may require extra configuration options for certain commands
    #[arg(short = 'X', long, value_name = "BOOL", help_heading = "FLAGS")]
    pub offline: bool,
    /// Re-run check or gather when input content changes
    #[arg(short = 'w', long, global = true, value_name = "BOOL", help_heading = "FLAGS")]
    pub watch: bool,
    /// Select automatic, native, or polling watch behavior
    #[arg(long, global = true, value_enum, default_value_t = WatchMode::Auto, requires = "watch", value_name = "MODE", help_heading = "OPTIONS")]
    pub watch_mode: WatchMode,
    /// Set the interval used by watch polling and Vale sync retries
    #[arg(short = 'p', long, value_parser = parse_watch_duration, value_name = "DURATION", help_heading = "OPTIONS")]
    pub poll_interval: Option<SignedDuration>,
    /// Limit concurrent work used for parallel processing
    #[arg(default_value_t = 10, short, long, value_name = "N", help_heading = "FLAGS")]
    pub threads: usize,
    /// Clear database cache data and downloaded Chromium artifacts
    #[arg(long = "clear-cache", value_name = "BOOL", help_heading = "FLAGS")]
    pub clear_cache: bool,
    /// Clear all data from database (all tables)
    #[arg(long = "reset-database", value_name = "BOOL", help_heading = "FLAGS")]
    pub reset_database: bool,
    /// Disable automatic cache cleanup on startup (expires old cache entries)
    #[arg(long = "no-clear-cache", value_name = "BOOL", help_heading = "FLAGS")]
    pub no_clear_cache: bool,
    /// Override default cache TTL (in seconds). Default is 2592000 (30 days)
    ///
    /// Set to 0 to disable caching
    #[arg(long = "cache-ttl", value_name = "SECONDS", env = CACHE_TTL, help_heading = "OPTIONS")]
    pub cache_ttl: Option<u64>,
    #[command(flatten)]
    pub verbose: Verbosity,
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(long, hide = true)]
    pub markdown_help: bool,
}
/// Container struct for working with subcommand run functions
#[derive(Builder, Clone, Debug, Default)]
#[builder(start_fn = init)]
pub struct CommandOptions {
    /// Path to file or folder to be used for input
    pub path: Option<PathBuf>,
    /// Git branch name
    pub branch: Option<String>,
    /// Git commit hash
    pub commit: Option<String>,
    /// Regex pattern of files to include at a given path desginated by `path`
    pub filter: Option<String>,
    /// Regex pattern of files to ignore at a given path desginated by `path`
    pub ignore: Option<String>,
    /// Path to file or folder to be used for output
    pub output: Option<PathBuf>,
    /// Runtime source selector for commands that process one selected input at a time
    pub selector: Option<Source>,
    /// Runtime source materialization action for commands that copy, symlink, or reference sources
    pub action: Option<SourceAction>,
    /// Path to the local database used for cache/history storage
    pub database_path: Option<PathBuf>,
    /// Path to reference file
    ///
    /// e.g. reference.pptx for exporting RAD to PowerPoint
    pub reference: Option<PathBuf>,
    /// Flag used to indicate if changed files should be obtained from a merge request
    #[builder(default)]
    pub merge_request: bool,
    /// Flag used to indicate if ACORN is running in offline mode
    #[builder(default)]
    pub offline: bool,
    /// Flag used to preview changes without writing files
    #[builder(default)]
    pub dry_run: bool,
    /// Flag used to disable colored output
    #[builder(default)]
    pub no_color: bool,
    /// Runtime output verbosity
    #[builder(default)]
    pub verbose: Verbosity,
    /// Flag used to disable local database access
    #[builder(default)]
    pub no_local_database: bool,
    /// Flag used to disable automatic GGUF quantization repository discovery
    #[builder(default)]
    pub no_fallback: bool,
    /// Maximum number of repositories considered during GGUF fallback discovery
    #[builder(default = 200)]
    pub search_limit: usize,
    /// Minimum number of downloads required for a GGUF fallback repository
    #[builder(default = DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT)]
    pub minimum_download_count: u64,
    /// Flag used to select a GGUF fallback repository interactively
    #[builder(default)]
    pub interactive: bool,
    /// Flag used to suppress output
    #[builder(default)]
    pub quiet: bool,
    /// Flag used to skip checksum verification when supported by a command
    #[builder(default)]
    pub skip_verify_checksum: bool,
    /// Flag used to indicate if offline mode is supported for the command
    #[builder(default)]
    pub supported: bool,
    /// Number of threads used for parallel processing
    #[builder(default = 10)]
    pub threads: usize,
}
/// Runtime watch configuration.
#[derive(Clone, Copy, Debug)]
pub struct WatchOptions {
    /// Watcher backend selection.
    pub mode: WatchMode,
    /// Filesystem and network polling interval.
    pub poll_interval: SignedDuration,
    /// Whether any watched source requires network polling.
    pub network: bool,
}
/// Resolve paths supported by structured export, including JSON Lines batches.
pub(crate) async fn resolve_export_paths(path: &Option<PathBuf>, options: &CommandOptions) -> ApiResult<Vec<PathBuf>> {
    let supported_extensions = SUPPORTED_EXTENSIONS.iter().cloned().chain([MimeType::JsonLines]).collect();
    resolve_paths_with_extensions(path, options, &[], supported_extensions).await
}
/// Returns a vector of PathBuf from the given options
///
/// If the options specify a merge request, the files from the current branch are returned.
/// If the options specify a commit, the files changed in the commit are returned.
/// If the options specify a branch, the files changed in the branch are returned.
/// If none of the above options are set, the files in the given path are returned.
/// If the options include a filter regex, only matching files are returned.
/// If the options include an ignore regex, it is applied to the files returned.
pub async fn resolve_paths(path: &Option<PathBuf>, options: &CommandOptions) -> ApiResult<Vec<PathBuf>> {
    resolve_paths_including(path, options, &[]).await
}
/// Resolve supported paths while also accepting exact extensionless file names.
pub(crate) async fn resolve_paths_including(path: &Option<PathBuf>, options: &CommandOptions, file_names: &[&str]) -> ApiResult<Vec<PathBuf>> {
    resolve_paths_with_extensions(path, options, file_names, SUPPORTED_EXTENSIONS.to_vec()).await
}
async fn resolve_paths_with_extensions(
    path: &Option<PathBuf>,
    options: &CommandOptions,
    file_names: &[&str],
    supported_extensions: Vec<MimeType>,
) -> ApiResult<Vec<PathBuf>> {
    let extensions = file_names.is_empty().then(|| {
        supported_extensions
            .iter()
            .map(MimeType::file_type)
            .chain(supported_extensions.contains(&MimeType::JsonLines).then(|| "ndjson".to_string()))
            .collect::<Vec<_>>()
    });
    let CommandOptions {
        branch,
        commit,
        filter,
        ignore,
        merge_request,
        ..
    } = options;
    let (files, local_base) = if *merge_request {
        (files_from_gitlab_merge_request(extensions.clone()).await, None)
    } else {
        match commit {
            | Some(hash) => (files_from_git_commit(hash, extensions.clone()), None),
            | None => match branch {
                | Some(name) => (files_from_git_branch(name, extensions.clone()), None),
                | None => {
                    let value = match path {
                        | Some(x) => x.clone(),
                        | None => PathBuf::from("."),
                    };
                    (files_all(value.clone(), extensions), Some(value))
                }
            },
        }
    };
    let patterns = vec![ignore.clone(), filter.clone().map(regex_inverse)]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let is_exact = path.as_ref().is_some_and(|value| value.is_file());
    let files = files
        .into_iter()
        .filter(|candidate| {
            let mime = MimeType::from_path(candidate);
            let supported_name = candidate
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| file_names.contains(&name));
            let supported_file = is_exact || supported_name || supported_extensions.contains(&mime);
            supported_file && (mime != MimeType::Markdown || is_exact || read_to_string(candidate).is_ok_and(ResearchActivity::is_markdown))
        })
        .collect::<Vec<_>>();
    match regex_join(&patterns) {
        | Some(pattern) => match local_base {
            | Some(root) => filter_ignored_with_root(files, Some(pattern), root),
            | None => filter_ignored(files, Some(pattern)),
        },
        | None => Ok(files),
    }
}
#[cfg(test)]
mod tests;
