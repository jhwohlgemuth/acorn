use super::config::ExecutionOrigin;
use super::parse_version_header;
use super::result::{ExecutionResult, RhaiExt};
use super::runtime::{verify_evaluate_entry_point, RhaiRuntime, Runtime};
use crate::analyzer::{Check, CheckCategory, CheckSeverity, Finding, Findings};
use crate::io::config::application::{ApplicationConfiguration, Entry, Hook};
use crate::io::workflow::{Change, WorkflowAction};
use acorn_core::util::constants::app::{MAX_EXEC_BYTES, SUPPORTED_EXEC_VERSION};
use acorn_core::util::MimeType;
use acorn_core::Location;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::validation::Validate as SchemaValidate;
use core::str::FromStr;
use std::fs::{create_dir, write};
use std::path::Path;
use tempfile::tempdir;
use validator::Validate;

const VALID_SOURCE: &str = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: []} }";

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

#[test]
fn application_configuration_has_hook() {
    let configuration = ApplicationConfiguration {
        scripts: Some(vec![Entry {
            path: "quality".to_string(),
            hooks: vec![Hook::Check, Hook::Format],
            absolute: false,
        }]),
        ..ApplicationConfiguration::default()
    };
    assert!(configuration.has_hook(Hook::Check));
    assert!(configuration.has_hook(Hook::Format));
    assert!(!configuration.has_hook(Hook::Link));
}
#[test]
fn application_configuration_validates_entries_and_duplicate_paths() {
    let configuration = ApplicationConfiguration {
        scripts: Some(vec![
            Entry {
                path: "quality".to_string(),
                hooks: vec![Hook::Check],
                absolute: false,
            },
            Entry {
                path: "quality".to_string(),
                hooks: vec![Hook::Format],
                absolute: false,
            },
        ]),
        ..ApplicationConfiguration::default()
    };
    let report = SchemaValidate::validate(&configuration).expect_err("duplicate paths fail validation");
    assert!(report.iter().any(|issue| issue.path == "scripts[1].path"));
}
#[test]
fn check_safe_missing_header_fails() {
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), "fn evaluate(context) {}")
        .iter()
        .any(Check::is_failure));
}
#[test]
fn check_safe_oversized_fails() {
    let source = format!("// @acorn-exec-v0.2.0\nfn evaluate(context) {{}}\n{}", "x".repeat(MAX_EXEC_BYTES));
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), &source).iter().any(Check::is_failure));
}
#[test]
fn check_safe_valid_source_passes() {
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), VALID_SOURCE)
        .iter()
        .all(|check| !check.is_failure()));
}
#[test]
fn check_safe_wrong_arity_fails() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(a,b) {}";
    assert!(RhaiRuntime::check_safe(Path::new("test.rhai"), source).iter().any(Check::is_failure));
}
#[test]
fn config_entry_validation_duplicate_hooks() {
    let entry = Entry {
        path: "foo".to_string(),
        hooks: vec![Hook::Check, Hook::Check],
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn config_entry_validation_empty_hooks() {
    let entry = Entry {
        path: "foo".to_string(),
        hooks: vec![],
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn config_entry_validation_parent_traversal() {
    let entry = Entry {
        path: "../escape".to_string(),
        hooks: vec![Hook::Check],
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn config_entry_validation_repeated_root() {
    let entry = Entry {
        path: ".acorn/quality".to_string(),
        hooks: vec![Hook::Check],
        absolute: false,
    };
    assert!(SchemaValidate::validate(&entry).is_err());
}
#[test]
fn config_hook_parse_invalid() {
    assert!(Hook::from_str("gather").is_err());
}
#[test]
fn config_hook_parse_valid() {
    assert_eq!(Hook::from_str("check").unwrap(), Hook::Check);
}
#[test]
fn configured_scripts_compile_completely_before_evaluation() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    write(script_root.join("first.rhai"), "// @acorn-exec-v0.2.0\nfn evaluate(context) { loop {} }").expect("first script");
    write(script_root.join("second.rhai"), "// @acorn-exec-v0.2.0\nfn evaluate(context) {").expect("second script");
    let configuration = ApplicationConfiguration {
        scripts: Some(vec![
            Entry {
                path: "first".to_string(),
                hooks: vec![Hook::Check],
                absolute: false,
            },
            Entry {
                path: "second".to_string(),
                hooks: vec![Hook::Check],
                absolute: false,
            },
        ]),
        ..ApplicationConfiguration::default()
    };
    let changes = vec![Change::Document {
        source: Location::from("activity.json"),
        data: Box::new(ResearchActivity::default()),
        content: String::new(),
        changed: false,
        linked: None,
        failures: Vec::new(),
        conflicts: Vec::new(),
    }];
    let error = configuration
        .check_scripts(&changes, temporary.path(), WorkflowAction::Check, ExecutionOrigin::Local)
        .expect_err("invalid second script must fail before the first script evaluates");
    assert!(error.to_string().contains("second.rhai` compile error"));
}
#[test]
fn configured_scripts_deny_non_local_callers() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    write(script_root.join("quality.rhai"), VALID_SOURCE).expect("script");
    let entry = Entry {
        path: "quality".to_string(),
        hooks: vec![Hook::Check],
        absolute: false,
    };
    let error = entry
        .resolve(temporary.path(), ExecutionOrigin::NonLocal)
        .expect_err("non-local execution must be denied");
    assert!(error.to_string().contains("unavailable to non-local callers"));
}
#[test]
fn configured_scripts_use_immutable_compiled_snapshot() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    let path = script_root.join("quality.rhai");
    write(&path, VALID_SOURCE).expect("script");
    let resolved = Entry {
        path: "quality".to_string(),
        hooks: vec![Hook::Check],
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
fn configured_validate_findings_use_schema_category() {
    let temporary = tempdir().expect("temporary project");
    let script_root = temporary.path().join(".acorn");
    create_dir(&script_root).expect("script directory");
    let source =
        "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [#{rule_id: \"rule-1\", severity: \"warning\", message: context.hook}]} }";
    write(script_root.join("quality.rhai"), source).expect("script");
    let resolved = Entry {
        path: "quality".to_string(),
        hooks: vec![Hook::Validate],
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
fn infer_explicit_rhai_extension() {
    assert_eq!(MimeType::from("script.rhai"), MimeType::Rhai);
}
#[test]
fn infer_extensionless_text_is_not_rhai() {
    assert_eq!(MimeType::infer(b"hello world"), None);
}
#[test]
fn infer_extensionless_valid_rhai_probe() {
    assert_eq!(MimeType::infer(VALID_SOURCE.as_bytes()), Some(MimeType::Rhai));
}
#[test]
fn infer_invalid_entry_is_not_rhai() {
    let source = b"// @acorn-exec-v0.2.0\nfn evaluate(a,b) {}";
    assert_eq!(MimeType::infer(source), None);
}
#[test]
fn module_loads() {
    assert_eq!(SUPPORTED_EXEC_VERSION.to_string(), "0.2.0");
}
#[test]
fn result_empty_rule_fails() {
    assert!(finding("").validate().is_err());
}
#[test]
fn result_rhai_ext_parses_findings() {
    let mut result = rhai::Map::new();
    result.insert("findings".into(), rhai::Dynamic::from(rhai::Array::new()));
    assert!(rhai::Dynamic::from(result).findings().expect("findings").is_empty());
}
#[test]
fn result_rhai_ext_rejects_bare_array() {
    assert!(rhai::Dynamic::from(rhai::Array::new()).findings().is_err());
}
#[test]
fn result_rhai_ext_rejects_invalid_finding_fields() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [#{rule_id: \"rule-1\", severity: \"error\", message: \"msg\", category: \"quality\"}]} }";
    let dynamic = RhaiRuntime::new().check(source, rhai::Map::new()).expect("script evaluates");
    assert_eq!(
        dynamic.findings().expect_err("category is host-owned"),
        "unknown finding field `category`"
    );
}
#[test]
fn result_rhai_ext_rejects_invalid_output() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{output: 1, findings: []} }";
    let dynamic = RhaiRuntime::new().check(source, rhai::Map::new()).expect("script evaluates");
    assert_eq!(
        dynamic.findings().expect_err("quality output must be unit"),
        "quality-check output must be omitted or unit"
    );
}
#[test]
fn result_rhai_ext_rejects_unknown_envelope_fields() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { #{findings: [], extra: true} }";
    let dynamic = RhaiRuntime::new().check(source, rhai::Map::new()).expect("script evaluates");
    assert_eq!(
        dynamic.findings().expect_err("unknown field must fail"),
        "unknown execution result field `extra`"
    );
}
#[test]
fn result_valid_finding_passes() {
    assert!(finding("rule-1").validate().is_ok());
}
#[test]
fn result_validates() {
    let result = ExecutionResult {
        output: (),
        findings: Findings::default(),
    };
    assert!(result.validate().is_ok());
}
#[test]
fn runtime_missing_entry_fails() {
    let ast = RhaiRuntime::new().compile("// @acorn-exec-v0.2.0\nfn other(x) {}").expect("compile");
    assert!(verify_evaluate_entry_point(&ast).is_err());
}
#[test]
fn runtime_operation_limit_stops_infinite_loop() {
    let source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { loop {} }";
    assert!(RhaiRuntime::new().check(source, rhai::Map::new()).is_err());
}
#[test]
fn runtime_valid_script_compiles_and_verifies() {
    let ast = RhaiRuntime::new().compile(VALID_SOURCE).expect("compile");
    assert!(verify_evaluate_entry_point(&ast).is_ok());
}
#[test]
fn runtime_wrong_arity_fails() {
    let ast = RhaiRuntime::new()
        .compile("// @acorn-exec-v0.2.0\nfn evaluate(a, b) {}")
        .expect("compile");
    assert!(verify_evaluate_entry_point(&ast).is_err());
}
#[test]
fn version_missing_header_fails() {
    assert!(parse_version_header("fn evaluate(context) {}").is_err());
}
#[test]
fn version_valid_header_parses() {
    assert_eq!(parse_version_header(VALID_SOURCE).unwrap(), SUPPORTED_EXEC_VERSION);
}
#[test]
fn version_wrong_version_fails() {
    assert!(parse_version_header("// @acorn-exec-v2.0.0\nfn evaluate(context) {}").is_err());
}
