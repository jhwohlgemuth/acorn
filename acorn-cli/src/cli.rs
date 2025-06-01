use clap::{Parser, Subcommand, ValueHint};
use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;

use crate::util::*;

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
#[command(author, version, about, verbatim_doc_comment)]
pub struct Arguments {
    /// Prevent communication with the internet - intended for disconnected local environments
    ///
    /// Note: Use of --offline may require extra configuration options for certain commands
    #[arg(long, value_name = "BOOL")]
    pub offline: bool,
    #[command(flatten)]
    pub verbose: Verbosity,
    #[command(subcommand)]
    pub command: Option<Commands>,
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum Commands {
    /// Perform static analysis on research activity data and apply standardized best practices
    ///
    ///     Check a file
    ///     $> acorn check --path /path/to/file.json
    ///
    ///     Check all supported files in a folder
    ///     $> acorn check --path /path/to/folder
    ///
    ///     Skip static analysis
    ///     $> acorn check --path /path/to/folder --skip validation
    ///
    ///     Skip schema validation
    ///     $> acorn check --path /path/to/folder --skip analysis,validation
    ///
    ///     Check file(s) that were changed in a given Git commit
    ///     $> acorn format --commit <commit hash>
    ///
    ///     Check file(s) that were changed in latest Git commit
    ///     $> acorn format --commit HEAD
    ///
    #[clap(verbatim_doc_comment)]
    Check {
        /// Path to check
        #[arg(default_value = "./", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Check files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH")]
        branch: Option<String>,
        /// Check files that were changed in a given Git commit
        #[arg(short, long, value_name = "COMMIT")]
        commit: Option<String>,
        /// Exit on first error
        #[arg(short, long = "exit-on-first-error", value_name = "BOOL")]
        exit_on_first_error: bool,
        /// Regex pattern applied to absolute paths of files that determines whether they should be included in checking process
        ///
        /// Only applies to `--path` values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$"
        #[arg(short, long, value_name = "REGEX")]
        ignore: Option<String>,
        /// Format JSON files in-place to maintain consistent format
        #[arg(short, long, value_name = "BOOL")]
        format: bool,
        /// Format files that were changed in a given merge request (Gitlab) or pull request (Github)
        #[arg(short, long = "merge-request", value_name = "BOOL")]
        merge_request: bool,
        /// Skip one or more available quality checks
        #[arg(short, long, value_name = "LIST", value_delimiter = ',')]
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
    #[clap(verbatim_doc_comment)]
    Doctor {
        /// Auto-correct systems issues where possible
        #[arg(short, long, value_name = "BOOL")]
        fix: bool,
        /// Start interactive TUI to selectively apply fixes
        #[arg(short, long, value_name = "BOOL")]
        interactive: bool,
        /// Choose to output a JSON issue report to stdout
        #[arg(short, long, value_name = "BOOL")]
        report: bool,
        /// Select one or more available diagnostic checks
        #[arg(default_value = "all", short, long, value_name = "LIST", value_delimiter = ',')]
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
    #[clap(verbatim_doc_comment)]
    Download {
        /// Path to buckets configuration file
        #[arg(default_value = "./buckets.json", short, long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        buckets: Option<PathBuf>,
        /// Path to output directory where the downloaded content should be saved
        #[arg(default_value = "./content", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        output: Option<PathBuf>,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Export research activity data to a specific target
    ///
    ///     $> acorn export --path /path/to/data
    ///
    ///     $> acorn export --path /path/to/data/data.json -vv
    ///
    ///     $> acorn export --path /path/to/data --target highlight --format powerpoint
    ///
    ///     $> acorn export -p /path/to/data -t poster -o /path/to/output
    ///
    ///     Export file(s) that were changed in a given Git commit
    ///     $> acorn export --commit <commit hash>
    ///
    ///     Export file(s) that were changed in latest Git commit
    ///     $> acorn export --commit HEAD
    ///
    #[clap(verbatim_doc_comment)]
    Export {
        /// Path of input files to be exported
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Export files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH")]
        branch: Option<String>,
        /// Export files that were changed in a given Git commit
        #[arg(short, long, value_name = "COMMIT")]
        commit: Option<String>,
        /// Export target file format
        #[arg(default_value_t, short, long, value_name = "FORMAT")]
        format: FileFormat,
        /// Export files that were changed in a given merge request (Gitlab) or pull request (Github)
        #[arg(short, long = "merge-request", value_name = "BOOL")]
        merge_request: bool,
        /// Path to output directory where the export artifacts should be saved
        #[arg(default_value = "./export", short, long, value_name = "DIRECTORY", value_hint = ValueHint::DirPath)]
        output: Option<PathBuf>,
        /// Path to reference file to be used when exporting to PowerPoint
        ///
        /// Only applies to `--format powerpoint`
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath)]
        reference: Option<PathBuf>,
        /// Value to use when creating export artifacts that require an aspect ratio
        ///
        /// Only applies to `--target highlight`
        #[arg(default_value_t, short, long, value_name = "SIZE")]
        size: Size,
        /// Print JSON schema for research activity data
        ///
        /// Note: Passing --schema will cause ACORN to exit after schema is printed
        #[arg(long, value_name = "BOOL")]
        schema: bool,
        /// Export target type
        #[arg(default_value_t, short, long, value_name = "TARGET")]
        target: Target,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Formats research activity data in place (inherently includes some elements of `acorn check`)
    ///
    ///     $> acorn format --path /path/to/file.json
    ///
    ///     $> acorn format --path /path/to/folder
    ///
    ///     Format file(s) that were changed in a given Git commit
    ///     $> acorn format --commit <commit hash>
    ///
    ///     Format file(s) that were changed in latest Git commit
    ///     $> acorn format --commit HEAD
    ///
    #[clap(verbatim_doc_comment)]
    Format {
        /// Path to look for files to format
        #[arg(default_value = "./", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Format files that were changed in a given Git branch
        #[arg(short, long, value_name = "BRANCH")]
        branch: Option<String>,
        /// Format files that were changed in a given Git commit
        #[arg(short, long, value_name = "COMMIT")]
        commit: Option<String>,
        /// Run format without making changes to target file(s). Will print a diff of changes.
        #[arg(short, long = "dry-run", value_name = "BOOL")]
        dry_run: bool,
        /// Regex pattern applied to absolute paths of files that determines whether they should be included in formatting process
        ///
        /// Only applies to `--path` values that point to a directory
        ///
        /// Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals
        ///
        /// Example: --ignore "[/]valid.json$"
        #[arg(short, long, value_name = "REGEX")]
        ignore: Option<String>,
        /// Format files that were changed in a given merge request (Gitlab) or pull request (Github)
        #[arg(short, long = "merge-request", value_name = "BOOL")]
        merge_request: bool,
        #[command(flatten)]
        verbose: Verbosity,
    },
}
