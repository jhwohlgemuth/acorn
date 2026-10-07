#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]

use super::*;
use crate::agent::{
    HydratedPrompt, InMemoryBackend, InferenceFinishReason, InferencePermissionPolicy, InferenceProvenance, InferenceRequest, InferenceResult,
};
use crate::analyzer::fix::{CheckReference, FixOptions, FixReport, FixSuggestionRequest, IntoFixes, FIX_ENVELOPE_VERSION};
use acorn_schema::agent::{FrontMatter, PromptFileAsset};
use schemars::JsonSchema;
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
fn fix_reference() -> CheckReference {
    CheckReference {
        category: "schema".to_string(),
        context: Some("Old title".to_string()),
        fingerprint: "a".repeat(64),
        id: "check-0001".to_string(),
        locator: Some("title".to_string()),
        message: "Title is invalid".to_string(),
        severity: "error".to_string(),
        standard: "research-activity-data".to_string(),
        uri: Some("activity.json".to_string()),
    }
}
fn fix_result(reference: &CheckReference) -> InferenceResult {
    InferenceResult {
        structured: Some(serde_json::json!({
            "fixes": [{
                "actions": [{
                    "instructions": "Replace the title with a valid value",
                    "kind": "manual"
                }],
                "automation": "manual",
                "confidence": 0.9,
                "effects": {
                    "elevation": false,
                    "filesystem_write": false,
                    "network": false,
                    "package_manager": false,
                    "process": false,
                    "restart": false,
                    "reversible": true
                },
                "finding_id": reference.id,
                "id": "fix-0001",
                "rationale": "The title requires human review.",
                "risk": "low",
                "source_fingerprint": reference.fingerprint,
                "status": "proposed",
                "summary": "Review the title",
                "target": "activity.json"
            }],
            "version": FIX_ENVELOPE_VERSION
        })),
        text: String::new(),
        ..inference_result()
    }
}
fn inference_request() -> InferenceRequest {
    InferenceRequest {
        agent: None,
        model: Some("test-model".to_string()),
        permission_policy: InferencePermissionPolicy::DenyMutation,
        prompt: HydratedPrompt {
            asset: PromptFileAsset::Summarize,
            body: "Summarize this source".to_string(),
            metadata: FrontMatter::init().name("summarize".to_string()).build(),
        },
        response_schema: None,
        timeout: 1_000.into(),
    }
}
fn inference_result() -> InferenceResult {
    InferenceResult {
        finish_reason: InferenceFinishReason::Stop,
        provenance: InferenceProvenance {
            agent: None,
            backend: "memory".to_string(),
            model: Some("test-model".to_string()),
            request_id: Some("registered-request".to_string()),
        },
        structured: None,
        text: "registered result".to_string(),
        usage: None,
    }
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
#[tokio::test]
async fn test_batch_preserves_order_and_omits_notifications() {
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
async fn test_invalid_documents_return_standard_errors() {
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
fn test_method_names_are_validated() {
    assert!(MethodName::from([""]).validate().is_err());
    assert!(MethodName::try_from(format!("{APPLICATION}.valid")).is_ok());
    assert!(MethodName::try_from("rpc.reserved").is_err());
    assert!(MethodName::try_from("other.invalid").is_err());
    assert_eq!(MethodName::from(["version"]).to_string(), format!("{APPLICATION}.version"));
}
#[tokio::test]
async fn test_mutation_and_offline_policies_are_enforced() {
    let effects = OperationEffects {
        mutation: true,
        network_read: true,
        ..OperationEffects::default()
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
async fn test_notifications_never_return_a_response() {
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
async fn test_registered_inference_matches_direct_service_and_requires_authorization() {
    let expected = inference_result();
    let backend = InMemoryBackend::new(expected.clone());
    let direct = crate::agent::run_inference(&backend, &inference_request()).await.unwrap();
    let registry = OperationRegistry::default().register_inference(backend).unwrap();
    let method = MethodName::from(["inference", "run"]);
    let params = serde_json::to_value(inference_request()).unwrap();
    let denied = registry.invoke(&method, params.clone(), InvocationContext::default()).await.unwrap_err();
    assert_eq!(denied.code, POLICY_DENIED);
    let registered = registry
        .invoke(
            &method,
            params,
            InvocationContext {
                allow_inference: true,
                ..InvocationContext::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(registered, serde_json::to_value(direct).unwrap());
}
#[tokio::test]
async fn test_registered_readiness_and_fix_suggestions_require_inference_authorization() {
    let reference = fix_reference();
    let registry = OperationRegistry::default()
        .register_inference(InMemoryBackend::new(fix_result(&reference)))
        .unwrap();
    let context = InvocationContext {
        allow_inference: true,
        ..InvocationContext::default()
    };
    let readiness = registry
        .invoke(
            &MethodName::from(["inference", "readiness"]),
            serde_json::to_value(inference_request()).unwrap(),
            context.clone(),
        )
        .await
        .unwrap();
    assert_eq!(readiness["ready"], true);
    let request = FixSuggestionRequest {
        options: FixOptions::default(),
        references: vec![reference],
    };
    let method = MethodName::from(["fixes", "suggest"]);
    let params = serde_json::to_value(request).unwrap();
    let denied = registry.invoke(&method, params.clone(), InvocationContext::default()).await.unwrap_err();
    assert_eq!(denied.code, POLICY_DENIED);
    let value = registry.invoke(&method, params, context).await.unwrap();
    let report: FixReport = serde_json::from_value(value).unwrap();
    assert_eq!(report.fixes().len(), 1);
    assert!(report.diagnostics.is_empty());
}
#[tokio::test]
async fn test_repository_workflow_is_equivalent_directly_and_over_json_rpc() {
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
