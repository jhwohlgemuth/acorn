use super::{Client, EndpointMode, InferenceError, Options};
use crate::{
    agent::{HydratedPrompt, InferenceBackend, InferenceFinishReason, InferencePermissionPolicy, InferenceRequest, InferenceUsage},
    io::api::Configuration,
    test::server::TestServer,
};
use acorn_schema::agent::{FrontMatter, PromptFileAsset};
use alloc::sync::Arc;
use axum::{extract::State as AxumState, http::StatusCode, routing::post, Json, Router};
use serde_json::{json, Value};
use std::sync::{Mutex, PoisonError};

#[derive(Clone, Default)]
struct ServerState {
    requests: Arc<Mutex<Vec<(EndpointMode, Value)>>>,
}
async fn chat(AxumState(state): AxumState<ServerState>, Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    let prompt = body
        .pointer("/messages/0/content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    state
        .requests
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((EndpointMode::ChatCompletions, body));
    response_for(EndpointMode::ChatCompletions, &prompt).await
}
fn client(server: &TestServer, mode: EndpointMode) -> Client {
    Client::new(Options::default().with_domain(&server.base_url), mode, "fake-model")
}
fn request(body: &str) -> InferenceRequest {
    InferenceRequest {
        agent: None,
        model: None,
        permission_policy: InferencePermissionPolicy::DenyMutation,
        prompt: HydratedPrompt {
            asset: PromptFileAsset::Summarize,
            body: body.to_string(),
            metadata: FrontMatter::init().name("test".to_string()).build(),
        },
        response_schema: None,
        timeout_ms: 10_000,
    }
}
async fn response_for(mode: EndpointMode, prompt: &str) -> (StatusCode, Json<Value>) {
    if prompt == "slow" {
        tokio::time::sleep(core::time::Duration::from_millis(100)).await;
    }
    let content = match prompt {
        | "invalid-structured" => r#"{"wrong":true}"#,
        | "structured" => r#"{"answer":42}"#,
        | _ => "same response",
    };
    match (prompt, mode) {
        | ("error", _) => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error":{"code":"rate_limit","message":"try later"}})),
        ),
        | ("malformed", _) => (StatusCode::OK, Json(json!({"unexpected":true}))),
        | ("refuse", EndpointMode::ChatCompletions) => (
            StatusCode::OK,
            Json(json!({
                "id":"chat-refusal",
                "model":"fake-model",
                "choices":[{"finish_reason":"content_filter","message":{"content":null,"refusal":"request refused"}}],
                "usage":{"prompt_tokens":4,"completion_tokens":2}
            })),
        ),
        | ("refuse", EndpointMode::Responses) => (
            StatusCode::OK,
            Json(json!({
                "id":"resp-refusal",
                "model":"fake-model",
                "status":"completed",
                "error":null,
                "incomplete_details":null,
                "output":[{"type":"message","content":[{"type":"refusal","refusal":"request refused"}]}],
                "usage":{"input_tokens":4,"output_tokens":2}
            })),
        ),
        | (_, EndpointMode::ChatCompletions) => (
            StatusCode::OK,
            Json(json!({
                "id":"chat-test",
                "model":"fake-model",
                "choices":[{"finish_reason":"stop","message":{"content":content,"refusal":null}}],
                "usage":{"prompt_tokens":4,"completion_tokens":2}
            })),
        ),
        | (_, EndpointMode::Responses) => (
            StatusCode::OK,
            Json(json!({
                "id":"resp-test",
                "model":"fake-model",
                "status":"completed",
                "error":null,
                "incomplete_details":null,
                "output":[{"type":"message","content":[{"type":"output_text","text":content}]}],
                "usage":{"input_tokens":4,"output_tokens":2}
            })),
        ),
    }
}
async fn responses(AxumState(state): AxumState<ServerState>, Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    let prompt = body.get("input").and_then(Value::as_str).unwrap_or_default().to_string();
    state
        .requests
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((EndpointMode::Responses, body));
    response_for(EndpointMode::Responses, &prompt).await
}
fn schema() -> Value {
    json!({
        "additionalProperties": false,
        "properties": {"answer": {"type": "integer"}},
        "required": ["answer"],
        "type": "object"
    })
}
async fn server() -> (TestServer, ServerState) {
    let state = ServerState::default();
    let router = Router::new()
        .route("/v1/chat/completions", post(chat))
        .route("/v1/responses", post(responses))
        .with_state(state.clone());
    (TestServer::start(router).await.expect("start OpenAI-compatible test server"), state)
}

