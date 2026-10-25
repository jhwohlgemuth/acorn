use super::capability::InvocationCapability;
use super::config::{ExecutionOrigin, Resolved};
use super::hooks::{HookExecution, HookOptions};
use super::manifest::{
    FilesystemCapability, NetworkCapability, ResourceLimits, ScriptAuthorization, ScriptCapabilities, ScriptManifest, ScriptManifestSource,
    SCRIPT_MANIFEST_VERSION,
};
use super::parse_version_header;
use super::result::{ExecutionResult, RhaiExt};
use super::runtime::{verify_evaluate_entry_point, RhaiRuntime, Runtime};
use crate::analyzer::{Check, CheckCategory, CheckSeverity};
use crate::io::config::application::{ApplicationConfiguration, Entry, Hook};
use crate::io::workflow::{Change, WorkflowAction};
use acorn_analysis::{Finding, Findings};
use acorn_core::util::constants::app::{MAX_EXEC_BYTES, SUPPORTED_EXEC_VERSION};
use acorn_core::util::MimeType;
use acorn_core::Location;
use acorn_core::OneOrMany;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::validation::Validate as SchemaValidate;
use alloc::collections::BTreeMap;
use core::str::FromStr;
use std::fs::{create_dir, read, write};
#[cfg(unix)]
use std::fs::{set_permissions, Permissions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use validator::Validate;

const VALID_SOURCE: &str = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: []} }";

fn authorization(capabilities: ScriptCapabilities) -> ScriptAuthorization {
    ScriptAuthorization {
        version: SUPPORTED_EXEC_VERSION.to_string(),
        capabilities,
        digest: "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string(),
        entry: "evaluate".to_string(),
        hooks: OneOrMany::Many(vec![Hook::Check, Hook::Format]),
        id: "quality".to_string(),
        limits: ResourceLimits::default(),
    }
}
fn finding(rule_id: &str) -> Finding {
    Finding {
        category: CheckCategory::Quality,
        context: None,
        fair: None,
        locator: None,
        message: "msg".to_string(),
        rule_id: rule_id.to_string(),
        severity: CheckSeverity::Info,
        success: true,
        uri: None,
    }
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/exec").join(name)
}
fn fixture_execution(name: &str, hook: WorkflowAction, data: &serde_json::Value, capabilities: ScriptCapabilities) -> HookExecution {
    let temporary = tempdir().expect("temporary script root");
    let path = temporary.path().join(name);
    write(&path, read(fixture(name)).expect("read fixture script")).expect("stage fixture script");
    #[cfg(unix)]
    set_permissions(&path, Permissions::from_mode(0o600)).expect("secure fixture script");
    let mut resolved = Resolved::direct(&path).expect("resolve fixture script");
    resolved.entry.id = Some(name.to_string());
    let authorization = authorization(capabilities);
    resolved
        .evaluate_hook(
            "activity.json",
            data,
            HookOptions::init()
                .maybe_authorization(Some(&authorization))
                .hook(hook)
                .maybe_manifest_source(Some("test-fixture"))
                .build(),
        )
        .expect("execute fixture script")
}
fn manifest(capabilities: ScriptCapabilities) -> ScriptManifest {
    ScriptManifest {
        version: SCRIPT_MANIFEST_VERSION.to_string(),
        scripts: OneOrMany::One(authorization(capabilities)),
    }
}

