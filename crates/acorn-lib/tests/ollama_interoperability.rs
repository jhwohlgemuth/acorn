#![allow(clippy::expect_used, clippy::panic)]
//! Ignored interoperability coverage for configured Ollama endpoints

use acorn::{
    agent::{InferenceRequest, PromptContext, PromptHydrator},
    analyzer::fix::{CheckReference, CheckReferences, Fix, FixAutomation, FixOptions, IntoFixes, Plan, Policy, Runner},
    io::api::{
        openai::{Client, EndpointMode, Options},
        Configuration,
    },
};
use acorn_schema::{
    agent::{PromptFileAsset, PromptTemplateConfiguration},
    research_activity::ResearchActivity,
    validation::Validate,
};
use owo_colors::OwoColorize;
use serde_json::json;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

const GENERAL_CONTENT: &str = "A field team mapped three wetland plots after spring rainfall. Plot B retained water longest and hosted the most amphibian calls. The team will repeat observations after the summer dry period before making restoration recommendations.";
struct DocumentOnlyRunner;
impl Runner for DocumentOnlyRunner {
    fn run(&self, _binary: &str, _arguments: &[&str]) -> acorn::io::ApiResult<String> {
        panic!("a safe document fix must not invoke a recipe")
    }
}
fn apply_automated_fix(fixes: &[&Fix], contents: &[u8]) {
    let automated = fixes
        .iter()
        .find(|fix| fix.automation == FixAutomation::SafeAutomatic)
        .map(|fix| (*fix).clone())
        .into_iter()
        .collect::<Vec<_>>();
    assert!(!automated.is_empty(), "Ollama returned no safe automatic fix");
    let directory = tempfile::tempdir().expect("create automated Ollama fix workspace");
    let input = directory.path().join("activity.json");
    fs::write(&input, contents).expect("write automated Ollama fix input");
    let plan = Plan::from_fixes(&automated, Policy::safe(false)).expect("resolve automated Ollama fix plan");
    let outcomes = plan
        .execute(directory.path(), &DocumentOnlyRunner, || true)
        .expect("apply automated Ollama fix");
    let fixed = fs::read(&input).expect("read automated Ollama fix output");
    assert!(!outcomes.is_empty(), "Ollama automated fix plan had no actions");
    assert_ne!(fixed, contents, "Ollama automated fix did not change its disposable input");
    println!(
        "{} I applied the safe fix to a disposable copy:\n{}",
        "[ACORN]".green().bold(),
        String::from_utf8_lossy(&fixed)
    );
    directory.close().expect("remove automated Ollama fix workspace");
}
fn client() -> Client {
    let base_url = required_environment("ACORN_TEST_OLLAMA_BASE_URL");
    let model = required_environment("ACORN_TEST_OLLAMA_MODEL");
    Client::ollama(Options::default().with_domain(&base_url), EndpointMode::ChatCompletions, model)
}
fn enabled() -> bool {
    std::env::var_os("ACORN_TEST_OLLAMA").is_some_and(|value| value == "1")
}
fn fix_reference() -> CheckReference {
    CheckReference {
        category: "schema".to_string(),
        context: Some(r#"{"title":"TBD"}"#.to_string()),
        fingerprint: "a".repeat(64),
        id: "check-0001".to_string(),
        locator: None,
        message: "Replace the placeholder title \"TBD\" in activity.json with \"Interoperability Activity\".".to_string(),
        rule_id: None,
        severity: "error".to_string(),
        standard: "research-activity-data".to_string(),
        uri: Some("activity.json".to_string()),
        vocabulary_advisory: None,
    }
}
fn general_content_request() -> InferenceRequest {
    let context = PromptContext {
        config: PromptTemplateConfiguration::init().max_items(2).max_words(60).build(),
        data: json!({"text": GENERAL_CONTENT}),
    };
    let prompt = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context).expect("hydrate general-content prompt");
    InferenceRequest {
        agent: None,
        model: None,
        permission_policy: acorn::agent::InferencePermissionPolicy::DenyMutation,
        prompt,
        response_schema: None,
        timeout: 120_000.into(),
    }
}
fn hydrated_request() -> (ResearchActivity, InferenceRequest) {
    let activity = serde_json::from_str::<ResearchActivity>(include_str!("../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json"))
        .expect("deserialize valid RAD fixture");
    activity.validate().expect("validate RAD fixture");
    let text = serde_json::to_string_pretty(&activity).expect("serialize valid RAD fixture");
    let context = PromptContext {
        config: PromptTemplateConfiguration::init().max_items(4).max_words(120).build(),
        data: json!({"text": text}),
    };
    let prompt = PromptHydrator::hydrate(PromptFileAsset::Summarize, &context).expect("hydrate synthetic prompt");
    (
        activity,
        InferenceRequest {
            agent: None,
            model: None,
            permission_policy: acorn::agent::InferencePermissionPolicy::DenyMutation,
            prompt,
            response_schema: None,
            timeout: 120_000.into(),
        },
    )
}
fn required_environment(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set when ACORN_TEST_OLLAMA=1"))
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build interoperability test runtime")
}
async fn smoke(contents: &[u8]) {
    let client = client();
    println!("{} I hydrated this general text:\n{GENERAL_CONTENT}", "[ACORN]".green().bold());
    println!(
        "{} Please summarize this general content without changing it.",
        "[ACORN → Ollama]".green().bold()
    );
    let general = client
        .infer_checked(&general_content_request())
        .await
        .expect("run general-content Ollama prompt");
    assert!(!general.text.trim().is_empty(), "Ollama general-content prompt returned no text");
    assert_eq!(general.provenance.backend, "ollama");
    println!("{} General summary:\n{}", "[Ollama → ACORN]".cyan().bold(), general.text);
    let (activity, request) = hydrated_request();
    let activity = serde_json::to_string_pretty(&activity).expect("serialize hydrated RAD fixture");
    println!(
        "{} I hydrated this valid RAD record for enrichment:\n{activity}",
        "[ACORN]".green().bold()
    );
    println!(
        "{} Please summarize this RAD record without changing it.",
        "[ACORN → Ollama]".green().bold()
    );
    let result = client.infer_checked(&request).await.expect("run hydrated Ollama prompt");
    assert!(!result.text.trim().is_empty(), "Ollama prompt returned no text");
    assert_eq!(result.provenance.backend, "ollama");
    println!(
        "{} Summary received ({} characters):\n{}",
        "[Ollama → ACORN]".cyan().bold(),
        result.text.chars().count(),
        result.text
    );
    println!("{} Now propose a safe, reviewable repair.", "[ACORN → Ollama]".green().bold());
    let report = vec![fix_reference()]
        .suggest(
            &client,
            &FixOptions {
                timeout: 120_000.into(),
                ..FixOptions::default()
            },
        )
        .await
        .expect("request Ollama fix suggestion");
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    let fixes = report.fixes();
    assert!(!fixes.is_empty(), "Ollama returned no validated fix suggestions");
    assert!(
        fixes.iter().all(|fix| fix.provenance.backend == "ollama"),
        "Ollama fix suggestions reported unexpected provenance"
    );
    println!("{} I validated {} Ollama fix suggestion(s)", "[ACORN]".green().bold(), fixes.len());
    let proposals = fixes
        .iter()
        .enumerate()
        .map(|(index, fix)| {
            let actions = serde_json::to_string_pretty(&fix.actions).expect("serialize validated Ollama fix actions");
            format!(
                "{}. {}\n   Target: {}\n   Why: {}\n   Confidence: {:.0}% | Risk: {:?} | Automation: {:?}\n   Actions:\n{}",
                index.saturating_add(1),
                fix.summary,
                fix.target,
                fix.rationale,
                f64::from(fix.confidence) * 100.0,
                fix.risk,
                fix.automation,
                actions
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    println!("{} Proposed fix:\n{proposals}", "[Ollama → ACORN]".cyan().bold());
    apply_automated_fix(&fixes, contents);
}
fn synthetic_input() -> (TempDir, PathBuf, Vec<u8>) {
    let directory = tempfile::tempdir().expect("create synthetic Ollama workspace");
    let path = directory.path().join("activity.json");
    let contents = br#"{"title":"TBD"}"#.to_vec();
    fs::write(&path, &contents).expect("write synthetic activity");
    (directory, path, contents)
}

#[test]
fn test_hydrated_request_uses_valid_rad_fixture() {
    let (activity, request) = hydrated_request();
    assert_eq!(activity.title, "Minimal Valid RAD");
    assert!(request.prompt.body.contains("Minimal Valid RAD"));
    assert!(general_content_request().prompt.body.contains(GENERAL_CONTENT));
}
#[test]
#[ignore = "requires ACORN_TEST_OLLAMA=1 and a configured Ollama endpoint and model"]
fn test_ollama_returns_text_and_validated_fix_suggestions_without_mutating_input() {
    if !enabled() {
        return;
    }
    let (directory, input, contents) = synthetic_input();
    println!(
        "{} I created a synthetic activity to review:\n{}",
        "[ACORN]".green().bold(),
        String::from_utf8_lossy(&contents)
    );
    runtime().block_on(smoke(&contents));
    assert_eq!(fs::read(&input).expect("read synthetic activity"), contents);
    directory.close().expect("remove synthetic workspace");
    println!(
        "{} The original activity stayed unchanged; the Ollama workspaces are removed.",
        "[ACORN]".green().bold()
    );
}
