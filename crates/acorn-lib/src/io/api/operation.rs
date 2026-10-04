//! Transport-neutral operation provenance and effects policy.
use acorn_core::prelude::String;
#[cfg(any(test, feature = "agentic", feature = "mcp-client"))]
use core::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Source transport for an operation invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationOrigin {
    /// Direct in-process invocation.
    Direct,
    /// Protobuf/gRPC invocation.
    Grpc,
    /// JSON-RPC HTTP invocation.
    JsonRpc,
    /// Normalized webhook invocation.
    Webhook,
}
/// One external effect declared by an operation or allowlisted tool.
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
#[cfg(any(test, feature = "agentic", feature = "mcp-client"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperationPolicyError {
    MutationApprovalRequired,
    Offline,
    UnavailableEffect,
}
/// Policy and provenance supplied to one registered operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationContext {
    /// Whether inference handlers may run.
    pub allow_inference: bool,
    /// Whether mutating handlers may run.
    pub allow_mutation: bool,
    /// Optional transport correlation identifier.
    pub correlation_id: Option<String>,
    /// Maximum serialized result size.
    pub max_output_bytes: usize,
    /// Whether external-system interaction is forbidden.
    pub offline: bool,
    /// Transport that initiated the invocation.
    pub origin: InvocationOrigin,
    /// Authenticated principal.
    pub principal: Principal,
}
/// Effect metadata enforced before an operation handler runs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationEffects {
    /// Whether the operation can access credentials.
    pub credentials: bool,
    /// Whether the operation can destroy or irreversibly replace data.
    pub destructive: bool,
    /// Whether the operation can read from the host filesystem.
    pub filesystem_read: bool,
    /// Whether the operation can write to the host filesystem.
    pub filesystem_write: bool,
    /// Whether the operation can invoke inference services.
    pub inference: bool,
    /// Whether the operation may change state.
    pub mutation: bool,
    /// Whether the operation can read from remote network services.
    pub network_read: bool,
    /// Whether the operation can start or communicate with processes.
    pub process: bool,
}
/// Authenticated caller identity supplied by a transport adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
            max_output_bytes: 1024 * 1024,
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
#[cfg(any(test, feature = "agentic", feature = "mcp-client"))]
impl fmt::Display for OperationPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::MutationApprovalRequired => formatter.write_str("Operation requires mutation approval"),
            | Self::Offline => formatter.write_str("Operation is unavailable while offline"),
            | Self::UnavailableEffect => formatter.write_str("Operation declares an unavailable effect"),
        }
    }
}
#[cfg(any(test, feature = "agentic", feature = "mcp-client"))]
pub(crate) fn authorize(effects: &OperationEffects, context: &InvocationContext) -> Result<(), OperationPolicyError> {
    let requires_mutation = effects.destructive || effects.filesystem_write || effects.mutation;
    let unavailable = effects.credentials || effects.process || (effects.inference && !context.allow_inference);
    let mutation_denied = requires_mutation && !context.allow_mutation;
    let offline = effects.network_read && context.offline;
    match (unavailable, mutation_denied, offline) {
        | (true, _, _) => Err(OperationPolicyError::UnavailableEffect),
        | (_, true, _) => Err(OperationPolicyError::MutationApprovalRequired),
        | (_, _, true) => Err(OperationPolicyError::Offline),
        | _ => Ok(()),
    }
}
