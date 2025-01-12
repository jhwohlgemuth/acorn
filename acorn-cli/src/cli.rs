use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

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
        #[arg(short = 'p', long = "path", value_name = "PATH", value_hint = clap::ValueHint::DirPath)]
        path: Option<std::path::PathBuf>,
        /// Path to configuration file
        #[arg(short = 'c', long = "config", value_name = "PATH", value_hint = clap::ValueHint::DirPath)]
        config: Option<std::path::PathBuf>,
        /// Format JSON files in-place to maintain consistent format
        #[arg(short = 'f', long = "format", value_name = "BOOL")]
        format: bool,
        #[command(flatten)]
        verbose: Verbosity,
    },
    /// Download research activity data, such as fact sheet files, from buckets and ACORN schemas
    ///
    ///     $> acorn download --config /path/to/bucket.json
    /// 
    ///     $> acorn download --config /path/to/bucket.json --output /path/to/output
    ///
    ///     $> acorn download --schema project
    ///
    #[clap(verbatim_doc_comment)]
    Download {
        /// Path to buckets configuration file
        #[arg(short = 'c', long = "config", value_name = "PATH", value_hint = clap::ValueHint::DirPath)]
        config: Option<std::path::PathBuf>,
        /// Path to output directory where the downloaded content should be saved
        #[arg(default_value = "./content", short = 'o', long = "output", value_name = "PATH", value_hint = clap::ValueHint::DirPath)]
        output: Option<std::path::PathBuf>,
        #[command(flatten)]
        verbose: Verbosity,
    },
}
