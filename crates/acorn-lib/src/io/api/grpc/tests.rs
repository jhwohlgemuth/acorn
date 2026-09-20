#![allow(clippy::arithmetic_side_effects, clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]
use super::client::{GrpcClient, GrpcClientConfig};
use super::proto::acorn_rpc_server::AcornRpc;
use super::proto::{invoke_response, request_id, BatchRequest, InvokeRequest, NotifyRequest, RequestId};
use super::server::{validate_transport, GrpcServer, GrpcService};
use super::{json_value, protobuf_value};
use crate::io::api::json_rpc::{InvocationContext, MethodName, OperationDefinition, OperationEffects, OperationRegistry, RpcError, INVALID_PARAMS};
use crate::io::api::Secret;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::time::Duration;
use futures::channel::oneshot;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::TcpListener;
use tonic::{Code, Request};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AddInput {
    left: i64,
    right: i64,
}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct AddOutput {
    total: i64,
}
struct CancellationGuard(Arc<AtomicBool>);

impl Drop for CancellationGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn add_request(identifier: i64, left: i64, right: i64) -> InvokeRequest {
    InvokeRequest {
        id: Some(RequestId {
            value: Some(request_id::Value::Number(identifier)),
        }),
        method: MethodName::from(["add"]).to_string(),
        params: Some(protobuf_value(json!({"left": left, "right": right})).unwrap()),
    }
}
fn authenticated<T>(message: T, token: &str) -> Request<T> {
    let mut request = Request::new(message);
    request.metadata_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
    request
}
fn registry() -> OperationRegistry {
    let definition = OperationDefinition::new::<AddInput, AddOutput>(MethodName::from(["add"]), OperationEffects::default()).unwrap();
    OperationRegistry::default()
        .register(definition, |input: AddInput, _| async move {
            Ok(AddOutput {
                total: input.left + input.right,
            })
        })
        .unwrap()
}
fn response_value(response: super::proto::InvokeResponse) -> Result<Value, RpcError> {
    match response.outcome {
        | Some(invoke_response::Outcome::Error(error)) => Err(error.into()),
        | Some(invoke_response::Outcome::Result(value)) => json_value(value),
        | None => Err(RpcError::new(crate::io::api::json_rpc::INTERNAL_ERROR, "Missing gRPC outcome")),
    }
}
fn slow_registry(cancelled: Arc<AtomicBool>) -> OperationRegistry {
    let definition = OperationDefinition::new::<AddInput, AddOutput>(MethodName::from(["slow"]), OperationEffects::default()).unwrap();
    registry()
        .register(definition, move |input: AddInput, _| {
            let cancelled = Arc::clone(&cancelled);
            async move {
                let _guard = CancellationGuard(cancelled);
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(AddOutput {
                    total: input.left + input.right,
                })
            }
        })
        .unwrap()
}

