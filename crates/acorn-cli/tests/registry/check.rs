//! Direct check command equivalence against the native routed analysis operation.
extern crate alloc;
use acorn::analyzer::{
    fix::FixOptions,
    host::{
        context::{AnalysisContext, AnalysisOptions, ExecutionExtension, ExecutionOptions, ReportExtension, ReportOptions},
        service::analyze_paths,
    },
};
use acorn::io::api::{
    openai::{Client as OpenAiClient, EndpointMode, Options},
    Configuration,
};
use acorn::io::mcp::{AnalysisFixInput, AnalysisInput, InputReference};
use acorn_core::options::CommonRuntime;
use acorn_core::util::MimeType;
use alloc::collections::BTreeSet;
use assert_cmd::Command;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpListener as SyncTcpListener;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::thread;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_direct_check_findings_match_native_analysis_offline() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let selected = "tests/fixtures/CITATION.cff";
    let skip = ["prose", "link", "quality", "readability", "fair", "crosswalk"];
    let native = AnalysisInput {
        input: InputReference::File(PathBuf::from(selected)),
        standard: None,
        format: None,
        skip: skip.iter().map(|name| (*name).to_string()).collect(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: false,
    }
    .analyze(&root, true)
    .await
    .expect("native analysis");
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(&root)
        .args([
            "--offline",
            "--no-local-database",
            "check",
            selected,
            "--output",
            "json",
            "--no-fail",
            "--skip",
            "prose,link,quality,readability,fair,crosswalk",
        ])
        .output()
        .expect("direct check");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    let json = &stdout[stdout.find('{').expect("JSON check output")..];
    let direct: Value = serde_json::from_str(json).expect("direct check JSON");
    let direct_findings = direct
        .get("checks")
        .and_then(Value::as_array)
        .expect("direct findings")
        .iter()
        .map(|check| {
            (
                check["category"].as_str().unwrap_or_default().to_string(),
                check["locator"].as_str().unwrap_or_default().to_string(),
                check["message"].as_str().unwrap_or_default().to_string(),
                check["severity"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    let native_findings = native
        .batches
        .iter()
        .flat_map(|batch| &batch.categories)
        .flat_map(|category| &category.findings)
        .filter(|finding| !finding.success)
        .map(|finding| {
            (
                finding.category.clone(),
                finding.locator.clone().unwrap_or_default(),
                finding.message.clone(),
                finding.severity.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(direct_findings, native_findings);
}
#[tokio::test]
async fn test_direct_directory_filters_match_native_selection() {
    let root = tempfile::tempdir().expect("temporary input directory");
    std::fs::write(
        root.path().join("bad.json"),
        include_str!("../../../../tests/fixtures/data/invalid_project_a/index.json"),
    )
    .expect("invalid metadata");
    std::fs::write(
        root.path().join("good.json"),
        include_str!("../../../../tests/fixtures/data/valid_project/index.json"),
    )
    .expect("valid metadata");
    let native = AnalysisInput {
        input: InputReference::Directory(PathBuf::from(".")),
        standard: None,
        format: None,
        skip: ["prose", "link", "quality", "readability", "fair", "crosswalk"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        filter: vec!["bad[.]json$".to_string()],
        ignore: vec!["good[.]json$".to_string()],
        disable_website_checks: false,
    }
    .analyze(root.path(), true)
    .await
    .expect("native filtered analysis");
    assert!(native
        .batches
        .iter()
        .flat_map(|batch| &batch.paths)
        .all(|path| path.ends_with("bad.json")));
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(root.path())
        .args([
            "--offline",
            "--no-local-database",
            "check",
            ".",
            "--filter",
            "bad[.]json$",
            "--ignore",
            "good[.]json$",
            "--output",
            "json",
            "--no-fail",
            "--skip",
            "prose,link,quality,readability,fair,crosswalk",
        ])
        .output()
        .expect("direct filtered check");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 check output");
    let direct: Value = serde_json::from_str(&stdout[stdout.find('{').expect("JSON output")..]).expect("direct filtered JSON");
    let direct_findings = direct
        .get("checks")
        .and_then(Value::as_array)
        .expect("direct findings")
        .iter()
        .map(|check| {
            (
                check["category"].as_str().unwrap_or_default().to_string(),
                check["message"].as_str().unwrap_or_default().to_string(),
                check["severity"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    let native_findings = native
        .batches
        .iter()
        .flat_map(|batch| &batch.categories)
        .flat_map(|category| &category.findings)
        .filter(|finding| !finding.success)
        .map(|finding| (finding.category.clone(), finding.message.clone(), finding.severity.clone()))
        .collect::<BTreeSet<_>>();
    assert_eq!(direct_findings, native_findings);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_direct_fix_proposals_match_native_preview_without_writing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let selected = "tests/fixtures/CITATION.cff";
    let original = std::fs::read(root.join(selected)).expect("original fixture");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("loopback listener");
    let address = listener.local_addr().expect("loopback address");
    let skip = ["prose", "link", "quality", "readability", "fair", "crosswalk"];
    let context = AnalysisContext::init()
        .analysis(
            AnalysisOptions::init()
                .skip(skip.iter().map(|name| (*name).to_string()).collect())
                .build(),
        )
        .execution(
            ExecutionOptions::init()
                .common(CommonRuntime {
                    quiet: true,
                    ..Default::default()
                })
                .extension(ExecutionExtension::init().disable_website_checks(true).build())
                .build(),
        )
        .report(
            ReportOptions::init()
                .common(CommonRuntime {
                    quiet: true,
                    ..Default::default()
                })
                .extension(ReportExtension::init().all(true).build())
                .build(),
        )
        .build();
    let report = analyze_paths(&[root.join(selected)], &context).await;
    let reference = report
        .normalize_checks(&FixOptions::default())
        .expect("normalized findings")
        .into_iter()
        .find(|reference| reference.vocabulary_advisory.is_none() && reference.uri.is_some())
        .expect("schema finding with a file target");
    let envelope = serde_json::json!({
        "fixes": [{
            "actions": [{
                "description": "Replace invalid email",
                "expected": "invalid email",
                "kind": "document-change",
                "replacement": "valid@example.org"
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
            "rationale": "Use a valid email address",
            "risk": "low",
            "source_fingerprint": reference.fingerprint,
            "status": "proposed",
            "summary": "Replace invalid email",
            "target": reference.uri.expect("file target")
        }],
        "version": "1"
    });
    let response = serde_json::json!({
        "id": "fixture-response",
        "model": "fixture-model",
        "choices": [{
            "message": {"content": envelope.to_string()},
            "finish_reason": "stop"
        }]
    });
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new().fallback(move || {
                let response = response.clone();
                async move { axum::Json(response) }
            }),
        )
        .await
        .expect("loopback inference server");
    });
    let base_url = format!("http://{address}");
    let backend = OpenAiClient::new(Options::default().with_domain(&base_url), EndpointMode::ChatCompletions, "fixture-model");
    let analysis = AnalysisInput {
        input: InputReference::File(PathBuf::from(selected)),
        standard: None,
        format: None,
        skip: skip.into_iter().map(str::to_string).collect(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: true,
    };
    let native = AnalysisFixInput { analysis }
        .analyze(
            &root,
            false,
            &backend,
            &FixOptions {
                model: Some("fixture-model".to_string()),
                ..FixOptions::default()
            },
        )
        .await
        .expect("native fix proposal");
    let direct = tokio::task::spawn_blocking({
        let root = root.clone();
        move || {
            let json = Command::cargo_bin("acorn")
                .expect("ACORN binary")
                .current_dir(&root)
                .args([
                    "--no-local-database",
                    "check",
                    selected,
                    "--output",
                    "json",
                    "--no-fail",
                    "--suggest-fixes",
                    "--backend",
                    "openai",
                    "--model",
                    "fixture-model",
                    "--base-url",
                    &base_url,
                    "--endpoint-mode",
                    "chat-completions",
                    "--disable-website-checks",
                    "--skip",
                    "prose,link,quality,readability,fair,crosswalk",
                ])
                .output()
                .expect("direct fix proposal");
            let diff = Command::cargo_bin("acorn")
                .expect("ACORN binary")
                .current_dir(&root)
                .args([
                    "--no-local-database",
                    "check",
                    selected,
                    "--output",
                    "diff",
                    "--no-fail",
                    "--suggest-fixes",
                    "--backend",
                    "openai",
                    "--model",
                    "fixture-model",
                    "--base-url",
                    &base_url,
                    "--endpoint-mode",
                    "chat-completions",
                    "--disable-website-checks",
                    "--skip",
                    "prose,link,quality,readability,fair,crosswalk",
                ])
                .output()
                .expect("direct fix preview");
            (json, diff)
        }
    })
    .await
    .expect("direct check task");
    server.abort();
    assert!(direct.0.status.success(), "{}", String::from_utf8_lossy(&direct.0.stderr));
    assert!(direct.1.status.success(), "{}", String::from_utf8_lossy(&direct.1.stderr));
    let preview = String::from_utf8(direct.1.stdout).expect("UTF-8 preview");
    assert!(native.differences.first().is_some_and(|difference| preview.contains(&difference.unified)));
    let stdout = String::from_utf8(direct.0.stdout).expect("UTF-8 output");
    let direct: Value = serde_json::from_str(&stdout[stdout.find('{').expect("JSON check output")..]).expect("direct check JSON");
    assert_eq!(
        direct.get("fixes"),
        Some(&serde_json::to_value(&native.fixes).expect("native fixes JSON"))
    );
    assert!(native.preview_diagnostics.is_empty());
    assert_eq!(native.differences.len(), 1);
    assert!(native
        .differences
        .first()
        .is_some_and(|difference| difference.unified.contains("valid@example.org")));
    assert_eq!(std::fs::read(root.join(selected)).expect("unchanged fixture"), original);
}
#[tokio::test]
async fn test_direct_git_branch_findings_match_native_selection() {
    let root = tempfile::tempdir().expect("temporary Git repository");
    let git = |arguments: &[&str]| {
        let output = ProcessCommand::new("git")
            .arg("-C")
            .arg(root.path())
            .args(arguments)
            .output()
            .expect("run Git");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "acorn@example.org"]);
    git(&["config", "user.name", "ACORN test"]);
    std::fs::write(root.path().join("README.txt"), "baseline").expect("baseline file");
    git(&["add", "README.txt"]);
    git(&["commit", "-m", "Baseline"]);
    git(&["checkout", "-b", "feature"]);
    std::fs::write(
        root.path().join("metadata.json"),
        include_str!("../../../../tests/fixtures/data/invalid_project_a/index.json"),
    )
    .expect("metadata file");
    git(&["add", "metadata.json"]);
    git(&["commit", "-m", "Add metadata"]);
    let native = AnalysisInput {
        input: InputReference::GitBranch("feature".to_string()),
        standard: None,
        format: None,
        skip: ["prose", "link", "quality", "readability", "fair", "crosswalk"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: false,
    }
    .analyze(root.path(), true)
    .await
    .expect("native Git analysis");
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(root.path())
        .args([
            "--offline",
            "--no-local-database",
            "check",
            "--branch",
            "feature",
            "--output",
            "json",
            "--no-fail",
            "--skip",
            "prose,link,quality,readability,fair,crosswalk",
        ])
        .output()
        .expect("direct Git check");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 check output");
    let direct: Value = serde_json::from_str(&stdout[stdout.find('{').expect("JSON output")..]).expect("direct Git JSON");
    let direct_findings = direct
        .get("checks")
        .and_then(Value::as_array)
        .expect("direct findings")
        .iter()
        .map(|check| {
            (
                check["category"].as_str().unwrap_or_default().to_string(),
                check["message"].as_str().unwrap_or_default().to_string(),
                check["severity"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    let native_findings = native
        .batches
        .iter()
        .flat_map(|batch| &batch.categories)
        .flat_map(|category| &category.findings)
        .filter(|finding| !finding.success)
        .map(|finding| (finding.category.clone(), finding.message.clone(), finding.severity.clone()))
        .collect::<BTreeSet<_>>();
    assert_eq!(direct_findings, native_findings);
}
#[test]
fn test_direct_merge_request_findings_match_native_selection() {
    let root = tempfile::tempdir().expect("temporary merge request root");
    std::fs::write(
        root.path().join("metadata.json"),
        include_str!("../../../../tests/fixtures/data/invalid_project_a/index.json"),
    )
    .expect("metadata file");
    let listener = SyncTcpListener::bind("127.0.0.1:0").expect("loopback GitLab API");
    let address = listener.local_addr().expect("loopback address");
    let body = r#"[{"new_path":"metadata.json"}]"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let server = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().expect("GitLab request");
            let mut request = [0_u8; 2048];
            let count = stream.read(&mut request).expect("read GitLab request");
            assert!(count > 0);
            stream.write_all(response.as_bytes()).expect("write GitLab response");
        }
    });
    let url = format!("http://{address}/api/v4");
    temp_env::with_vars(
        [
            ("CI_API_V4_URL", Some(url.as_str())),
            ("CI_MERGE_REQUEST_PROJECT_ID", Some("42")),
            ("CI_MERGE_REQUEST_IID", Some("7")),
        ],
        || {
            let native = tokio::runtime::Runtime::new()
                .expect("runtime")
                .block_on(
                    AnalysisInput {
                        input: InputReference::GitMergeRequest,
                        standard: None,
                        format: None,
                        skip: ["prose", "link", "quality", "readability", "fair", "crosswalk"]
                            .into_iter()
                            .map(str::to_string)
                            .collect(),
                        filter: Vec::new(),
                        ignore: Vec::new(),
                        disable_website_checks: true,
                    }
                    .analyze(root.path(), false),
                )
                .expect("native merge request analysis");
            let output = Command::cargo_bin("acorn")
                .expect("ACORN binary")
                .current_dir(root.path())
                .args([
                    "--no-local-database",
                    "check",
                    "--merge-request",
                    "--disable-website-checks",
                    "--output",
                    "json",
                    "--no-fail",
                    "--skip",
                    "prose,link,quality,readability,fair,crosswalk",
                ])
                .output()
                .expect("direct merge request check");
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            let stdout = String::from_utf8(output.stdout).expect("UTF-8 check output");
            let direct: Value = serde_json::from_str(&stdout[stdout.find('{').expect("JSON output")..]).expect("direct merge request JSON");
            let direct_findings = direct
                .get("checks")
                .and_then(Value::as_array)
                .expect("direct findings")
                .iter()
                .map(|check| {
                    (
                        check["category"].as_str().unwrap_or_default().to_string(),
                        check["message"].as_str().unwrap_or_default().to_string(),
                        check["severity"].as_str().unwrap_or_default().to_string(),
                    )
                })
                .collect::<BTreeSet<_>>();
            let native_findings = native
                .batches
                .iter()
                .flat_map(|batch| &batch.categories)
                .flat_map(|category| &category.findings)
                .filter(|finding| !finding.success)
                .map(|finding| (finding.category.clone(), finding.message.clone(), finding.severity.clone()))
                .collect::<BTreeSet<_>>();
            assert_eq!(direct_findings, native_findings);
        },
    );
    server.join().expect("GitLab server finished");
}
#[cfg(unix)]
#[test]
fn test_direct_oci_findings_match_native_analysis_without_layer_pull() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = tempfile::tempdir().expect("fake ORAS directory");
    let binary = temp.path().join("oras");
    let log = temp.path().join("calls.log");
    let fixtures = root.join("tests/fixtures/oci");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\ncase \"$1\" in\n  --version|version|-version|-v) printf '%s\\n' 'oras version 1.3.0' ;;\n  manifest) if [ \"$3\" = \"--descriptor\" ]; then cat '{}'; else cat '{}'; fi ;;\n  *) exit 97 ;;\nesac\n",
        log.display(), fixtures.join("descriptor.json").display(), fixtures.join("manifest-annotated.json").display()
    );
    std::fs::write(&binary, script).expect("fake ORAS executable");
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).expect("executable permission");
    let path = std::env::var_os("PATH")
        .map(|value| format!("{}:{}", temp.path().display(), value.to_string_lossy()))
        .unwrap_or_else(|| temp.path().display().to_string());
    let reference = "oci://registry.example.org/models/fixture:latest";
    temp_env::with_var("PATH", Some(path), || {
        let native = tokio::runtime::Runtime::new()
            .expect("runtime")
            .block_on(
                AnalysisInput {
                    input: InputReference::Oci(reference.to_string()),
                    standard: Some(acorn_schema::standard::Standard::Oci),
                    format: None,
                    skip: Vec::new(),
                    filter: Vec::new(),
                    ignore: Vec::new(),
                    disable_website_checks: false,
                }
                .analyze(&root, false),
            )
            .expect("native OCI analysis");
        let output = Command::cargo_bin("acorn")
            .expect("ACORN binary")
            .current_dir(&root)
            .args(["--no-local-database", "check", reference, "--all", "--output", "json", "--no-fail"])
            .output()
            .expect("direct OCI check");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8(output.stdout).expect("UTF-8 check output");
        let direct: Value = serde_json::from_str(&stdout[stdout.find('{').expect("JSON output")..]).expect("direct OCI JSON");
        let direct_findings = direct
            .get("checks")
            .and_then(Value::as_array)
            .expect("direct findings")
            .iter()
            .map(|check| {
                (
                    check["category"].as_str().unwrap_or_default().to_string(),
                    check["message"].as_str().unwrap_or_default().to_string(),
                    check["severity"].as_str().unwrap_or_default().to_string(),
                    check["success"].as_bool().unwrap_or(false),
                )
            })
            .collect::<BTreeSet<_>>();
        let native_findings = native
            .batches
            .iter()
            .flat_map(|batch| &batch.categories)
            .flat_map(|category| &category.findings)
            .map(|finding| {
                (
                    finding.category.clone(),
                    finding.message.clone(),
                    finding.severity.clone(),
                    finding.success,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(direct_findings, native_findings);
        let calls = std::fs::read_to_string(&log).expect("ORAS call log");
        assert!(!calls.contains("pull"));
    });
}
#[tokio::test]
async fn test_direct_piped_json_findings_match_native_text_input() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = include_str!("../../../../tests/fixtures/data/invalid_project_a/index.json");
    let native = AnalysisInput {
        input: InputReference::Text(content.to_string()),
        standard: None,
        format: Some(MimeType::Json),
        skip: ["prose", "link", "quality", "readability", "fair", "crosswalk"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: false,
    }
    .analyze(&root, true)
    .await
    .expect("native piped analysis");
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .current_dir(&root)
        .args([
            "--offline",
            "--no-local-database",
            "check",
            "--format",
            "json",
            "--output",
            "json",
            "--no-fail",
            "--skip",
            "prose,link,quality,readability,fair,crosswalk",
        ])
        .write_stdin(content)
        .output()
        .expect("direct piped check");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 check output");
    let direct: Value = serde_json::from_str(&stdout[stdout.find('{').expect("JSON output")..]).expect("direct piped JSON");
    let direct_findings = direct
        .get("checks")
        .and_then(Value::as_array)
        .expect("direct findings")
        .iter()
        .map(|check| {
            (
                check["category"].as_str().unwrap_or_default().to_string(),
                check["message"].as_str().unwrap_or_default().to_string(),
                check["severity"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    let native_findings = native
        .batches
        .iter()
        .flat_map(|batch| &batch.categories)
        .flat_map(|category| &category.findings)
        .filter(|finding| !finding.success)
        .map(|finding| (finding.category.clone(), finding.message.clone(), finding.severity.clone()))
        .collect::<BTreeSet<_>>();
    assert_eq!(direct_findings, native_findings);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_direct_remote_findings_match_native_analysis() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("loopback listener");
    let address = listener.local_addr().expect("loopback address");
    let content = include_str!("../../../../tests/fixtures/CITATION.cff");
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/yaml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{content}",
        content.len()
    );
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.expect("accepted request");
            let mut request = [0_u8; 2048];
            let count = stream.read(&mut request).await.expect("read request");
            assert!(count > 0);
            stream.write_all(response.as_bytes()).await.expect("write response");
        }
    });
    let url = format!("http://{address}/CITATION.cff");
    let native = AnalysisInput {
        input: InputReference::Url(url.clone()),
        standard: None,
        format: None,
        skip: ["prose", "link", "quality", "readability", "fair", "crosswalk"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: true,
    }
    .analyze(&root, false)
    .await
    .expect("native remote analysis");
    let direct = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("acorn")
            .expect("ACORN binary")
            .current_dir(&root)
            .args([
                "--no-local-database",
                "check",
                &url,
                "--output",
                "json",
                "--no-fail",
                "--disable-website-checks",
                "--skip",
                "prose,link,quality,readability,fair,crosswalk",
            ])
            .output()
            .expect("direct remote check")
    })
    .await
    .expect("direct check task");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    let stdout = String::from_utf8(direct.stdout).expect("UTF-8 check output");
    let direct: Value = serde_json::from_str(&stdout[stdout.find('{').expect("JSON output")..]).expect("direct remote JSON");
    let direct_findings = direct
        .get("checks")
        .and_then(Value::as_array)
        .expect("direct findings")
        .iter()
        .map(|check| {
            (
                check["category"].as_str().unwrap_or_default().to_string(),
                check["message"].as_str().unwrap_or_default().to_string(),
                check["severity"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    let native_findings = native
        .batches
        .iter()
        .flat_map(|batch| &batch.categories)
        .flat_map(|category| &category.findings)
        .filter(|finding| !finding.success)
        .map(|finding| (finding.category.clone(), finding.message.clone(), finding.severity.clone()))
        .collect::<BTreeSet<_>>();
    assert_eq!(direct_findings, native_findings);
    server.await.expect("server finished");
}
