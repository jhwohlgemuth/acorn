use super::super::chat::{execution_root, path_hint, source_matches, validate_interactive, ChatDriver, NoRecipeRunner};
use crate::cli::{Arguments, Commands};
use acorn::agent::{
    ChatEvent, ChatRole, InMemoryBackend, InferenceBackend, InferenceCapabilities, InferenceFinishReason, InferenceFuture, InferencePermissionPolicy,
    InferenceProvenance, InferenceRequest, InferenceResult,
};
use acorn::analyzer::{
    fix::{
        CheckReference, FindingReference, Fix, FixAction, FixAutomation, FixEffects, FixPreviewTarget, FixProvenance, FixReport, FixRisk, FixStatus,
        Outcome, Plan, Policy,
    },
    host::{context::AnalysisContext, service::analyze_paths},
};
use acorn::util::StringConversion;
use acorn_core::util::MimeType;
use acorn_core::Noul;
use acorn_core::{
    time::{Milliseconds, UnitOfTime},
    Location,
};
use alloc::sync::Arc;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

struct FailureBackend;
struct PendingBackend;
struct RecordingBackend {
    requests: Arc<Mutex<Vec<InferenceRequest>>>,
}
impl InferenceBackend for FailureBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }

    fn infer<'a>(&'a self, _request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(async { Err(color_eyre::eyre::eyre!("transport unavailable")) })
    }
}
impl InferenceBackend for PendingBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }

    fn infer<'a>(&'a self, _request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(core::future::pending())
    }
}
impl InferenceBackend for RecordingBackend {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities::default()
    }

    fn infer<'a>(&'a self, request: &'a InferenceRequest) -> InferenceFuture<'a> {
        self.requests.lock().expect("request lock").push(request.clone());
        Box::pin(async { Ok(result(InferenceFinishReason::Stop, "The CFF file has one finding.")) })
    }
}
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}
fn proposed_document_fix(target: &str) -> Fix {
    let reference = CheckReference {
        category: "metadata".to_string(),
        context: None,
        fingerprint: "a".repeat(64),
        id: "check-1".to_string(),
        locator: None,
        message: "invalid value".to_string(),
        rule_id: None,
        severity: "error".to_string(),
        standard: "research-activity-data".to_string(),
        uri: Some("activity.json".to_string()),
        vocabulary_advisory: None,
    };
    Fix::init()
        .actions(vec![FixAction::DocumentChange {
            description: "replace value".to_string(),
            expected: "old".to_string(),
            replacement: "new".to_string(),
        }])
        .automation(FixAutomation::ConfirmationRequired)
        .confidence(Noul::ONE)
        .effects(FixEffects {
            filesystem_write: true,
            reversible: true,
            ..FixEffects::default()
        })
        .finding(FindingReference::Check(reference))
        .id("fix-1")
        .provenance(FixProvenance {
            agent: None,
            backend: "test".to_string(),
            model: None,
            prompt_version: "1".to_string(),
            request_id: None,
        })
        .rationale("replace the invalid value")
        .risk(FixRisk::Low)
        .status(FixStatus::Proposed)
        .summary("Replace value")
        .target(target)
        .build()
}
fn result(reason: InferenceFinishReason, text: &str) -> InferenceResult {
    InferenceResult {
        finish_reason: reason,
        provenance: InferenceProvenance {
            backend: "memory".to_string(),
            agent: None,
            model: None,
            request_id: None,
        },
        structured: None,
        text: text.to_string(),
        usage: None,
    }
}