#[tokio::test]
async fn test_capabilities_and_debug_keep_configuration_explicit_and_secret_free() {
    let (server, _) = server().await;
    let options = Options::with_token("private-token").build().with_domain(&server.base_url);
    let client = Client::new(options, EndpointMode::Responses, "fake-model").with_structured_output(true);
    let capabilities = client.capabilities();
    assert!(capabilities.structured_output);
    assert!(!capabilities.attachments);
    assert!(!capabilities.mutating_tools);
    assert!(!capabilities.streaming);
    assert!(!capabilities.tools);
    assert!(!format!("{client:?}").contains("private-token"));
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_endpoint_modes_forward_hydrated_prompt_body_verbatim() {
    let (server, state) = server().await;
    let body = "Instructions\n\n# Input Data (ZON)\n\n```zon\ntext:source\n```";
    client(&server, EndpointMode::Responses)
        .infer_checked(&request(body))
        .await
        .expect("responses inference");
    client(&server, EndpointMode::ChatCompletions)
        .infer_checked(&request(body))
        .await
        .expect("chat inference");
    {
        let requests = state.requests.lock().unwrap_or_else(PoisonError::into_inner);
        let responses_body = requests
            .iter()
            .find_map(|(mode, request)| (*mode == EndpointMode::Responses).then_some(request))
            .expect("responses request body");
        let chat_body = requests
            .iter()
            .find_map(|(mode, request)| (*mode == EndpointMode::ChatCompletions).then_some(request))
            .expect("chat request body");
        assert_eq!(responses_body.get("input").and_then(Value::as_str), Some(body));
        assert_eq!(chat_body.pointer("/messages/0/content").and_then(Value::as_str), Some(body));
    }
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_endpoint_modes_return_equivalent_normalized_results() {
    let (server, _) = server().await;
    let responses_result = client(&server, EndpointMode::Responses)
        .infer_checked(&request("plain"))
        .await
        .expect("responses inference");
    let chat_result = client(&server, EndpointMode::ChatCompletions)
        .infer_checked(&request("plain"))
        .await
        .expect("chat inference");
    assert_eq!(responses_result.text, chat_result.text);
    assert_eq!(responses_result.finish_reason, chat_result.finish_reason);
    assert_eq!(responses_result.structured, chat_result.structured);
    assert_eq!(responses_result.usage, chat_result.usage);
    assert_eq!(responses_result.provenance.backend, "openai");
    assert_eq!(chat_result.provenance.backend, "openai");
    assert_eq!(responses_result.provenance.model, chat_result.provenance.model);
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_endpoint_modes_send_distinct_structured_output_shapes() {
    let (server, state) = server().await;
    let structured = InferenceRequest {
        response_schema: Some(schema()),
        ..request("structured")
    };
    let responses_result = client(&server, EndpointMode::Responses)
        .with_structured_output(true)
        .infer_checked(&structured)
        .await
        .expect("responses structured inference");
    let chat_result = client(&server, EndpointMode::ChatCompletions)
        .with_structured_output(true)
        .infer_checked(&structured)
        .await
        .expect("chat structured inference");
    assert_eq!(responses_result.structured, Some(json!({"answer":42})));
    assert_eq!(chat_result.structured, responses_result.structured);
    {
        let requests = state.requests.lock().unwrap_or_else(PoisonError::into_inner);
        let responses_body = requests
            .iter()
            .find_map(|(mode, body)| (*mode == EndpointMode::Responses).then_some(body))
            .expect("responses request body");
        let chat_body = requests
            .iter()
            .find_map(|(mode, body)| (*mode == EndpointMode::ChatCompletions).then_some(body))
            .expect("chat request body");
        assert_eq!(responses_body.pointer("/text/format/type").and_then(Value::as_str), Some("json_schema"));
        assert_eq!(responses_body.pointer("/text/format/schema"), Some(&schema()));
        assert_eq!(chat_body.pointer("/response_format/type").and_then(Value::as_str), Some("json_schema"));
        assert_eq!(chat_body.pointer("/response_format/json_schema/schema"), Some(&schema()));
        assert_eq!(responses_body.get("store"), Some(&Value::Bool(false)));
        assert_eq!(chat_body.get("store"), Some(&Value::Bool(false)));
    }
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_invalid_structured_output_and_schema_are_rejected_locally() {
    let (server, state) = server().await;
    let invalid_output = InferenceRequest {
        response_schema: Some(schema()),
        ..request("invalid-structured")
    };
    let output_error = client(&server, EndpointMode::Responses)
        .infer_checked(&invalid_output)
        .await
        .expect_err("invalid structured output should fail");
    assert!(matches!(output_error, InferenceError::SchemaViolation(_)));
    let invalid_schema = InferenceRequest {
        response_schema: Some(json!({"type":"invented"})),
        ..request("structured")
    };
    let schema_error = client(&server, EndpointMode::Responses)
        .infer_checked(&invalid_schema)
        .await
        .expect_err("invalid schema should fail");
    assert!(matches!(schema_error, InferenceError::InvalidSchema(_)));
    assert_eq!(state.requests.lock().unwrap_or_else(PoisonError::into_inner).len(), 1);
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_malformed_success_response_is_rejected() {
    let (server, _) = server().await;
    let result = client(&server, EndpointMode::ChatCompletions).infer_checked(&request("malformed")).await;
    assert!(matches!(result, Err(InferenceError::InvalidResponse(_))));
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_offline_policy_allows_loopback_and_rejects_remote_endpoints() {
    let (server, _) = server().await;
    let local = client(&server, EndpointMode::Responses).with_offline(true);
    assert!(local.infer_checked(&request("plain")).await.is_ok());
    let remote = Client::new(Options::default(), EndpointMode::Responses, "fake-model").with_offline(true);
    assert!(matches!(remote.infer_checked(&request("plain")).await, Err(InferenceError::Offline(_))));
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_provider_errors_and_refusals_are_typed() {
    let (server, _) = server().await;
    let client = client(&server, EndpointMode::Responses);
    let provider = client.infer_checked(&request("error")).await.expect_err("provider error");
    assert!(matches!(
        provider,
        InferenceError::Provider {
            code: Some(code),
            status: 429,
            ..
        } if code == "rate_limit"
    ));
    let refusal = client.infer_checked(&request("refuse")).await.expect("typed refusal");
    assert_eq!(refusal.finish_reason, InferenceFinishReason::Refusal);
    assert_eq!(refusal.text, "request refused");
    assert_eq!(
        refusal.usage,
        Some(InferenceUsage {
            input_tokens: Some(4),
            output_tokens: Some(2)
        })
    );
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_remote_authentication_and_transport_security_are_checked_before_io() {
    let missing = Client::new(Options::default(), EndpointMode::Responses, "fake-model");
    assert!(matches!(
        missing.infer_checked(&request("plain")).await,
        Err(InferenceError::MissingAuthentication)
    ));
    let insecure = Client::new(
        Options::with_token("private-token").build().with_domain("http://example.com"),
        EndpointMode::Responses,
        "fake-model",
    );
    assert!(matches!(
        insecure.infer_checked(&request("plain")).await,
        Err(InferenceError::InsecureEndpoint(_))
    ));
}
#[tokio::test]
async fn test_structured_output_fallback_parses_without_advertised_capability() {
    let (server, state) = server().await;
    let structured = InferenceRequest {
        response_schema: Some(schema()),
        ..request("structured")
    };
    let result = client(&server, EndpointMode::Responses)
        .infer_checked(&structured)
        .await
        .expect("structured fallback");
    assert_eq!(result.structured, Some(json!({"answer":42})));
    {
        let requests = state.requests.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(requests.first().and_then(|(_, body)| body.get("text")), None);
    }
    server.stop().await.expect("stop test server");
}
#[tokio::test]
async fn test_timeout_is_enforced() {
    let (server, _) = server().await;
    let timed = InferenceRequest {
        timeout_ms: 10,
        ..request("slow")
    };
    let result = client(&server, EndpointMode::Responses).infer_checked(&timed).await;
    assert!(matches!(result, Err(InferenceError::Timeout(_))));
    server.stop().await.expect("stop test server");
}
