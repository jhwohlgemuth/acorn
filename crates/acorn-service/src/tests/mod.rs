#![allow(clippy::indexing_slicing)]
use super::*;
use acorn_analysis::{readability::ReadabilityType, CheckCategory};
use acorn_core::prelude::{vec, ToString, Vec};
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_schema::hardware::Memory;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::Standard;
use acorn_schema::vocabulary::VocabularyFuzzyMode;
use futures::executor::block_on;

const VALID_RESEARCH_ACTIVITY: &str = include_str!("../../../../tests/fixtures/data/valid_project/index.json");

#[test]
fn test_json_rpc_keeps_version_informational_and_registers_canonical_operations() {
    let registry = OperationRegistry::acorn(AcornService, "9.8.7", None).expect("operation registry");
    assert_eq!(
        registry.names().into_iter().map(|name| name.to_string()).collect::<Vec<_>>(),
        [
            "acorn.operations.replay",
            "acorn.operations.status",
            "acorn.readability.check",
            "acorn.research-activity.validate",
            "acorn.standard.validate",
            "acorn.version",
            "acorn.workflows.process",
        ]
    );
    let dispatch = block_on(registry.dispatch(
        br#"{"jsonrpc":"2.0","id":1,"method":"acorn.version","params":{}}"#,
        InvocationContext::default(),
    ));
    let body = dispatch.body.expect("JSON-RPC response body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON-RPC response");
    assert_eq!(value["result"]["version"], "9.8.7");
    let dispatch = block_on(registry.dispatch(
        br#"{"jsonrpc":"2.0","id":2,"method":"acorn.operations.status","params":{"operationKey":"fixture"}}"#,
        InvocationContext::default(),
    ));
    let body = dispatch.body.expect("JSON-RPC response body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON-RPC response");
    assert_eq!(value["error"]["code"], -32003);
}
#[test]
fn test_json_rpc_rejects_duplicate_fields_and_oversized_input() {
    let registry = OperationRegistry::acorn(AcornService, "9.8.7", None).expect("operation registry");
    let duplicate = block_on(registry.dispatch(
        br#"{"jsonrpc":"2.0","id":1,"id":2,"method":"acorn.version"}"#,
        InvocationContext::default(),
    ));
    let body = duplicate.body.expect("error response");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON-RPC response");
    assert_eq!(value["error"]["code"], -32700);
    let oversized = block_on(registry.dispatch(
        br#"{"jsonrpc":"2.0","id":1,"method":"acorn.version"}"#,
        InvocationContext {
            max_input: Memory::kb(0.0),
            ..InvocationContext::default()
        },
    ));
    let body = oversized.body.expect("error response");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("JSON-RPC response");
    assert_eq!(value["error"]["code"], -32002);
}
#[test]
fn test_readability_supports_every_canonical_metric() {
    for metric in [
        ReadabilityType::ARI,
        ReadabilityType::CLI,
        ReadabilityType::FKGL,
        ReadabilityType::FRES,
        ReadabilityType::GFI,
        ReadabilityType::Lix,
        ReadabilityType::SMOG,
    ] {
        let response = AcornService
            .check_readability(CheckReadabilityRequest {
                metric,
                source_uri: None,
                text: "This is a complete sentence for deterministic analysis.".to_string(),
                threshold: None,
            })
            .expect("readability response");
        assert!(response.value.is_finite());
    }
}
#[test]
fn test_rejects_unsafe_and_duplicate_repository_paths() {
    let request = |paths: &[&str]| ProcessRepositoryRequest {
        actions: vec![WorkflowAction::Validate],
        files: paths
            .iter()
            .map(|path| RepositoryFileInput {
                content: "{}".to_string(),
                format: Some(MimeType::Json),
                path: (*path).to_string(),
            })
            .collect(),
        project: "project".to_string(),
        revision: "revision".to_string(),
    };
    assert!(AcornService.process_repository(request(&["../secret.json"])).is_err());
    assert!(AcornService.process_repository(request(&["a.json", "a.json"])).is_err());
}
#[test]
fn test_repository_actions_are_distinct_and_validated() {
    let request = |actions| ProcessRepositoryRequest {
        actions,
        files: vec![RepositoryFileInput {
            content: "{\"z\":1}".to_string(),
            format: Some(MimeType::Json),
            path: "activity.json".to_string(),
        }],
        project: "project".to_string(),
        revision: "revision".to_string(),
    };
    let validated = AcornService
        .process_repository(request(vec![WorkflowAction::Validate]))
        .expect("validate repository");
    assert!(!validated.reports[0].changed);
    assert!(validated.files.is_empty());
    let checked = AcornService
        .process_repository(request(vec![WorkflowAction::Check]))
        .expect("check repository");
    assert!(checked.reports[0].changed);
    assert!(checked.files.is_empty());
    let formatted = AcornService
        .process_repository(request(vec![WorkflowAction::Format]))
        .expect("format repository");
    assert!(formatted.reports[0].changed);
    assert_eq!(formatted.files.len(), 1);
    assert!(AcornService.process_repository(request(Vec::new())).is_err());
    assert!(AcornService
        .process_repository(request(vec![WorkflowAction::Validate, WorkflowAction::Validate]))
        .is_err());
    let denied = AcornService
        .process_repository(request(vec![WorkflowAction::Enrich]))
        .expect_err("enrichment requires a host capability");
    assert_eq!(denied.kind, ServiceErrorKind::PolicyDenied);
    let linked = AcornService
        .process_repository(ProcessRepositoryRequest {
            actions: vec![WorkflowAction::Link],
            files: vec![RepositoryFileInput {
                content: serde_json::to_string(&ResearchActivity::default()).expect("research activity JSON"),
                format: Some(MimeType::Json),
                path: "activity.json".to_string(),
            }],
            project: "project".to_string(),
            revision: "revision".to_string(),
        })
        .expect("link repository");
    assert_eq!(linked.files.len(), 1);
    assert_eq!(linked.files[0].path, "activity.jsonld");
}
#[test]
fn test_repository_does_not_downgrade_invalid_rad_markdown() {
    let content = ResearchActivity::default().to_markdown().replace("## Contact", "## Unknown");
    let result = AcornService.process_repository(ProcessRepositoryRequest {
        actions: vec![WorkflowAction::Validate],
        files: vec![RepositoryFileInput {
            content,
            format: Some(MimeType::Markdown),
            path: "activity.md".to_string(),
        }],
        project: "project".to_string(),
        revision: "revision".to_string(),
    });
    assert!(result.is_err());
}
#[test]
fn test_repository_output_is_sorted_by_portable_path() {
    let response = AcornService
        .process_repository(ProcessRepositoryRequest {
            actions: vec![WorkflowAction::Validate],
            files: vec![
                RepositoryFileInput {
                    content: "{\"z\":1}".to_string(),
                    format: Some(MimeType::Json),
                    path: "z.json".to_string(),
                },
                RepositoryFileInput {
                    content: "{\"a\":1}".to_string(),
                    format: Some(MimeType::Json),
                    path: "a.json".to_string(),
                },
            ],
            project: "project".to_string(),
            revision: "revision".to_string(),
        })
        .expect("portable repository processing");
    assert_eq!(
        response.reports.iter().map(|report| report.path.as_str()).collect::<Vec<_>>(),
        ["a.json", "z.json"]
    );
}
#[test]
fn test_research_activity_validation_reports_effective_vocabulary_policy() {
    let relaxed = AcornService
        .validate_research_activity(ValidateResearchActivityRequest {
            content: VALID_RESEARCH_ACTIVITY.to_string(),
            format: MimeType::Json,
            source_uri: Some("fixture.json".to_string()),
            vocabulary_fuzzy: VocabularyFuzzyMode::Relaxed,
        })
        .expect("research activity response");
    assert!(relaxed.valid);
    assert_eq!(relaxed.vocabulary_fuzzy, Some(VocabularyFuzzyMode::Relaxed));
    assert!(relaxed.diagnostics.iter().any(|diagnostic| diagnostic.code.starts_with("vocabulary.")));
    let off = AcornService
        .validate_research_activity(ValidateResearchActivityRequest {
            content: VALID_RESEARCH_ACTIVITY.to_string(),
            format: MimeType::Json,
            source_uri: None,
            vocabulary_fuzzy: VocabularyFuzzyMode::Off,
        })
        .expect("research activity response");
    assert!(off.diagnostics.is_empty());
    assert_eq!(off.vocabulary_fuzzy, Some(VocabularyFuzzyMode::Off));
}
#[test]
fn test_research_activity_vocabulary_request_defaults_and_rejects_unknown_modes() {
    let request: ValidateResearchActivityRequest = serde_json::from_value(serde_json::json!({
        "content": "{}",
        "format": "json"
    }))
    .expect("default request");
    assert_eq!(request.vocabulary_fuzzy, VocabularyFuzzyMode::Relaxed);
    assert!(serde_json::from_value::<ValidateResearchActivityRequest>(serde_json::json!({
        "content": "{}",
        "format": "json",
        "vocabularyFuzzy": "unknown"
    }))
    .is_err());
}
#[test]
fn test_standard_validation_supports_text_and_rejects_malformed_datacite() {
    let text = AcornService
        .validate_standard(ValidateStandardRequest {
            content: "Portable standards validation.".to_string(),
            format: MimeType::Text,
            source_uri: None,
            standard: Standard::Text,
        })
        .expect("text validation response");
    assert!(text.valid);
    let datacite = AcornService
        .validate_standard(ValidateStandardRequest {
            content: "{}".to_string(),
            format: MimeType::Json,
            source_uri: Some("fixture.json".to_string()),
            standard: Standard::Datacite,
        })
        .expect("DataCite validation response");
    assert!(!datacite.valid);
    assert!(datacite
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.path.as_deref().is_some_and(|path| path.starts_with("fixture.json"))));
}
#[test]
fn test_workflow_action_maps_to_check_category() {
    assert_eq!(CheckCategory::from(WorkflowAction::Validate), CheckCategory::Schema);
    assert_eq!(CheckCategory::from(WorkflowAction::Check), CheckCategory::Quality);
    assert_eq!(CheckCategory::from(WorkflowAction::Enrich), CheckCategory::Quality);
    assert_eq!(CheckCategory::from(WorkflowAction::Format), CheckCategory::Quality);
    assert_eq!(CheckCategory::from(WorkflowAction::Link), CheckCategory::Quality);
}
