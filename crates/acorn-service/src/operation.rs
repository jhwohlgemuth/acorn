//! Transport-neutral operation provenance and effect policy.
use crate::{ServiceError, ServiceErrorKind, ServiceResult};
use acorn_core::prelude::String;
use serde::{Deserialize, Serialize};

/// Source transport for an invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
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
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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
/// Policy and provenance supplied to one operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InvocationContext {
    /// Whether inference handlers may run.
    pub allow_inference: bool,
    /// Whether mutating handlers may run.
    pub allow_mutation: bool,
    /// Optional transport correlation identifier.
    pub correlation_id: Option<String>,
    /// Maximum serialized request size.
    pub max_input_bytes: usize,
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
            max_input_bytes: 1024 * 1024,
            max_output_bytes: 1024 * 1024,
            offline: false,
            origin: InvocationOrigin::Direct,
            principal: Principal { identifier: "local".into() },
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
pub(crate) fn authorize(effects: &OperationEffects, context: &InvocationContext) -> ServiceResult<()> {
    let mutating = effects.destructive || effects.filesystem_write || effects.mutation;
    let unavailable = effects.credentials || effects.process || (effects.inference && !context.allow_inference);
    match () {
        | () if unavailable => Err(ServiceError::new(
            ServiceErrorKind::PolicyDenied,
            "Operation declares an unavailable effect",
        )),
        | () if mutating && !context.allow_mutation => Err(ServiceError::new(ServiceErrorKind::PolicyDenied, "Operation requires mutation approval")),
        | () if effects.network_read && context.offline => Err(ServiceError::new(
            ServiceErrorKind::PolicyDenied,
            "Operation is unavailable while offline",
        )),
        | () => Ok(()),
    }
}
