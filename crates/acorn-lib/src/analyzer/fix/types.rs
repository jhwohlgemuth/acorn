use super::{
    BTreeMap, Box, Deserialize, Error, JsonSchema, OneOrMany, Serialize, String, Vec, VocabularyAdvisory, DEFAULT_MAX_CONTEXT_BYTES,
    DEFAULT_MAX_EXCERPT_BYTES, DEFAULT_MAX_FINDINGS_PER_BATCH, MAX_MESSAGE_BYTES,
};
use acorn_core::time::Milliseconds;
use acorn_core::Noul;
/// Stable category for a host discovery finding.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::AsRefStr, strum::Display, strum::EnumIs)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DiscoveryCategory {
    /// Graphics accelerator diagnostics.
    Gpu,
    /// Physical and swap memory diagnostics.
    Memory,
    /// Network interface diagnostics.
    Network,
    /// Required and optional software diagnostics.
    Software,
    /// Operating-system diagnostics.
    System,
    /// Unrecognized diagnostic category.
    Unknown,
}
/// Stable status for a host discovery finding.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::Display, strum::EnumIs)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum DiscoveryStatus {
    /// The observed state meets the requirement.
    Healthy,
    /// The observed state needs attention.
    Attention,
    /// ACORN could not determine the state.
    Unavailable,
}
/// Reference to the finding that motivated a fix.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "source")]
pub enum FindingReference {
    /// Analyzer check reference.
    Check(CheckReference),
    /// Host discovery finding reference.
    Discovery(DiscoveryReference),
}
/// Proposal action that contains no executable command specification.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "kind")]
pub enum FixAction {
    /// Describe a bounded replacement in one document.
    DocumentChange {
        /// Human-readable description of the change.
        description: String,
        /// Exact source value expected before applying the change.
        expected: String,
        /// Proposed replacement value.
        replacement: String,
    },
    /// Explain a remediation that must be completed by a person.
    Manual {
        /// Bounded manual instructions.
        instructions: String,
    },
    /// Refer to a recipe compiled into ACORN without supplying its command.
    RegisteredRecipe {
        /// Opaque trusted recipe identifier.
        recipe_id: String,
    },
}
/// Automation level proposed for a fix.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize, strum::EnumIs)]
#[serde(rename_all = "kebab-case")]
pub enum FixAutomation {
    /// The proposal can be applied automatically after trusted planning.
    SafeAutomatic,
    /// The proposal requires explicit confirmation.
    ConfirmationRequired,
    /// The proposal requires an interactive workflow.
    InteractiveOnly,
    /// The proposal can only be followed manually.
    Manual,
}
#[derive(Debug, Error)]
pub(super) enum FixError {
    #[error("Fix action count does not match its status")]
    ActionCountMismatch,
    #[error("Fix context limit exceeds the {DEFAULT_MAX_CONTEXT_BYTES}-byte ceiling")]
    ContextLimitExceeded,
    #[error("Fix context limit must be greater than zero")]
    ContextLimitZero,
    #[error("Fix finding excerpt exceeds the configured limit")]
    ExcerptExceeded,
    #[error("Fix excerpt limit exceeds the {DEFAULT_MAX_EXCERPT_BYTES}-byte ceiling")]
    ExcerptLimitExceeded,
    #[error("Fix excerpt limit must be greater than zero")]
    ExcerptLimitZero,
    #[error("Fix finding exceeds the configured context limit")]
    FindingContextExceeded,
    #[error("Fix finding limit exceeds the {DEFAULT_MAX_FINDINGS_PER_BATCH}-finding ceiling")]
    FindingLimitExceeded,
    #[error("Fix finding limit must be greater than zero")]
    FindingLimitZero,
    #[error("Fix actions are empty, duplicated, or invalid")]
    InvalidActions,
    #[error("Fix confidence is invalid or below the configured threshold")]
    InvalidConfidence,
    #[error("Fix finding reference contains an empty required field")]
    InvalidFinding,
    #[error("Fix finding reference contains an invalid SHA-256 fingerprint")]
    InvalidFingerprint,
    #[error("Fix identifier cannot be empty")]
    InvalidIdentifier,
    #[error("Fix minimum confidence must be finite and between zero and one")]
    InvalidMinimumConfidence,
    #[error("Inference returned a fix status reserved for trusted execution")]
    InvalidStatus,
    #[error("Fix finding message exceeds {MAX_MESSAGE_BYTES} bytes")]
    MessageExceeded,
    #[error("Fix summary and rationale are required")]
    MissingDescription,
    #[error("Fix suggestion request contains duplicate finding identifiers")]
    RepeatedFinding,
    #[error("Fix source fingerprint is stale or invented")]
    StaleFingerprint,
    #[error("Fix inference timeout must be greater than zero")]
    TimeoutZero,
    #[error("Fix target is unsafe or was not supplied by its finding")]
    UnsafeTarget,
}
/// Risk classification for a proposed fix.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize, strum::EnumIs)]
#[serde(rename_all = "lowercase")]
pub enum FixRisk {
    /// Low-risk, bounded proposal.
    Low,
    /// Proposal requiring additional review.
    Medium,
    /// Proposal with significant potential impact.
    High,
}
/// Lifecycle state of a fix proposal.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize, strum::EnumIs)]
#[serde(rename_all = "lowercase")]
pub enum FixStatus {
    /// Proposed but not applied.
    Proposed,
    /// No safe proposal is available.
    Unsupported,
    /// Proposal failed local validation or policy.
    Rejected,
    /// Proposal no longer matches its source.
    Stale,
    /// Trusted execution applied and verified the fix.
    Applied,
    /// Trusted execution failed.
    Failed,
}
/// Fixes grouped by analysis standard and category.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryFixes {
    /// Finding category.
    pub category: String,
    /// Ordered fixes in this group.
    pub fixes: FixBatch,
    /// Analysis standard, absent for discovery findings.
    pub standard: Option<String>,
}
/// Normalized analyzer check supplied to proposal generation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckReference {
    /// Analyzer check category.
    pub category: String,
    /// Bounded source context.
    pub context: Option<String>,
    /// SHA-256 fingerprint of canonical identity and bounded context.
    pub fingerprint: String,
    /// Request-local finding identifier.
    pub id: String,
    /// Optional check locator.
    pub locator: Option<String>,
    /// Bounded diagnostic message.
    pub message: String,
    /// Stable rule identifier supplied by the analyzer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    /// Check severity.
    pub severity: String,
    /// Metadata standard used by the analysis batch.
    pub standard: String,
    /// Optional source URI.
    pub uri: Option<String>,
    /// Structured controlled-vocabulary evidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary_advisory: Option<Box<VocabularyAdvisory>>,
}
/// Normalized discovery finding supplied to proposal generation.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryReference {
    /// Discovery category.
    pub category: DiscoveryCategory,
    /// Structured, secret-free context.
    pub details: BTreeMap<String, String>,
    /// SHA-256 fingerprint of the observation.
    pub fingerprint: String,
    /// Request-local finding identifier.
    pub id: String,
    /// Expected state.
    pub expected: String,
    /// Observed state.
    pub observed: String,
    /// Discovery status captured at observation time.
    pub status: DiscoveryStatus,
    /// Diagnostic subject.
    pub subject: String,
}
/// Reviewable proposal linked to one source finding.
#[derive(bon::Builder, Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields)]
pub struct Fix {
    /// Proposed non-executable actions.
    pub actions: Vec<FixAction>,
    /// Proposed automation level.
    pub automation: FixAutomation,
    /// Model or provider confidence from zero through one.
    pub confidence: Noul,
    /// Declared effects that trusted planning must enforce.
    pub effects: FixEffects,
    /// Source finding.
    pub finding: FindingReference,
    /// Request-local fix identifier.
    pub id: String,
    /// Secret-free provenance.
    pub provenance: FixProvenance,
    /// Human-readable rationale.
    pub rationale: String,
    /// Risk classification.
    pub risk: FixRisk,
    /// Proposal lifecycle state.
    pub status: FixStatus,
    /// Concise proposal summary.
    pub summary: String,
    /// Finding-owned target or request-local finding identifier.
    pub target: String,
}
/// Ordered proposal collection.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct FixBatch {
    /// Fixes in deterministic order.
    pub values: OneOrMany<Fix>,
}
/// Non-fatal diagnostic produced while suggesting one batch.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixDiagnostic {
    /// Finding identifiers affected by the diagnostic.
    pub finding_ids: Vec<String>,
    /// Bounded public diagnostic.
    pub message: String,
}
/// Effects declared by a proposal without granting permission to perform them.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixEffects {
    /// Whether the proposal could require elevation.
    pub elevation: bool,
    /// Whether the proposal could modify files.
    pub filesystem_write: bool,
    /// Whether the proposal could use a package manager.
    pub package_manager: bool,
    /// Whether the proposal could access a network.
    pub network: bool,
    /// Whether the proposal could start a process.
    pub process: bool,
    /// Whether the proposed change is reversible.
    pub reversible: bool,
    /// Whether the proposal could require a restart.
    pub restart: bool,
}
/// Bounded suggestion options shared by direct and registered invocation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FixOptions {
    /// Optional backend-specific agent selection.
    pub agent: Option<String>,
    /// Include already successful checks when alternatives are requested.
    pub include_successful: bool,
    /// Maximum aggregate context bytes in one inference request.
    pub max_context_bytes: usize,
    /// Maximum source excerpt bytes retained for one finding.
    pub max_excerpt_bytes: usize,
    /// Maximum findings in one inference request.
    pub max_findings_per_batch: usize,
    /// Minimum accepted proposal confidence.
    pub min_confidence: f32,
    /// Optional backend-specific model selection.
    pub model: Option<String>,
    /// Inference timeout.
    pub timeout: Milliseconds,
}
/// Secret-free origin metadata for a fix proposal.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixProvenance {
    /// Agent identifier when supplied by the backend.
    pub agent: Option<String>,
    /// Stable backend kind.
    pub backend: String,
    /// Model identifier when supplied by the backend.
    pub model: Option<String>,
    /// Prompt/envelope contract version.
    pub prompt_version: String,
    /// Secret-free request correlation identifier.
    pub request_id: Option<String>,
}
/// Grouped fix proposal report.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixReport {
    /// Fix groups in deterministic order.
    pub categories: Vec<CategoryFixes>,
    /// Non-fatal batch diagnostics.
    pub diagnostics: Vec<FixDiagnostic>,
}
/// Registered-operation input containing already normalized findings.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixSuggestionRequest {
    /// Suggestion policy and request limits.
    pub options: FixOptions,
    /// Normalized analyzer findings.
    pub references: Vec<CheckReference>,
}
/// Additive fix counts by lifecycle state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FixSummary {
    /// Applied fixes.
    pub applied: usize,
    /// Failed fixes.
    pub failed: usize,
    /// Proposed fixes.
    pub proposed: usize,
    /// Rejected fixes.
    pub rejected: usize,
    /// Stale fixes.
    pub stale: usize,
    /// Unsupported findings.
    pub unsupported: usize,
}
impl From<&str> for DiscoveryCategory {
    fn from(value: &str) -> Self {
        match value {
            | "gpu" => Self::Gpu,
            | "memory" => Self::Memory,
            | "network" => Self::Network,
            | "software" => Self::Software,
            | "system" => Self::System,
            | _ => Self::Unknown,
        }
    }
}
impl FixEffects {
    pub(super) const fn combine(self, other: Self) -> Self {
        Self {
            elevation: self.elevation || other.elevation,
            filesystem_write: self.filesystem_write || other.filesystem_write,
            package_manager: self.package_manager || other.package_manager,
            network: self.network || other.network,
            process: self.process || other.process,
            reversible: self.reversible && other.reversible,
            restart: self.restart || other.restart,
        }
    }
    pub(super) fn recipe(package_manager: bool) -> Self {
        Self {
            filesystem_write: true,
            network: true,
            package_manager,
            process: true,
            ..Self::default()
        }
    }
}
