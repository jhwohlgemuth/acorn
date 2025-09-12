//! # Command line interface (CLI) utilities
//!
//! Common utilities and structs used to create a command line interface using ACORN schemas
use crate::util::{files_all, files_from_git_branch, files_from_git_commit, filter_ignored, git_branch_name};
use bon::Builder;
use clap::ValueEnum;
use derive_more::Display;
use serde::Serialize;
use std::path::PathBuf;

/// Catagories available when analyzing ("checking") research activity data
///
/// Used primarily by ACORN CLI
#[derive(Clone, Debug, Display, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Check {
    /// Static analysis of prose
    #[display("analysis")]
    Analysis,
    /// Folder structure and file naming
    #[display("conventions")]
    Conventions,
    /// Readability of prose
    #[display("readability")]
    Readability,
    /// Schema validation via Rust type system
    #[display("validation")]
    Validation,
}
/// Catagories available when performing system diagnostics before using ACORN
///
/// Used primarily by ACORN CLI
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Diagnostic {
    /// All available diagnostics
    #[default]
    #[display("all")]
    All,
    /// System information (e.g. CPU count, OS, etc...)
    #[display("system")]
    System,
    /// Memory information and usage
    #[display("memory")]
    Memory,
    /// Network information
    #[display("network")]
    Network,
    /// Graphics Processing Unit (GPU) information
    #[display("gpu")]
    Gpu,
    /// Check for installed software
    #[display("software")]
    Software,
}
/// Target export file formats available when exporting research activity data using acorn
///
/// Used primarily by ACORN CLI
#[derive(Clone, Debug, Default, Display, ValueEnum, Serialize)]
pub enum FileFormat {
    /// Portable Document Format
    #[default]
    #[display("pdf")]
    Pdf,
    /// Microsoft PowerPoint
    ///
    /// Only available for certain targets (e.g. highlights)
    #[display("powerpoint")]
    Powerpoint,
}
/// Target artifact aspect ratio size available when exporting research activity data using acorn
///
/// Used primarily by ACORN CLI
#[derive(Clone, Debug, Default, Display, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    /// Standard size (4:3)
    #[display("standard")]
    Standard,
    /// Widescreen size (16:9)
    #[default]
    #[display("widescreen")]
    Widescreen,
}
/// Target artifact types available when exporting research activity data using acorn
///
/// Used primarily by ACORN CLI
#[derive(Clone, Copy, Debug, Default, Display, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
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
/// Target artifact branding available when exporting research activity data using acorn
///
/// Used primarily by ACORN CLI
#[derive(Default, Clone, Copy, Debug, Display)]
pub enum TargetLabel {
    /// National Security Sciences Directorate
    ///
    /// See <https://www.ornl.gov/science-area/national-security>
    #[default]
    #[display("National Security Sciences")]
    Nssd,
    /// Biological and Environmental Systems Sciences Directorate
    ///
    /// See <https://www.ornl.gov/directorate/bessd>
    #[display("Biological and Environmental Systems Sciences")]
    Bessd,
    /// ORNL Water Power Program
    ///
    /// See <https://www.ornl.gov/waterpower>
    #[display("Water Power Program")]
    Wpp,
    /// General ORNL Branding
    #[display("Solving Big Problems")]
    Ornl,
}
/// Container struct for working with ACORN CLI subcommand run functions
#[derive(Builder, Clone, Debug, Default)]
#[builder(start_fn = init)]
pub struct Options {
    /// Path to file or folder to be used for input
    pub path: Option<PathBuf>,
    /// Git branch name
    pub branch: Option<String>,
    /// Git commit hash
    pub commit: Option<String>,
    /// Regex pattern of files to ignore at a given path desginated by `path`
    pub ignore: Option<String>,
    /// Path to file or folder to be used for output
    pub output: Option<PathBuf>,
    /// Path to reference file
    ///
    /// e.g. reference.pptx for exporting RAD to PowerPoint
    pub reference: Option<PathBuf>,
    /// Artifact aspect ratio size
    pub size: Option<Size>,
    /// Artifact target type
    pub target: Option<Target>,
    /// Flag used to indicate if a single error should cause the process to exit
    #[builder(default)]
    pub exit_on_first_error: bool,
    /// Flag used to indicate if changed files should be obtained from a merge request
    #[builder(default)]
    pub merge_request: bool,
}
impl TargetLabel {
    /// Returns a string representing the folder name for the given TargetLabel
    pub fn folder(self) -> String {
        match self {
            | TargetLabel::Bessd => "bessd".to_string(),
            | TargetLabel::Wpp => "wpp".to_string(),
            | TargetLabel::Nssd => "nssd".to_string(),
            | TargetLabel::Ornl => TargetLabel::default().folder().to_owned(),
        }
    }
    /// Returns a TargetLabel based on the given organization name
    pub fn from_organization(name: &str) -> Self {
        match name {
            | "Biological and Environmental Systems Science Directorate" => TargetLabel::Bessd,
            | "National Security Sciences Division" => TargetLabel::Nssd,
            | "Oak Ridge National Laboratory" => TargetLabel::Ornl,
            | "Water Power Program" => TargetLabel::Wpp,
            | _ => TargetLabel::default(),
        }
    }
}
/// Returns a vector of PathBuf from the given options
///
/// If the options specify a merge request, the files from the current branch are returned.
/// If the options specify a commit, the files changed in the commit are returned.
/// If the options specify a branch, the files changed in the branch are returned.
/// If none of the above options are set, the files in the given path are returned.
/// If the options include an ignore regex, it is applied to the files returned.
pub fn paths_from_options(path: &Option<PathBuf>, options: &Option<Options>) -> Vec<PathBuf> {
    let extensions = Some(vec!["JSON", "YAML"]);
    match options {
        | Some(Options {
            branch,
            commit,
            ignore,
            merge_request,
            ..
        }) => {
            let files = if *merge_request {
                match git_branch_name() {
                    | Some(name) => files_from_git_branch(&name, extensions),
                    | None => vec![],
                }
            } else {
                match commit {
                    | Some(hash) => files_from_git_commit(hash, extensions),
                    | None => match branch {
                        | Some(name) => files_from_git_branch(name, extensions),
                        | None => {
                            let value = match path {
                                | Some(x) => x.clone(),
                                | None => PathBuf::from("."),
                            };
                            files_all(value, extensions)
                        }
                    },
                }
            };
            filter_ignored(files, ignore.clone())
        }
        | None => {
            let value = match path {
                | Some(x) => x.clone(),
                | None => PathBuf::from("."),
            };
            files_all(value, extensions)
        }
    }
}
