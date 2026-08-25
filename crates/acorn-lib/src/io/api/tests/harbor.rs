use crate::io::api::harbor::{endpoint, params, Options};
use crate::io::api::{render, IntoHeaders, RemoteResource};
use crate::io::config::{RegistryKind, RegistryProfile};
use crate::param;
use temp_env::with_var;
use urlencoding::encode;

fn profile() -> RegistryProfile {
    RegistryProfile {
        kind: RegistryKind::Harbor,
        endpoint: "http://127.0.0.1:5000".to_string(),
        plain_http: true,
        ..Default::default()
    }
}
#[test]
fn test_splits_nested_repository_and_encodes_path_segments() {
    let profile = profile();
    let options = Options::from_repository(&profile, "ai/models/qwen").unwrap();
    assert_eq!(options.project, "ai");
    assert_eq!(encode(options.repository), "models%2Fqwen");
    assert_eq!(encode("sha256:abc"), "sha256%3Aabc");
}
#[test]
fn test_rejects_repository_without_project() {
    assert!(Options::from_repository(&profile(), "qwen").is_err());
}
#[test]
fn test_renders_artifact_requests_with_shared_api_resources() {
    let endpoint = endpoint(&profile()).unwrap();
    let params = vec![
        param!(TemplateValue, "project", "ai"),
        param!(TemplateValue, "repository", "models%2Fqwen"),
        param!(TemplateValue, "reference", "v1"),
    ];
    let context = endpoint.context(Some(params));
    let request = endpoint
        .resources
        .iter()
        .find(|resource| resource.name == "artifact")
        .map(|resource| render(&resource.template, &context));
    assert_eq!(
        request.as_deref(),
        Some("http://127.0.0.1:5000/api/v2.0/projects/ai/repositories/models%2Fqwen/artifacts/v1")
    );
}
#[test]
fn test_uses_basic_auth_when_profile_has_a_username() {
    with_var("ACORN_TEST_HARBOR_SECRET", Some("secret"), || {
        let profile = RegistryProfile {
            credential_env: Some("ACORN_TEST_HARBOR_SECRET".to_string()),
            username: Some("robot".to_string()),
            ..profile()
        };
        let headers = params(&profile, Vec::new()).unwrap().into_headers();
        assert_eq!(
            headers.get("authorization").and_then(|value| value.to_str().ok()),
            Some("Basic cm9ib3Q6c2VjcmV0")
        );
    });
}
