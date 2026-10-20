use super::*;
use acorn::io::api::powerautomate::local::ApplyStatus;
use acorn::io::config::ApplicationConfiguration;
use acorn_host::fs::file_checksum;
use acorn_schema::research_activity::{Logbook, LogbookEntry};
use clap_verbosity_flag::Verbosity;
use std::env::temp_dir;
use std::process::Command;

fn arguments(projects: Vec<String>) -> Report {
    let directory = temp_dir().join(format!("acorn-report-command-{}", nanoid::nanoid!()));
    create_dir_all(&directory).expect("report test directory");
    let config = directory.join("acorn.json");
    write(
        &config,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/automation/powerautomate/config.json"
        )),
    )
    .expect("report configuration");
    Report {
        chrome_path: None,
        combine: false,
        commit: Some("0123456789abcdef".to_string()),
        config: Some(config),
        dry_run: true,
        from: None,
        full_timestamp: false,
        job_id: Some("200".to_string()),
        output: directory.join("reports"),
        pipeline_id: Some("100".to_string()),
        projects,
        root: directory,
        through: "2026-10-01T00:00:00Z".to_string(),
        verbose: Verbosity::default(),
    }
}
fn report_fixture_arguments() -> Report {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/report");
    let mut arguments = arguments(Vec::new());
    arguments.config = Some(root.join("config.json"));
    arguments.from = Some("2026-10-01T00:00:00Z".to_string());
    arguments.root = root;
    arguments.through = "2026-11-01T00:00:00Z".to_string();
    arguments
}

