use super::*;
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};

fn portable_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z][a-z0-9_-]{0,7}", 1..5).prop_map(|segments| segments.join("/"))
}
fn supported_uri() -> impl Strategy<Value = String> {
    ("[a-z][a-z0-9]{0,7}", 1_u16..=u16::MAX, portable_path()).prop_flat_map(|(host, port, path)| {
        prop_oneof![
            Just(format!("http://{host}.example.org:{port}/{path}")),
            Just(format!("https://{host}.example.org:{port}/{path}")),
            Just(format!("ssh://git@{host}.example.org:{port}/{path}")),
            Just(format!("file:/{path}")),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/location/tests.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_location_detailed_revision_round_trips_through_serde(uri in supported_uri(), revision in "[A-Za-z0-9._/-]{1,32}") {
        let value = serde_json::json!({"scheme": "https", "uri": uri, "revision": revision});
        let location = serde_json::from_value::<Location>(value).expect("generated detailed location should deserialize");
        let encoded = serde_json::to_value(&location).expect("generated detailed location should serialize");
        let decoded = serde_json::from_value::<Location>(encoded).expect("serialized detailed location should deserialize");
        prop_assert_eq!(decoded, location);
    }
    #[test]
    fn test_location_normalization_is_idempotent(path in portable_path()) {
        let decorated = format!("./{path}/../tail");
        let normalized = Location::from(decorated.as_str()).normalize();
        prop_assert_eq!(Location::from(normalized.as_str()).normalize(), normalized);
    }
    #[test]
    fn test_location_supported_values_round_trip_through_display(value in prop_oneof![portable_path(), supported_uri()]) {
        let location = Location::from(value.as_str());
        prop_assert_eq!(Location::from(location.to_string().as_str()), location);
    }
}
#[test]
fn test_default_repository_is_local_git() {
    assert!(Repository::default().is_local());
}
#[test]
fn test_location_classifies_and_normalizes_portable_paths_and_oci_references() {
    assert!(Location::from("model.gguf").is_path());
    assert!(Location::from("/models/model.gguf").is_absolute());
    assert!(Location::from("C:\\models\\model.gguf").is_absolute());
    assert_eq!(Location::from("docs/../README.md").normalize(), "README.md");
    assert!(Location::from("s3://bucket/key").is_s3());
    assert!(Location::from("registry.example.org/org/model:latest").is_oci_reference());
    assert!(!Location::from("https://example.org/model").is_local());
}
#[test]
fn test_location_classifies_non_loopback_http_locations() {
    assert!(Location::from("https://example.org/data").is_non_loopback_http_or_https());
    assert!(!Location::from("http://localhost/data").is_non_loopback_http_or_https());
    assert!(!Location::from("https://127.0.0.1/data").is_non_loopback_http_or_https());
    assert!(!Location::from("https://127.0.0.2/data").is_non_loopback_http_or_https());
    assert!(!Location::from("https://[::1]/data").is_non_loopback_http_or_https());
    assert!(!Location::from("https://").is_non_loopback_http_or_https());
    assert!(!Location::from("file:///tmp/data").is_non_loopback_http_or_https());
}
#[test]
fn test_location_preserves_registry_ports_and_ipv6() {
    assert_eq!(
        Location::from("https://registry.example.org:5000").registry().as_deref(),
        Some("registry.example.org:5000")
    );
    assert_eq!(
        Location::from("https://[2001:db8::1]:5000").registry().as_deref(),
        Some("[2001:db8::1]:5000")
    );
}
#[test]
fn test_location_preserves_ssh_uri_components_and_serialization() {
    let value = "ssh://git@example.org:2222/group/project.git";
    let location = Location::from(value);
    assert_eq!(location.scheme(), Scheme::SSH);
    assert_eq!(location.uri().as_deref(), Some(value));
    assert_eq!(location.host().as_deref(), Some("example.org"));
    assert_eq!(location.port(), Some(2222));
    assert_eq!(location.path().as_deref(), Some("/group/project.git"));
    let serialized = serde_json::to_string(&location).expect("location serializes");
    let round_trip: Location = serde_json::from_str(&serialized).expect("location deserializes");
    assert_eq!(round_trip, location);
}
#[test]
fn test_location_round_trips_supported_uri_schemes() {
    [
        ("file:///tmp/acorn.json", "file:/tmp/acorn.json", Scheme::File),
        ("http://user@example.org:8080/data", "http://user@example.org:8080/data", Scheme::HTTP),
        ("https://user@example.org:8443/data", "https://user@example.org:8443/data", Scheme::HTTPS),
        (
            "ssh://git@example.org:2222/group/project.git",
            "ssh://git@example.org:2222/group/project.git",
            Scheme::SSH,
        ),
    ]
    .into_iter()
    .for_each(|(value, canonical, scheme)| {
        let location = Location::from(value);
        assert_eq!(location.scheme(), scheme);
        assert_eq!(location.uri().as_deref(), Some(canonical));
    });
    assert_eq!(serde_json::to_string(&Scheme::SSH).expect("serialize SSH scheme"), "\"ssh\"");
    assert_eq!(serde_json::from_str::<Scheme>("\"ssh\"").expect("deserialize SSH scheme"), Scheme::SSH);
}
#[test]
fn test_repository_classification_requires_provider_domain() {
    let github = Repository::from_remote("https://github.com/openai/codex", "gitlab.com").expect("GitHub repository");
    assert!(github.is_git_hub());
    assert_eq!(github.project_path().as_deref(), Some("openai/codex"));
    assert!(Repository::from_remote("https://example.org/group/project", "gitlab.com").is_none());
}
#[test]
fn test_repository_gitlab_builder() {
    let repository = Repository::gitlab().location(Location::from("https://gitlab.com/group/project")).call();
    assert!(repository.is_gitlab());
    assert_eq!(repository.project_path().as_deref(), Some("group/project"));
}