#[cfg(unix)]
#[test]
fn test_absolute_script_rejects_group_writable_file() {
    let temporary = tempdir().expect("temporary script root");
    let path = temporary.path().join("writable.rhai");
    write(&path, VALID_SOURCE).expect("script");
    set_permissions(&path, Permissions::from_mode(0o666)).expect("writable script permissions");
    let error = Resolved::direct(&path).expect_err("writable absolute script must be denied");
    assert!(error.to_string().contains("group/world-writable"));
}
#[test]
fn test_application_configuration_has_hook() {
    let configuration = ApplicationConfiguration {
        scripts: Some(OneOrMany::One(Entry {
            id: None,
            path: "quality".to_string(),
            hooks: OneOrMany::Many(vec![Hook::Check, Hook::Format]),
            absolute: false,
        })),
        ..ApplicationConfiguration::default()
    };
    assert!(configuration.has_hook(Hook::Check));
    assert!(configuration.has_hook(Hook::Format));
    assert!(!configuration.has_hook(Hook::Link));
    assert_eq!(WorkflowAction::from(Hook::Check), WorkflowAction::Check);
}
#[test]
fn test_application_configuration_validates_entries_and_duplicate_paths() {
    let configuration = ApplicationConfiguration {
        scripts: Some(OneOrMany::Many(vec![
            Entry {
                id: None,
                path: "quality".to_string(),
                hooks: OneOrMany::One(Hook::Check),
                absolute: false,
            },
            Entry {
                id: None,
                path: "quality".to_string(),
                hooks: OneOrMany::One(Hook::Format),
                absolute: false,
            },
        ])),
        ..ApplicationConfiguration::default()
    };
    let report = SchemaValidate::validate(&configuration).expect_err("duplicate paths fail validation");
    assert!(report.iter().any(|issue| issue.path == "scripts[1].path"));
}
#[test]
fn test_check_safe_missing_header_fails() {
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), "fn evaluate(context) {}")
        .iter()
        .any(Check::is_failure));
}
#[test]
fn test_check_safe_oversized_fails() {
    let source = format!("// @acorn-exec-v0.2.0\nfn evaluate(context) {{}}\n{}", "x".repeat(MAX_EXEC_BYTES));
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), &source).iter().any(Check::is_failure));
}
#[test]
fn test_check_safe_valid_source_passes() {
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), VALID_SOURCE)
        .iter()
        .all(|check| !check.is_failure()));
}
#[test]
fn test_check_safe_wrong_arity_fails() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(a,b) {}";
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), source).iter().any(Check::is_failure));
}
#[test]
fn test_config_entry_validation_duplicate_hooks() {
    let entry = Entry {
        id: None,
        path: "foo".to_string(),
        hooks: OneOrMany::Many(vec![Hook::Check, Hook::Check]),
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn test_config_entry_validation_empty_hooks() {
    let entry = Entry {
        id: None,
        path: "foo".to_string(),
        hooks: OneOrMany::Many(Vec::new()),
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn test_config_entry_validation_parent_traversal() {
    let entry = Entry {
        id: None,
        path: "../escape".to_string(),
        hooks: OneOrMany::One(Hook::Check),
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn test_config_entry_validation_repeated_root() {
    let entry = Entry {
        id: None,
        path: ".acorn/quality".to_string(),
        hooks: OneOrMany::One(Hook::Check),
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn test_config_hook_parse_invalid() {
    assert!(Hook::from_str("gather").is_err());
}
#[test]
fn test_config_hook_parse_valid() {
    assert_eq!(Hook::from_str("check").unwrap(), Hook::Check);
}
#[test]
fn test_configured_scripts_compile_completely_before_evaluation() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    write(script_root.join("first.rhai"), "// @acorn-exec-v0.2.0\nfn evaluate(context) { loop {} }").expect("first script");
    write(script_root.join("second.rhai"), "// @acorn-exec-v0.2.0\nfn evaluate(context) {").expect("second script");
    let configuration = ApplicationConfiguration {
        scripts: Some(OneOrMany::Many(vec![
            Entry {
                id: None,
                path: "first".to_string(),
                hooks: OneOrMany::One(Hook::Check),
                absolute: false,
            },
            Entry {
                id: None,
                path: "second".to_string(),
                hooks: OneOrMany::One(Hook::Check),
                absolute: false,
            },
        ])),
        ..ApplicationConfiguration::default()
    };
    let changes = vec![Change::Document {
        changed: false,
        conflicts: Vec::new(),
        decisions: Vec::new(),
        content: String::new(),
        data: Box::new(ResearchActivity::default()),
        failures: Vec::new(),
        linked: None,
        original: String::new(),
        source: Location::from("activity.json"),
        vocabulary_reports: Vec::new(),
    }];
    let error = configuration
        .check_scripts(&changes, temporary.path(), WorkflowAction::Check, ExecutionOrigin::Local)
        .expect_err("invalid second script must fail before the first script evaluates");
    assert!(error.to_string().contains("second.rhai` compile error"));
}
#[test]
fn test_configured_scripts_deny_non_local_callers() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    write(script_root.join("quality.rhai"), VALID_SOURCE).expect("script");
    let entry = Entry {
        id: None,
        path: "quality".to_string(),
        hooks: OneOrMany::One(Hook::Check),
        absolute: false,
    };
    let error = entry
        .resolve(temporary.path(), ExecutionOrigin::NonLocal)
        .expect_err("non-local execution must be denied");
    assert!(error.to_string().contains("unavailable to non-local callers"));
}
#[test]
fn test_configured_scripts_use_immutable_compiled_snapshot() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    let path = script_root.join("quality.rhai");
    write(&path, VALID_SOURCE).expect("script");
    let resolved = Entry {
        id: None,
        path: "quality".to_string(),
        hooks: OneOrMany::One(Hook::Check),
        absolute: false,
    }
    .resolve(temporary.path(), ExecutionOrigin::Local)
    .expect("prepared script");
    write(&path, "invalid after preparation").expect("changed script");
    assert!(resolved
        .check("activity.json", &ResearchActivity::default(), 0, WorkflowAction::Check)
        .expect("prepared AST evaluates")
        .is_empty());
}
#[test]
fn test_configured_validate_findings_use_schema_category() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    let source =
        "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [#{rule_id: \"rule-1\", severity: \"warning\", message: context.hook}]} }";
    write(script_root.join("quality.rhai"), source).expect("script");
    let resolved = Entry {
        id: None,
        path: "quality".to_string(),
        hooks: OneOrMany::One(Hook::Validate),
        absolute: false,
    }
    .resolve(temporary.path(), ExecutionOrigin::Local)
    .expect("prepared script");
    let checks = resolved
        .check("activity.json", &ResearchActivity::default(), 0, WorkflowAction::Validate)
        .expect("validate hook evaluates");
    let check = checks.first().expect("validate finding");
    assert_eq!(check.category, CheckCategory::Schema);
    assert_eq!(check.message, "validate");
}
#[test]
fn test_effectful_runtime_reads_and_proposes_without_writing() {
    let temporary = tempdir().expect("temporary capability root");
    write(temporary.path().join("input.txt"), "content").expect("input");
    let roots = BTreeMap::from([("project".to_string(), temporary.path().to_path_buf())]);
    let authorization = authorization(ScriptCapabilities {
        read: Some(FilesystemCapability {
            max_bytes: 1024,
            max_files: 1,
            roots: roots.clone(),
        }),
        write: Some(FilesystemCapability {
            max_bytes: 1024,
            max_files: 1,
            roots,
        }),
        ..ScriptCapabilities::default()
    });
    let capability = InvocationCapability::new(&authorization, false, WorkflowAction::Format).expect("capability");
    let runtime = RhaiRuntime::with_capability(&capability, &authorization.limits);
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { let value = read_text(\"project\", \"input.txt\"); write_text(\"project\", \"output.txt\", value); #{findings: []} }";
    let result = runtime.check(source, rhai::Map::new());
    assert!(result.is_ok());
    let proposals = capability.proposals().expect("proposals");
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals.first().expect("one proposal").content, "content");
    assert!(!temporary.path().join("output.txt").exists());
}
#[test]
fn test_filesystem_read_enforces_distinct_file_limit() {
    let temporary = tempdir().expect("temporary capability root");
    write(temporary.path().join("first.txt"), "first").expect("first input");
    write(temporary.path().join("second.txt"), "second").expect("second input");
    let authorization = authorization(ScriptCapabilities {
        read: Some(FilesystemCapability {
            max_bytes: 1024,
            max_files: 1,
            roots: BTreeMap::from([("project".to_string(), temporary.path().to_path_buf())]),
        }),
        ..ScriptCapabilities::default()
    });
    let capability = InvocationCapability::new(&authorization, false, WorkflowAction::Check).expect("capability");
    let runtime = RhaiRuntime::with_capability(&capability, &authorization.limits);
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { read_text(\"project\", \"first.txt\"); read_text(\"project\", \"second.txt\"); #{findings: []} }";
    let error = runtime.check(source, rhai::Map::new()).expect_err("file limit must fail");
    assert!(error.to_string().contains("file limit"));
}
#[cfg(unix)]
#[test]
fn test_filesystem_read_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let temporary = tempdir().expect("temporary capability root");
    let outside = tempdir().expect("outside root");
    write(outside.path().join("secret.txt"), "secret").expect("outside input");
    symlink(outside.path(), temporary.path().join("escape")).expect("symlink");
    let authorization = authorization(ScriptCapabilities {
        read: Some(FilesystemCapability {
            max_bytes: 1024,
            max_files: 1,
            roots: BTreeMap::from([("project".to_string(), temporary.path().to_path_buf())]),
        }),
        ..ScriptCapabilities::default()
    });
    let capability = InvocationCapability::new(&authorization, false, WorkflowAction::Check).expect("capability");
    let runtime = RhaiRuntime::with_capability(&capability, &authorization.limits);
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { read_text(\"project\", \"escape/secret.txt\"); #{findings: []} }";
    assert!(runtime.check(source, rhai::Map::new()).is_err());
}
#[test]
fn test_filesystem_write_is_unavailable_to_check_hooks() {
    let temporary = tempdir().expect("temporary capability root");
    let roots = BTreeMap::from([("project".to_string(), temporary.path().to_path_buf())]);
    let authorization = authorization(ScriptCapabilities {
        write: Some(FilesystemCapability {
            max_bytes: 1024,
            max_files: 1,
            roots,
        }),
        ..ScriptCapabilities::default()
    });
    let capability = InvocationCapability::new(&authorization, false, WorkflowAction::Check).expect("capability");
    let runtime = RhaiRuntime::with_capability(&capability, &authorization.limits);
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { write_text(\"project\", \"output.txt\", \"content\"); #{findings: []} }";
    assert!(runtime.check(source, rhai::Map::new()).is_err());
    assert!(!temporary.path().join("output.txt").exists());
}
#[test]
fn test_fixture_check_hook_reports_draft_activity() {
    let execution = fixture_execution(
        "check-draft.rhai",
        WorkflowAction::Check,
        &serde_json::json!({"meta": {"draft": true}, "title": "Draft activity"}),
        ScriptCapabilities::default(),
    );
    let check = execution.checks.first().expect("draft finding");
    assert_eq!(check.category, CheckCategory::Quality);
    assert_eq!(check.locator.as_deref(), Some("meta.draft"));
    assert_eq!(check.message, "Project metadata is still marked as draft");
}
#[test]
fn test_fixture_enrich_hook_adds_keywords() {
    let execution = fixture_execution(
        "enrich-keywords.rhai",
        WorkflowAction::Enrich,
        &serde_json::json!({"meta": {"keywords": []}, "title": "Research activity"}),
        ScriptCapabilities {
            mutation: true,
            ..ScriptCapabilities::default()
        },
    );
    let document = execution.document.expect("enriched document");
    let document = serde_json::from_str::<serde_json::Value>(&document).expect("valid JSON document");
    assert_eq!(
        document.pointer("/meta/keywords"),
        Some(&serde_json::json!(["reproducibility", "research software"]))
    );
}
#[test]
fn test_fixture_format_hook_normalizes_title() {
    let execution = fixture_execution(
        "format-title.rhai",
        WorkflowAction::Format,
        &serde_json::json!({"meta": {"id": "activity-1"}, "title": "  research title  "}),
        ScriptCapabilities {
            mutation: true,
            ..ScriptCapabilities::default()
        },
    );
    let document = execution.document.expect("formatted document");
    let document = serde_json::from_str::<serde_json::Value>(&document).expect("valid JSON document");
    assert_eq!(document.get("title"), Some(&serde_json::json!("Normalized research title")));
}
#[test]
fn test_fixture_link_hook_proposes_summary_file() {
    let temporary = tempdir().expect("temporary output root");
    let execution = fixture_execution(
        "link-summary.rhai",
        WorkflowAction::Link,
        &serde_json::json!({"meta": {"id": "activity-1"}, "title": "Linked activity"}),
        ScriptCapabilities {
            write: Some(FilesystemCapability {
                max_bytes: 1024,
                max_files: 1,
                roots: BTreeMap::from([("output".to_string(), temporary.path().to_path_buf())]),
            }),
            ..ScriptCapabilities::default()
        },
    );
    let proposal = execution.files.first().expect("summary proposal");
    assert_eq!(proposal.content, "Linked activity");
    assert_eq!(proposal.path, temporary.path().join("activity-summary.txt"));
    assert!(!proposal.path.exists());
}
#[test]
fn test_fixture_validate_hook_rejects_missing_identifier() {
    let execution = fixture_execution(
        "validate-identifier.rhai",
        WorkflowAction::Validate,
        &serde_json::json!({"meta": {"id": ""}, "title": "Unidentified activity"}),
        ScriptCapabilities::default(),
    );
    let check = execution.checks.first().expect("identifier finding");
    assert_eq!(check.category, CheckCategory::Schema);
    assert_eq!(check.severity, CheckSeverity::Error);
    assert_eq!(check.locator.as_deref(), Some("meta.id"));
}
#[test]
fn test_format_hook_returns_a_structured_document_proposal_and_audit() {
    let temporary = tempdir().expect("temporary script root");
    let script = temporary.path().join("format.rhai");
    write(
        &script,
        "// @acorn-exec-v0.2.0\nfn evaluate(context) { context.proposed.test = \"changed\"; #{findings: [], output: #{document: context.proposed}} }",
    )
    .expect("script");
    let mut resolved = Resolved::direct(&script).expect("resolved script");
    resolved.entry.id = Some("quality".to_string());
    let execution = resolved
        .evaluate_hook(
            "activity.json",
            &serde_json::json!({"test": "original"}),
            HookOptions::init()
                .maybe_authorization(Some(&authorization(ScriptCapabilities {
                    mutation: true,
                    ..ScriptCapabilities::default()
                })))
                .hook(WorkflowAction::Format)
                .maybe_manifest_source(Some("inline"))
                .build(),
        )
        .expect("hook execution");
    let document = execution.document.expect("document proposal");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&document).expect("JSON document").get("test"),
        Some(&serde_json::Value::String("changed".to_string()))
    );
    assert_eq!(execution.audit.script_id, "quality");
    assert_eq!(execution.audit.proposal_digests.len(), 1);
}
#[test]
fn test_infer_explicit_rhai_extension() {
    assert_eq!(MimeType::from("script.rhai"), MimeType::Rhai);
}
#[test]
fn test_infer_extensionless_text_is_not_rhai() {
    assert_eq!(MimeType::infer(b"hello world"), None);
}
#[test]
fn test_infer_extensionless_valid_rhai_probe() {
    assert_eq!(MimeType::infer(VALID_SOURCE.as_bytes()), Some(MimeType::Rhai));
}
#[test]
fn test_infer_rhai_header_is_independent_of_compilation() {
    let source = b"// @acorn-exec-v0.2.0\nfn evaluate(a,b) {}";
    assert_eq!(MimeType::infer(source), Some(MimeType::Rhai));
    assert!(RhaiRuntime::validate_source(Path::new("invalid.rhai"), core::str::from_utf8(source).expect("UTF-8 source")).is_err());
}
#[test]
fn test_inline_effectful_manifest_requires_explicit_configuration() {
    let source = ScriptManifestSource::Inline(Box::new(manifest(ScriptCapabilities {
        mutation: true,
        ..ScriptCapabilities::default()
    })));
    assert!(source.resolve(false).is_err());
    assert!(source.resolve(true).is_ok());
}
#[test]
fn test_inline_manifest_deserializes_from_application_configuration() {
    let configuration = ApplicationConfiguration::parse(format!(
        r#"{{"scriptManifest":{{"version":"{SCRIPT_MANIFEST_VERSION}","scripts":[]}},"scripts":[]}}"#
    ));
    assert!(configuration.is_ok());
    assert!(configuration.expect("configuration").script_manifest.is_some());
}
#[test]
fn test_manifest_deserializes_simple_names() {
    let content = format!(
        r#"{{"version":"{SCRIPT_MANIFEST_VERSION}","scripts":[{{"id":"quality","version":"{SUPPORTED_EXEC_VERSION}","entry":"evaluate","digest":"sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad","hooks":["check"],"capabilities":{{"read":null,"write":null,"network":null}}}}]}}"#
    );
    let manifest = ScriptManifest::parse(Path::new("manifest.json"), &content).expect("manifest parses");
    assert!(SchemaValidate::validate(&manifest).is_ok());
    let authorization = manifest.scripts.first().expect("authorization");
    assert_eq!(authorization.entry, "evaluate");
    assert_eq!(authorization.version, SUPPORTED_EXEC_VERSION.to_string());
}
#[test]
fn test_manifest_rejects_an_unsupported_script_abi() {
    let mut manifest = manifest(ScriptCapabilities::default());
    manifest.scripts.first_mut().expect("authorization").version = "9.0.0".to_string();
    assert!(manifest.validate().is_err());
}
#[test]
fn test_module_loads() {
    assert_eq!(SUPPORTED_EXEC_VERSION.to_string(), "0.2.0");
}
#[test]
fn test_network_capability_denies_offline_requests() {
    let authorization = authorization(ScriptCapabilities {
        network: Some(NetworkCapability {
            hosts: OneOrMany::One("example.com".to_string()),
            max_requests: 1,
            max_response_bytes: 1024,
            timeout: 100,
        }),
        ..ScriptCapabilities::default()
    });
    let capability = InvocationCapability::new(&authorization, true, WorkflowAction::Check).expect("capability");
    let runtime = RhaiRuntime::with_capability(&capability, &authorization.limits);
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { http_get(\"https://example.com\"); #{findings: []} }";
    let error = runtime.check(source, rhai::Map::new()).expect_err("offline request must fail");
    assert!(error.to_string().contains("offline"));
}
#[test]
fn test_network_capability_denies_unlisted_and_private_targets() {
    let authorization = authorization(ScriptCapabilities {
        network: Some(NetworkCapability {
            hosts: OneOrMany::One("127.0.0.1".to_string()),
            max_requests: 2,
            max_response_bytes: 1024,
            timeout: 100,
        }),
        ..ScriptCapabilities::default()
    });
    let capability = InvocationCapability::new(&authorization, false, WorkflowAction::Check).expect("capability");
    let runtime = RhaiRuntime::with_capability(&capability, &authorization.limits);
    let unlisted = "// @acorn-exec-v0.2.0\nfn evaluate(context) { http_get(\"https://example.com\"); #{findings: []} }";
    let private = "// @acorn-exec-v0.2.0\nfn evaluate(context) { http_get(\"https://127.0.0.1\"); #{findings: []} }";
    assert!(runtime
        .check(unlisted, rhai::Map::new())
        .expect_err("unlisted host")
        .to_string()
        .contains("not allowlisted"));
    assert!(runtime
        .check(private, rhai::Map::new())
        .expect_err("private address")
        .to_string()
        .contains("prohibited address"));
}
#[test]
fn test_result_empty_rule_fails() {
    assert!(finding("").validate().is_err());
}
#[test]
fn test_result_rhai_ext_parses_findings() {
    let mut result = rhai::Map::new();
    result.insert("findings".into(), rhai::Dynamic::from(rhai::Array::new()));
    assert!(rhai::Dynamic::from(result).findings().expect("findings").is_empty());
}
#[test]
fn test_result_rhai_ext_rejects_bare_array() {
    assert!(rhai::Dynamic::from(rhai::Array::new()).findings().is_err());
}
#[test]
fn test_result_rhai_ext_rejects_invalid_finding_fields() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [#{rule_id: \"rule-1\", severity: \"error\", message: \"msg\", category: \"quality\"}]} }";
    let dynamic = RhaiRuntime::new().check(source, rhai::Map::new()).expect("script evaluates");
    assert_eq!(
        dynamic.findings().expect_err("category is host-owned"),
        "unknown finding field `category`"
    );
}
#[test]
fn test_result_rhai_ext_rejects_invalid_output() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{output: 1, findings: []} }";
    let dynamic = RhaiRuntime::new().check(source, rhai::Map::new()).expect("script evaluates");
    assert_eq!(
        dynamic.findings().expect_err("quality output must be unit"),
        "quality-check output must be omitted or unit"
    );
}
#[test]
fn test_result_rhai_ext_rejects_unknown_envelope_fields() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [], extra: true} }";
    let dynamic = RhaiRuntime::new().check(source, rhai::Map::new()).expect("script evaluates");
    assert_eq!(
        dynamic.findings().expect_err("unknown field must fail"),
        "unknown execution result field `extra`"
    );
}
#[test]
fn test_result_valid_finding_passes() {
    assert!(finding("rule-1").validate().is_ok());
}
#[test]
fn test_result_validates() {
    let result = ExecutionResult {
        output: (),
        findings: Findings::default(),
    };
    assert!(result.validate().is_ok());
}
#[test]
fn test_runtime_missing_entry_fails() {
    let ast = RhaiRuntime::new().compile("// @acorn-exec-v0.2.0\nfn other(x) {}").expect("compile");
    assert!(verify_evaluate_entry_point(&ast).is_err());
}
#[test]
fn test_runtime_operation_limit_stops_infinite_loop() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { loop {} }";
    assert!(RhaiRuntime::new().check(source, rhai::Map::new()).is_err());
}
#[test]
fn test_runtime_valid_script_compiles_and_verifies() {
    let ast = RhaiRuntime::new().compile(VALID_SOURCE).expect("compile");
    assert!(verify_evaluate_entry_point(&ast).is_ok());
}
#[test]
fn test_runtime_wrong_arity_fails() {
    let ast = RhaiRuntime::new()
        .compile("// @acorn-exec-v0.2.0\nfn evaluate(a, b) {}")
        .expect("compile");
    assert!(verify_evaluate_entry_point(&ast).is_err());
}
#[test]
fn test_version_missing_header_fails() {
    assert!(parse_version_header("fn evaluate(context) {}").is_err());
}
#[test]
fn test_version_valid_header_parses() {
    assert_eq!(parse_version_header(VALID_SOURCE).unwrap(), SUPPORTED_EXEC_VERSION);
}
#[test]
fn test_version_wrong_version_fails() {
    assert!(parse_version_header("// @acorn-exec-v2.0.0\nfn evaluate(context) {}").is_err());
}
