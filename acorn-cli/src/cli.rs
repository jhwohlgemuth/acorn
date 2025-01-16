use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use clap_verbosity_flag::Verbosity;
use derive_more::Display;
use serde::Serialize;
use std::path::PathBuf;

/// Multitool for validating and enabling research activity data
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
///⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀Accessible Content Organization for Research Needs
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
    /// Perfrom static analysis on research activity data and apply standardized best practices
    ///
    ///     $> acorn lint --path /path/to/file.json
    ///
    ///     $> acorn lint --path /path/to/folder
    ///
    ///     $> acorn lint --path /path/to/folder --format
    ///
    #[clap(verbatim_doc_comment)]
    Lint {
        /// Path to check
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Path to configuration file
        #[arg(short = 'c', long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        config: Option<PathBuf>,
        /// Format JSON files in-place to maintain consistent format
        #[arg(short, long, value_name = "BOOL")]
        format: bool,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Download research activity data, such as fact sheet files, from buckets and ACORN schemas
    ///
    ///     $> acorn download --buckets /path/to/buckets.json
    ///
    ///     $> acorn download --buckets /path/to/buckets.json --output /path/to/output
    ///
    ///     $> acorn download --schema project
    ///
    #[clap(verbatim_doc_comment)]
    Download {
        /// Path to buckets configuration file
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
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
    ///     $> acorn export --path /path/to/data --target pdf
    ///
    ///     $> acorn export --path /path/to/data --target pdf --output /path/to/output
    ///
    #[clap(verbatim_doc_comment)]
    Export {
        /// Path to files for export
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Export target type
        #[arg(default_value_t, short, long, value_name = "TARGET")]
        target: Target,
        /// Path to output directory where the export artifacts should be saved
        #[arg(default_value = "./export", short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        output: Option<PathBuf>,
        #[command(flatten)]
        verbose: Verbosity,
    },
}
