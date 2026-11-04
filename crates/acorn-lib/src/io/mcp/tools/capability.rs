//! Versioned capability contracts and deterministic executor selection.

use super::{CapabilityHandler, ToolHandler, ToolRegistry};
use crate::error::ApiResult;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_core::validation::ValidationError;
use acorn_schema::agent::tools::{NeedleToolDefinition, ToolEffects};
use acorn_schema::hardware::memory::Memory;
use acorn_schema::validation::{is_identifier, Validate, ValidationReport};
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use core::future::Future;
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// Maximum serialized catalog size accepted by Needle.
pub const MAX_CAPABILITY_CATALOG_BYTES: usize = 32 * 1024;
const MAX_PROMPT_CATALOG_BYTES: usize = 22 * 1024;
/// Readiness established outside registration, with a reason for unavailable bindings.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum BindingAvailability {
    /// The binding is ready for selection.
    Ready,
    /// The binding cannot run in the current host/configuration.
    Unavailable {
        /// Human-readable feature, host, asset, or configuration reason.
        reason: String,
    },
}
/// The implementation class used only for deterministic binding preference.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BindingKind {
    /// In-process Rust implementation.
    Native,
    /// Explicitly configured local model or service.
    Local,
    /// Explicitly configured external service.
    Remote,
}
/// Invalid capability contracts, selections, and catalogs.
#[derive(Debug, Error)]
pub enum CapabilityError {
    /// More than one eligible external binding exists.
    #[error("Capability '{0}' has ambiguous external bindings; select one explicitly")]
    AmbiguousBindings(String),
    /// Serialization of the capability catalog failed.
    #[error("Failed to serialize capability catalog — {0}")]
    CatalogSerialization(String),
    /// The catalog exceeds Needle's bounded input.
    #[error("Capability catalog exceeds the {MAX_CAPABILITY_CATALOG_BYTES}-byte Needle limit; narrow the explicit capability scope")]
    CatalogTooLarge,
    /// A binding identifier is already registered.
    #[error("Duplicate binding ID '{0}'")]
    DuplicateBinding(String),
    /// A capability identifier is already registered with another contract.
    #[error("Capability '{0}' is already registered with a different contract")]
    ConflictingCapability(String),
    /// A capability identifier is already registered.
    #[error("Duplicate capability ID '{0}'")]
    DuplicateCapability(String),
    /// No capability was selected for a scoped catalog.
    #[error("Capability scope is empty; select at least one capability")]
    EmptyScope,
    /// A binding identifier is malformed.
    #[error("Binding requires valid IDs")]
    InvalidBindingId,
    /// A capability contract failed validation.
    #[error("Invalid capability — {0}")]
    InvalidDefinition(String),
    /// The binding contract differs from the canonical capability.
    #[error("Binding '{binding}' does not match capability '{capability}' contract")]
    MismatchedContract {
        /// Binding identifier.
        binding: String,
        /// Capability identifier.
        capability: String,
    },
    /// No binding is eligible for the requested capability.
    #[error("Capability '{0}' has no available binding")]
    NoAvailableBinding(String),
    /// A caller selected a binding barred by readiness or policy.
    #[error("Explicit binding '{binding}' is unavailable for capability '{capability}': {reason}")]
    UnavailableExplicitBinding {
        /// Binding identifier.
        binding: String,
        /// Capability identifier.
        capability: String,
        /// Readiness or policy reason.
        reason: String,
    },
    /// A scoped catalog names a capability with no eligible binding.
    #[error("Capability scope '{0}' has no available binding")]
    UnavailableScope(String),
    /// A requested capability identifier is unknown.
    #[error("Unknown capability '{0}'")]
    UnknownCapability(String),
    /// A caller selected an unknown binding.
    #[error("Unknown explicit binding '{binding}' for capability '{capability}'")]
    UnknownExplicitBinding {
        /// Binding identifier.
        binding: String,
        /// Capability identifier.
        capability: String,
    },
    /// A scoped catalog names an unknown capability.
    #[error("Unknown capability scope '{0}'")]
    UnknownScope(String),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Family {
    Check,
    Diff,
    Doctor,
    Download,
    Exec,
    Export,
    Format,
    Gather,
    Import,
    Link,
    Report,
    Swhid,
    Sync,
    Unknown,
}
/// One implementation of a capability contract.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BindingDefinition {
    /// Stable configured binding identifier.
    ///
    /// The current convention is `<modifier>:<namespace>.<operation>[::<scope>...]`, for example `native:acorn.export::render`.
    /// The modifier identifies the executor binding; the corresponding capability ID omits it.
    pub id: String,
    /// Capability implemented by this binding, using `<namespace>.<operation>[::<scope>...]` (for example, `acorn.export::render`).
    pub capability_id: String,
    /// Contract version implemented by this binding.
    pub version: SemanticVersion,
    /// Executor class.
    pub kind: BindingKind,
    /// Binding input schema; must match the capability exactly.
    pub input_schema: Value,
    /// Binding output schema; must match the capability exactly.
    pub output_schema: Value,
    /// Binding effects; must match the capability exactly.
    pub effects: ToolEffects,
    /// Current readiness and its reason.
    pub availability: BindingAvailability,
    /// The trusted handler can run a restricted local mode while offline.
    pub offline_safe: bool,
}
/// A stable, versioned operation independent of its executor.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CapabilityDefinition {
    /// Namespaced capability identifier using
    /// `<namespace>.<operation>[::<scope>...]` (for example, `acorn.export::render`).
    pub id: String,
    /// Contract version; bindings must declare this exact version.
    pub version: SemanticVersion,
    /// Bounded description shown to the router.
    pub description: String,
    /// Canonical input JSON Schema.
    pub input_schema: Value,
    /// Canonical output JSON Schema.
    pub output_schema: Value,
    /// Declared operation effects.
    pub effects: ToolEffects,
}
#[derive(Clone)]
pub(super) struct CapabilityEntry {
    definition: CapabilityDefinition,
    bindings: Vec<RegisteredBinding>,
}
struct CapabilityIdentifier<'a>(&'a str);
/// Caller constraints used when resolving a compatible binding.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CapabilitySelectionContext {
    /// Reject external bindings when true.
    pub offline: bool,
    /// Permit external bindings when true and online.
    pub allow_remote: bool,
}
#[derive(Clone)]
struct RegisteredBinding {
    definition: BindingDefinition,
    handler: CapabilityHandler,
}
/// Deterministic prompt-scoped catalog and its measured retention.
#[derive(Clone, Debug, PartialEq)]
pub struct ScopedCatalog {
    /// Explicit retained capability IDs, or `None` when the full catalog fits.
    pub ids: Option<Vec<String>>,
    /// Serialized bytes sent to Needle.
    pub bytes: Memory,
    /// Number of capabilities omitted to respect the catalog limit.
    pub omitted: usize,
    /// Serialized bytes before prompt scoping.
    pub unscoped_bytes: Memory,
}
/// Resolved capability and one eligible executor binding.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SelectedBinding {
    /// Canonical operation contract.
    pub capability: CapabilityDefinition,
    /// The selected executor contract.
    pub binding: BindingDefinition,
}
impl BindingAvailability {
    fn reason(&self, binding: &BindingDefinition, context: CapabilitySelectionContext) -> Option<String> {
        match self {
            | Self::Unavailable { reason } => Some(reason.clone()),
            | Self::Ready if binding.kind == BindingKind::Remote && context.offline => Some("offline policy denies remote bindings".to_string()),
            | Self::Ready if binding.effects.open_world && context.offline && !binding.offline_safe => {
                Some("offline policy denies this open-world binding".to_string())
            }
            | Self::Ready if binding.kind == BindingKind::Remote && !context.allow_remote => Some("caller policy denies remote bindings".to_string()),
            | Self::Ready => None,
        }
    }
}
impl BindingDefinition {
    /// Declare a typed executor contract for one capability.
    pub fn new<I: JsonSchema, O: JsonSchema>(
        id: impl Into<String>,
        capability_id: impl Into<String>,
        version: SemanticVersion,
        kind: BindingKind,
        effects: ToolEffects,
        availability: BindingAvailability,
    ) -> Self {
        Self {
            id: id.into(),
            capability_id: capability_id.into(),
            version,
            kind,
            input_schema: schema::<I>(),
            output_schema: schema::<O>(),
            effects,
            availability,
            offline_safe: false,
        }
    }
    /// Mark a trusted native or local handler as capable of an offline-safe mode.
    pub fn with_offline_support(self) -> Self {
        Self { offline_safe: true, ..self }
    }
}
impl CapabilityDefinition {
    /// Create a capability from typed input and output contracts.
    pub fn new<I: JsonSchema, O: JsonSchema>(
        id: impl Into<String>,
        version: SemanticVersion,
        description: impl Into<String>,
        effects: ToolEffects,
    ) -> Self {
        Self {
            id: id.into(),
            version,
            description: description.into(),
            input_schema: schema::<I>(),
            output_schema: schema::<O>(),
            effects,
        }
    }
}
impl Validate for CapabilityDefinition {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if CapabilityIdentifier(&self.id).validate().is_err() || !self.id.contains('.') {
            report.add("id", ValidationError::new("invalid").with_message("Capability requires a namespaced ID"));
        }
        if self.version == SemanticVersion::default() {
            report.add(
                "version",
                ValidationError::new("invalid").with_message("Capability requires a nonzero version"),
            );
        }
        if self.description.trim().is_empty() {
            report.add(
                "description",
                ValidationError::new("required").with_message("Capability requires a description"),
            );
        }
        if self.input_schema.get("type").and_then(Value::as_str) != Some("object") {
            report.add(
                "input_schema",
                ValidationError::new("invalid").with_message("Capability requires an object input schema"),
            );
        }
        if jsonschema::validator_for(&self.input_schema).is_err() {
            report.add(
                "input_schema",
                ValidationError::new("invalid").with_message("Capability input schema must be valid JSON Schema"),
            );
        }
        if jsonschema::validator_for(&self.output_schema).is_err() {
            report.add(
                "output_schema",
                ValidationError::new("invalid").with_message("Capability output schema must be valid JSON Schema"),
            );
        }
        report.finish()
    }
}
impl Validate for CapabilityIdentifier<'_> {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        let scoped = match self.0.split_once(':') {
            | Some((prefix, scoped)) if !scoped.starts_with(':') => is_identifier(prefix) && scoped.split("::").all(is_identifier),
            | _ => self.0.split("::").all(is_identifier),
        };
        if self.0.len() > 128 || !scoped {
            report.add("id", ValidationError::new("invalid").with_message("Identifier is malformed"));
        }
        report.finish()
    }
}
impl Family {
    fn group(self) -> usize {
        match self {
            | Self::Check | Self::Diff | Self::Doctor => 0,
            | Self::Exec | Self::Gather | Self::Swhid => 1,
            | Self::Export => 2,
            | Self::Format | Self::Link => 3,
            | Self::Download | Self::Import => 4,
            | Self::Report | Self::Sync => 5,
            | Self::Unknown => 6,
        }
    }
    fn keyword(self) -> &'static str {
        match self {
            | Self::Check => "check",
            | Self::Diff => "diff",
            | Self::Doctor => "doctor",
            | Self::Download => "download",
            | Self::Exec => "exec",
            | Self::Export => "export",
            | Self::Format => "format",
            | Self::Gather => "gather",
            | Self::Import => "import",
            | Self::Link => "link",
            | Self::Report => "report",
            | Self::Swhid => "swhid",
            | Self::Sync => "sync",
            | Self::Unknown => "",
        }
    }
    fn prompt_score(self, definition: &CapabilityDefinition, words: &BTreeSet<String>) -> usize {
        let name = definition.id.split('.').nth(1).unwrap_or_default();
        let family = self.keyword();
        let aliases: &[&str] = match self {
            | Self::Check => &["analyze", "inspect", "validate", "validation", "quality", "findings"],
            | Self::Diff => &[
                "compare",
                "difference",
                "jaccard",
                "jaro",
                "levenshtein",
                "similarity",
                "patch",
                "cosine",
                "dice",
            ],
            | Self::Doctor => &["diagnostic", "diagnostics", "readiness", "memory", "gpu"],
            | Self::Download => &["acquire", "fetch", "weights", "bucket"],
            | Self::Exec => &["script", "rhai", "hook", "manifest"],
            | Self::Export => &["render", "pdf", "powerpoint", "archive", "bagit", "jsonl"],
            | Self::Format => &["normalize", "normalization", "enrich", "formatter", "crosswalk"],
            | Self::Gather => &["extract", "identifier", "identifiers", "pid", "doi", "orcid", "rrid"],
            | Self::Import => &["persist", "endpoint", "hugging", "metadata"],
            | Self::Link => &["linked", "context", "jsonld"],
            | Self::Report => &["publish", "publication", "highlights", "logbook"],
            | Self::Swhid => &["hash", "revision", "identifier"],
            | Self::Sync => &["opencode", "vscode", "goose", "configuration", "editor"],
            | Self::Unknown => &[],
        };
        let family_score: usize = match (words.contains(family), aliases.iter().any(|alias| words.contains(*alias))) {
            | (true, _) => 200,
            | (false, true) if self == Self::Exec => 260,
            | (false, true) => 120,
            | (false, false) => 0,
        };
        let name_score = name
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|part| words.contains(*part))
            .count()
            .saturating_mul(35);
        let description_score = definition
            .description
            .to_ascii_lowercase()
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|word| word.len() >= 4 && words.contains(*word))
            .count()
            .saturating_mul(3);
        family_score.saturating_add(name_score).saturating_add(description_score)
    }
}
impl From<&str> for Family {
    fn from(id: &str) -> Self {
        let scoped = id.split_once(':').map_or(id, |(_, scoped)| scoped);
        match scoped.split('.').nth(1).unwrap_or_default().split(['_', ':']).next().unwrap_or_default() {
            | "check" => Self::Check,
            | "diff" => Self::Diff,
            | "doctor" => Self::Doctor,
            | "download" => Self::Download,
            | "exec" => Self::Exec,
            | "export" => Self::Export,
            | "format" => Self::Format,
            | "gather" => Self::Gather,
            | "import" => Self::Import,
            | "link" => Self::Link,
            | "report" => Self::Report,
            | "swhid" => Self::Swhid,
            | "sync" => Self::Sync,
            | _ => Self::Unknown,
        }
    }
}
impl From<&String> for Family {
    fn from(id: &String) -> Self {
        Self::from(id.as_str())
    }
}
impl From<String> for Family {
    fn from(id: String) -> Self {
        Self::from(id.as_str())
    }
}
impl ToolRegistry {
    #[cfg(feature = "needle")]
    pub(crate) fn has_capabilities(&self) -> bool {
        !self.capabilities.is_empty()
    }
    pub(super) fn current_handler(&self, selected: &SelectedBinding, context: CapabilitySelectionContext) -> Option<CapabilityHandler> {
        self.capabilities
            .iter()
            .find(|entry| entry.definition == selected.capability)
            .and_then(|entry| entry.bindings.iter().find(|binding| binding.definition == selected.binding))
            .filter(|binding| binding.definition.availability.reason(&binding.definition, context).is_none())
            .map(|binding| binding.handler.clone())
    }
    /// Add one canonical capability, rejecting duplicate IDs.
    pub fn register_capability(mut self, definition: CapabilityDefinition) -> ApiResult<Self> {
        definition
            .validate()
            .map_err(|errors| CapabilityError::InvalidDefinition(errors.to_string()).into())
            .and_then(|()| {
                if self.capabilities.iter().any(|entry| entry.definition.id == definition.id) {
                    Err(CapabilityError::DuplicateCapability(definition.id).into())
                } else {
                    self.capabilities.push(CapabilityEntry {
                        definition,
                        bindings: Vec::new(),
                    });
                    self.capabilities.sort_by(|left, right| left.definition.id.cmp(&right.definition.id));
                    Ok(self)
                }
            })
    }
    pub(super) fn register_dynamic_capability(self, definition: CapabilityDefinition) -> ApiResult<Self> {
        match self.capabilities.iter().find(|entry| entry.definition.id == definition.id) {
            | Some(entry) if entry.definition == definition => Ok(self),
            | Some(_) => Err(CapabilityError::ConflictingCapability(definition.id).into()),
            | None => self.register_capability(definition),
        }
    }
    /// Add one typed executor after checking its version, schemas, and effects.
    pub fn register_capability_binding<I, O, F, Fut>(self, definition: BindingDefinition, handler: F) -> ApiResult<Self>
    where
        I: DeserializeOwned + JsonSchema + Send + 'static,
        O: Serialize + JsonSchema + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        let matches_types = definition.input_schema == schema::<I>() && definition.output_schema == schema::<O>();
        match matches_types {
            | false => Err(CapabilityError::MismatchedContract {
                binding: definition.id.clone(),
                capability: definition.capability_id.clone(),
            }
            .into()),
            | true => {
                let handler = ToolHandler::new::<I, O, F, Fut>(definition.id.clone(), Arc::new(handler));
                self.register_dynamic_binding(definition, handler.into())
            }
        }
    }
    pub(super) fn register_dynamic_binding(mut self, definition: BindingDefinition, handler: CapabilityHandler) -> ApiResult<Self> {
        let duplicate = self
            .capabilities
            .iter()
            .flat_map(|entry| &entry.bindings)
            .any(|binding| binding.definition.id == definition.id);
        if CapabilityIdentifier(&definition.id).validate().is_err() || CapabilityIdentifier(&definition.capability_id).validate().is_err() {
            Err(CapabilityError::InvalidBindingId.into())
        } else if duplicate {
            Err(CapabilityError::DuplicateBinding(definition.id).into())
        } else {
            self.capabilities
                .iter_mut()
                .find(|entry| entry.definition.id == definition.capability_id)
                .ok_or_else(|| CapabilityError::UnknownCapability(definition.capability_id.clone()))
                .map_err(Into::into)
                .and_then(|entry| {
                    let canonical = &entry.definition;
                    let matches_contract = definition.version == canonical.version
                        && definition.input_schema == canonical.input_schema
                        && definition.output_schema == canonical.output_schema
                        && definition.effects == canonical.effects;
                    if !matches_contract {
                        Err(CapabilityError::MismatchedContract {
                            binding: definition.id.clone(),
                            capability: canonical.id.clone(),
                        }
                        .into())
                    } else {
                        entry.bindings.push(RegisteredBinding { definition, handler });
                        entry.bindings.sort_by(|left, right| left.definition.id.cmp(&right.definition.id));
                        Ok(())
                    }
                })
                .map(|()| self)
        }
    }
    /// Resolve an eligible binding: explicit choice, configured preference, native, then local.
    /// Equally ranked remote alternatives require a caller choice.
    pub fn select_capability_binding(
        &self,
        capability_id: &str,
        explicit_binding: Option<&str>,
        preferred_binding: Option<&str>,
        context: CapabilitySelectionContext,
    ) -> ApiResult<SelectedBinding> {
        self.capabilities
            .iter()
            .find(|entry| entry.definition.id == capability_id)
            .ok_or_else(|| CapabilityError::UnknownCapability(capability_id.to_string()))
            .map_err(Into::into)
            .and_then(|entry| {
                let eligible = |binding: &RegisteredBinding| binding.definition.availability.reason(&binding.definition, context).is_none();
                if let Some(id) = explicit_binding {
                    match entry.bindings.iter().find(|binding| binding.definition.id == id) {
                        | Some(binding) => match binding.definition.availability.reason(&binding.definition, context) {
                            | None => Ok(SelectedBinding {
                                capability: entry.definition.clone(),
                                binding: binding.definition.clone(),
                            }),
                            | Some(reason) => Err(CapabilityError::UnavailableExplicitBinding {
                                binding: id.to_string(),
                                capability: capability_id.to_string(),
                                reason,
                            }
                            .into()),
                        },
                        | None => Err(CapabilityError::UnknownExplicitBinding {
                            binding: id.to_string(),
                            capability: capability_id.to_string(),
                        }
                        .into()),
                    }
                } else {
                    let preferred =
                        preferred_binding.and_then(|id| entry.bindings.iter().find(|binding| binding.definition.id == id && eligible(binding)));
                    let native = entry
                        .bindings
                        .iter()
                        .find(|binding| binding.definition.kind == BindingKind::Native && eligible(binding));
                    let local = entry
                        .bindings
                        .iter()
                        .find(|binding| binding.definition.kind == BindingKind::Local && eligible(binding));
                    let remote = entry
                        .bindings
                        .iter()
                        .filter(|binding| binding.definition.kind == BindingKind::Remote && eligible(binding))
                        .collect::<Vec<_>>();
                    let chosen = preferred.or(native).or(local).or(match remote.as_slice() {
                        | [one] => Some(*one),
                        | _ => None,
                    });
                    match chosen {
                        | Some(binding) => Ok(SelectedBinding {
                            capability: entry.definition.clone(),
                            binding: binding.definition.clone(),
                        }),
                        | None if remote.len() > 1 => Err(CapabilityError::AmbiguousBindings(capability_id.to_string()).into()),
                        | None => Err(CapabilityError::NoAvailableBinding(capability_id.to_string()).into()),
                    }
                }
            })
    }
    /// Serialize an explicitly scoped capability catalog within Needle's byte cap.
    /// Binding alternatives are deduplicated behind their capability contract.
    pub fn capability_catalog(&self, scope: Option<&[String]>, context: CapabilitySelectionContext) -> ApiResult<String> {
        self.serialize_capability_catalog(scope, context)
            .and_then(|serialized| match serialized.len() <= MAX_CAPABILITY_CATALOG_BYTES {
                | true => Ok(serialized),
                | false => Err(CapabilityError::CatalogTooLarge.into()),
            })
    }
    /// Choose a deterministic prompt shortlist when the eligible catalog exceeds Needle's limit.
    pub fn scope_for_prompt(&self, prompt: &str, context: CapabilitySelectionContext) -> ApiResult<ScopedCatalog> {
        self.serialize_capability_catalog(None, context).and_then(|unscoped| {
            let unscoped_bytes = unscoped.len();
            match unscoped_bytes <= MAX_PROMPT_CATALOG_BYTES {
                | true => Ok(ScopedCatalog {
                    ids: None,
                    bytes: Memory::from(unscoped_bytes as u64),
                    omitted: 0,
                    unscoped_bytes: Memory::from(unscoped_bytes as u64),
                }),
                | false => {
                    let words = prompt
                        .to_ascii_lowercase()
                        .split(|character: char| !character.is_ascii_alphanumeric())
                        .filter(|word| !word.is_empty())
                        .map(str::to_string)
                        .collect::<BTreeSet<_>>();
                    let mut ranked = self
                        .capabilities
                        .iter()
                        .filter(|entry| {
                            entry
                                .bindings
                                .iter()
                                .any(|binding| binding.definition.availability.reason(&binding.definition, context).is_none())
                        })
                        .map(|entry| {
                            (
                                Family::from(&entry.definition.id).prompt_score(&entry.definition, &words),
                                entry.definition.id.clone(),
                            )
                        })
                        .collect::<Vec<_>>();
                    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
                    let groups = ranked
                        .iter()
                        .map(|(_, id)| Family::from(id).group())
                        .fold(Vec::<usize>::new(), |mut groups, group| {
                            if !groups.contains(&group) {
                                groups.push(group);
                            }
                            groups
                        })
                        .into_iter()
                        .take(2)
                        .collect::<Vec<_>>();
                    let mut grouped = ranked
                        .iter()
                        .filter(|(_, id)| groups.contains(&Family::from(id).group()))
                        .map(|(_, id)| id.clone())
                        .collect::<Vec<_>>();
                    grouped.extend(
                        [format!("{APPLICATION}.check"), format!("{APPLICATION}.export::bag")]
                            .into_iter()
                            .filter_map(|id| {
                                ranked
                                    .iter()
                                    .find(|(_, available)| available == &id)
                                    .map(|(_, available)| available.clone())
                            }),
                    );
                    grouped.sort();
                    grouped.dedup();
                    let outside_group = ranked.len().saturating_sub(grouped.len());
                    self.serialize_capability_catalog(Some(&grouped), context)
                        .and_then(|catalog| match catalog.len() <= MAX_PROMPT_CATALOG_BYTES {
                            | true => Ok(ScopedCatalog {
                                ids: Some(grouped.clone()),
                                bytes: Memory::from(catalog.len() as u64),
                                omitted: outside_group,
                                unscoped_bytes: Memory::from(unscoped_bytes as u64),
                            }),
                            | false => ranked
                                .into_iter()
                                .filter(|(_, id)| groups.contains(&Family::from(id).group()))
                                .try_fold((Vec::<String>::new(), 0_usize, 2_usize), |(ids, omitted, bytes), (_, id)| {
                                    let mut candidate = ids.clone();
                                    candidate.push(id);
                                    self.serialize_capability_catalog(Some(&candidate), context).map(|candidate_catalog| {
                                        match candidate_catalog.len() <= MAX_PROMPT_CATALOG_BYTES {
                                            | true => (candidate, omitted, candidate_catalog.len()),
                                            | false => (ids, omitted.saturating_add(1), bytes),
                                        }
                                    })
                                })
                                .and_then(|(ids, omitted, bytes)| match ids.is_empty() {
                                    | true => Err(CapabilityError::CatalogTooLarge.into()),
                                    | false => Ok(ScopedCatalog {
                                        ids: Some({
                                            let mut ids = ids;
                                            ids.sort();
                                            ids
                                        }),
                                        bytes: Memory::from(bytes as u64),
                                        omitted: omitted.saturating_add(outside_group),
                                        unscoped_bytes: Memory::from(unscoped_bytes as u64),
                                    }),
                                }),
                        })
                }
            }
        })
    }
    fn serialize_capability_catalog(&self, scope: Option<&[String]>, context: CapabilitySelectionContext) -> ApiResult<String> {
        let valid_scope = match scope {
            | Some([]) => Err(CapabilityError::EmptyScope.into()),
            | Some(ids) => ids
                .iter()
                .try_for_each(|id| match self.capabilities.iter().find(|entry| &entry.definition.id == id) {
                    | None => Err(CapabilityError::UnknownScope(id.clone()).into()),
                    | Some(entry)
                        if !entry
                            .bindings
                            .iter()
                            .any(|binding| binding.definition.availability.reason(&binding.definition, context).is_none()) =>
                    {
                        Err(CapabilityError::UnavailableScope(id.clone()).into())
                    }
                    | Some(_) => Ok(()),
                }),
            | None => Ok(()),
        };
        valid_scope.and_then(|()| {
            let definitions = self
                .capabilities
                .iter()
                .filter(|entry| {
                    scope.is_none_or(|ids| ids.contains(&entry.definition.id))
                        && entry
                            .bindings
                            .iter()
                            .any(|binding| binding.definition.availability.reason(&binding.definition, context).is_none())
                })
                .map(|entry| NeedleToolDefinition {
                    name: entry.definition.id.clone(),
                    description: entry.definition.description.clone(),
                    parameters: catalog_schema(&entry.definition.input_schema, false),
                    triggers: Vec::new(),
                })
                .collect::<Vec<_>>();
            serde_json::to_string(&definitions).map_err(|why| CapabilityError::CatalogSerialization(why.to_string()).into())
        })
    }
}
pub(super) fn catalog_schema(schema: &Value, named_schemas: bool) -> Value {
    match schema {
        | Value::Object(fields) => Value::Object(
            fields
                .iter()
                .filter(|(key, _)| named_schemas || !matches!(key.as_str(), "$schema" | "title" | "description"))
                .map(|(key, value)| {
                    let projected = match key.as_str() {
                        | "properties" | "patternProperties" | "$defs" | "definitions" | "dependentSchemas" => catalog_schema(value, true),
                        | "const" | "default" | "enum" | "examples" => value.clone(),
                        | _ => catalog_schema(value, false),
                    };
                    (key.clone(), projected)
                })
                .collect(),
        ),
        | Value::Array(values) => Value::Array(values.iter().map(|value| catalog_schema(value, false)).collect()),
        | value => value.clone(),
    }
}
fn schema<T: JsonSchema>() -> Value {
    serde_json::to_value(schema_for!(T)).unwrap_or_else(|_| serde_json::json!({ "type": "object" }))
}
