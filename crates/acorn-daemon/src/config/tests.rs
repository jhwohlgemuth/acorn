use super::*;
use std::fs::write;
use tempfile::TempDir;

fn job_jsonc(root: &Path, id: &str, input: &str, triggers: &str) -> String {
    let root = root.to_string_lossy().replace('\\', "/");
    format!(
        r#"{{
            "schemaVersion": "2.0.0",
            "daemon": {{
                "jobs": [{{
                    "id": "{id}",
                    "root": "{root}",
                    "input": "{input}",
                    "kind": "check",
                    "triggers": {triggers}
                }}]
            }}
        }}"#
    )
}
fn load_text(temp: &TempDir, source: &str) -> Result<DaemonConfig, ConfigError> {
    let path = temp.path().join(".acorn.jsonc");
    write(&path, source).expect("write daemon configuration fixture");
    DaemonConfig::load(path)
}
fn project() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("create daemon configuration fixture directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("create fixture project root");
    write(root.join("input.json"), "{}").expect("write fixture project input");
    (temp, root)
}

#[test]
fn test_config_accepts_defaults_and_valid_job() {
    let (temp, root) = project();
    let config = load_text(&temp, &job_jsonc(&root, "daily-check", "input.json", r#"{"onChange":true}"#)).expect("valid daemon configuration");
    assert_eq!(config.bind().grpc(), Some("127.0.0.1:50051".parse().expect("default gRPC address")));
    assert_eq!(config.bind().rpc(), Some("127.0.0.1:8001".parse().expect("default RPC address")));
    let job = config.jobs().first().expect("configured daemon job");
    assert_eq!(job.id().as_str(), "daily-check");
    assert!(job.schedule().on_change());
}
#[test]
fn test_config_rejects_duplicate_ids() {
    let (temp, root) = project();
    let root = root.to_string_lossy().replace('\\', "/");
    let source = format!(
        r#"{{"schemaVersion":"2.0.0","daemon":{{"jobs":[
            {{"id":"duplicate","root":"{root}","input":"input.json","kind":"check","triggers":{{"onChange":true}}}},
            {{"id":"duplicate","root":"{root}","input":"input.json","kind":"validate","triggers":{{"everySeconds":60}}}}
        ]}}}}"#
    );
    assert!(matches!(load_text(&temp, &source), Err(ConfigError::Validation(_))));
}
#[test]
fn test_config_rejects_input_failures() {
    let (temp, root) = project();
    for input in ["missing.json", "https://example.org/input.json", "-", "../input.json"] {
        assert!(
            load_text(&temp, &job_jsonc(&root, "input-check", input, r#"{"onChange":true}"#)).is_err(),
            "input {input}"
        );
    }
}
#[test]
fn test_config_rejects_invalid_roots() {
    let temp = tempfile::tempdir().expect("create daemon configuration fixture directory");
    let relative = Path::new("relative-root");
    assert!(matches!(
        load_text(&temp, &job_jsonc(relative, "root-check", "input.json", r#"{"onChange":true}"#)),
        Err(ConfigError::Validation(_))
    ));
    let missing = temp.path().join("missing-root");
    assert!(matches!(
        load_text(&temp, &job_jsonc(&missing, "root-check", "input.json", r#"{"onChange":true}"#)),
        Err(ConfigError::InvalidRoot { .. })
    ));
}
#[test]
fn test_config_rejects_missing_triggers_and_zero_interval() {
    let (temp, root) = project();
    assert!(matches!(
        load_text(&temp, &job_jsonc(&root, "trigger-check", "input.json", "{}")),
        Err(ConfigError::Source { .. })
    ));
    assert!(load_text(&temp, &job_jsonc(&root, "trigger-check", "input.json", r#"{"everySeconds":0}"#)).is_err());
}
#[test]
fn test_config_rejects_no_listener() {
    let temp = tempfile::tempdir().expect("create daemon configuration fixture directory");
    let source = r#"{"schemaVersion":"2.0.0","daemon":{"bind":{"grpc":null,"rpc":null}}}"#;
    assert!(matches!(load_text(&temp, source), Err(ConfigError::Validation(_))));
}
#[cfg(unix)]
#[test]
fn test_config_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let (temp, root) = project();
    let outside = temp.path().join("outside.json");
    write(&outside, "{}").expect("write outside fixture input");
    symlink(&outside, root.join("linked.json")).expect("create escaping input symlink");
    assert!(matches!(
        load_text(&temp, &job_jsonc(&root, "escape-check", "linked.json", r#"{"onChange":true}"#)),
        Err(ConfigError::InputEscape { .. })
    ));
}
#[test]
fn test_config_rejects_unsupported_version_and_unknown_fields() {
    let temp = tempfile::tempdir().expect("create daemon configuration fixture directory");
    assert!(matches!(
        load_text(&temp, r#"{"schemaVersion":"1.0.0","daemon":{}}"#),
        Err(ConfigError::Source { .. })
    ));
    assert!(load_text(&temp, r#"{"schemaVersion":"2.0.0","unknown":true,"daemon":{}}"#).is_err());
}
#[test]
fn test_config_supports_listener_disablement_and_tls() {
    let temp = tempfile::tempdir().expect("create daemon configuration fixture directory");
    let source = r#"{
        "schemaVersion": "2.0.0",
        "daemon": {
            "bind": {"grpc": null, "rpc": "127.0.0.1:9001"},
            "tls": {"cert": "cert.pem", "key": "key.pem"}
        }
    }"#;
    let config = load_text(&temp, source).expect("valid explicit listener configuration");
    assert_eq!(config.bind().grpc(), None);
    assert_eq!(config.bind().rpc(), Some("127.0.0.1:9001".parse().expect("valid fixture socket address")));
    let tls = config.tls().expect("configured TLS paths");
    assert_eq!(tls.cert(), Path::new("cert.pem"));
    assert_eq!(tls.key(), Path::new("key.pem"));
}
