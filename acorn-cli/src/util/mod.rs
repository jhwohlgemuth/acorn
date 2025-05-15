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
    pub output: Option<PathBuf>,
    pub reference: Option<PathBuf>,
    pub size: Option<Size>,
    pub target: Option<Target>,
}
impl TargetLabel {
    pub fn get_folder(self) -> String {
        match self {
            | TargetLabel::Bessd => "bessd".to_string(),
            | TargetLabel::Wpp => "wpp".to_string(),
            | TargetLabel::Nssd => "nssd".to_string(),
            | TargetLabel::Ornl => TargetLabel::default().get_folder().to_owned(),
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
