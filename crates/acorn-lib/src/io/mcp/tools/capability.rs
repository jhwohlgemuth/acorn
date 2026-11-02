//! Versioned capability contracts and deterministic executor selection.

use super::{CapabilityHandler, ToolHandler, ToolRegistry};
use crate::error::ApiResult;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::SemanticVersion;
use acorn_core::validation::ValidationError;
use acorn_schema::agent::tools::{NeedleToolDefinition, ToolEffects};
use acorn_schema::validation::{is_identifier, Validate, ValidationReport};
use alloc::sync::Arc;
use core::future::Future;
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// Maximum serialized catalog size accepted by Needle.
pub const MAX_CAPABILITY_CATALOG_BYTES: usize = 32 * 1024;
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
/// One implementation of a capability contract.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BindingDefinition {
    /// Stable configured binding identifier.
    pub id: String,
    /// Capability implemented by this binding.
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
    /// Namespaced capability identifier.
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
        if !valid_id(&self.id) || !self.id.contains('.') {
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
    #[cfg(feature = "mcp-client")]
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
        if !valid_id(&definition.id) || !valid_id(&definition.capability_id) {
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
                    parameters: entry.definition.input_schema.clone(),
                    triggers: Vec::new(),
                })
                .collect::<Vec<_>>();
            serde_json::to_string(&definitions)
                .map_err(|why| CapabilityError::CatalogSerialization(why.to_string()).into())
                .and_then(|serialized| {
                    if serialized.len() > MAX_CAPABILITY_CATALOG_BYTES {
                        Err(CapabilityError::CatalogTooLarge.into())
                    } else {
                        Ok(serialized)
                    }
                })
        })
    }
}
fn schema<T: JsonSchema>() -> Value {
    serde_json::to_value(schema_for!(T)).unwrap_or_else(|_| serde_json::json!({ "type": "object" }))
}
fn valid_id(id: &str) -> bool {
    id.len() <= 128 && id.split(':').all(is_identifier)
}
