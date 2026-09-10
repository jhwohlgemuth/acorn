#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
use super::*;
use crate::io::api;
use axum::body::{to_bytes, Body};
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Request, StatusCode};
use schemars::JsonSchema;
use tower::ServiceExt;

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
fn service() -> http::RpcHttpService {
    http::RpcHttpService::new(
        OperationRegistry::acorn().unwrap(),
        api::Secret::from("secret".to_string()),
        InvocationContext::default(),
    )
}
#[tokio::test]
async fn batch_preserves_order_and_omits_notifications() {
    let method = MethodName::from(["add"]);
    let body = serde_json::to_vec(&serde_json::json!([
        {"jsonrpc": JSON_RPC_VERSION, "id": "first", "method": method, "params": {"left": 1, "right": 2}},
        {"jsonrpc": JSON_RPC_VERSION, "method": method, "params": {"left": 5, "right": 8}},
        {"jsonrpc": JSON_RPC_VERSION, "id": 3, "method": method, "params": {"left": 10, "right": 20}}
    ]))
    .unwrap();
    let response = registry().dispatch(&body, InvocationContext::default()).await;
    let value: Value = serde_json::from_slice(response.body.as_deref().unwrap()).unwrap();
    assert_eq!(value[0]["id"], "first");
    assert_eq!(value[0]["result"]["total"], 3);
    assert_eq!(value[1]["id"], 3);
    assert_eq!(value[1]["result"]["total"], 30);
}
#[tokio::test]
async fn endpoint_authenticates_and_dispatches() {
    let body = serde_json::to_vec(&serde_json::json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": 1,
        "method": MethodName::from(["version"])
    }))
    .unwrap();
    let request = Request::post("/rpc")
        .header(AUTHORIZATION, "Bearer secret")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();
    let response = service().router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["id"], 1);
    assert_eq!(value["result"]["version"], env!("CARGO_PKG_VERSION"));
}
#[tokio::test]
async fn endpoint_rejects_missing_authentication_and_large_bodies() {
    let unauthenticated = Request::post("/rpc")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let response = service().router().oneshot(unauthenticated).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let oversized = Request::post("/rpc")
        .header(AUTHORIZATION, "Bearer secret")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from("12345"))
        .unwrap();
    let response = service().with_max_body_bytes(4).router().oneshot(oversized).await.unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let unsupported = Request::post("/rpc")
        .header(AUTHORIZATION, "Bearer secret")
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from("{}"))
        .unwrap();
    let response = service().router().oneshot(unsupported).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
#[tokio::test]
async fn invalid_documents_return_standard_errors() {
    let add = MethodName::from(["add"]);
    let missing = MethodName::from(["missing"]);
    let version = MethodName::from(["version"]);
    let cases = [
        (
            format!(r#"{{"jsonrpc":"{JSON_RPC_VERSION}","id":1,"method":}}"#).into_bytes(),
            PARSE_ERROR,
        ),
        (
            format!(r#"{{"jsonrpc":"{JSON_RPC_VERSION}","id":1,"method":"{add}","method":"{version}"}}"#).into_bytes(),
            PARSE_ERROR,
        ),
        (
            format!(r#"{{"jsonrpc":"{JSON_RPC_VERSION}","id":1,"method":"{add}"}} trailing"#).into_bytes(),
            PARSE_ERROR,
        ),
        (b"[]".to_vec(), INVALID_REQUEST),
        (format!(r#"{{"jsonrpc":"1.0","id":1,"method":"{add}"}}"#).into_bytes(), INVALID_REQUEST),
        (
            format!(r#"{{"jsonrpc":"{JSON_RPC_VERSION}","id":1.5,"method":"{add}"}}"#).into_bytes(),
            INVALID_REQUEST,
        ),
        (
            format!(r#"{{"jsonrpc":"{JSON_RPC_VERSION}","id":1,"method":"{missing}"}}"#).into_bytes(),
            METHOD_NOT_FOUND,
        ),
        (
            format!(r#"{{"jsonrpc":"{JSON_RPC_VERSION}","id":1,"method":"{add}","params":{{"left":"bad","right":2}}}}"#).into_bytes(),
            INVALID_PARAMS,
        ),
    ];
    for (body, code) in cases {
        let response = registry().dispatch(&body, InvocationContext::default()).await;
        let value: Value = serde_json::from_slice(response.body.as_deref().unwrap()).unwrap();
        assert_eq!(value["error"]["code"], code);
    }
}
#[test]
fn method_names_are_validated() {
    assert!(MethodName::from([""]).validate().is_err());
    assert!(MethodName::try_from(format!("{APPLICATION}.valid")).is_ok());
    assert!(MethodName::try_from("rpc.reserved").is_err());
    assert!(MethodName::try_from("other.invalid").is_err());
    assert_eq!(MethodName::from(["version"]).to_string(), format!("{APPLICATION}.version"));
}
#[tokio::test]
async fn mutation_and_offline_policies_are_enforced() {
    let effects = OperationEffects {
        destructive: false,
        open_world: true,
        read_only: false,
    };
    let method = MethodName::from(["mutate"]);
    let definition = OperationDefinition::new::<AddInput, AddOutput>(method.clone(), effects).unwrap();
    let registry = OperationRegistry::default()
        .register(definition, |input: AddInput, _| async move {
            Ok(AddOutput {
                total: input.left + input.right,
            })
        })
        .unwrap();
    let params = serde_json::json!({"left": 1, "right": 2});
    let mutation = registry.invoke(&method, params.clone(), InvocationContext::default()).await.unwrap_err();
    assert_eq!(mutation.code, POLICY_DENIED);
    let offline = registry
        .invoke(
            &method,
            params,
            InvocationContext {
                allow_mutation: true,
                offline: true,
                ..InvocationContext::default()
            },
        )
        .await
        .unwrap_err();
    assert_eq!(offline.code, POLICY_DENIED);
}
#[tokio::test]
async fn notifications_never_return_a_response() {
    let body = serde_json::to_vec(&serde_json::json!({
        "jsonrpc": JSON_RPC_VERSION,
        "method": MethodName::from(["add"]),
        "params": {"left": 1, "right": 2}
    }))
    .unwrap();
    let response = registry().dispatch(&body, InvocationContext::default()).await;
    assert_eq!(response.status_code, 204);
    assert!(response.body.is_none());
}
#[tokio::test]
async fn repository_workflow_is_equivalent_directly_and_over_json_rpc() {
    let registry = OperationRegistry::acorn().unwrap();
    let params = serde_json::json!({
        "workflow": "repository-quality",
        "input": {
            "actions": ["validate", "format"],
            "files": [{"path": "metadata.json", "content": "{\"z\":1,\"a\":2}", "format": "json"}],
            "project": "30",
            "revision": "abc123"
        }
    });
    let direct = registry
        .invoke(&MethodName::from(["workflows", "process"]), params.clone(), InvocationContext::default())
        .await
        .unwrap();
    let request = serde_json::to_vec(&serde_json::json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": "workflow",
        "method": MethodName::from(["workflows", "process"]),
        "params": params
    }))
    .unwrap();
    let dispatched = registry.dispatch(&request, InvocationContext::default()).await;
    let response: Value = serde_json::from_slice(dispatched.body.as_deref().unwrap()).unwrap();
    assert_eq!(response["result"], direct);
}
