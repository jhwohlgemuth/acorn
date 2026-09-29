//! Proposal-only fix models and inference-backed suggestion orchestration.

use crate::agent::{
    run_inference, HydratedPrompt, InferenceBackend, InferencePermissionPolicy, InferenceProvenance, InferenceRequest, InferenceResult,
    PromptContext, PromptHydrator,
};
use crate::analyzer::host::service::AnalysisReport;
use crate::io::{ApiResult, Fingerprint, Parse};
use crate::util::StringConversion;
use acorn_core::prelude::{format, String, ToString, Vec};
use acorn_core::time::Milliseconds;
use acorn_core::util::StringExt;
use acorn_core::validation::{digest, ValidationError};
use acorn_host::fs::SafePath;
use acorn_schema::{
    agent::{PromptFileAsset, PromptTemplateConfiguration},
    validation::{Validate, ValidationReport},
    OneOrMany,
};
use alloc::collections::{BTreeMap, BTreeSet};
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use core::{convert::identity, fmt};
use futures::{stream, StreamExt};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
mod discovery;
mod execution;
mod plan;
mod service;
mod types;
pub use discovery::{HostProvider, Provider};
#[cfg(feature = "analysis")]
pub use execution::CommandRunner;
pub use execution::{Outcome, Runner, Verifier};
pub use plan::{Plan, Policy};
use types::FixError;
pub use types::{
    CategoryFixes, CheckReference, DiscoveryCategory, DiscoveryReference, DiscoveryStatus, FindingReference, Fix, FixAction, FixAutomation, FixBatch,
    FixDiagnostic, FixEffects, FixOptions, FixProvenance, FixReport, FixRisk, FixStatus, FixSuggestionRequest, FixSummary,
};
/// Maximum aggregate finding context sent in one inference request by default.
pub const DEFAULT_MAX_CONTEXT_BYTES: usize = 128 * 1024;
/// Maximum UTF-8 byte length of one source excerpt by default.
pub const DEFAULT_MAX_EXCERPT_BYTES: usize = 4 * 1024;
/// Maximum number of findings sent in one inference request by default.
pub const DEFAULT_MAX_FINDINGS_PER_BATCH: usize = 32;
/// Version of the proposal-only fix envelope.
pub const FIX_ENVELOPE_VERSION: &str = "1";
const MAX_MESSAGE_BYTES: usize = 1024;
/// Adds proposal orchestration methods to normalized check-reference collections.
#[async_trait]
pub trait CheckReferences {
    /// Partition references using the configured count and context ceilings.
    fn batch(self, options: &FixOptions) -> Vec<Vec<CheckReference>>;
    /// Build one structured inference request for these references.
    fn build(&self, options: &FixOptions) -> ApiResult<InferenceRequest>;
    /// Suggest fixes through one transport-neutral backend.
    async fn suggest<B>(self, backend: &B, options: &FixOptions) -> ApiResult<FixReport>
    where
        B: InferenceBackend + Sync;
    /// Validate per-reference and collection-wide invariants.
    fn validate_with(&self, options: &FixOptions) -> ApiResult<()>;
}
/// Flattens a grouped fix container into deterministic fix order.
pub trait IntoFixes {
    /// Return all contained fixes in deterministic group order.
    fn fixes(&self) -> Vec<&Fix>;
}
impl CategoryFixes {
    fn key(fix: &Fix) -> (Option<String>, String) {
        match &fix.finding {
            | FindingReference::Check(reference) => (Some(reference.standard.clone()), reference.category.clone()),
            | FindingReference::Discovery(reference) => (None, reference.category.to_string()),
        }
    }
}
impl CheckReference {
    fn context_bytes(&self) -> usize {
        self.context.as_ref().map_or(0, String::len).saturating_add(self.message.len())
    }
}
impl Validate for CheckReference {
    fn validate(&self) -> Result<(), ValidationReport> {
        let fields = vec![
            self.category.as_str(),
            self.message.as_str(),
            self.severity.as_str(),
            self.standard.as_str(),
        ]
        .trim_and_compact();
        let invalid_finding = [!is_identifier(&self.id), fields.len() != 4].into_iter().any(identity);
        let prefixed_fingerprint = format!("sha256:{}", self.fingerprint);
        let valid_fingerprint = digest(&prefixed_fingerprint).is_ok();
        let valid_message = self.message.len() <= MAX_MESSAGE_BYTES;
        let valid_context = self.context.as_ref().is_none_or(|value| value.len() <= DEFAULT_MAX_EXCERPT_BYTES);
        match (invalid_finding, valid_fingerprint, valid_message, valid_context) {
            | (true, _, _, _) => Err(FixError::InvalidFinding.report("")),
            | (_, false, _, _) => Err(FixError::InvalidFingerprint.report("fingerprint")),
            | (_, _, false, _) => Err(FixError::MessageExceeded.report("message")),
            | (_, _, _, false) => Err(FixError::ExcerptExceeded.report("context")),
            | _ => Ok(()),
        }
    }
}
impl Validate for FixAction {
    fn validate(&self) -> Result<(), ValidationReport> {
        let valid = match self {
            | Self::DocumentChange {
                description,
                expected,
                replacement,
            } => !description.trim().is_empty() && !expected.is_empty() && expected != replacement,
            | Self::Manual { instructions } => !instructions.trim().is_empty(),
            | Self::RegisteredRecipe { recipe_id } => is_identifier(recipe_id),
        };
        match valid {
            | true => Ok(()),
            | false => Err(ValidationReport::from_error(
                "",
                ValidationError::new("fix_action").with_message("Fix action is empty or invalid"),
            )),
        }
    }
}
impl FixError {
    fn report(self, path: &str) -> ValidationReport {
        ValidationReport::from_error(path, ValidationError::new("fix").with_message(self.to_string()))
    }
}
impl Default for FixOptions {
    fn default() -> Self {
        Self {
            agent: None,
            include_successful: false,
            max_context_bytes: DEFAULT_MAX_CONTEXT_BYTES,
            max_excerpt_bytes: DEFAULT_MAX_EXCERPT_BYTES,
            max_findings_per_batch: DEFAULT_MAX_FINDINGS_PER_BATCH,
            min_confidence: 0.0,
            model: None,
            timeout: Milliseconds::new(30_000),
        }
    }
}
impl Validate for FixOptions {
    fn validate(&self) -> Result<(), ValidationReport> {
        let valid_confidence = self.min_confidence.is_finite() && (0.0..=1.0).contains(&self.min_confidence);
        let limits = (self.max_context_bytes, self.max_excerpt_bytes, self.max_findings_per_batch, self.timeout);
        match (limits, valid_confidence) {
            | ((0, _, _, _), _) => Err(FixError::ContextLimitZero.report("maxContextBytes")),
            | ((value, _, _, _), _) if value > DEFAULT_MAX_CONTEXT_BYTES => Err(FixError::ContextLimitExceeded.report("maxContextBytes")),
            | ((_, 0, _, _), _) => Err(FixError::ExcerptLimitZero.report("maxExcerptBytes")),
            | ((_, value, _, _), _) if value > DEFAULT_MAX_EXCERPT_BYTES => Err(FixError::ExcerptLimitExceeded.report("maxExcerptBytes")),
            | ((_, _, 0, _), _) => Err(FixError::FindingLimitZero.report("maxFindingsPerBatch")),
            | ((_, _, value, _), _) if value > DEFAULT_MAX_FINDINGS_PER_BATCH => Err(FixError::FindingLimitExceeded.report("maxFindingsPerBatch")),
            | ((_, _, _, _), false) => Err(FixError::InvalidMinimumConfidence.report("minConfidence")),
            | ((_, _, _, timeout), _) if timeout.is_zero() => Err(FixError::TimeoutZero.report("timeout")),
            | _ => Ok(()),
        }
    }
}
impl From<InferenceProvenance> for FixProvenance {
    fn from(value: InferenceProvenance) -> Self {
        Self {
            agent: value.agent,
            backend: value.backend,
            model: value.model,
            prompt_version: FIX_ENVELOPE_VERSION.to_string(),
            request_id: value.request_id,
        }
    }
}
impl FixReport {
    fn from_parts(fixes: Vec<Fix>, diagnostics: Vec<FixDiagnostic>) -> Self {
        let categories = fixes
            .into_iter()
            .fold(BTreeMap::<(Option<String>, String), Vec<Fix>>::new(), |mut groups, fix| {
                groups.entry(CategoryFixes::key(&fix)).or_default().push(fix);
                groups
            })
            .into_iter()
            .map(|((standard, category), values)| CategoryFixes {
                category,
                fixes: FixBatch { values: values.into() },
                standard,
            })
            .collect();
        Self { categories, diagnostics }
    }
}
impl From<Vec<Fix>> for FixReport {
    fn from(value: Vec<Fix>) -> Self {
        Self::from_parts(value, Vec::new())
    }
}
impl IntoFixes for FixReport {
    fn fixes(&self) -> Vec<&Fix> {
        self.categories.iter().flat_map(|category| category.fixes.values.iter()).collect()
    }
}
impl fmt::Display for FixReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let heading = match self.categories.is_empty() && self.diagnostics.is_empty() {
            | true => Ok(()),
            | false => writeln!(formatter, "\nSuggested fixes"),
        };
        heading
            .and_then(|()| {
                self.categories.iter().try_for_each(|category| {
                    category
                        .fixes
                        .values
                        .iter()
                        .try_for_each(|fix| writeln!(formatter, "- [{}] {}", category.category, fix.summary))
                })
            })
            .and_then(|()| {
                self.diagnostics
                    .iter()
                    .try_for_each(|diagnostic| writeln!(formatter, "Fix suggestion — {}", diagnostic.message))
            })
    }
}
impl From<&FixReport> for FixSummary {
    fn from(value: &FixReport) -> Self {
        value.fixes().into_iter().fold(Self::default(), |summary, fix| match fix.status {
            | FixStatus::Applied => FixSummary {
                applied: summary.applied.saturating_add(1),
                ..summary
            },
            | FixStatus::Failed => FixSummary {
                failed: summary.failed.saturating_add(1),
                ..summary
            },
            | FixStatus::Proposed => FixSummary {
                proposed: summary.proposed.saturating_add(1),
                ..summary
            },
            | FixStatus::Rejected => FixSummary {
                rejected: summary.rejected.saturating_add(1),
                ..summary
            },
            | FixStatus::Stale => FixSummary {
                stale: summary.stale.saturating_add(1),
                ..summary
            },
            | FixStatus::Unsupported => FixSummary {
                unsupported: summary.unsupported.saturating_add(1),
                ..summary
            },
        })
    }
}
impl From<FixReport> for FixSummary {
    fn from(value: FixReport) -> Self {
        Self::from(&value)
    }
}
fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-'))
}
fn validate_options(options: &FixOptions) -> ApiResult<()> {
    options.validate().map_err(|why| eyre!(why))
}
#[cfg(test)]
mod tests;
