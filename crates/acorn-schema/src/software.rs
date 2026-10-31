//! Portable software identities and diagnostic observations.

use acorn_core::{
    prelude::{String, ToString},
    util::SemanticVersion,
};
use core::fmt;
/// Software probed by ACORN host diagnostics.
pub const SOFTWARE: [Software; 13] = [
    Software::Acorn,
    Software::Chromium,
    Software::Codex,
    Software::CodexAcp,
    Software::Git,
    Software::Node,
    Software::Npm,
    Software::Npx,
    Software::OpenCode,
    Software::Ollama,
    Software::Oras,
    Software::Pandoc,
    Software::Vale,
];
/// A software product recognized by ACORN.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Software {
    /// ACORN.
    Acorn,
    /// Chromium or a compatible Chrome executable.
    Chromium,
    /// Codex CLI.
    Codex,
    /// Codex ACP adapter.
    CodexAcp,
    /// Git.
    Git,
    /// Node.js.
    Node,
    /// npm.
    Npm,
    /// npx.
    Npx,
    /// OpenCode.
    OpenCode,
    /// Ollama model server CLI.
    Ollama,
    /// ORAS.
    Oras,
    /// Pandoc.
    Pandoc,
    /// An unrecognized software product.
    Unknown,
    /// Vale.
    Vale,
}
/// One portable software-probe observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoftwareSnapshot {
    /// Display name used as the finding subject.
    pub name: String,
    /// Resolved executable path when installed.
    pub path: Option<String>,
    /// Bounded version output when available.
    pub version: Option<SemanticVersion>,
}
impl fmt::Display for Software {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Acorn => "ACORN",
            | Self::Chromium => "Chromium",
            | Self::Codex => "Codex",
            | Self::CodexAcp => "Codex ACP",
            | Self::Git => "Git",
            | Self::Node => "Node.js",
            | Self::Npm => "npm",
            | Self::Npx => "npx",
            | Self::OpenCode => "OpenCode",
            | Self::Ollama => "Ollama",
            | Self::Oras => "ORAS",
            | Self::Pandoc => "Pandoc",
            | Self::Unknown => "unknown",
            | Self::Vale => "Vale",
        })
    }
}
impl From<&str> for Software {
    fn from(value: &str) -> Self {
        match value {
            | "ACORN" => Self::Acorn,
            | "Chromium" => Self::Chromium,
            | "Codex" => Self::Codex,
            | "Codex ACP" => Self::CodexAcp,
            | "Git" => Self::Git,
            | "Node.js" => Self::Node,
            | "npm" => Self::Npm,
            | "npx" => Self::Npx,
            | "OpenCode" => Self::OpenCode,
            | "Ollama" => Self::Ollama,
            | "ORAS" => Self::Oras,
            | "Pandoc" => Self::Pandoc,
            | "Vale" => Self::Vale,
            | _ => Self::Unknown,
        }
    }
}
impl Software {
    /// Return the canonical executable name used to invoke this software.
    pub const fn command(self) -> &'static str {
        match self {
            | Self::Acorn => "acorn",
            | Self::Chromium => "chromium",
            | Self::Codex => "codex",
            | Self::CodexAcp => "codex-acp",
            | Self::Git => "git",
            | Self::Node => "node",
            | Self::Npm => "npm",
            | Self::Npx => "npx",
            | Self::OpenCode => "opencode",
            | Self::Ollama => "ollama",
            | Self::Oras => "oras",
            | Self::Pandoc => "pandoc",
            | Self::Unknown => "",
            | Self::Vale => "vale",
        }
    }
}
impl SoftwareSnapshot {
    /// Construct a portable observation from host-provided values.
    pub fn new(software: Software, path: Option<String>, version: Option<SemanticVersion>) -> Self {
        Self {
            name: software.to_string(),
            path,
            version,
        }
    }
}