#[tokio::test]
async fn authentication_and_batch_limits_use_transport_statuses() {
    let service = GrpcService::new(registry(), Secret::from("secret".to_string()), InvocationContext::default())
        .unwrap()
        .with_max_batch_length(1);
    let unauthenticated = service.invoke(Request::new(add_request(1, 1, 2))).await.unwrap_err();
    assert_eq!(unauthenticated.code(), Code::Unauthenticated);
    let oversized = service
        .batch(authenticated(
            BatchRequest {
                requests: vec![add_request(1, 1, 2), add_request(2, 3, 4)],
            },
            "secret",
        ))
        .await
        .unwrap_err();
    assert_eq!(oversized.code(), Code::ResourceExhausted);
}
#[tokio::test]
async fn batch_preserves_request_order() {
    let service = GrpcService::new(registry(), Secret::from("secret".to_string()), InvocationContext::default()).unwrap();
    let response = service
        .batch(authenticated(
            BatchRequest {
                requests: vec![add_request(1, 1, 2), add_request(2, 10, 20)],
            },
            "secret",
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(response.responses.len(), 2);
    assert_eq!(response_value(response.responses[0].clone()).unwrap(), json!({"total": 3}));
    assert_eq!(response_value(response.responses[1].clone()).unwrap(), json!({"total": 30}));
}
#[test]
fn client_and_server_transport_policies_are_enforced() {
    let token = Secret::from("secret".to_string());
    assert!(GrpcClientConfig::new("http://127.0.0.1:50051", token.clone())
        .with_offline(true)
        .endpoint()
        .is_ok());
    assert!(GrpcClientConfig::new("http://example.com:50051", token.clone()).endpoint().is_err());
    assert!(GrpcClientConfig::new("https://example.com:50051", token)
        .with_offline(true)
        .endpoint()
        .is_err());
    assert!(validate_transport("127.0.0.1:50051".parse().unwrap(), None).is_ok());
    assert!(validate_transport("0.0.0.0:50051".parse().unwrap(), None).is_err());
}
#[tokio::test]
async fn grpc_and_json_rpc_produce_equivalent_results_and_errors() {
    let registry = registry();
    let service = GrpcService::new(registry.clone(), Secret::from("secret".to_string()), InvocationContext::default()).unwrap();
    let direct = registry
        .invoke(&MethodName::from(["add"]), json!({"left": 2, "right": 5}), InvocationContext::default())
        .await
        .unwrap();
    let json_request = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": MethodName::from(["add"]),
        "params": {"left": 2, "right": 5}
    }))
    .unwrap();
    let dispatched = registry.dispatch(&json_request, InvocationContext::default()).await;
    let json_response: Value = serde_json::from_slice(dispatched.body.as_deref().unwrap()).unwrap();
    let grpc_response = service.invoke(authenticated(add_request(1, 2, 5), "secret")).await.unwrap().into_inner();
    assert_eq!(response_value(grpc_response).unwrap(), direct);
    assert_eq!(json_response["result"], direct);
    let invalid = InvokeRequest {
        params: Some(protobuf_value(json!({"left": "bad", "right": 5})).unwrap()),
        ..add_request(2, 0, 0)
    };
    let error = response_value(service.invoke(authenticated(invalid, "secret")).await.unwrap().into_inner()).unwrap_err();
    assert_eq!(error.code, INVALID_PARAMS);
}
#[tokio::test]
async fn network_client_server_enforce_auth_deadlines_and_cancellation() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let cancelled = Arc::new(AtomicBool::new(false));
    let server = GrpcServer::new(
        slow_registry(Arc::clone(&cancelled)),
        Secret::from("secret".to_string()),
        InvocationContext::default(),
    )
    .unwrap()
    .with_request_timeout(Duration::from_millis(50));
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server_task = tokio::spawn(server.serve_with_shutdown(address, async move {
        let _ = shutdown_rx.await;
    }));
    tokio::time::sleep(Duration::from_millis(30)).await;
    let endpoint = format!("http://{address}");
    let mut client =
        GrpcClient::connect(GrpcClientConfig::new(endpoint.clone(), Secret::from("secret".to_string())).with_request_timeout(Duration::from_secs(1)))
            .await
            .unwrap();
    let result = client.invoke(add_request(1, 4, 6)).await.unwrap();
    assert_eq!(response_value(result).unwrap(), json!({"total": 10}));
    let mut unauthorized =
        GrpcClient::connect(GrpcClientConfig::new(endpoint, Secret::from("wrong".to_string())).with_request_timeout(Duration::from_secs(1)))
            .await
            .unwrap();
    let status = unauthorized.invoke(add_request(2, 1, 1)).await.unwrap_err();
    assert_eq!(status.code(), Code::Unauthenticated);
    let slow = InvokeRequest {
        method: MethodName::from(["slow"]).to_string(),
        ..add_request(3, 1, 2)
    };
    let status = client.invoke(slow).await.unwrap_err();
    assert_eq!(status.code(), Code::Cancelled);
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(cancelled.load(Ordering::SeqCst));
    shutdown_tx.send(()).unwrap();
    server_task.await.unwrap().unwrap();
}
#[tokio::test]
async fn notification_invokes_without_an_operation_response() {
    let invocations = Arc::new(AtomicUsize::new(0));
    let definition = OperationDefinition::new::<AddInput, AddOutput>(MethodName::from(["notify"]), OperationEffects::default()).unwrap();
    let registry = OperationRegistry::default()
        .register(definition, {
            let invocations = Arc::clone(&invocations);
            move |input: AddInput, _| {
                let invocations = Arc::clone(&invocations);
                async move {
                    invocations.fetch_add(1, Ordering::SeqCst);
                    Ok(AddOutput {
                        total: input.left + input.right,
                    })
                }
            }
        })
        .unwrap();
    let service = GrpcService::new(registry, Secret::from("secret".to_string()), InvocationContext::default()).unwrap();
    let notification = NotifyRequest {
        method: MethodName::from(["notify"]).to_string(),
        params: Some(protobuf_value(json!({"left": 1, "right": 2})).unwrap()),
    };
    service.notify(authenticated(notification, "secret")).await.unwrap();
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
}
#[test]
fn protobuf_values_round_trip_json_shapes() {
    let value = json!({
        "array": [null, true, 1.5, "value"],
        "object": {"nested": false}
    });
    assert_eq!(json_value(protobuf_value(value.clone()).unwrap()).unwrap(), value);
    assert!(protobuf_value(json!(9_007_199_254_740_992_u64)).is_err());
}
#[tokio::test]
async fn transport_failures_and_empty_credentials_are_reported_by_client() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let endpoint = format!("http://{address}");
    let empty = GrpcClient::connect(GrpcClientConfig::new(endpoint.clone(), Secret::from(String::new()))).await;
    assert!(empty.is_err());
    let unavailable =
        GrpcClient::connect(GrpcClientConfig::new(endpoint, Secret::from("secret".to_string())).with_request_timeout(Duration::from_millis(50)))
            .await;
    assert!(unavailable.is_err());
}