#[tokio::test]
async fn test_absolute_confirmed_metadata_path_can_be_attached() {
    let root = fixtures().canonicalize().expect("canonical fixture root");
    let source = root.join("CITATION.cff");
    let backend = InMemoryBackend::new(result(InferenceFinishReason::Stop, "Attached"));
    let mut driver = ChatDriver::new(&backend, root, true, None, None, Milliseconds::new(3000));
    driver
        .attach(Location::from(source.to_string_lossy().as_ref()), None)
        .await
        .expect("attach absolute path");
    assert!(driver.state().target().is_some());
}
#[tokio::test]
async fn test_cancellation_does_not_record_an_assistant_turn() {
    let backend = InMemoryBackend::new(result(InferenceFinishReason::Stop, "Late reply"));
    let mut driver = ChatDriver::new(&backend, fixtures(), true, None, None, Milliseconds::new(3000));
    assert!(driver.reply_with_cancel("Check this", async {}).await.is_err());
    assert!(matches!(driver.state().events().last(), Some(ChatEvent::Cancelled)));
    assert!(!driver
        .state()
        .events()
        .iter()
        .any(|event| matches!(event, ChatEvent::Turn(turn) if turn.role == ChatRole::Assistant)));
}
#[test]
fn test_fix_execution_root_matches_only_the_reviewed_path() {
    let path = PathBuf::from("/work/data/CITATION.cff");
    assert_eq!(execution_root(&path, "data/CITATION.cff").expect("matching root"), PathBuf::from("/work"));
    assert!(execution_root(&path, "other/CITATION.cff").is_err());
    assert!(execution_root(&path, "../CITATION.cff").is_err());
}
#[test]
fn test_interactive_rejects_explicit_needle_flags_and_offline_acp() {
    for flag in ["--max-rounds", "--threshold", "--custom-location"] {
        let value = if flag == "--custom-location" {
            "file:///model"
        } else if flag == "--threshold" {
            "0.5"
        } else {
            "3"
        };
        let parsed = Arguments::try_parse_from(["acorn", "ask", "--interactive", flag, value]).expect("parse interactive flag");
        let arguments = parsed
            .command
            .and_then(|command| match command {
                | Commands::Infer(arguments) => Some(arguments),
                | _ => None,
            })
            .expect("expected infer arguments");
        assert!(validate_interactive(&arguments, false).is_err(), "{flag}");
    }
    let parsed = Arguments::try_parse_from(["acorn", "ask", "--interactive"]).expect("parse interactive");
    let arguments = parsed
        .command
        .and_then(|command| match command {
            | Commands::Infer(arguments) => Some(arguments),
            | _ => None,
        })
        .expect("expected infer arguments");
    assert!(validate_interactive(&arguments, true).is_err());
    assert!(validate_interactive(&arguments, false).is_ok());
}
#[tokio::test]
async fn test_interactive_requires_terminal_before_creating_history() {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        let parsed = Arguments::try_parse_from(["acorn", "ask", "--interactive"]).expect("parse interactive ask");
        let arguments = parsed
            .command
            .and_then(|command| match command {
                | Commands::Infer(arguments) => Some(arguments),
                | _ => None,
            })
            .expect("infer arguments");
        let error = super::super::run(&arguments, false).await.expect_err("TTY required");
        assert!(error.to_string().contains("TTY"));
    }
}
#[tokio::test]
async fn test_multi_turn_reply_is_grounded_in_typed_findings_and_denies_mutation() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let backend = RecordingBackend { requests: requests.clone() };
    let mut driver = ChatDriver::new(&backend, fixtures(), true, None, None, Milliseconds::new(3000));
    driver.attach(Location::from("CITATION.cff"), None).await.expect("attach");
    driver.reply("What did the check find?").await.expect("first reply");
    driver.reply("Explain that again.").await.expect("second reply");
    let requests = requests.lock().expect("request lock");
    assert_eq!(requests.len(), 2);
    assert!(requests.first().expect("first request").prompt.body.contains("citation-file-format"));
    assert!(requests.get(1).expect("second request").prompt.body.contains("What did the check find?"));
    assert_eq!(
        requests.first().expect("first request").permission_policy,
        InferencePermissionPolicy::DenyMutation
    );
    assert!(driver
        .state()
        .events()
        .iter()
        .any(|event| matches!(event, ChatEvent::Turn(turn) if turn.role == ChatRole::Tool)));
}
#[test]
fn test_natural_language_path_hint_requires_a_real_local_path() {
    let root = fixtures();
    assert_eq!(path_hint(&root, "Please check CITATION.cff"), Some("CITATION.cff".to_string()));
    assert_eq!(path_hint(&root, "Please check invented.json"), None);
}
#[tokio::test]
async fn test_refusal_cancel_and_missing_target_are_explicit() {
    let backend = InMemoryBackend::new(result(InferenceFinishReason::Refusal, "No"));
    let mut driver = ChatDriver::new(&backend, fixtures(), true, None, None, Milliseconds::new(3000));
    assert!(driver.reply("Help me").await.is_err());
    assert!(matches!(driver.state().events().last(), Some(ChatEvent::Failed(_))));
    assert!(driver.attach(Location::from("missing.json"), None).await.is_err());
    assert!(driver.state().target().is_none());
    driver.cancel();
    assert!(matches!(driver.state().events().last(), Some(ChatEvent::Cancelled)));
    driver.clear();
    assert!(driver.state().events().is_empty());
}
#[test]
fn test_review_gate_excludes_recipe_and_manual_actions() {
    let mut fix = proposed_document_fix("activity.json");
    assert!(fix.is_reviewable_document_change());
    fix.actions = vec![FixAction::RegisteredRecipe {
        recipe_id: "run".to_string(),
    }];
    assert!(!fix.is_reviewable_document_change());
    fix.actions = vec![FixAction::Manual {
        instructions: "edit by hand".to_string(),
    }];
    assert!(!fix.is_reviewable_document_change());
}
#[test]
fn test_review_rejects_invalid_replacement_and_stale_content() {
    let directory = tempfile::tempdir().expect("temporary fix root");
    let root = directory.path().canonicalize().expect("canonical root");
    let path = root.join("activity.json");
    let original = include_str!("../../../../../../tests/fixtures/data/valid_project/index.json");
    fs::write(&path, original).expect("write source");
    let target = FixPreviewTarget {
        fix_ids: vec!["fix-1".to_string()],
        original: original.to_string(),
        proposed: "{".to_string(),
        source_format: MimeType::Json,
        standard: acorn_schema::standard::Standard::ResearchActivityData,
        target: "activity.json".to_string(),
    };
    assert!(target.reviewed_diff().is_err());
    assert!(source_matches(&path, &root, original));
    fs::write(&path, "changed").expect("change source");
    assert!(!source_matches(&path, &root, original));
}
#[tokio::test]
async fn test_reviewed_document_plan_applies_and_rolls_back() {
    let directory = tempfile::tempdir().expect("temporary fix root");
    let root = directory.path().canonicalize().expect("canonical root");
    let path = root.join("CITATION.cff");
    let original = include_str!("../../../../../../tests/fixtures/CITATION.cff");
    fs::write(&path, original).expect("source fixture");
    let mut options = AnalysisContext::default();
    options.analysis.standard = acorn_schema::standard::Standard::CitationFileFormat;
    options.execution.common.offline = true;
    options.execution.extension.disable_website_checks = true;
    options.execution.extension.skip_sync = true;
    let report = analyze_paths(core::slice::from_ref(&path), &options).await;
    let mut fix = proposed_document_fix(&path.file_name_with_parent());
    fix.actions = vec![FixAction::DocumentChange {
        description: "update title".to_string(),
        expected: "title: ACORN".to_string(),
        replacement: "title: ACORN TEST".to_string(),
    }];
    let preview = FixReport::from(vec![fix.clone()]).build_preview(&report, false);
    assert_eq!(preview.targets.len(), 1, "preview diagnostics: {:?}", preview.diagnostics);
    let target = preview.targets.first().expect("one preview target");
    assert!(target.reviewed_diff().expect("valid diff").contains("ACORN TEST"));
    assert!(source_matches(&path, &root, original));
    let plan = Plan::from_fixes(core::slice::from_ref(&fix), Policy::confirmed(true)).expect("confirmed plan");
    let execution_root = execution_root(&path, &target.target).expect("execution root");
    let applied = plan
        .execute(&execution_root, &NoRecipeRunner, || {
            fs::read_to_string(&path).is_ok_and(|content| content == target.proposed)
        })
        .expect("apply plan");
    assert!(matches!(applied.as_slice(), [Outcome::Applied(_)]));
    fs::write(&path, original).expect("reset source");
    let rolled_back = plan.execute(&execution_root, &NoRecipeRunner, || false).expect("rollback plan");
    assert!(matches!(rolled_back.as_slice(), [Outcome::RolledBack(_)]));
    assert_eq!(fs::read_to_string(&path).expect("restored source"), original);
    #[cfg(unix)]
    {
        let permissions = fs::metadata(&root).expect("directory metadata").permissions();
        let mut readonly = permissions.clone();
        readonly.set_readonly(true);
        fs::set_permissions(&root, readonly).expect("make destination readonly");
        let probe = root.join("write-probe");
        if fs::write(&probe, "probe").is_err() {
            assert!(plan.execute(&execution_root, &NoRecipeRunner, || true).is_err(), "write must fail");
        } else {
            fs::remove_file(&probe).expect("remove write probe");
        }
        fs::set_permissions(&root, permissions).expect("restore destination permissions");
        assert_eq!(fs::read_to_string(&path).expect("source after failed write"), original);
    }
}
#[tokio::test]
async fn test_transport_failure_and_timeout_keep_the_session_usable() {
    let failure = FailureBackend;
    let mut failed = ChatDriver::new(&failure, fixtures(), true, None, None, Milliseconds::new(3000));
    assert!(failed.reply("What is this?").await.is_err());
    assert!(matches!(failed.state().events().last(), Some(ChatEvent::Failed(_))));
    let pending = PendingBackend;
    let mut timed_out = ChatDriver::new(&pending, fixtures(), true, None, None, Milliseconds::new(1));
    assert!(timed_out.reply("What is this?").await.is_err());
    assert!(matches!(timed_out.state().events().last(), Some(ChatEvent::Failed(message)) if message.contains("timed out")));
}
