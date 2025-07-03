use acorn_lib::util::{files_all, files_from_git_branch, files_from_git_commit, git_branch_name};
use bon::Builder;
use clap::ValueEnum;
use derive_more::Display;
use serde::Serialize;
use std::path::PathBuf;

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
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Diagnostic {
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
#[derive(Clone, Copy, Debug, Default, Display, ValueEnum, Serialize)]
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
#[derive(Default, Clone, Copy, Debug, Display)]
pub enum TargetLabel {
    #[default]
    #[display("National Security Sciences")]
    Nssd,
    #[display("Biological and Environmental Systems Sciences")]
    Bessd,
    #[display("Water Power Program")]
    Wpp,
    #[display("Solving Big Problems")]
    Ornl,
}
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct Options {
    pub path: Option<PathBuf>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub ignore: Option<String>,
    pub output: Option<PathBuf>,
    pub reference: Option<PathBuf>,
    pub size: Option<Size>,
    pub target: Option<Target>,
    #[builder(default)]
    pub exit_on_first_error: bool,
    #[builder(default)]
    pub merge_request: bool,
}
impl TargetLabel {
    pub fn folder(self) -> String {
        match self {
            | TargetLabel::Bessd => "bessd".to_string(),
            | TargetLabel::Wpp => "wpp".to_string(),
            | TargetLabel::Nssd => "nssd".to_string(),
            | TargetLabel::Ornl => TargetLabel::default().folder().to_owned(),
        }
    }
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
pub fn paths_from_options(path: &Option<PathBuf>, options: &Option<Options>) -> Vec<PathBuf> {
    let extension = Some("json");
    match options {
        | Some(Options {
            branch,
            commit,
            ignore,
            merge_request,
            ..
        }) => {
            if *merge_request {
                match git_branch_name() {
                    | Some(name) => files_from_git_branch(&name, extension),
                    | None => vec![],
                }
            } else {
                match commit {
                    | Some(hash) => files_from_git_commit(hash, extension),
                    | None => match branch {
                        | Some(name) => files_from_git_branch(name, extension),
                        | None => {
                            let value = match path {
                                | Some(x) => x.clone(),
                                | None => PathBuf::from("."),
                            };
                            files_all(value, extension, ignore.clone())
                        }
                    },
                }
            }
        }
        | None => {
            let value = match path {
                | Some(x) => x.clone(),
                | None => PathBuf::from("."),
            };
            files_all(value, extension, None)
        }
    }
}
