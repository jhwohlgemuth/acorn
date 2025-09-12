use acorn_lib::analyzer::readability::ReadabilityType;
use acorn_lib::util::cli::*;
use clap::builder::styling::{Ansi256Color, AnsiColor};
use clap::builder::Styles;
use clap::{ArgAction, Parser, Subcommand, ValueHint};
use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;

const PRIMARY_COLOR: u8 = 135;
const STYLES: Styles = Styles::styled()
    .header(Ansi256Color(PRIMARY_COLOR).on_default().bold())
    .usage(Ansi256Color(PRIMARY_COLOR).on_default().bold())
    .literal(AnsiColor::White.on_default())
    .placeholder(Ansi256Color(183).on_default());

/// Multitool for maintaining sustainable research activity data
///
///⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣄⣀⠀⠀⠀
///⠀⠀⠀⠀⠀⠀⠀⣀⠀⢴⣶⠀⢶⣦⠀⢄⣀⠀⠠⢾⣿⠿⠿⠿⠿⢦⠀
///⠀⠀⠀⠀⠀⠀⠺⠿⠇⢸⣿⣇⠘⣿⣆⠘⣿⡆⠠⣄⡀⠀⠀⠀⠀⠀⠀   .---.    .--.      .--.    ___ .-.     ___ .-.   
///⠀⠀⠀⠀⢀⣴⣶⣶⣤⣄⡉⠛⠀⢹⣿⡄⢹⣿⡀⢻⣧⠀⡀⠀⠀⠀⠀⠀ / .-, \  /    \    /    \  (   )   \   (   )   \
///⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡈⠓⠀⣿⣧⠈⢿⡆⠸⡄⠀⠀⠀ (__) ; | |  .-. ;  |  .-. ;  | ' .-. ;   |  .-. .
///⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣈⠙⢆⠘⣿⡀⢻⠀⠀ ⠀  .'`  | |  |(___) | |  | |  |  / (___)  | |  | |
///⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠹⣧⠈⠀⠀ ⠀ / .'| | |  |      | |  | |  | |         | |  | | ⠀
///⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠈⠃⠀⠀⠀ | /  | | |  | ___  | |  | |  | |         | |  | |
///⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀ ; |  ; | |  '(   ) | '  | |  | |         | |  | | ⠀
///⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀ ' `-'  | '  `-' |  '  `-' /  | |         | |  | |
///⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀⠀`.__.'_.  `.__,'    `.__.'  (___)       (___)(___)
///⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀
///⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀Accessible Content Optimization for Research Needs
/// ⠀ ⣿⣿⠿⠿⠿⠿⠿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
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
    /// Prevent communication with the internet - intended for disconnected local environments
    ///
    /// Note: Use of --offline may require extra configuration options for certain commands
    #[arg(short = 'X', long, value_name = "BOOL", help_heading = "FLAGS")]
    pub offline: bool,
    /// Limit number of threads used by rayon for parallel processing
    ///
    /// See Rayon documentation for more information
    #[arg(default_value_t, short, long, value_name = "N", help_heading = "FLAGS")]
    pub threads: usize,
    #[command(flatten)]
    pub verbose: Verbosity,
    #[arg(short = 'V', long, help = "Prints version information", help_heading = "FLAGS")]
    pub version: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(short, long, action = ArgAction::Help, global = true, help = "Print help (see a summary with '-h')", help_heading = "FLAGS")]
    pub help: Option<bool>,
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
    ///     $> acorn check /path/to/folder --skip validation
    ///
    ///     Skip schema validation
    ///     $> acorn check /path/to/folder --skip analysis,validation
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
    Check {
        /// Path to check
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
        path: Option<PathBuf>,
        /// Check files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
        branch: Option<String>,
        /// Check files that were changed in a given Git commit or commit range
        ///
        ///     Check files that were changed in the last 10 commits
        ///     &> acorn check --commit HEAD~10..HEAD
        ///
        /// See https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection for more information of commit ranges
        #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
        commit: Option<String>,
        /// Disable website checks that require internet connection
        #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
        disable_website_checks: bool,
        /// Exit on first error
        #[arg(short, long = "exit-on-first-error", value_name = "BOOL", help_heading = "FLAGS")]
        exit_on_first_error: bool,
        /// Regex pattern applied to absolute paths of files that determines whether they should be included in checking process
        ///
        /// Only applies to `--path` values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$"
        #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
        ignore: Option<String>,
        /// Format JSON files in-place to maintain consistent format
        #[arg(short, long, value_name = "BOOL", help_heading = "FLAGS")]
        format: bool,
        /// Format files that were changed in a given merge request (Gitlab) or pull request (Github)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        /// Readability test to use
        #[arg(default_value_t, short, long = "readability-test", value_name = "METRIC", help_heading = "OPTIONS")]
        readability_metric: ReadabilityType,
        /// Skip one or more available quality checks
        #[arg(short, long, value_name = "LIST", value_delimiter = ',', help_heading = "OPTIONS")]
        skip: Vec<Check>,
        #[command(flatten)]
        verbose: Verbosity,
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
    ///     Download research activity data from a list of buckets
    ///     $> acorn download --buckets /path/to/buckets.json
    ///
    ///     Download research activity data to a specific output directory
    ///     $> acorn download --buckets /path/to/buckets.json --output /path/to/output
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Download {
        /// Path to buckets configuration file
        #[arg(default_value = "./buckets.json", short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
        buckets: Option<PathBuf>,
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
    ///     Export file(s) that were changed in a given Git commit
    ///     $> acorn export --commit <commit hash>
    ///
    ///     Export file(s) that were changed in latest Git commit
    ///     $> acorn export --commit HEAD
    ///
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Export {
        /// Path of input files to be exported
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
        path: Option<PathBuf>,
        /// Export files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH", help_heading = "OPTIONS")]
        branch: Option<String>,
        /// Export files that were changed in a given Git commit
        #[arg(short, long, value_name = "COMMIT", help_heading = "OPTIONS")]
        commit: Option<String>,
        /// Export target file format
        #[arg(default_value_t, short, long, value_name = "FORMAT", help_heading = "OPTIONS")]
        format: FileFormat,
        /// Export files that were changed in a given merge request (Gitlab) or pull request (Github)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        /// Path to output directory where the export artifacts should be saved
        #[arg(default_value = "./export", short, long, value_name = "DIRECTORY", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
        output: Option<PathBuf>,
        /// Path to reference file to be used when exporting to PowerPoint
        ///
        /// Only applies to `--format powerpoint`
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath, help_heading = "OPTIONS")]
        reference: Option<PathBuf>,
        /// Value to use when creating export artifacts that require an aspect ratio
        ///
        /// Only applies to `--target highlight`
        #[arg(default_value_t, short, long, value_name = "SIZE", help_heading = "OPTIONS")]
        size: Size,
        /// Export target type
        #[arg(default_value_t, short, long, value_name = "TARGET", help_heading = "OPTIONS")]
        target: Target,
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
    Format {
        /// Path to look for files to format
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
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
        /// Regex pattern applied to absolute paths of files that determines whether they should be included in formatting process
        ///
        /// Only applies to path values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$"
        #[arg(short, long, value_name = "REGEX", help_heading = "OPTIONS")]
        ignore: Option<String>,
        /// Format files that were changed in a given merge request (Gitlab) or pull request (Github)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Add linked data context to research activity data
    ///
    ///     $> acorn link /path/to/data/
    #[clap(verbatim_doc_comment, next_help_heading = "FLAGS")]
    Link {
        /// Path to look for files to process
        #[arg(default_value = "./", required = false, value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "OPTIONS")]
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
        /// Processes files that were changed in a given merge request (Gitlab) or pull request (Github)
        #[arg(short, long = "merge-request", value_name = "BOOL", help_heading = "FLAGS")]
        merge_request: bool,
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
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum SchemaCommands {
    /// Print research activity data (RAD) JSON schema to stdout
    Rad {},
    /// Print research activity identifier (RAiD) metadata JSON schema to stdout
    Raid {},
}

#[cfg(test)]
mod tests;
