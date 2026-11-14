#![allow(clippy::indexing_slicing)]
use super::*;
use acorn_analysis::{readability::ReadabilityType, CheckCategory, CheckSeverity, FairEvidence};
use acorn_core::prelude::{vec, ToString, Vec};
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_schema::codec::DocumentCodec;
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
fn test_research_activity_rejects_invalid_identifiers_at_every_typed_location() {
    let identifier = serde_json::json!({"value":"https://ror.org/01qz5mb56", "scheme":"ROR", "schemeUri":"https://ror.org"});
    let mut document: serde_json::Value = serde_json::from_str(VALID_RESEARCH_ACTIVITY).expect("fixture");
    document["meta"]["organizations"] = serde_json::json!([{
        "name":"Example organization", "identifiers":[identifier.clone()],
        "roles":[{"organization":identifier.clone(), "role":"Publisher"}]
    }]);
    document["meta"]["contributors"] = serde_json::json!([{
        "name":"Example contributor", "identifiers":[identifier.clone()], "affiliations":[identifier.clone()], "roles":[]
    }]);
    document["meta"]["relatedResources"] = serde_json::json!([{"identifier":identifier.clone(), "relationType":"References"}]);
    document["meta"]["outputs"] = serde_json::json!([{
        "title":"Example output", "type":"dataset", "keywords":[], "alternateIdentifiers":[identifier.clone()],
        "relatedIdentifiers":[{"identifier":identifier, "relationType":"References"}]
    }]);
    document["meta"]["ror"] = serde_json::json!(["https://ror.org/01qz5mb56"]);
    let validate = |document: &serde_json::Value| {
        AcornService
            .validate_research_activity(ValidateResearchActivityRequest {
                content: document.to_string(),
                format: MimeType::Json,
                source_uri: None,
                vocabulary_fuzzy: VocabularyFuzzyMode::Off,
            })
            .expect("validation response")
    };
    let valid = validate(&document);
    assert!(valid.valid, "{:?}", valid.diagnostics);
    for (pointer, path) in [
        ("/meta/contributors/0/affiliations/0/value", "meta.contributors[0].affiliations[0].value"),
        ("/meta/contributors/0/identifiers/0/value", "meta.contributors[0].identifiers[0].value"),
        ("/meta/organizations/0/identifiers/0/value", "meta.organizations[0].identifiers[0].value"),
        (
            "/meta/organizations/0/roles/0/organization/value",
            "meta.organizations[0].roles[0].organization.value",
        ),
        (
            "/meta/outputs/0/alternateIdentifiers/0/value",
            "meta.outputs[0].alternateIdentifiers[0].value",
        ),
        (
            "/meta/outputs/0/relatedIdentifiers/0/identifier/value",
            "meta.outputs[0].relatedIdentifiers[0].identifier.value",
        ),
        ("/meta/relatedResources/0/identifier/value", "meta.relatedResources[0].identifier.value"),
        ("/meta/ror/0", "meta.ror[0]"),
    ] {
        let mut invalid = document.clone();
        *invalid.pointer_mut(pointer).expect("identifier location") = serde_json::json!("https://ror.org/01qz5mb56xxx");
        let response = validate(&invalid);
        assert!(!response.valid, "malformed identifier accepted at {path}");
        assert!(
            response
                .diagnostics
                .iter()
                .any(|issue| issue.path.as_deref() == Some(path) && issue.code == "ROR"),
            "{path}: {:?}",
            response.diagnostics
        );
    }
}
#[test]
fn test_research_activity_rejects_invalid_typed_identifier_in_every_format() {
    let mut document: serde_json::Value = serde_json::from_str(VALID_RESEARCH_ACTIVITY).expect("fixture");
    document["meta"]["organizations"] = serde_json::json!([{
        "name":"Organization", "roles":[], "identifiers":[{
            "value":"https://ror.org/01qz5mb56xxx", "scheme":"ROR"
        }]
    }]);
    let activity: ResearchActivity = serde_json::from_value(document).expect("typed activity");
    for format in [MimeType::Json, MimeType::Jsonc, MimeType::Markdown, MimeType::Yaml, MimeType::Zon] {
        let content = activity.encode(&format).expect("encode activity");
        let response = AcornService
            .validate_research_activity(ValidateResearchActivityRequest {
                content,
                format: format.clone(),
                source_uri: None,
                vocabulary_fuzzy: VocabularyFuzzyMode::Off,
            })
            .expect("response");
        assert!(!response.valid, "malformed ROR accepted in {format:?}");
        assert!(
            response.diagnostics.iter().any(|issue| issue.code == "ROR"),
            "{format:?}: {:?}",
            response.diagnostics
        );
    }
}
#[test]
fn test_research_activity_validation_includes_every_fair_rule_description() {
    let result = AcornService
        .validate_research_activity(ValidateResearchActivityRequest {
            content: VALID_RESEARCH_ACTIVITY.to_string(),
            format: MimeType::Json,
            source_uri: None,
            vocabulary_fuzzy: VocabularyFuzzyMode::Relaxed,
        })
        .unwrap();
    let findings = FairEvidence::default().assess();
    assert_eq!(findings.len(), 41);
    for finding in findings {
        let code = finding.indicator.id.to_string();
        let diagnostic = result.diagnostics.iter().find(|item| item.code == code).expect("FAIR diagnostic");
        assert_eq!(diagnostic.category, CheckCategory::Fair);
        assert_eq!(
            diagnostic.message.matches(&finding.indicator.description).count(),
            1,
            "{code}: {}",
            diagnostic.message
        );
    }
}
#[test]
fn test_research_activity_validation_keeps_advisories_with_schema_errors() {
    let content = VALID_RESEARCH_ACTIVITY.replace(r#""title": "Valid Project""#, r#""title": "x""#);
    let result = AcornService
        .validate_research_activity(ValidateResearchActivityRequest {
            content,
            format: MimeType::Json,
            source_uri: Some("example.json".to_string()),
            vocabulary_fuzzy: VocabularyFuzzyMode::Relaxed,
        })
        .unwrap();
    assert!(!result.valid);
    assert_eq!(result.vocabulary_fuzzy, Some(VocabularyFuzzyMode::Relaxed));
    assert!(result
        .diagnostics
        .iter()
        .any(|item| item.category == CheckCategory::Schema && item.severity == CheckSeverity::Error));
    assert!(result
        .diagnostics
        .iter()
        .any(|item| item.category == CheckCategory::Prose && item.code.starts_with("vocabulary.")));
    for severity in [CheckSeverity::Suggestion, CheckSeverity::Info] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|item| item.category == CheckCategory::Fair && item.severity == severity),
            "missing {severity}"
        );
    }
    assert!(result
        .diagnostics
        .iter()
        .all(|item| item.path.as_deref().is_some_and(|path| path.starts_with("example.json:"))));
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
    assert!(relaxed
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.starts_with("vocabulary."))
        .all(|diagnostic| diagnostic.category == CheckCategory::Prose));
    let off = AcornService
        .validate_research_activity(ValidateResearchActivityRequest {
            content: VALID_RESEARCH_ACTIVITY.to_string(),
            format: MimeType::Json,
            source_uri: None,
            vocabulary_fuzzy: VocabularyFuzzyMode::Off,
        })
        .expect("research activity response");
    assert!(!off.diagnostics.is_empty());
    assert!(off.diagnostics.iter().all(|diagnostic| diagnostic.category == CheckCategory::Fair));
    assert_eq!(off.vocabulary_fuzzy, Some(VocabularyFuzzyMode::Off));
}
#[test]
fn test_research_activity_validation_skips_advisories_when_parsing_fails() {
    let result = AcornService
        .validate_research_activity(ValidateResearchActivityRequest {
            content: r#"{"title":1}"#.to_string(),
            format: MimeType::Json,
            source_uri: None,
            vocabulary_fuzzy: VocabularyFuzzyMode::Relaxed,
        })
        .unwrap();
    assert!(!result.valid);
    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .all(|item| item.category == CheckCategory::Schema && item.severity == CheckSeverity::Error));
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
    let dublin_core = AcornService
        .validate_standard(ValidateStandardRequest {
            content: r#"{
                "@id": "https://example.org/work",
                "http://purl.org/dc/terms/title": { "@value": "A title", "@language": "en" }
            }"#
            .to_string(),
            format: MimeType::LdJson,
            source_uri: Some("fixture.jsonld".to_string()),
            standard: Standard::DublinCore,
        })
        .expect("Dublin Core validation response");
    assert!(dublin_core.valid);
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
