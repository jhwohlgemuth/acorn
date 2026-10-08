#![allow(clippy::expect_used, clippy::panic)]
//! Ignored interoperability coverage for configured ACP agents

use acorn::{
    agent::{InferenceRequest, PromptContext, PromptHydrator},
    analyzer::fix::{CheckReference, CheckReferences, Fix, FixAutomation, FixOptions, IntoFixes, Plan, Policy, Runner},
    io::acp::{AgentProfile, Client, Codex, OpenCode},
    util::constants::env::TEST_ACP_AGENT,
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
    assert!(!automated.is_empty(), "ACP returned no safe automatic fix");
    let directory = tempfile::tempdir().expect("create automated ACP fix workspace");
    let input = directory.path().join("activity.json");
    fs::write(&input, contents).expect("write automated ACP fix input");
    let plan = Plan::from_fixes(&automated, Policy::safe(false)).expect("resolve automated ACP fix plan");
    let outcomes = plan
        .execute(directory.path(), &DocumentOnlyRunner, || true)
        .expect("apply automated ACP fix");
    let fixed = fs::read(&input).expect("read automated ACP fix output");
    assert!(!outcomes.is_empty(), "automated ACP fix plan had no actions");
    assert_ne!(fixed, contents, "automated ACP fix did not change its disposable input");
    println!(
        "{} I applied the safe fix to a disposable copy:\n{}",
        "[ACORN]".green().bold(),
        String::from_utf8_lossy(&fixed)
    );
    directory.close().expect("remove automated ACP fix workspace");
}
fn enabled(agent_id: &str) -> bool {
    std::env::var(TEST_ACP_AGENT).is_ok_and(|value| value == agent_id)
}
fn fix_reference() -> CheckReference {
    CheckReference {
        category: "schema".to_string(),
        context: Some(r#"{"title":"TBD"}"#.to_string()),
        fingerprint: "a".repeat(64),
        id: "check-0001".to_string(),
        locator: None,
        message: "Replace the placeholder title \"TBD\" in activity.json with \"Interoperability Activity\".".to_string(),
        severity: "error".to_string(),
        standard: "research-activity-data".to_string(),
        uri: Some("activity.json".to_string()),
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
fn run_agent_interoperability<A: AgentProfile>() {
    let agent_id = A::AGENT.command();
    if !enabled(agent_id) {
        return;
    }
    let (directory, input, contents) = synthetic_input();
    let request_label = format!("[ACORN → {agent_id}]");
    let response_label = format!("[{agent_id} → ACORN]");
    println!(
        "{} I created a synthetic activity to review:\n{}",
        "[ACORN]".green().bold(),
        String::from_utf8_lossy(&contents)
    );
    let client = Client::new(A::process_config(None, directory.path().to_path_buf()));
    let runtime = runtime();
    println!("{} Are you ready to review it?", request_label.as_str().green().bold());
    let readiness = runtime.block_on(client.protocol_readiness()).expect("negotiate ACP readiness");
    assert_eq!(readiness.protocol_version, 1);
    println!(
        "{} Ready (protocol v{})",
        response_label.as_str().cyan().bold(),
        readiness.protocol_version
    );
    println!("{} I hydrated this general text:\n{GENERAL_CONTENT}", "[ACORN]".green().bold());
    println!(
        "{} Please summarize this general content without changing it.",
        request_label.as_str().green().bold()
    );
    let general = runtime
        .block_on(client.infer_checked(&general_content_request()))
        .expect("run general-content ACP prompt");
    assert!(!general.text.trim().is_empty(), "ACP general-content prompt returned no text");
    println!("{} General summary:\n{}", response_label.as_str().cyan().bold(), general.text);
    let (activity, request) = hydrated_request();
    let activity = serde_json::to_string_pretty(&activity).expect("serialize hydrated RAD fixture");
    println!(
        "{} I hydrated this valid RAD record for enrichment:\n{activity}",
        "[ACORN]".green().bold()
    );
    println!(
        "{} Please summarize this RAD record without changing it.",
        request_label.as_str().green().bold()
    );
    let result = runtime.block_on(client.infer_checked(&request)).expect("run hydrated ACP prompt");
    assert!(!result.text.trim().is_empty(), "ACP prompt returned no text");
    println!(
        "{} Summary received ({} characters):\n{}",
        response_label.as_str().cyan().bold(),
        result.text.chars().count(),
        result.text
    );
    println!("{} Now propose a safe, reviewable repair.", request_label.as_str().green().bold());
    let report = runtime
        .block_on(vec![fix_reference()].suggest(
            &client,
            &FixOptions {
                timeout: 120_000.into(),
                ..FixOptions::default()
            },
        ))
        .expect("request ACP fix suggestion");
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    let fixes = report.fixes();
    assert!(!fixes.is_empty(), "ACP returned no validated fix suggestions");
    println!("{} I validated {} fix suggestion(s)", "[ACORN]".green().bold(), fixes.len());
    let proposals = fixes
        .iter()
        .enumerate()
        .map(|(index, fix)| {
            let actions = serde_json::to_string_pretty(&fix.actions).expect("serialize validated ACP fix actions");
            format!(
                "{}. {}\n   Target: {}\n   Why: {}\n   Confidence: {:.0}% | Risk: {:?} | Automation: {:?}\n   Actions:\n{}",
                index.saturating_add(1),
                fix.summary,
                fix.target,
                fix.rationale,
                fix.confidence * 100.0,
                fix.risk,
                fix.automation,
                actions
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    println!("{} Proposed fix:\n{proposals}", response_label.as_str().cyan().bold());
    apply_automated_fix(&fixes, &contents);
    assert_eq!(fs::read(&input).expect("read synthetic activity"), contents);
    directory.close().expect("reap ACP session workspace");
    println!(
        "{} The original activity stayed unchanged; the ACP workspace is reaped.",
        "[ACORN]".green().bold()
    );
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build interoperability test runtime")
}
fn synthetic_input() -> (TempDir, PathBuf, Vec<u8>) {
    let directory = tempfile::tempdir().expect("create synthetic ACP workspace");
    let path = directory.path().join("activity.json");
    let contents = br#"{"title":"TBD"}"#.to_vec();
    fs::write(&path, &contents).expect("write synthetic activity");
    (directory, path, contents)
}

#[test]
#[ignore = "requires ACORN_TEST_ACP_AGENT=codex and a configured local codex-acp adapter"]
fn test_codex_acp_returns_text_and_validated_fix_suggestions_without_mutating_input() {
    run_agent_interoperability::<Codex>();
}
#[test]
fn test_hydrated_request_uses_valid_rad_fixture() {
    let (activity, request) = hydrated_request();
    assert_eq!(activity.title, "Minimal Valid RAD");
    assert!(request.prompt.body.contains("Minimal Valid RAD"));
    assert!(general_content_request().prompt.body.contains(GENERAL_CONTENT));
}
#[test]
#[ignore = "requires ACORN_TEST_ACP_AGENT=opencode and a configured local OpenCode ACP agent"]
fn test_opencode_acp_returns_text_and_validated_fix_suggestions_without_mutating_input() {
    run_agent_interoperability::<OpenCode>();
}
