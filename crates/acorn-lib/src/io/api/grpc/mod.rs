//! Versioned Protobuf/gRPC transport for the shared operation registry.
use crate::io::api::json_rpc::{RequestId as OperationRequestId, RpcError as OperationError, INTERNAL_ERROR, INVALID_PARAMS};
use alloc::collections::BTreeMap;
use prost_types::{value::Kind, ListValue, Struct, Value as ProtobufValue};
use serde_json::{Map, Number, Value};

pub mod client;
pub mod server;
/// Generated `acorn.rpc.v1` protocol messages and services.
#[allow(missing_docs, clippy::std_instead_of_core)]
pub mod proto {
    tonic::include_proto!("acorn.rpc.v1");
}

/// Default maximum number of requests accepted in one gRPC batch.
pub const DEFAULT_MAX_BATCH_LENGTH: usize = 128;
/// Default maximum encoded or decoded gRPC message size.
pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 1024 * 1024;
/// Default request deadline in seconds.
pub const DEFAULT_REQUEST_TIMEOUT_SECONDS: u64 = 30;
const MAX_SAFE_PROTOBUF_INTEGER: u64 = 9_007_199_254_740_991;
impl From<proto::RpcError> for OperationError {
    fn from(error: proto::RpcError) -> Self {
        Self {
            code: error.code,
            data: error.data.and_then(|value| json_value(value).ok()),
            message: error.message,
        }
    }
}
impl TryFrom<proto::RequestId> for OperationRequestId {
    type Error = OperationError;

    fn try_from(identifier: proto::RequestId) -> Result<Self, Self::Error> {
        match identifier.value {
            | Some(proto::request_id::Value::Number(value)) => Ok(Self::Number(value)),
            | Some(proto::request_id::Value::Text(value)) => Ok(Self::String(value)),
            | None => Err(OperationError::new(INVALID_PARAMS, "Request identifier is missing")),
        }
    }
}
impl From<OperationRequestId> for proto::RequestId {
    fn from(identifier: OperationRequestId) -> Self {
        let value = match identifier {
            | OperationRequestId::Number(value) => proto::request_id::Value::Number(value),
            | OperationRequestId::String(value) => proto::request_id::Value::Text(value),
        };
        Self { value: Some(value) }
    }
}
impl From<OperationError> for proto::RpcError {
    fn from(error: OperationError) -> Self {
        Self {
            code: error.code,
            data: error.data.and_then(|value| protobuf_value(value).ok()),
            message: error.message,
        }
    }
}
fn empty_params() -> Value {
    Value::Object(Map::new())
}
fn json_number(value: f64) -> Option<Number> {
    let integer = (value.fract() == 0.0).then(|| value.to_string());
    integer
        .as_deref()
        .and_then(|value| value.parse::<i64>().ok())
        .map(Number::from)
        .or_else(|| integer.as_deref().and_then(|value| value.parse::<u64>().ok()).map(Number::from))
        .or_else(|| Number::from_f64(value))
}
/// Convert a Protobuf `Value` into its JSON representation.
pub fn json_value(value: ProtobufValue) -> Result<Value, OperationError> {
    match value.kind {
        | Some(Kind::BoolValue(value)) => Ok(Value::Bool(value)),
        | Some(Kind::ListValue(value)) => value.values.into_iter().map(json_value).collect::<Result<Vec<_>, _>>().map(Value::Array),
        | Some(Kind::NullValue(_)) | None => Ok(Value::Null),
        | Some(Kind::NumberValue(value)) => json_number(value)
            .map(Value::Number)
            .ok_or_else(|| OperationError::new(INVALID_PARAMS, "Protobuf value contains a non-finite number")),
        | Some(Kind::StringValue(value)) => Ok(Value::String(value)),
        | Some(Kind::StructValue(value)) => value
            .fields
            .into_iter()
            .map(|(key, value)| json_value(value).map(|value| (key, value)))
            .collect::<Result<Map<_, _>, _>>()
            .map(Value::Object),
    }
}
fn protobuf_number(value: &Number) -> Result<f64, OperationError> {
    let safe_integer = value
        .as_i64()
        .map(|value| value.unsigned_abs() <= MAX_SAFE_PROTOBUF_INTEGER)
        .or_else(|| value.as_u64().map(|value| value <= MAX_SAFE_PROTOBUF_INTEGER))
        .unwrap_or(true);
    match (safe_integer, value.as_f64()) {
        | (false, _) => Err(OperationError::new(
            INTERNAL_ERROR,
            "JSON integer exceeds Protobuf's lossless numeric range",
        )),
        | (_, Some(value)) => Ok(value),
        | (_, None) => Err(OperationError::new(INTERNAL_ERROR, "JSON number cannot be represented by Protobuf")),
    }
}
/// Convert a JSON value into its Protobuf representation.
pub fn protobuf_value(value: Value) -> Result<ProtobufValue, OperationError> {
    let kind = match value {
        | Value::Array(values) => values
            .into_iter()
            .map(protobuf_value)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| Kind::ListValue(ListValue { values })),
        | Value::Bool(value) => Ok(Kind::BoolValue(value)),
        | Value::Null => Ok(Kind::NullValue(0)),
        | Value::Number(value) => protobuf_number(&value).map(Kind::NumberValue),
        | Value::Object(values) => values
            .into_iter()
            .map(|(key, value)| protobuf_value(value).map(|value| (key, value)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(|fields| Kind::StructValue(Struct { fields })),
        | Value::String(value) => Ok(Kind::StringValue(value)),
    };
    kind.map(|kind| ProtobufValue { kind: Some(kind) })
}

#[cfg(test)]
mod tests;
