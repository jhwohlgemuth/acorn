#[cfg(feature = "template")]
use super::prompt::{Prompt, PromptFileAssetExt, Rendered};
use super::*;
#[cfg(feature = "template")]
use acorn_schema::agent::PromptTemplateConfiguration;
use acorn_schema::agent::{FrontMatter, PromptFileAsset};
use acorn_schema::standard::zon;
use serde_json::json;

#[cfg(feature = "template")]
fn context(data: serde_json::Value) -> PromptContext<serde_json::Value> {
    PromptContext {
        config: PromptTemplateConfiguration::init()
            .include_analogy(true)
            .include_examples(true)
            .include_implicit(false)
            .include_practice(true)
            .language("French".to_string())
            .max_items(3)
            .max_words(42)
            .min_confidence(0.75)
            .text("legacy source".to_string())
            .build(),
        data,
    }
}
fn request() -> InferenceRequest {
    InferenceRequest {
        agent: None,
        model: Some("test-model".to_string()),
        permission_policy: InferencePermissionPolicy::DenyMutation,
        prompt: HydratedPrompt {
            asset: PromptFileAsset::Summarize,
            body: "Summarize this source".to_string(),
            metadata: FrontMatter::init()
                .description("Summary prompt".to_string())
                .name("summarize".to_string())
                .build(),
        },
        response_schema: None,
        timeout_ms: 1_000,
    }
}
fn result() -> InferenceResult {
    InferenceResult {
        finish_reason: InferenceFinishReason::Stop,
        provenance: InferenceProvenance {
            agent: None,
            backend: "memory".to_string(),
            model: Some("test-model".to_string()),
            request_id: Some("test-request".to_string()),
        },
        structured: Some(json!({"summary": "done"})),
        text: "done".to_string(),
        usage: Some(InferenceUsage {
            input_tokens: Some(3),
            output_tokens: Some(1),
        }),
    }
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_appends_complete_zon_data_by_default() {
    let prompt = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context(json!({"text": "source"}))).unwrap();
    assert!(prompt.body.ends_with("# Input Data (ZON)\n\n```zon\ntext:source\n```"));
    assert_eq!(prompt.body.matches("# Input Data").count(), 1);
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_expands_fence_around_backticks() {
    let prompt = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context(json!({"text": "literal ``` marker"}))).unwrap();
    assert!(prompt.body.contains("# Input Data (ZON)\n\n````zon\n"));
    assert!(prompt.body.ends_with("\n````"));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_rejects_invalid_frontmatter() {
    let prompt = Prompt::<Rendered>::new("---\nname: [\n---\nbody".to_string());
    let error = PromptFileAsset::Summarize.hydrate(prompt).unwrap_err().into_error();
    assert!(error.to_string().contains("frontmatter"));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_rejects_missing_required_data() {
    let error = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context(json!({}))).unwrap_err();
    assert!(error.to_string().contains("Failed to render prompt template"));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_rejects_non_object_data() {
    let error = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context(json!("source"))).unwrap_err();
    assert!(error.to_string().contains("must serialize as an object"));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_rejects_reserved_data_keys() {
    let error = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context(json!({"config": {}}))).unwrap_err();
    assert!(error.to_string().contains("reserved key 'config'"));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_rejects_unknown_assets() {
    let error = PromptHydrator::hydrate(PromptFileAsset::Unknown("missing.prompt".to_string()), &context(json!({}))).unwrap_err();
    assert!(error.to_string().contains("missing.prompt"));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_rejects_unresolved_template_syntax() {
    let prompt = Prompt::<Rendered>::new("---\nname: summarize\n---\n{{ missing }}".to_string());
    let error = PromptFileAsset::Summarize.hydrate(prompt).unwrap_err().into_error();
    assert!(error.to_string().contains("unresolved template syntax"));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_renders_every_embedded_prompt() {
    let cases = [
        (PromptFileAsset::Eli5, "eli5", "Keep output under 42 words"),
        (PromptFileAsset::ExtractClaim, "extract-claim", "Extract at most 3 distinct claims"),
        (PromptFileAsset::FindGaps, "find-gaps", "hydrated source text"),
        (PromptFileAsset::Summarize, "summarize", "Keep output under 42 words"),
        (PromptFileAsset::Teach, "teach", "If true, provide 2-3 concrete"),
        (PromptFileAsset::Translate, "translate", "English-to-French"),
    ];
    cases.into_iter().for_each(|(asset, name, expected)| {
        let prompt = PromptHydrator::hydrate(asset, &context(json!({"language": "French", "text": "hydrated source text"})))
            .expect("embedded prompt should hydrate");
        assert!(prompt.body.contains("hydrated source text"));
        assert!(prompt.body.contains(expected));
        assert!(!prompt.body.contains("{{"));
        assert_eq!(prompt.metadata.name, name);
        assert!(prompt.metadata.description().is_some());
        let output = prompt.metadata.output.as_ref().expect("embedded prompt should declare output metadata");
        assert!(output.format.is_some());
        assert!(output.schema.is_some());
    });
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_renders_multiline_data_into_valid_frontmatter() {
    let source = "first line\n\"quoted\" second line";
    let prompt = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context(json!({"text": source}))).unwrap();
    let encoded = prompt
        .body
        .split_once("# Input Data (ZON)\n\n```zon\n")
        .map(|(_, encoded)| encoded.trim_end_matches("\n```"))
        .expect("prompt should contain one ZON data block");
    assert_eq!(zon::decode_value(encoded).unwrap(), json!({"text": source}));
}
#[cfg(feature = "template")]
#[test]
fn test_hydrate_supports_pretty_json_for_troubleshooting() {
    let prompt =
        PromptHydrator::hydrate_with_format(PromptFileAsset::Summarize, &context(json!({"text": "source"})), PromptDataFormat::Json).unwrap();
    assert!(prompt.body.ends_with("# Input Data (JSON)\n\n```json\n{\n  \"text\": \"source\"\n}\n```"));
}
#[tokio::test]
async fn test_in_memory_backend_returns_deterministic_result() {
    let expected = result();
    let actual = run_inference(&InMemoryBackend::new(expected.clone()), &request()).await.unwrap();
    assert_eq!(actual, expected);
}
#[test]
fn test_inference_contract_round_trips_without_transport_or_secret_fields() {
    let encoded = serde_json::to_value(request()).unwrap();
    let decoded: InferenceRequest = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded, request());
    let text = encoded.to_string();
    assert!(!text.contains("api_key"));
    assert!(!text.contains("http_status"));
    assert!(!text.contains("permission_request_id"));
}
#[tokio::test]
async fn test_inference_service_rejects_empty_prompts_and_zero_timeouts() {
    let backend = InMemoryBackend::new(result());
    let empty = InferenceRequest {
        prompt: HydratedPrompt {
            body: " ".to_string(),
            ..request().prompt
        },
        ..request()
    };
    assert!(run_inference(&backend, &empty).await.unwrap_err().to_string().contains("cannot be empty"));
    let no_timeout = InferenceRequest { timeout_ms: 0, ..request() };
    assert!(run_inference(&backend, &no_timeout)
        .await
        .unwrap_err()
        .to_string()
        .contains("greater than zero"));
}
