use acorn::io::{
    files_all, files_from_git_branch, files_from_git_commit, files_from_gitlab_merge_request, filter_ignored, filter_ignored_with_root, ApiResult,
};
use acorn::prelude::PathBuf;
use acorn::util::constants::env::{CACHE_TTL, DATABASE_BACKEND, DATABASE_PATH, NO_LOCAL_DATABASE};
use acorn::util::{regex_inverse, regex_join};
use bon::Builder;
use clap::builder::{
    styling::{Ansi256Color, AnsiColor},
    Styles,
};
use clap::{ArgAction, Parser, Subcommand, ValueHint};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre;

pub mod arguments;
use arguments::{CheckCategoryArgument, DatabaseBackend, Diagnostic, ExecutorArgument, FileFormat, ReadabilityTypeArgument, Standard};

/// Return type for main function
pub type Void = eyre::Result<(), eyre::Report>;
const PRIMARY_COLOR: u8 = 135;
const STYLES: Styles = Styles::styled()
    .header(Ansi256Color(PRIMARY_COLOR).on_default().bold())
    .usage(Ansi256Color(PRIMARY_COLOR).on_default().bold())
    .literal(AnsiColor::White.on_default())
    .placeholder(Ansi256Color(183).on_default());
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
    /// Flag used to suppress output
    #[builder(default)]
    pub quiet: bool,
    /// Flag used to indicate if offline mode is supported for the command
    #[builder(default)]
    pub supported: bool,
    /// Number of threads used for parallel processing
    #[builder(default = 10)]
    pub threads: usize,
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
    let extensions = Some(vec!["JSON", "YAML"]);
    let CommandOptions {
        branch,
        commit,
        filter,
        ignore,
        merge_request,
        ..
    } = options;
    let (files, local_base) = if *merge_request {
        (files_from_gitlab_merge_request(extensions).await, None)
    } else {
        match commit {
            | Some(hash) => (files_from_git_commit(hash, extensions), None),
            | None => match branch {
                | Some(name) => (files_from_git_branch(name, extensions), None),
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

    match regex_join(&patterns) {
        | Some(pattern) => match local_base {
            | Some(root) => filter_ignored_with_root(files, Some(pattern), root),
            | None => filter_ignored(files, Some(pattern)),
        },
        | None => Ok(files),
    }
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
    disable_help_flag = true,
    disable_version_flag = true,
    next_help_heading = "FLAGS",
    subcommand_help_heading = "COMMANDS",
    override_usage = "acorn [FLAGS] [COMMAND]"
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
    /// Limit number of threads used by rayon for parallel processing
    ///
    /// See Rayon documentation for more information
    #[arg(default_value_t = 10, short, long, value_name = "N", help_heading = "FLAGS")]
    pub threads: usize,
    /// Clear cache data from database (link_cache, validation_history, research_activity_cache tables)
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
    #[arg(short = 'V', long, help = "Prints version information", help_heading = "FLAGS")]
    pub version: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(short, long, action = ArgAction::Help, global = true, help = "Print help (see a summary with '-h')", help_heading = "FLAGS")]
    pub help: Option<bool>,
    #[arg(long, hide = true)]
    pub markdown_help: bool,
}
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
    #[clap(
        verbatim_doc_comment,
        next_help_heading = "FLAGS",
        override_usage = "acorn check [OPTIONS] [FLAGS] <PATH>"
    )]
    #[command(visible_alias = "lint")]
    Check {
        /// Path to check
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
        path: Option<PathBuf>,
        /// Check files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
        branch: Option<String>,
        /// Check files that were changed in a given Git commit or commit range
        ///
        ///     Check files that were changed in the last 10 commits
        ///     &> acorn check --commit HEAD~10..HEAD
        ///
        /// See <https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection> for more information of commit ranges
        #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
        commit: Option<String>,
        /// Disable website checks that require internet connection
        #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
        disable_website_checks: bool,
        /// Include all check results, including suggestions and info messages
        #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
        all: bool,
        /// Exit on first error
        #[arg(short, long = "exit-on-first-error", value_name = "BOOL", help_heading = "FLAGS")]
        exit_on_first_error: bool,
        /// Regular expression pattern(s) applied to absolute paths of files to include during checks
        ///
        /// Only files matching at least one pattern will be processed
        #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        filter: Vec<String>,
        /// Regular expression pattern(s) applied to absolute paths of files to exclude from checking
        ///
        /// Only applies to `--path` values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$,[/]draft.json$"
        #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        ignore: Vec<String>,
        /// Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        /// Prevent `acorn check` from exiting with a non-zero exit code when errors are found
        ///
        /// Useful for certain scenarios such as debugging
        ///
        /// > CAUTION: Use at your own risk
        #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
        no_fail: bool,
        /// Suppress output from external commands (e.g., Vale) while preserving ACORN output — useful for piping
        #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
        raw: bool,
        /// Readability test to use
        #[arg(default_value_t, short, long = "readability-test", value_name = "METRIC", help_heading = "OPTIONS")]
        readability_metric: ReadabilityTypeArgument,
        /// Skip one or more available quality check categories
        #[arg(short = 'S', long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        skip: Vec<CheckCategoryArgument>,
        /// Skip checksum verification when downloading files (e.g., downloading Vale to perform static analysis of prose)
        /// > CAUTION: Use at your own risk
        #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
        skip_verify_checksum: bool,
        /// Metadata standard to validate against
        #[arg(default_value_t, short, long, value_name = "STANDARD", help_heading = "OPTIONS")]
        standard: Standard,
        /// Use compact output format instead of detailed output
        #[arg(short = 't', long, value_name = "BOOL", help_heading = "FLAGS")]
        terse: bool,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Create research resources like GitLab runners, MCP servers, etc.
    ///
    ///     $> acorn create runner --config ./acorn.json
    ///
    ///     $> acorn create runner --group 12345
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
    Doctor {
        /// Auto-correct systems issues where possible
        #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
        fix: bool,
        /// Start interactive TUI to selectively apply fixes
        #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
        interactive: bool,
        /// Choose to output a JSON issue report to stdout
        #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
        report: bool,
        /// Select one or more available diagnostic checks
        #[arg(default_value = "all", short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        check: Vec<Diagnostic>,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Download research activity data from buckets
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
    ///     Download using default configuration in current directory (first match):
    ///     .acorn.json, .acorn.yaml, .acorn.yml
    ///     $> acorn download
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    #[command(visible_alias = "fetch")]
    Download {
        /// Bucket repository URL
        #[arg(value_name = "URL", required = false, value_hint = ValueHint::Url, help_heading = "ARGS")]
        url: Option<String>,
        /// Path to configuration file (alternative to URL argument)
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, conflicts_with = "url", help_heading = "OPTIONS")]
        config: Option<PathBuf>,
        /// Regular expression pattern(s) used to include files while downloading
        ///
        /// Only files matching at least one pattern will be processed
        #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        filter: Vec<String>,
        /// Regular expression pattern(s) to ignore while downloading
        ///
        /// Values augment built-in ignores and can be provided as a comma-separated list
        #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        ignore: Vec<String>,
        /// Path to output directory where the downloaded content should be saved
        #[arg(default_value = "./content", short, long, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
        output: Option<PathBuf>,
        #[command(flatten)]
        verbose: Verbosity,
    },
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
    ///     Export file(s) that were changed in a given Git commit
    ///     $> acorn export --commit <commit hash>
    ///
    ///     Export file(s) that were changed in latest Git commit
    ///     $> acorn export --commit HEAD
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Export {
        /// Path of input files to be exported
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
        path: Option<PathBuf>,
        /// Export files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
        branch: Option<String>,
        /// Combine all export artifacts into a single artifact
        #[arg(long = "combine", value_name = "BOOL", help_heading = "FLAGS")]
        combine: bool,
        /// Export files that were changed in a given Git commit
        #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
        commit: Option<String>,
        /// Export target file format
        #[arg(default_value_t, short, long, value_name = "FORMAT", help_heading = "OPTIONS")]
        format: FileFormat,
        /// Regular expression pattern(s) applied to absolute paths of files to include during export
        ///
        /// Only files matching at least one pattern will be processed
        #[arg(long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        filter: Vec<String>,
        /// Regular expression pattern(s) applied to absolute paths of files to exclude from export process
        ///
        /// Only applies to `--path` values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$,[/]draft.json$"
        #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        ignore: Vec<String>,
        /// Export files that were changed in a given merge request (Gitlab) or pull request (GitHub)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        /// Path to output directory where the export artifacts should be saved
        #[arg(default_value = "./export", short, long, value_name = "DIRECTORY", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
        output: Option<PathBuf>,
        /// Suppress output from external commands while preserving ACORN output — useful for piping
        #[arg(long, value_name = "BOOL", help_heading = "FLAGS")]
        raw: bool,
        /// Path to reference file to be used when exporting to PowerPoint
        ///
        /// Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
        reference: Option<PathBuf>,
        /// Metadata standard to validate against
        #[arg(default_value_t, short, long, value_name = "STANDARD", help_heading = "OPTIONS")]
        standard: Standard,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Formats research activity data in place (inherently includes some elements of `acorn check`)
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
    Format {
        /// Path to look for files to format
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
        path: Option<PathBuf>,
        /// Format files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
        branch: Option<String>,
        /// Format files that were changed in a given Git commit
        #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
        commit: Option<String>,
        /// Run format without making changes to target file(s). Will print a diff of changes.
        #[arg(short, long = "dry-run", value_name = "BOOL", help_heading = "FLAGS")]
        dry_run: bool,
        /// Regex pattern applied to absolute paths of files to include in formatting process
        ///
        /// Only files matching the pattern will be processed
        #[arg(long, value_name = "REGEX", help_heading = "OPTIONS")]
        filter: Option<String>,
        /// Regex pattern applied to absolute paths of files that determines whether they should be included in formatting process
        ///
        /// Only applies to path values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$"
        #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
        ignore: Option<String>,
        /// Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        /// Metadata standard to validate against
        #[arg(default_value_t, short, long, value_name = "STANDARD", help_heading = "OPTIONS")]
        standard: Standard,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Gather (extract) research activity data
    ///
    ///     $> acorn gather /path/to/data/
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    #[command(visible_alias = "harvest")]
    Gather {
        /// Path to look for files to process
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
        path: Option<PathBuf>,
        /// Regex pattern applied to absolute paths of files to include in processing
        ///
        /// Only files matching the pattern will be processed
        #[arg(long, value_name = "REGEX", help_heading = "OPTIONS")]
        filter: Option<String>,
        /// Regex pattern applied to absolute paths of files that determines whether they should be included in processing
        ///
        /// Only applies to path values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$"
        #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
        ignore: Option<String>,
        /// Process files that were changed in a given merge request (GitLab) or pull request (GitHub)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        /// Metadata standard to validate against
        #[arg(default_value_t, short, long, value_name = "STANDARD", help_heading = "OPTIONS")]
        standard: Standard,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Add linked data context to research activity data
    ///
    ///     $> acorn link /path/to/data/
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Link {
        /// Path to look for files to process
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::AnyPath, help_heading = "OPTIONS")]
        path: Option<PathBuf>,
        /// Process files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
        branch: Option<String>,
        /// Process files that were changed in a given Git commit
        #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
        commit: Option<String>,
        /// Run link without making changes to target file(s). Will print a diff of changes.
        #[arg(short, long = "dry-run", value_name = "BOOL", help_heading = "FLAGS")]
        dry_run: bool,
        /// Regex pattern applied to absolute paths of files that determines whether they should be included in processing
        ///
        /// Only applies to path values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$"
        #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
        ignore: Option<String>,
        /// Processes files that were changed in a given merge request (Gitlab) or pull request (GitHub)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        /// Metadata standard to validate against
        #[arg(default_value_t, short, long, value_name = "STANDARD", help_heading = "OPTIONS")]
        standard: Standard,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Print research activity data (RAD) or research activity identifier (RAiD) metadata JSON schema to stdout
    ///
    ///     Print RAD JSON schema (default)
    ///     $> acorn schema
    ///
    ///     Print RAD JSON schema
    ///     $> acorn schema rad
    ///
    ///     Print RAiD JSON schema
    ///     $> acorn schema raid
    #[clap(verbatim_doc_comment)]
    #[command(long_about = None)]
    Schema {
        #[command(subcommand)]
        command: Option<SchemaCommands>,
    },
    /// Launch the interactive terminal user interface (TUI)
    ///
    ///     $> acorn tui
    ///
    #[clap(verbatim_doc_comment)]
    Tui,
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum CreateCommands {
    /// Model context provider (MCP) server
    Mcp,
    /// GitLab runner
    Runner {
        /// Path to configuration file for creating the resource
        #[arg(short, long, default_value = "./", value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
        config: Option<PathBuf>,
        /// Runner description (alternative to providing a configuration file)
        #[arg(short, long, value_name = "DESCRIPTION", help_heading = "OPTIONS")]
        description: Option<String>,
        /// GitLab runner executor type
        #[arg(short, long, value_name = "TYPE", help_heading = "OPTIONS")]
        executor: Option<ExecutorArgument>,
        /// GitLab group ID to create a runner for (alternative to providing a configuration file)
        #[arg(short, long, conflicts_with = "project", value_name = "ID", help_heading = "OPTIONS")]
        group: Option<u64>,
        /// GitLab repository domain override for runner creation (e.g., code.ornl.gov)
        #[arg(short = 'r', long, alias = "domain", value_name = "REPO", help_heading = "OPTIONS")]
        repo: Option<String>,
        /// GitLab project ID to create a runner for (alternative to providing a configuration file)
        #[arg(short, long, conflicts_with = "group", value_name = "ID", help_heading = "OPTIONS")]
        project: Option<u64>,
        /// Runner name (used as Docker container name)
        #[arg(short = 'n', long, value_name = "NAME", help_heading = "OPTIONS")]
        name: Option<String>,
        /// Hosting server location (for use during registration)
        #[arg(short, long, value_name = "SERVER", help_heading = "OPTIONS")]
        server: Option<String>,
        /// Tags to register the runner with (alternative to providing a configuration file)
        #[arg(
            short,
            long,
            conflicts_with = "untagged",
            value_name = "LIST",
            value_delimiter = ',',
            help_heading = "OPTIONS"
        )]
        tags: Vec<String>,
        /// Whether this runner can pick up jobs without matching tags (alternative to providing a configuration file)
        #[arg(short, long, conflicts_with = "tags", help_heading = "FLAGS")]
        untagged: bool,
        #[command(flatten)]
        verbose: Verbosity,
    },
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum SchemaCommands {
    /// Print research activity data (RAD) JSON schema to stdout
    Rad,
    /// Print research activity identifier (RAiD) metadata JSON schema to stdout
    Raid,
}

#[cfg(test)]
mod tests;
