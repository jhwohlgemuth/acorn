#![allow(clippy::panic)] // Unexpected review callbacks must fail the test, even on expected-error paths.
use super::*;
use acorn_schema::research_activity::{GraduationCandidate, GraduationTarget, Logbook, LogbookEntry};
use std::fs;

fn fixture(directory: &Path, extension: &str) -> PathBuf {
    let path = directory.join(format!("index.{extension}"));
    let activity = ResearchActivity {
        logbook: Some(
            Logbook::init()
                .entries(vec![
                    LogbookEntry::init()
                        .identifier("person-one")
                        .project_id("acorn")
                        .timestamp("2026-10-24T12:00:00Z")
                        .category("personnel")
                        .state("in-progress")
                        .summary("Example, Avery")
                        .build(),
                    LogbookEntry::init()
                        .identifier("person-two")
                        .project_id("acorn")
                        .timestamp("2026-10-24T13:00:00Z")
                        .category("personnel")
                        .state("in-progress")
                        .summary("Example, Morgan")
                        .build(),
                ])
                .build(),
        ),
        ..ResearchActivity::default()
    };
    fs::write(&path, activity.encode(&format(&path).unwrap()).unwrap()).unwrap();
    path
}
fn preview(index: &Path) -> Result<Value> {
    execute(
        index,
        &IntakeRunArguments {
            dry_run: true,
            ..Default::default()
        },
        ask,
    )
}
#[test]
fn test_ambiguous_selection_is_rejected() {
    let candidate = GraduationCandidate {
        entry_identifier: "entry".into(),
        target: GraduationTarget::Contributors,
        value: "Example, Avery".into(),
    };
    assert!(validate_selection(&GraduationCandidates(vec![candidate.clone(), candidate]), &["entry".into()]).is_err());
}
#[test]
fn test_backup_failure_preserves_source() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let before = fs::read_to_string(&path).unwrap();
    let backup = format!(
        "{}.graduation-{}.before",
        path.display(),
        digest_text(&before).trim_start_matches("sha256:")
    );
    fs::create_dir(backup).unwrap();
    assert!(apply(&path, &["person-one".into()], &digest_text(&before)).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), before);
}
#[test]
fn test_candidates_and_selected_graduation_preserve_evidence() {
    for extension in ["json", "jsonc", "yaml", "zonf"] {
        let directory = tempfile::tempdir().unwrap();
        let path = fixture(directory.path(), extension);
        let before = fs::read_to_string(&path).unwrap();
        let review = preview(&path).unwrap();
        assert_eq!(review.get("candidates").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        let receipt = apply(&path, &["person-one".into()], review.get("baseDigest").unwrap().as_str().unwrap()).unwrap();
        assert_eq!(fs::read_to_string(receipt.get("backup").unwrap().as_str().unwrap()).unwrap(), before);
        let after = fs::read_to_string(&path).unwrap();
        let value = decode(&after, &format(&path).unwrap()).unwrap();
        let original = decode(&before, &format(&path).unwrap()).unwrap();
        assert_eq!(value.meta.contributors.as_ref().unwrap().len(), 1);
        assert_eq!(value.meta.contributors.as_ref().unwrap().first().unwrap().name, "Example, Avery");
        assert_eq!(value.title, original.title);
        let entries = &value.logbook.as_ref().unwrap().entries;
        assert_eq!(entries.len(), 2);
        assert!(entries.first().unwrap().archived);
        assert!(!entries.get(1).unwrap().archived);
        assert_eq!(preview(&path).unwrap().get("candidates").unwrap().as_array().unwrap().len(), 1);
        assert!(apply(&path, &["person-one".into()], &digest_text(&before)).is_err());
        assert!(apply(&path, &["person-one".into()], &digest_text(&after)).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), after);
    }
}
#[test]
fn test_invalid_and_unknown_selections_never_write() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let before = fs::read_to_string(&path).unwrap();
    for accepted in [vec![], vec!["unknown".into()], vec!["person-one".into(), "unknown".into()]] {
        assert!(apply(&path, &accepted, &digest_text(&before)).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
    }
    fs::write(&path, "{ invalid }").unwrap();
    assert!(preview(&path).is_err());
    assert!(apply(&path, &["person-one".into()], &digest_text("{ invalid }")).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "{ invalid }");
}
#[test]
fn test_no_candidates_is_read_only() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("index.json");
    let content = ResearchActivity::default().encode(&MimeType::Json).unwrap();
    fs::write(&path, &content).unwrap();
    assert!(preview(&path).unwrap().get("candidates").unwrap().as_array().unwrap().is_empty());
    assert_eq!(fs::read_to_string(path).unwrap(), content);
}
#[test]
fn test_run_dry_run_wins_over_force_in_every_format() {
    for extension in ["json", "jsonc", "yaml", "yml", "zonf"] {
        let directory = tempfile::tempdir().unwrap();
        let path = fixture(directory.path(), extension);
        let before = fs::read(&path).unwrap();
        let options = IntakeRunArguments {
            dry_run: true,
            force: true,
            ..Default::default()
        };
        let plan = execute(&path, &options, |_| panic!("dry-run must not prompt without --interactive")).unwrap();
        assert_eq!(plan.get("status").unwrap(), "dry-run");
        assert_eq!(plan.get("accepted").unwrap().as_array().unwrap().len(), 2);
        assert_ne!(plan.get("baseDigest"), plan.get("proposedDigest"));
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
#[test]
fn test_run_explicit_selection_requires_digest_and_skips_prompts() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let before = fs::read_to_string(&path).unwrap();
    let options = IntakeRunArguments {
        accept: vec!["person-two".into()],
        ..Default::default()
    };
    assert!(execute(&path, &options, |_| panic!("explicit selection must not prompt")).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    let options = IntakeRunArguments {
        expected_digest: Some(digest_text(&before)),
        ..options
    };
    let receipt = execute(&path, &options, |_| panic!("explicit selection must not prompt")).unwrap();
    assert_eq!(receipt.get("accepted").unwrap(), &json!(["person-two"]));
    let after = decode(&fs::read_to_string(&path).unwrap(), &MimeType::Json).unwrap();
    assert!(!after.logbook.as_ref().unwrap().entries.first().unwrap().archived);
    assert!(after.logbook.as_ref().unwrap().entries.get(1).unwrap().archived);
}
#[test]
fn test_run_force_accepts_all_eligible_entries_and_replay_is_unchanged() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let mut activity = decode(&fs::read_to_string(&path).unwrap(), &MimeType::Json).unwrap();
    activity.logbook.as_mut().unwrap().entries.push(
        LogbookEntry::init()
            .identifier("research-only")
            .project_id("acorn")
            .timestamp("2026-10-24T14:00:00Z")
            .category("research")
            .state("in-progress")
            .summary("A highlight with no canonical promotion")
            .build(),
    );
    fs::write(&path, activity.encode(&MimeType::Json).unwrap()).unwrap();
    let before = fs::read_to_string(&path).unwrap();
    let options = IntakeRunArguments {
        force: true,
        ..Default::default()
    };
    let receipt = execute(&path, &options, |_| panic!("--force must not prompt")).unwrap();
    assert_eq!(receipt.get("accepted").unwrap(), &json!(["person-one", "person-two"]));
    assert_eq!(fs::read_to_string(receipt.get("backup").unwrap().as_str().unwrap()).unwrap(), before);
    let after = fs::read_to_string(&path).unwrap();
    let activity = decode(&after, &MimeType::Json).unwrap();
    assert_eq!(activity.meta.contributors.as_ref().unwrap().len(), 2);
    assert!(!activity.logbook.unwrap().entries.get(2).unwrap().archived);
    let replay = execute(&path, &options, |_| panic!("--force must not prompt")).unwrap();
    assert_eq!(replay.get("status").unwrap(), "unchanged");
    assert_eq!(fs::read_to_string(&path).unwrap(), after);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}
#[test]
fn test_run_force_preserves_validation_digest_and_backup_guards() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let before = fs::read_to_string(&path).unwrap();
    let options = IntakeRunArguments {
        force: true,
        expected_digest: Some("sha256:stale".into()),
        ..Default::default()
    };
    assert!(execute(&path, &options, |_| panic!("--force must not prompt")).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    let options = IntakeRunArguments {
        expected_digest: None,
        ..options
    };
    let backup = format!(
        "{}.graduation-{}.before",
        path.display(),
        digest_text(&before).trim_start_matches("sha256:")
    );
    fs::create_dir(&backup).unwrap();
    assert!(execute(&path, &options, |_| panic!("--force must not prompt")).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    fs::remove_dir(backup).unwrap();
    fs::write(&path, "{ invalid }").unwrap();
    assert!(execute(&path, &options, |_| panic!("--force must not prompt")).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "{ invalid }");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}
#[test]
fn test_run_interactive_cancellation_never_writes() {
    for answers in [
        vec![PromptOutcome::Cancelled],
        vec![PromptOutcome::Selected(true), PromptOutcome::Cancelled],
        vec![
            PromptOutcome::Selected(true),
            PromptOutcome::Selected(true),
            PromptOutcome::Selected(false),
        ],
        vec![PromptOutcome::Selected(true), PromptOutcome::Selected(true), PromptOutcome::Cancelled],
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = fixture(directory.path(), "json");
        let before = fs::read(&path).unwrap();
        let options = IntakeRunArguments {
            interactive: true,
            ..Default::default()
        };
        let mut answers = answers.into_iter();
        assert!(execute(&path, &options, |_| Ok(answers.next().unwrap())).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
#[test]
fn test_run_interactive_dry_run_and_declining_all_never_write() {
    for dry_run in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = fixture(directory.path(), "json");
        let before = fs::read(&path).unwrap();
        let options = IntakeRunArguments {
            dry_run,
            interactive: true,
            ..Default::default()
        };
        let mut answers = [PromptOutcome::Selected(dry_run), PromptOutcome::Selected(false)].into_iter();
        let result = execute(&path, &options, |_| Ok(answers.next().unwrap())).unwrap();
        assert!(answers.next().is_none());
        assert_eq!(result.get("status").unwrap(), if dry_run { "dry-run" } else { "unchanged" });
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
#[test]
fn test_run_interactive_escapes_control_characters_in_values() {
    let candidates = GraduationCandidates(vec![GraduationCandidate {
        entry_identifier: "entry".into(),
        target: GraduationTarget::Contributors,
        value: "Example\n\u{1b}[2J".into(),
    }]);
    let options = IntakeRunArguments {
        interactive: true,
        ..Default::default()
    };
    let selected = select(&candidates, &options, &mut |prompt| {
        assert!(!prompt.contains(['\n', '\u{1b}']));
        assert!(prompt.contains("\\n\\u{1b}[2J"));
        Ok(PromptOutcome::Selected(true))
    })
    .unwrap();
    assert_eq!(selected, ["entry"]);
}
#[test]
fn test_run_interactive_rechecks_index_after_confirmation() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let before = fs::read_to_string(&path).unwrap();
    let mut changed = decode(&before, &MimeType::Json).unwrap();
    changed.title = "Concurrent edit".into();
    let changed = changed.encode(&MimeType::Json).unwrap();
    let options = IntakeRunArguments {
        interactive: true,
        ..Default::default()
    };
    let error = execute(&path, &options, |prompt| {
        if prompt.starts_with("Apply ") {
            fs::write(&path, &changed).unwrap();
        }
        Ok(PromptOutcome::Selected(true))
    })
    .unwrap_err();
    assert!(error.to_string().contains("Index changed since review"));
    assert_eq!(fs::read_to_string(&path).unwrap(), changed);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}
#[test]
fn test_run_interactive_selects_each_candidate_and_confirms_once() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let options = IntakeRunArguments {
        interactive: true,
        ..Default::default()
    };
    let mut answers = [false, true, true].into_iter();
    let mut prompts = Vec::new();
    let receipt = execute(&path, &options, |prompt| {
        prompts.push(prompt.to_string());
        Ok(PromptOutcome::Selected(answers.next().unwrap()))
    })
    .unwrap();
    assert_eq!(receipt.get("accepted").unwrap(), &json!(["person-two"]));
    assert!(prompts.first().unwrap().contains("person-one -> Contributors: \"Example, Avery\""));
    assert!(prompts.get(1).unwrap().contains("person-two -> Contributors: \"Example, Morgan\""));
    assert!(prompts.last().unwrap().contains("Apply 1 selected promotion"));
    assert!(answers.next().is_none());
}
#[test]
fn test_run_without_approval_preserves_the_index() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture(directory.path(), "json");
    let before = fs::read(&path).unwrap();
    let error = execute(&path, &IntakeRunArguments::default(), |_| panic!("interactive mode must be explicit")).unwrap_err();
    assert!(error.to_string().contains("Graduation requires approval"));
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}
