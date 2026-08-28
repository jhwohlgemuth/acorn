#![allow(clippy::indexing_slicing)]
use super::*;
use acorn::io::InputOutput;
use acorn::prelude::PathBuf;
use acorn::schema::research_activity::ResearchActivity;
use acorn::Location;
use assert_cmd::Command;

#[test]
fn test_explicit_empty_stdin_fails() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args(["--offline", "--no-local-database", "gather", "-"])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No content received from standard input"));
}
#[test]
fn test_filter_and_ignore_patterns_compose() {
    let values = vec!["keep.md".to_string(), "skip.md".to_string(), "skip.txt".to_string()];
    let filters = FilterSet::compile(&["skip".to_string()], &[r"[.]md$".to_string()]).unwrap();
    assert_eq!(expand(&values, &filters, None), vec!["keep.md".to_string()]);
}
#[tokio::test]
async fn test_load_accepts_location() {
    let document = SourceDocument::load(Location::from("10.1234/example"), true).await.unwrap();
    assert_eq!(document.format, "pid");
}
#[test]
fn test_cli_citation_format_takes_precedence_over_environment() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .env("ACORN_CITATION_FORMAT", "invalid")
        .args([
            "-v",
            "--no-local-database",
            "gather",
            "--raw",
            "--resolve",
            "--citation-format",
            "apa",
            "--text",
            "plain-text-without-identifiers",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("('invalid' is unsupported format)"));
}
#[test]
fn test_invalid_citation_format_warns_and_uses_ieee() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .env("ACORN_CITATION_FORMAT", "invalid")
        .args([
            "-v",
            "--no-local-database",
            "gather",
            "--raw",
            "--resolve",
            "--text",
            "plain-text-without-identifiers",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("IEEE"));
    assert!(stderr.contains("('invalid' is unsupported format)"));
}
#[test]
fn test_max_depth_limits_directory_expansion() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/gather/depth");
    let input = vec![root.display().to_string()];
    let filters = FilterSet::compile(&[], &[]).unwrap();
    assert!(expand(&input, &filters, Some(0)).is_empty());
    assert_eq!(expand(&input, &filters, Some(1)).len(), 1);
    assert_eq!(expand(&input, &filters, Some(2)).len(), 2);
    assert_eq!(expand(&input, &filters, None).len(), 3);
}
#[test]
fn test_piped_stdin_does_not_require_dash() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args(["--offline", "--no-local-database", "gather", "--format", "json"])
        .write_stdin("doi:10.1234/example")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(report.contains(r#""identifier": "10.1234/example""#));
    assert!(report.contains(r#""source": "<stdin>""#));
}
#[test]
fn test_console_format_renders_current_gather_results() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "--offline",
            "--no-local-database",
            "gather",
            "--text",
            "doi:10.1234/example",
            "--format",
            "console",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(report.contains("ACORN gather:"));
    assert!(report.contains("10.1234/example"));
}
#[test]
fn test_raw_format_prints_only_discovered_identifiers() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "--offline",
            "--no-local-database",
            "gather",
            "--text",
            "doi:10.1234/example https://ror.org/01qz5mb56 doi:10.1234/example",
            "--raw",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "10.1234/example\nhttps://ror.org/01qz5mb56\n");
}
#[test]
fn test_raw_format_normalizes_arxiv_urls() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "--offline",
            "--no-local-database",
            "gather",
            "--text",
            "https://arxiv.org/pdf/2106.09685v2.pdf",
            "--raw",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "arXiv:2106.09685v2\n");
}
#[test]
fn test_raw_format_silently_returns_nonzero_for_load_failures() {
    let path = acorn::prelude::temp_dir().join(format!("acorn-invalid-{}.pdf", nanoid::nanoid!()));
    write_file(path.clone(), "not a PDF".to_string()).unwrap();
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args(["--offline", "--no-local-database", "gather", "--raw", path.to_str().unwrap()])
        .output()
        .unwrap();
    let _ = remove_file(path);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
#[test]
fn test_verbose_raw_format_writes_load_failures_to_stderr() {
    let path = acorn::prelude::temp_dir().join(format!("acorn-invalid-{}.pdf", nanoid::nanoid!()));
    write_file(path.clone(), "not a PDF".to_string()).unwrap();
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args(["-v", "--offline", "--no-local-database", "gather", "--raw", path.to_str().unwrap()])
        .output()
        .unwrap();
    let _ = remove_file(path);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid PDF"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("ACORN gather:"));
}
#[test]
fn test_terse_format_suppresses_discovery_table() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "-vv",
            "--offline",
            "--no-local-database",
            "gather",
            "--terse",
            "--text",
            "doi:10.1234/example",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("found"));
    assert!(!stdout.contains("ACORN gather:"));
    assert!(!stdout.trim_start().starts_with('{'));
}
#[test]
fn test_quiet_suppresses_gather_stdout() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args([
            "--offline",
            "--no-local-database",
            "gather",
            "--text",
            "doi:10.1234/example",
            "--format",
            "console",
            "--quiet",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty());
}
#[test]
fn test_gather_persists_project_candidate_and_reports_replay() {
    let database_path = acorn::prelude::temp_dir().join(format!("acorn-gather-{}.db", nanoid::nanoid!()));
    let run = || {
        Command::cargo_bin("acorn")
            .unwrap()
            .args([
                "--offline",
                "--database-path",
                database_path.to_str().unwrap(),
                "gather",
                "--text",
                "doi:10.1234/example https://orcid.org/0000-0002-2057-9115",
                "--format",
                "json",
            ])
            .output()
            .unwrap()
    };
    let created = run();
    assert!(created.status.success(), "{}", String::from_utf8_lossy(&created.stderr));
    let report: serde_json::Value = serde_json::from_slice(&created.stdout).unwrap();
    assert_eq!(report["summary"]["candidates"]["created"], 1);
    assert_eq!(report["candidates"].as_array().unwrap().len(), 1);
    assert!(report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["category"] == "link" && check["message"] == "created"));
    let replayed = run();
    assert!(replayed.status.success(), "{}", String::from_utf8_lossy(&replayed.stderr));
    let report: serde_json::Value = serde_json::from_slice(&replayed.stdout).unwrap();
    assert_eq!(report["summary"]["candidates"]["unchanged"], 1);
    assert!(report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["category"] == "link" && check["message"] == "unchanged"));
}
#[test]
fn test_osti_rejects_offline_mode_before_network_access() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args(["--offline", "--no-local-database", "gather", "--osti", "projects", "ACORN"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--osti cannot be used with --offline"));
}
#[test]
fn test_osti_conflicts_with_schema_analysis_options() {
    let output = Command::cargo_bin("acorn")
        .unwrap()
        .args(["--no-local-database", "gather", "--osti", "projects", "ACORN", "--standard", "rads"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
}
#[tokio::test]
async fn test_enrich_sources_emits_json_lines_without_rewriting_sources() {
    let mut data = ResearchActivity::default();
    data.contact.identifier = Some("0000-0002-1825-0097".to_string());
    data.meta.ror = Some(vec!["https://ror.org/01qz5mb56".to_string()]);
    let content = serde_json::to_string(&data).unwrap();
    let source = SourceDocument::init().content(content).format("json").source("activity.json").build();
    let (documents, enriched, failures) = SourceDocuments::from(vec![source]).enrich(true).await;
    assert_eq!(documents.0.len(), 1);
    assert_eq!(enriched.len(), 1);
    assert!(failures.is_empty());
    assert!(serde_json::from_str::<ResearchActivity>(&enriched[0]).is_ok());
    let yaml = serde_norway::to_string(&data).unwrap();
    assert!(ResearchActivity::parse(&yaml).is_some());
}
