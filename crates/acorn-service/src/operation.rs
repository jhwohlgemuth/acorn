//! Portable operation provenance and effect policy.
use acorn_core::prelude::{String, ToString};
pub use acorn_schema::hardware::Memory;
use core::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Source transport for an operation invocation.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationOrigin {
    /// Direct in-process invocation.
    Direct,
    /// Protobuf/gRPC invocation.
    Grpc,
    /// JSON-RPC invocation.
    JsonRpc,
    /// Normalized webhook invocation.
    Webhook,
}
/// One external effect declared by an operation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationEffect {
    /// Access credentials.
    Credentials,
    /// Destroy or irreversibly replace data.
    Destructive,
    /// Read from the host filesystem.
    FilesystemRead,
    /// Write to the host filesystem.
    FilesystemWrite,
    /// Invoke inference services.
    Inference,
    /// Change state.
    Mutation,
    /// Read from remote network services.
    NetworkRead,
    /// Start or communicate with processes.
    Process,
}
/// Reason an operation's declared effects are not permitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationPolicyError {
    /// Mutation approval was required but absent.
    MutationApprovalRequired,
    /// Network access was requested while offline.
    Offline,
    /// The composition cannot provide a declared effect.
    UnavailableEffect,
}
/// Policy and provenance supplied to one operation.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InvocationContext {
    /// Whether inference handlers may run.
    pub allow_inference: bool,
    /// Whether mutating handlers may run.
    pub allow_mutation: bool,
    /// Optional transport correlation identifier.
    pub correlation_id: Option<String>,
    /// Maximum serialized request size.
    pub max_input: Memory,
    /// Maximum serialized result size.
    pub max_output: Memory,
    /// Whether external-system interaction is forbidden.
    pub offline: bool,
    /// Transport that initiated the invocation.
    pub origin: InvocationOrigin,
    /// Authenticated principal.
    pub principal: Principal,
}
/// Effect metadata enforced before an operation handler runs.
#[derive(Clone, Copy, Debug, Default, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OperationEffects {
    /// Whether credentials may be accessed.
    pub credentials: bool,
    /// Whether data may be irreversibly changed.
    pub destructive: bool,
    /// Whether the filesystem may be read.
    pub filesystem_read: bool,
    /// Whether the filesystem may be written.
    pub filesystem_write: bool,
    /// Whether inference may be invoked.
    pub inference: bool,
    /// Whether state may change.
    pub mutation: bool,
    /// Whether remote services may be read.
    pub network_read: bool,
    /// Whether processes may be invoked.
    pub process: bool,
}
/// Authenticated caller identity supplied by an adapter.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Principal {
    /// Stable caller identifier.
    pub identifier: String,
}
impl Default for InvocationContext {
    fn default() -> Self {
        Self {
            allow_inference: false,
            allow_mutation: false,
            correlation_id: None,
            max_input: Memory::mb(1),
            max_output: Memory::mb(1),
            offline: false,
            origin: InvocationOrigin::Direct,
            principal: Principal {
                identifier: "local".to_string(),
            },
        }
    }
}
impl FromIterator<OperationEffect> for OperationEffects {
    fn from_iter<T: IntoIterator<Item = OperationEffect>>(effects: T) -> Self {
        effects.into_iter().fold(Self::default(), |mut declared, effect| {
            match effect {
                | OperationEffect::Credentials => declared.credentials = true,
                | OperationEffect::Destructive => declared.destructive = true,
                | OperationEffect::FilesystemRead => declared.filesystem_read = true,
                | OperationEffect::FilesystemWrite => declared.filesystem_write = true,
                | OperationEffect::Inference => declared.inference = true,
                | OperationEffect::Mutation => declared.mutation = true,
                | OperationEffect::NetworkRead => declared.network_read = true,
                | OperationEffect::Process => declared.process = true,
            }
            declared
        })
    }
}
impl fmt::Display for OperationPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::MutationApprovalRequired => formatter.write_str("Operation requires mutation approval"),
            | Self::Offline => formatter.write_str("Operation is unavailable while offline"),
            | Self::UnavailableEffect => formatter.write_str("Operation declares an unavailable effect"),
        }
    }
}
/// Enforce an operation's declared effects against an invocation context.
pub fn authorize(effects: &OperationEffects, context: &InvocationContext) -> Result<(), OperationPolicyError> {
    let requires_mutation = effects.destructive || effects.filesystem_write || effects.mutation;
    let unavailable = effects.credentials || effects.process || (effects.inference && !context.allow_inference);
    match () {
        | () if unavailable => Err(OperationPolicyError::UnavailableEffect),
        | () if requires_mutation && !context.allow_mutation => Err(OperationPolicyError::MutationApprovalRequired),
        | () if effects.network_read && context.offline => Err(OperationPolicyError::Offline),
        | () => Ok(()),
    }
}
