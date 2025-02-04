use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use clap_verbosity_flag::Verbosity;
use derive_more::Display;
use serde::Serialize;
use std::path::PathBuf;

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
    #[command(flatten)]
    pub verbose: Verbosity,
    #[command(subcommand)]
    pub command: Option<Commands>,
}
#[derive(Clone, Debug, Display, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Check {
    #[display("analysis")]
    // Static analysis of prose
    Analysis,
    // Schema validation via Rust type system
    #[display("validation")]
    Validation,
}
#[derive(Clone, Debug, Default, Display, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    #[default]
    #[display("standard")]
    // Standard size (4:3)
    Standard,
    // Widescreen size (16:9)
    #[display("widescreen")]
    Widescreen,
}
#[derive(Clone, Debug, Default, Display, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    #[default]
    #[display("fact-sheet")]
    FactSheet,
    #[display("highlight")]
    Highlight,
    #[display("poster")]
    Poster,
}
#[derive(Debug, Subcommand)]
#[command(long_about = None)]
pub enum Commands {
    /// Perform static analysis on research activity data and apply standardized best practices
    ///
    ///     $> acorn check --path /path/to/file.json
    ///
    ///     $> acorn check --path /path/to/folder
    ///
    #[clap(verbatim_doc_comment)]
    Check {
        /// Path to check
        #[arg(default_value = "./", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Path to configuration file
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        config: Option<PathBuf>,
        /// Format JSON files in-place to maintain consistent format
        #[arg(short, long, value_name = "BOOL")]
        format: bool,
        /// Skip one or more available quality checks
        #[arg(short, long, value_name = "LIST", value_delimiter = ',')]
        skip: Vec<Check>,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Download research activity data from buckets
    ///
    ///     $> acorn download --buckets /path/to/buckets.json
    ///
    ///     $> acorn download --buckets /path/to/buckets.json --output /path/to/output
    ///
    #[clap(verbatim_doc_comment)]
    Download {
        /// Path to buckets configuration file
        #[arg(default_value = "./buckets.json", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
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
    ///     $> acorn export --path /path/to/data --target highlight --size widescreen
    ///
    ///     $> acorn export -p /path/to/data -t poster -o /path/to/output
    ///
    #[clap(verbatim_doc_comment)]
    Export {
        /// Path of input files to be exported
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Export target type
        #[arg(default_value_t, short, long, value_name = "TARGET")]
        target: Target,
        /// Value to use when creating export artifacts that require an aspect ratio (only applies to `--target highlight`)
        #[arg(default_value_t, short, long, value_name = "SIZE")]
        size: Size,
        /// Path to output directory where the export artifacts should be saved
        #[arg(default_value = "./export", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        output: Option<PathBuf>,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Formats research activity data in place (inherently includes some elements of `acorn check`)
    ///
    ///     $> acorn format --path /path/to/file.json
    ///
    ///     $> acorn format --path /path/to/folder
    ///
    #[clap(verbatim_doc_comment)]
    Format {
        /// Path to check
        #[arg(default_value = "./", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Path to configuration file
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        config: Option<PathBuf>,
        #[command(flatten)]
        verbose: Verbosity,
    },
}
