//! Analyzer result values independent of Vale runtime configuration
use acorn_core::prelude::{String, Vec};
use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Vale output severity.
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValeOutputItemSeverity {
    /// Warning that should strongly be considered for correction.
    #[display("warning")]
    Warning,
    /// Error that should be corrected.
    #[display("error")]
    Error,
    /// Suggested improvement.
    #[default]
    #[display("suggestion")]
    Suggestion,
}
/// Parsed Vale output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValeOutput {
    /// Output findings.
    pub items: Vec<ValeOutputItem>,
}
/// One Vale output finding.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValeOutputItem {
    pub(crate) action: ValeOutputItemAction,
    /// Name of the Vale check.
    pub check: String,
    pub(crate) description: String,
    /// Source line number.
    pub line: u32,
    pub(crate) link: String,
    /// Finding message.
    pub message: String,
    /// Finding severity.
    pub severity: ValeOutputItemSeverity,
    /// Starting and ending source characters.
    pub span: Vec<u32>,
    /// Source text matched by Vale.
    #[serde(rename = "Match")]
    pub word_match: String,
}
/// Suggested action attached to a Vale finding.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValeOutputItemAction {
    pub(crate) name: String,
    pub(crate) params: Option<Vec<String>>,
}
impl ValeOutputItem {
    /// Construct a finding from the fields used by portable analysis and rendering.
    pub fn new(
        check: impl Into<String>,
        line: u32,
        message: impl Into<String>,
        severity: ValeOutputItemSeverity,
        span: Vec<u32>,
        word_match: impl Into<String>,
    ) -> Self {
        Self {
            action: ValeOutputItemAction {
                name: String::new(),
                params: None,
            },
            check: check.into(),
            description: String::new(),
            line,
            link: String::new(),
            message: message.into(),
            severity,
            span,
            word_match: word_match.into(),
        }
    }
}
