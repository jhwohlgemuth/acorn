use super::*;
use acorn_schema::research_activity::{Logbook, LogbookEntry};
use clap_verbosity_flag::Verbosity;
use std::env::temp_dir;

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
        commit: Some("0123456789abcdef".to_string()),
        config: Some(config),
        dry_run: true,
        from: None,
        job_id: Some("200".to_string()),
        output: directory.join("reports"),
        pipeline_id: Some("100".to_string()),
        projects,
        root: directory,
        through: "2026-10-01T00:00:00Z".to_string(),
        verbose: Verbosity::default(),
    }
}

#[test]
fn test_prepare_accepts_offline_dry_run_and_project_selection() {
    let prepared = prepare(&arguments(vec!["pilot".to_string()]), true).expect("prepared report");
    assert_eq!(prepared.0.gitlab_project, "42");
    assert_eq!(prepared.1, "0123456789abcdef");
    assert_eq!(prepared.2.len(), 1);
    assert_eq!(prepared.2.first().expect("selected project").project_id, "pilot");
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
    let (config, commit, _) = prepare(&arguments, true).expect("prepared report");
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
        let (config, commit, _) = prepare(&arguments, true).expect("prepared report");
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
