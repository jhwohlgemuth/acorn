use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

/// Multi-tool for verifying myriad aspects of research activity data
///
///   .---.    .--.      .--.    ___ .-.     ___ .-.   
///  / .-, \  /    \    /    \  (   )   \   (   )   \  
/// (__) ; | |  .-. ;  |  .-. ;  | ' .-. ;   |  .-. .  
///   .'`  | |  |(___) | |  | |  |  / (___)  | |  | |  
///  / .'| | |  |      | |  | |  | |         | |  | |  
/// | /  | | |  | ___  | |  | |  | |         | |  | |  
/// ; |  ; | |  '(   ) | '  | |  | |         | |  | |  
/// ' `-'  | '  `-' |  '  `-' /  | |         | |  | |  
/// `.__.'_.  `.__,'    `.__.'  (___)       (___)(___) 
/// Accessible Content Organization for Research Needs
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
    /// Validate a JSON file or JSON files in a given directory
    ///
    ///     $> acorn check /path/to/file.json
    ///
    ///     $> acorn check /path/to/folder
    ///
    #[clap(verbatim_doc_comment)]
    Check {
        /// Path to check
        #[arg(value_name = "PATH", value_hint = clap::ValueHint::DirPath)]
        path: Option<std::path::PathBuf>,
        /// Path to configuration file
        #[arg(short = 'c', long = "config", value_name = "PATH", value_hint = clap::ValueHint::DirPath)]
        config: Option<std::path::PathBuf>,
    },
}
