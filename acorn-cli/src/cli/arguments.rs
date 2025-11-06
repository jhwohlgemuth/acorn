use clap::ValueEnum;
use derive_more::Display;
/// Categories available when analyzing ("checking") research activity data
#[derive(Clone, Debug, PartialEq, PartialOrd, ValueEnum)]
pub enum CheckCategory {
    Conventions,
    Prose,
    Readability,
    Schema,
}
/// Categories available when performing system diagnostics before using ACORN
#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum Diagnostic {
    #[default]
    All,
    System,
    Memory,
    Network,
    Gpu,
    Software,
}
/// Target export file formats available when exporting research activity data using acorn
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum FileFormat {
    #[default]
    #[display("pdf")]
    Pdf,
    #[display("pptx")]
    Powerpoint,
}
/// Readability Type
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum ReadabilityTypeArgument {
    /// Automated Readability Index (ARI)
    Ari,
    /// Coleman-Liau Index (CLI)
    #[display("cli")]
    Cli,
    /// Flesch-Kincaid Grade Level (FKGL)
    #[default]
    #[display("fkgl")]
    Fkgl,
    /// Flesch Reading Ease (FRES)
    #[display("fres")]
    Fres,
    /// Gunning Fog Index (GFI)
    #[display("gfi")]
    Gfi,
    /// Lix (abbreviation of Swedish läsbarhetsindex)
    #[display("lix")]
    Lix,
    /// SMOG Index (SMOG)
    #[display("smog")]
    Smog,
}
/// Target artifact aspect ratio size available when exporting research activity data using acorn
#[derive(Clone, Debug, Default, Display, ValueEnum)]
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
#[derive(Clone, Copy, Debug, Default, Display, ValueEnum)]
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
