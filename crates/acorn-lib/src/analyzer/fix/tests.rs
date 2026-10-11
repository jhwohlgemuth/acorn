use super::execution::{apply_vocabulary_replacement, VocabularyDocument, VocabularyTarget};
use super::plan::Recipe;
use super::service::{validate_envelope, validate_suggested_fix, PendingReference, PendingReferences, SuggestedFix, SuggestionEnvelope};
use super::*;
use crate::agent::{InMemoryBackend, InferenceCapabilities, InferenceFinishReason, InferenceFuture, InferenceResult, InferenceUsage};
use crate::analyzer::{check, CheckCategory, CheckSeverity};
use acorn_core::prelude::Box;
use acorn_core::util::{constants::app::APPLICATION, MimeType};
use acorn_core::Noul;
use acorn_schema::{
    standard::Standard::ResearchActivityData,
    vocabulary::{VocabularyFuzzyMode, VocabularyOutcome},
};
use core::cell::RefCell;
use serde_json::json;
use std::fs;
use tempfile::tempdir;
#[derive(Clone)]
struct FailingBackend;
#[derive(Default)]
struct RecordingRunner(RefCell<Vec<(String, Vec<String>)>>);
impl InferenceBackend for FailingBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }
    fn infer<'a>(&'a self, _request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(core::future::ready(Err(eyre!("fixture backend failure"))))
    }
}
impl Runner for RecordingRunner {
    fn run(&self, binary: &str, arguments: &[&str]) -> ApiResult<String> {
        self.0
            .borrow_mut()
            .push((binary.to_string(), arguments.iter().map(|value| value.to_string()).collect()));
        Ok(String::new())
    }
}
fn action() -> FixAction {
    FixAction::DocumentChange {
        description: "Replace an invalid title".to_string(),
        expected: "Old title".to_string(),
        replacement: "New title".to_string(),
    }
}
fn planned_document_fix() -> Fix {
    let reference = reference("check-0001");
    validate_suggested_fix(suggested(&reference), &reference, &FixOptions::default(), result(&reference).provenance).unwrap()
}
fn reference(id: &str) -> CheckReference {
    CheckReference {
        category: "schema".to_string(),
        context: Some("Old title".to_string()),
        fingerprint: "a".repeat(64),
        id: id.to_string(),
        locator: Some("title".to_string()),
        message: "Title is invalid".to_string(),
        rule_id: None,
        severity: "error".to_string(),
        standard: "research-activity-data".to_string(),
        uri: Some("activity.json".to_string()),
        vocabulary_advisory: None,
    }
}
fn result(reference: &CheckReference) -> InferenceResult {
    InferenceResult {
        finish_reason: InferenceFinishReason::Stop,
        provenance: InferenceProvenance {
            agent: Some("fixture-agent".to_string()),
            backend: "memory".to_string(),
            model: Some("fixture-model".to_string()),
            request_id: Some("fixture-request".to_string()),
        },
        structured: Some(json!({
            "fixes": [{
                "actions": [{
                    "description": "Replace an invalid title",
                    "expected": "Old title",
                    "kind": "document-change",
                    "replacement": "New title"
                }],
                "automation": "safe-automatic",
                "confidence": 0.95,
                "effects": {
                    "elevation": false,
                    "filesystem_write": true,
                    "network": false,
                    "package_manager": false,
                    "process": false,
                    "restart": false,
                    "reversible": true
                },
                "finding_id": reference.id,
                "id": "fix-0001",
                "rationale": "The replacement satisfies the title requirement.",
                "risk": "low",
                "source_fingerprint": reference.fingerprint,
                "status": "proposed",
                "summary": "Replace the invalid title",
                "target": "activity.json"
            }],
            "version": FIX_ENVELOPE_VERSION
        })),
        text: String::new(),
        usage: Some(InferenceUsage {
            input_tokens: Some(10),
            output_tokens: Some(20),
        }),
    }
}
fn suggested(reference: &CheckReference) -> SuggestedFix {
    SuggestedFix {
        actions: vec![action()],
        automation: FixAutomation::SafeAutomatic,
        confidence: Noul::new(0.95).expect("valid unit value"),
        effects: FixEffects {
            filesystem_write: true,
            reversible: true,
            ..FixEffects::default()
        },
        finding_id: reference.id.clone(),
        id: "fix-0001".to_string(),
        rationale: "The replacement satisfies the title requirement.".to_string(),
        risk: FixRisk::Low,
        source_fingerprint: reference.fingerprint.clone(),
        status: FixStatus::Proposed,
        summary: "Replace the invalid title".to_string(),
        target: "activity.json".to_string(),
    }
}
fn vocabulary_reference(id: &str, outcome: VocabularyOutcome, replacement: Option<&str>) -> CheckReference {
    let mut reference = CheckReference {
        category: "prose".to_string(),
        context: Some("chemistrx".to_string()),
        fingerprint: "b".repeat(64),
        id: id.to_string(),
        locator: Some("meta.keywords[0]".to_string()),
        message: "Use canonical vocabulary value 'chemistry' instead of 'chemistrx'.".to_string(),
        rule_id: Some("vocabulary.relaxed-match".to_string()),
        severity: "warning".to_string(),
        standard: "research-activity-data".to_string(),
        uri: Some("activity.json".to_string()),
        vocabulary_advisory: Some(Box::new(VocabularyAdvisory {
            candidates: Vec::new(),
            complete: true,
            edit_distance: Some(1),
            evidence: Some("fuzzy".to_string()),
            mode: VocabularyFuzzyMode::Relaxed,
            method: Some("optimal-string-alignment".to_string()),
            original: "chemistrx".to_string(),
            outcome,
            policy_revision: "vocabulary-v2".to_string(),
            replacement: replacement.map(str::to_string),
            runner_up_margin: Some(Noul::new(0.2).expect("valid unit value")),
            score: Some(Noul::new(0.9).expect("valid unit value")),
            source_locator: "meta.keywords[0]".to_string(),
            variant: Some("restricted-optimal-string-alignment".to_string()),
            version: 1,
            vocabulary: "keywords".to_string(),
            vocabulary_revision: "keywords-vocabulary-v2".to_string(),
        })),
    };
    reference.fingerprint = reference.computed_fingerprint();
    reference
}
#[test]
fn test_batch_limits_count_and_context_deterministically() {
    let options = FixOptions {
        max_context_bytes: 64,
        max_findings_per_batch: 2,
        ..FixOptions::default()
    };
    let references = ["check-0001", "check-0002", "check-0003"].into_iter().map(reference).collect::<Vec<_>>();
    let batches = references.batch(&options);
    assert_eq!(batches.len(), 2);
    assert_eq!(batches.first().map(Vec::len), Some(2));
    assert_eq!(batches.get(1).map(Vec::len), Some(1));
}
#[test]
fn test_build_request_hydrates_bounded_finding_data_and_schema() {
    let request = vec![reference("check-0001")].build(&FixOptions::default()).unwrap();
    assert!(request.prompt.body.contains("check-0001"));
    assert!(request.prompt.body.contains("Title is invalid"));
    let schema = request.response_schema.expect("fix request must include a response schema");
    assert!(request.prompt.body.contains("# Response Schema"));
    assert!(request.prompt.body.contains(&serde_json::to_string_pretty(&schema).unwrap()));
}
#[test]
fn test_check_reference_serialization_omits_optional_vocabulary_advisory() {
    let ordinary = serde_json::to_value(reference("check-0001")).unwrap();
    assert!(ordinary.get("vocabulary_advisory").is_none());
    let vocabulary = serde_json::to_value(vocabulary_reference("check-0001", VocabularyOutcome::FuzzyAccepted, Some("chemistry"))).unwrap();
    assert!(vocabulary.get("vocabulary_advisory").is_some());
}
#[test]
fn test_check_reference_validation_rejects_unknown_vocabulary_mode() {
    let mut reference = serde_json::to_value(vocabulary_reference("check-0001", VocabularyOutcome::FuzzyAccepted, Some("chemistry")))
        .expect("reference should serialize");
    *reference.pointer_mut("/vocabulary_advisory/mode").expect("advisory mode") = json!("unknown");
    assert!(serde_json::from_value::<CheckReference>(reference).is_err());
}
#[test]
fn test_fix_actions_reject_executable_fields() {
    let value = json!({
        "command": "rm",
        "kind": "registered-recipe",
        "recipe_id": "document.format"
    });
    assert!(serde_json::from_value::<FixAction>(value).is_err());
}
#[test]
fn test_fix_report_groups_and_summarizes_proposals() {
    let reference = reference("check-0001");
    let fix = validate_suggested_fix(suggested(&reference), &reference, &FixOptions::default(), result(&reference).provenance).unwrap();
    let report = FixReport::from_parts(vec![fix], Vec::new());
    let decoded: FixReport = serde_json::from_value(serde_json::to_value(&report).unwrap()).unwrap();
    assert_eq!(decoded, report);
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.fixes().len(), 1);
    let summary: FixSummary = report.into();
    assert_eq!(summary.proposed, 1);
}
#[test]
fn test_normalization_sorts_and_bounds_check_fields() {
    let context = "é".repeat(DEFAULT_MAX_EXCERPT_BYTES);
    let second = check!(
        CheckCategory::Schema,
        false,
        severity: CheckSeverity::Warning,
        message: "second",
        context: context,
        uri: "z.json",
    );
    let first = check!(
        CheckCategory::Schema,
        false,
        severity: CheckSeverity::Error,
        message: "first",
        uri: "a.json",
    );
    let pending = vec![
        PendingReference {
            check: &second,
            standard: "research-activity-data".to_string(),
        },
        PendingReference {
            check: &first,
            standard: "research-activity-data".to_string(),
        },
    ];
    let references = pending.normalize(&FixOptions::default()).unwrap();
    assert_eq!(references.first().and_then(|reference| reference.uri.as_deref()), Some("a.json"));
    assert_eq!(references.get(1).map(|reference| reference.id.as_str()), Some("check-0002"));
    assert!(references
        .get(1)
        .and_then(|reference| reference.context.as_ref())
        .is_some_and(|context| context.len() <= DEFAULT_MAX_EXCERPT_BYTES));
    assert!(references
        .iter()
        .all(|reference| digest(&format!("sha256:{}", reference.fingerprint)).is_ok()));
}
#[test]
fn test_options_deserialization_rejects_invalid_minimum_confidence() {
    [0.0, 1.0].into_iter().for_each(|min_confidence| {
        let options = serde_json::from_value::<FixOptions>(json!({ "min_confidence": min_confidence })).expect("valid minimum confidence");
        assert_eq!(options.min_confidence, Noul::new(min_confidence).expect("valid unit value"));
    });
    [-0.1, 1.1].into_iter().for_each(|min_confidence| {
        assert!(serde_json::from_value::<FixOptions>(json!({ "min_confidence": min_confidence })).is_err());
    });
}
#[test]
fn test_options_reject_requests_above_fixed_ceiling() {
    let options = FixOptions {
        max_findings_per_batch: DEFAULT_MAX_FINDINGS_PER_BATCH.saturating_add(1),
        ..FixOptions::default()
    };
    assert!(options.validate().is_err());
}
#[test]
fn test_planner_coalesces_repeated_node_recipe() {
    let mut node = planned_document_fix();
    node.actions = vec![FixAction::RegisteredRecipe {
        recipe_id: if cfg!(target_os = "macos") {
            "node.homebrew".to_string()
        } else if cfg!(windows) {
            "node.winget".to_string()
        } else {
            "unsupported".to_string()
        },
    }];
    node.automation = FixAutomation::InteractiveOnly;
    node.effects = FixEffects {
        filesystem_write: true,
        network: true,
        package_manager: true,
        process: true,
        ..FixEffects::default()
    };
    if cfg!(any(target_os = "macos", windows)) {
        let plan = Plan::from_fixes(&[node.clone(), node.clone(), node], Policy::confirmed(false)).unwrap();
        assert_eq!(plan.preview().len(), 1);
    } else {
        assert!(Plan::from_fixes(&[node], Policy::confirmed(false)).is_err());
    }
}
#[test]
fn test_planner_rejects_registered_recipe_effects_that_model_output_understates() {
    let mut recipe = planned_document_fix();
    recipe.actions = vec![FixAction::RegisteredRecipe {
        recipe_id: "vale.install".to_string(),
    }];
    recipe.automation = FixAutomation::ConfirmationRequired;
    recipe.effects = FixEffects {
        network: true,
        process: true,
        ..FixEffects::default()
    };
    assert!(Plan::from_fixes(&[recipe], Policy::confirmed(false)).is_err());
}
#[test]
fn test_planner_rejects_stale_document_preconditions_and_unapproved_recipes() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("activity.json"), "Different title").unwrap();
    let document = Plan::from_fixes(&[planned_document_fix()], Policy::safe(false)).unwrap();
    assert!(document.execute(directory.path(), &RecordingRunner::default(), || true).is_err());
    let mut recipe = planned_document_fix();
    recipe.actions = vec![FixAction::RegisteredRecipe {
        recipe_id: "vale.install".to_string(),
    }];
    recipe.automation = FixAutomation::ConfirmationRequired;
    recipe.effects = FixEffects {
        filesystem_write: true,
        network: true,
        process: true,
        ..FixEffects::default()
    };
    assert!(Plan::from_fixes(&[recipe], Policy::safe(false)).is_err());
    let mut offline_recipe = planned_document_fix();
    offline_recipe.actions = vec![FixAction::RegisteredRecipe {
        recipe_id: "vale.install".to_string(),
    }];
    offline_recipe.automation = FixAutomation::ConfirmationRequired;
    offline_recipe.effects = FixEffects {
        filesystem_write: true,
        network: true,
        process: true,
        ..FixEffects::default()
    };
    assert!(Plan::from_fixes(&[offline_recipe], Policy::confirmed(true)).is_err());
}
#[test]
fn test_preview_applies_document_changes_atomically_and_preserves_other_fixes() {
    let mut first = planned_document_fix();
    first.actions.push(FixAction::DocumentChange {
        description: "Require missing text".to_string(),
        expected: "Absent text".to_string(),
        replacement: "Replacement".to_string(),
    });
    let mut second = planned_document_fix();
    second.id = "fix-0002".to_string();
    second.actions = vec![FixAction::DocumentChange {
        description: "Replace title".to_string(),
        expected: "Old title".to_string(),
        replacement: "Final title".to_string(),
    }];
    let report = FixReport::from(vec![first, second]);
    let sources = [PreviewSource::Available {
        content: "Old title".to_string(),
        source_format: MimeType::Json,
        standard: ResearchActivityData,
        target: "activity.json".to_string(),
    }];
    let preview = report.project(&sources);
    assert_eq!(preview.targets.len(), 1);
    assert_eq!(preview.targets.first().map(|target| target.proposed.as_str()), Some("Final title"));
    assert_eq!(
        preview.targets.first().and_then(|target| target.fix_ids.first()).map(String::as_str),
        Some("fix-0002")
    );
    assert_eq!(preview.diagnostics.len(), 1);
}
#[test]
fn test_preview_reports_ambiguous_sources_without_projecting() {
    let report = FixReport::from(vec![planned_document_fix()]);
    let source = PreviewSource::Available {
        content: "Old title".to_string(),
        source_format: MimeType::Json,
        standard: ResearchActivityData,
        target: "activity.json".to_string(),
    };
    let preview = report.project(&[source.clone(), source]);
    assert!(preview.targets.is_empty());
    assert_eq!(preview.diagnostics.len(), 1);
    assert_eq!(
        preview.diagnostics.first().map(|diagnostic| diagnostic.summary.as_str()),
        Some("Ambiguous preview target")
    );
}
#[test]
fn test_preview_retains_document_changes_from_mixed_actions_and_omits_net_zero_targets() {
    let mut mixed = planned_document_fix();
    mixed.actions.push(FixAction::Manual {
        instructions: "Review the title".to_string(),
    });
    let mixed_report = FixReport::from(vec![mixed]);
    let source = PreviewSource::Available {
        content: "Old title".to_string(),
        source_format: MimeType::Json,
        standard: ResearchActivityData,
        target: "activity.json".to_string(),
    };
    let mixed_preview = mixed_report.project(core::slice::from_ref(&source));
    let mut reverse = planned_document_fix();
    reverse.id = "fix-0002".to_string();
    reverse.actions = vec![FixAction::DocumentChange {
        description: "Restore title".to_string(),
        expected: "New title".to_string(),
        replacement: "Old title".to_string(),
    }];
    let net_zero = FixReport::from(vec![planned_document_fix(), reverse]).project(&[source]);
    assert_eq!(mixed_preview.targets.first().map(|target| target.proposed.as_str()), Some("New title"));
    assert_eq!(mixed_preview.diagnostics.len(), 1);
    assert!(net_zero.targets.is_empty());
    assert_eq!(
        net_zero.diagnostics.first().map(|diagnostic| diagnostic.summary.as_str()),
        Some("No net document change")
    );
}
#[test]
fn test_registered_recipe_registry_exposes_only_curated_vectors() {
    assert_eq!(Recipe::Chromium.command(), (APPLICATION, ["download", "chromium"].as_slice()));
    assert_eq!(Recipe::HomebrewNode.command(), ("brew", ["install", "node"].as_slice()));
    assert_eq!(Recipe::Vale.command(), (APPLICATION, ["download", "vale"].as_slice()));
    assert_eq!(
        Recipe::WinGetNode.command(),
        (
            "winget",
            [
                "install",
                "--id",
                "OpenJS.NodeJS.LTS",
                "--exact",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ]
            .as_slice()
        )
    );
}
#[test]
fn test_registered_recipes_run_only_fixed_argument_vectors() {
    let mut recipe = planned_document_fix();
    recipe.actions = vec![FixAction::RegisteredRecipe {
        recipe_id: "vale.install".to_string(),
    }];
    recipe.automation = FixAutomation::ConfirmationRequired;
    recipe.effects = FixEffects {
        filesystem_write: true,
        network: true,
        process: true,
        ..FixEffects::default()
    };
    let runner = RecordingRunner::default();
    let plan = Plan::from_fixes(&[recipe], Policy::confirmed(false)).unwrap();
    assert_eq!(
        plan.execute(tempdir().unwrap().path(), &runner, || true).unwrap(),
        [Outcome::Applied("run vale.install".to_string())]
    );
    assert_eq!(
        runner.0.into_inner(),
        [(
            std::env::current_exe().unwrap().display().to_string(),
            vec!["download".to_string(), "vale".to_string()]
        )]
    );
}
#[tokio::test]
async fn test_suggestion_failures_are_reported_without_partial_fixes() {
    let report = vec![reference("check-0001")]
        .suggest(&FailingBackend, &FixOptions::default())
        .await
        .unwrap();
    assert!(report.fixes().is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert!(report
        .diagnostics
        .first()
        .is_some_and(|diagnostic| diagnostic.message.contains("fixture backend failure")));
}
#[tokio::test]
async fn test_suggestion_service_builds_vocabulary_fix_without_inference() {
    let report = vec![vocabulary_reference("check-0001", VocabularyOutcome::FuzzyAccepted, Some("chemistry"))]
        .suggest(&FailingBackend, &FixOptions::default())
        .await
        .unwrap();
    assert!(report.diagnostics.is_empty());
    let fixes = report.fixes();
    assert_eq!(fixes.len(), 1);
    let fix = fixes.first().expect("deterministic vocabulary fix");
    assert_eq!(fix.automation, FixAutomation::SafeAutomatic);
    assert_eq!(fix.provenance.backend, "host");
    assert_eq!(
        fix.actions,
        [FixAction::DocumentChange {
            description: "Use canonical keywords vocabulary value".to_string(),
            expected: "chemistrx".to_string(),
            replacement: "chemistry".to_string(),
        }]
    );
}
#[tokio::test]
async fn test_suggestion_service_returns_validated_grouped_fixes() {
    let reference = reference("check-0001");
    let backend = InMemoryBackend::new(result(&reference));
    let report = vec![reference]
        .suggest(
            &backend,
            &FixOptions {
                min_confidence: Noul::new(0.9).expect("valid unit value"),
                ..FixOptions::default()
            },
        )
        .await
        .unwrap();
    assert!(report.diagnostics.is_empty());
    let fixes = report.fixes();
    assert_eq!(fixes.first().map(|fix| fix.provenance.backend.as_str()), Some("memory"));
    let summary: FixSummary = report.into();
    assert_eq!(summary.proposed, 1);
}
#[test]
fn test_suggestion_validation_rejects_ambiguous_vocabulary_replacement() {
    let reference = vocabulary_reference("check-0001", VocabularyOutcome::Ambiguous, None);
    assert!(validate_suggested_fix(suggested(&reference), &reference, &FixOptions::default(), result(&reference).provenance).is_err());
}
#[test]
fn test_suggestion_validation_rejects_duplicate_actions_and_unsafe_targets() {
    let reference = reference("check-0001");
    let mut duplicate = suggested(&reference);
    duplicate.actions.push(action());
    assert!(validate_suggested_fix(duplicate, &reference, &FixOptions::default(), result(&reference).provenance).is_err());
    let mut unsafe_target = suggested(&reference);
    unsafe_target.target = "../activity.json".to_string();
    assert!(validate_suggested_fix(unsafe_target, &reference, &FixOptions::default(), result(&reference).provenance).is_err());
    let mut low_confidence = suggested(&reference);
    low_confidence.confidence = Noul::ZERO;
    let options = FixOptions {
        min_confidence: Noul::new(0.5).expect("valid unit value"),
        ..FixOptions::default()
    };
    assert!(validate_suggested_fix(low_confidence, &reference, &options, result(&reference).provenance).is_err());
}
#[test]
fn test_suggestion_validation_rejects_unknown_and_stale_findings() {
    let reference = reference("check-0001");
    let unknown = SuggestionEnvelope {
        fixes: vec![SuggestedFix {
            finding_id: "check-9999".to_string(),
            ..suggested(&reference)
        }],
        version: FIX_ENVELOPE_VERSION.to_string(),
    };
    assert!(validate_envelope(
        unknown,
        core::slice::from_ref(&reference),
        &FixOptions::default(),
        result(&reference).provenance
    )
    .is_err());
    let mut second = suggested(&reference);
    second.id = "fix-0002".to_string();
    let duplicate_target = SuggestionEnvelope {
        fixes: vec![suggested(&reference), second],
        version: FIX_ENVELOPE_VERSION.to_string(),
    };
    assert!(validate_envelope(
        duplicate_target,
        core::slice::from_ref(&reference),
        &FixOptions::default(),
        result(&reference).provenance
    )
    .is_err());
    let mut stale = suggested(&reference);
    stale.source_fingerprint = "b".repeat(64);
    assert!(validate_suggested_fix(stale, &reference, &FixOptions::default(), result(&reference).provenance).is_err());
}
#[test]
fn test_trusted_document_plan_applies_and_rolls_back_on_failed_verification() {
    let directory = tempdir().unwrap();
    let target = directory.path().join("activity.json");
    fs::write(&target, "Old title").unwrap();
    let plan = Plan::from_fixes(&[planned_document_fix()], Policy::safe(false)).unwrap();
    assert_eq!(plan.preview(), ["write activity.json"]);
    let applied = plan.execute(directory.path(), &RecordingRunner::default(), || true).unwrap();
    assert_eq!(applied, vec![Outcome::Applied("write activity.json".to_string())]);
    assert_eq!(fs::read_to_string(&target).unwrap(), "New title");
    fs::write(&target, "Old title").unwrap();
    let rollback = Plan::from_fixes(&[planned_document_fix()], Policy::safe(false))
        .unwrap()
        .execute(directory.path(), &RecordingRunner::default(), || false)
        .unwrap();
    assert_eq!(rollback, vec![Outcome::RolledBack("write activity.json".to_string())]);
    assert_eq!(fs::read_to_string(&target).unwrap(), "Old title");
}
#[test]
fn test_vocabulary_fix_deduplicates_scalars_but_preserves_funder_records() {
    let mut data = serde_json::from_str::<acorn_schema::research_activity::ResearchActivity>(include_str!(
        "../../../../../tests/fixtures/data/valid_project/index.json"
    ))
    .expect("research activity fixture");
    data.meta.keywords = vec!["chemistrx".to_string(), "chemistry".to_string()];
    let data = apply_vocabulary_replacement(data, &VocabularyTarget::Keyword(0), "chemistrx", "chemistry").expect("keyword replacement");
    let funder_count = data.meta.funders.as_ref().map_or(0, |values| values.iter().count());
    let data = data.stable_deduplicate();
    assert_eq!(data.meta.keywords, ["chemistry"]);
    assert_eq!(data.meta.funders.as_ref().map_or(0, |values| values.iter().count()), funder_count);
}
#[test]
fn test_vocabulary_fix_revalidates_resolver_evidence_and_fingerprint() {
    let reference = vocabulary_reference("check-0001", VocabularyOutcome::FuzzyAccepted, Some("chemistry"));
    assert!(reference.verified_vocabulary_replacement().is_ok());
    let mut stale = reference;
    stale.message.push('!');
    assert!(stale.verified_vocabulary_replacement().is_err());
}
#[test]
fn test_vocabulary_target_parses_supported_locators() {
    assert_eq!(
        VocabularyTarget::try_from("contact.affiliation"),
        Ok(VocabularyTarget::ContactAffiliation)
    );
    assert_eq!(VocabularyTarget::try_from("meta.funders[2].name"), Ok(VocabularyTarget::Funder(2)));
    assert_eq!(VocabularyTarget::try_from("meta.technology[1]"), Ok(VocabularyTarget::Technology(1)));
    assert!(VocabularyTarget::try_from("meta.unknown[0]").is_err());
}