#[tokio::test]
async fn test_general_title_page_writes_pdf_when_chrome_is_configured() {
    let Some(chrome_path) = std::env::var_os("ACORN_TEST_CHROME_PATH").map(std::path::PathBuf::from) else {
        return;
    };
    let directory = tempfile::tempdir().expect("scratch directory");
    let mut arguments = report_fixture_arguments();
    arguments.chrome_path = Some(chrome_path);
    arguments.output = directory.path().join("reports");
    let (_, _, _, selected) = prepare(&arguments, true).expect("report fixture configuration");
    arguments.write_title_page(&selected, true).await.expect("title page");
    let pdf = arguments.output.join("title-page.pdf");
    assert!(read(&pdf).expect("title PDF").starts_with(b"%PDF"));
    let extracted = Command::new("pdftotext").arg(&pdf).arg("-").output().expect("extract title text");
    assert!(extracted.status.success());
    let text = String::from_utf8_lossy(&extracted.stdout);
    assert!(text.contains("Project Activity Highlights"));
    assert!(text.contains("General updates"));
    assert!(text.contains("Ordered replacement compute hardware"));
}
#[test]
fn test_global_entries_are_selected_from_project_fixtures() {
    let arguments = report_fixture_arguments();
    let (_, _, _, selected) = prepare(&arguments, true).expect("report fixture configuration");
    let updates = arguments.global_updates(&selected).expect("global updates");
    assert_eq!(updates.entries.len(), 3);
    assert!(updates.entries.iter().all(|entry| entry.global));
    assert_eq!(
        updates.entries.iter().map(|entry| entry.project_id.as_str()).collect::<Vec<_>>(),
        ["watershed-sensing", "materials-discovery", "grid-resilience"]
    );
    selected.iter().for_each(|folder| {
        let path = folder.path.as_deref().expect("project fixture path");
        let (_, project_updates, _) =
            prepare_project(&arguments, "000000000000", &folder.id, path, arguments.from.as_deref()).expect("project updates");
        assert!(project_updates.entries.iter().all(|entry| !entry.global));
    });
}
#[tokio::test]
async fn test_local_jsonl_apply_updates_report_source_and_renders_when_configured() {
    let chrome_path = std::env::var_os("ACORN_TEST_CHROME_PATH").map(std::path::PathBuf::from);
    let directory = tempfile::tempdir().expect("scratch directory");
    let root = directory.path();
    let project = root.join("acorn");
    create_dir_all(&project).expect("project directory");
    let index = project.join("index.json");
    let original = serde_json::to_vec_pretty(&ResearchActivity::default()).expect("baseline activity");
    write(&index, &original).expect("baseline index");
    let inbox = root.join("logbook.jsonl");
    let line = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/automation/powerautomate/logbook.jsonl"
    ))
    .lines()
    .next()
    .expect("fixture submission");
    write(&inbox, format!("{line}\n")).expect("fixture inbox");
    let local_config = root.join("local.json");
    write(
        &local_config,
        serde_json::to_vec(&serde_json::json!({
            "schemaVersion": "2.0.0",
            "folders": [{
                "id": "acorn",
                "members": ["scientist@example.org"],
                "path": format!("file://{}", index.display())
            }]
        }))
        .expect("local configuration"),
    )
    .expect("local configuration file");
    let report_config = root.join("report.json");
    write(
        &report_config,
        serde_json::to_vec(&serde_json::json!({
            "schemaVersion": "2.0.0",
            "folders": [{
                "id": "acorn",
                "members": ["scientist@example.org"],
                "path": "acorn/index.json"
            }],
            "options": {"powerAutomate": {
                "callback_destination": "pilot-flow",
                "gitlab_project": "42",
                "package": "automated-artifacts"
            }}
        }))
        .expect("report configuration"),
    )
    .expect("report configuration file");
    let mut arguments = Report {
        chrome_path,
        combine: false,
        commit: Some("0000000000000000".to_string()),
        config: Some(report_config),
        dry_run: true,
        from: Some("2026-10-01T00:00:00Z".to_string()),
        full_timestamp: false,
        job_id: None,
        output: root.join("reports"),
        pipeline_id: None,
        projects: vec!["acorn".to_string()],
        root: root.to_path_buf(),
        through: "2026-11-01T00:00:00Z".to_string(),
        verbose: Verbosity::default(),
    };
    let (config, _routing, baseline_commit, selected) = prepare(&arguments, true).expect("report configuration accepted");
    assert_eq!(selected.len(), 1);
    let options = gitlab::Options::default()
        .with_identifier("42")
        .with_path("acorn/index.json")
        .with_project_id("acorn");
    let queue = OperationQueue::from(root.join("report-operations.sqlite"));
    let baseline = report_project(&arguments, &config, &options, &queue, &baseline_commit, true)
        .await
        .expect("baseline report");
    assert_eq!(baseline.status, ReportStatus::NoChanges);
    assert!(baseline.entry_identifiers.is_empty());
    let (local_configuration, local_config_path) = ApplicationConfiguration::load_local_intake(&local_config).expect("local intake configuration");
    let staged = local_configuration.stage(&local_config_path, &inbox).expect("stage submission");
    assert_eq!(staged.staged, vec!["11111111-1111-4111-8111-111111111111"]);
    assert!(staged.rejected.is_empty());
    assert_eq!(read(&index).expect("unchanged staged index"), original);
    let submission_id = staged.staged.first().expect("staged submission");
    assert_eq!(
        local_configuration.apply(&local_config_path, submission_id).expect("apply proposal"),
        ApplyStatus::Applied
    );
    assert_eq!(
        local_configuration.apply(&local_config_path, submission_id).expect("repeat apply"),
        ApplyStatus::AlreadyApplied
    );
    let updated = ResearchActivity::read(&index).expect("updated activity");
    let entries = &updated.logbook.expect("updated logbook").entries;
    assert_eq!(entries.len(), 1);
    let entry = entries.first().expect("applied entry");
    assert_eq!(entry.project_id, "acorn");
    assert_eq!(entry.summary, "Ran pilot calibration with \"quoted\" notes");
    let entry_id = entry.identifier.clone();
    let (_, selected_updates, _) =
        prepare_project(&arguments, "1111111111111111", "acorn", "acorn/index.json", arguments.from.as_deref()).expect("updated report selection");
    assert_eq!(selected_updates.entries.len(), 1);
    assert_eq!(selected_updates.entries.first().expect("selected entry").identifier, entry_id);
    let Some(_) = arguments.chrome_path.as_ref() else {
        return;
    };
    arguments.commit = Some("1111111111111111".to_string());
    let (_, _, updated_commit, _) = prepare(&arguments, true).expect("updated report configuration");
    let manifest = report_project(&arguments, &config, &options, &queue, &updated_commit, true)
        .await
        .expect("updated report");
    assert_eq!(manifest.status, ReportStatus::Published);
    assert_eq!(manifest.entry_identifiers, vec![entry_id]);
    assert_eq!(manifest.project_id, "acorn");
    assert_eq!(manifest.source_commit, updated_commit);
    let report_directory = arguments.output.join("acorn/20261101T000000Z-111111111111");
    let pdf_path = report_directory.join("report.pdf");
    let pdf = read(&pdf_path).expect("report PDF");
    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(
        manifest.checksum.as_deref(),
        Some(file_checksum(&pdf_path, None).expect("PDF checksum").checksum_value.as_str())
    );
    let stored: ReportManifest =
        serde_json::from_slice(&read(report_directory.join("manifest.json")).expect("stored manifest")).expect("parse stored manifest");
    assert_eq!(stored, manifest);
    let extracted = Command::new("pdftotext").arg(&pdf_path).arg("-").output().expect("extract PDF text");
    assert!(extracted.status.success());
    assert!(String::from_utf8_lossy(&extracted.stdout).contains("Ran pilot calibration with \"quoted\" notes"));
}
#[test]
fn test_prepare_accepts_offline_dry_run_and_project_selection() {
    let prepared = prepare(&arguments(vec!["pilot".to_string()]), true).expect("prepared report");
    assert_eq!(prepared.0.gitlab_project, "42");
    assert_eq!(prepared.2, "0123456789abcdef");
    assert_eq!(prepared.3.len(), 1);
    assert_eq!(prepared.3.first().expect("selected folder").id, "pilot");
}
#[test]
fn test_prepare_rejects_missing_context_and_unknown_selection() {
    let mut missing_commit = arguments(Vec::new());
    missing_commit.commit = None;
    assert!(prepare(&missing_commit, true).is_err());
    let publish_offline = Report {
        dry_run: false,
        ..arguments(Vec::new())
    };
    assert!(prepare(&publish_offline, true).is_err());
    let publish_without_job = Report {
        dry_run: false,
        job_id: None,
        pipeline_id: None,
        ..arguments(Vec::new())
    };
    assert!(prepare(&publish_without_job, false).is_err());
    assert!(prepare(&arguments(vec!["unknown".to_string()]), true).is_err());
}
#[test]
fn test_replay_manifest_requeues_notification_without_publication() {
    let mut arguments = arguments(Vec::new());
    arguments.dry_run = false;
    let config = prepare(
        &Report {
            dry_run: true,
            ..arguments.clone()
        },
        false,
    )
    .expect("prepared configuration")
    .0;
    let window = ReportingWindow::new(Some("2026-09-01T00:00:00Z"), "2026-10-01T00:00:00Z").expect("reporting window");
    let manifest = ReportManifest::new(
        "automated-artifacts",
        "pilot",
        "0123456789abcdef",
        &window,
        Vec::new(),
        "100",
        "200",
        None,
        "https://gitlab.example/package/manifest.json",
    )
    .expect("report manifest");
    let version = manifest.version.clone();
    let queue = OperationQueue::from(temp_dir().join(format!("acorn-report-replay-{}.db", nanoid::nanoid!())));
    let replayed = replay_manifest(&arguments, &config, &queue, manifest).expect("manifest replay");
    assert_eq!(replayed.version, version);
    assert_eq!(
        queue
            .callback_state(&format!("webhook:report-no-changes:{version}"))
            .expect("callback state"),
        Some(acorn::io::api::webhooks::store::OperationState::Queued)
    );
}
#[tokio::test]
async fn test_report_project_no_change_writes_only_manifest() {
    let arguments = arguments(Vec::new());
    let project_directory = arguments.root.join("pilot");
    create_dir_all(&project_directory).expect("project directory");
    write(
        project_directory.join("index.json"),
        serde_json::to_vec_pretty(&ResearchActivity::default()).expect("activity fixture"),
    )
    .expect("activity file");
    let (config, _routing, commit, _) = prepare(&arguments, true).expect("prepared report");
    let options = gitlab::Options::default()
        .with_identifier("42")
        .with_path("pilot/index.json")
        .with_project_id("pilot");
    let queue = OperationQueue::from(temp_dir().join(format!("acorn-report-no-change-{}.db", nanoid::nanoid!())));
    let manifest = report_project(&arguments, &config, &options, &queue, &commit, true)
        .await
        .expect("no-change report");
    assert_eq!(manifest.status, ReportStatus::NoChanges);
    assert!(manifest.entry_identifiers.is_empty());
    let output = arguments.output.join("pilot/20261001T000000Z-0123456789ab");
    assert!(output.join("manifest.json").is_file());
    assert!(!output.join("report.pdf").exists());
}
#[tokio::test]
async fn test_report_project_with_entries_writes_pdf_when_chrome_is_configured() {
    let mut arguments = arguments(Vec::new());
    arguments.chrome_path = std::env::var_os("ACORN_TEST_CHROME_PATH").map(std::path::PathBuf::from);
    if arguments.chrome_path.is_some() {
        let project_directory = arguments.root.join("pilot");
        create_dir_all(&project_directory).expect("project directory");
        let activity = ResearchActivity {
            logbook: Some(
                Logbook::init()
                    .entries(vec![LogbookEntry::init()
                        .category("software")
                        .identifier("release")
                        .project_id("pilot")
                        .state("completed")
                        .summary("Released the pilot")
                        .timestamp("2026-09-18T12:00:00Z")
                        .build()])
                    .build(),
            ),
            ..ResearchActivity::default()
        };
        write(
            project_directory.join("index.json"),
            serde_json::to_vec_pretty(&activity).expect("activity fixture"),
        )
        .expect("activity file");
        let (config, _routing, commit, _) = prepare(&arguments, true).expect("prepared report");
        let options = gitlab::Options::default()
            .with_identifier("42")
            .with_path("pilot/index.json")
            .with_project_id("pilot");
        let queue = OperationQueue::from(temp_dir().join(format!("acorn-report-pdf-{}.db", nanoid::nanoid!())));
        let manifest = report_project(&arguments, &config, &options, &queue, &commit, true)
            .await
            .expect("published report");
        assert_eq!(manifest.status, ReportStatus::Published);
        let output = arguments.output.join("pilot/20261001T000000Z-0123456789ab/report.pdf");
        let pdf = read(output).expect("report PDF");
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 1_000);
    }
}
