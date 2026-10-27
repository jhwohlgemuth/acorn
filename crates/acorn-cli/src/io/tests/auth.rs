use super::super::auth::config::{resolve, resolve_source};
use std::fs::write;

#[test]
fn test_resolve_auth_profile_preserves_order_and_explicit_path() {
    let directory = tempfile::tempdir().expect("temporary ACRE directory");
    let path = directory.path().join("custom.json");
    write(&path, include_str!("../../../../../tests/fixtures/oauth2/config/ordered.json")).expect("fixture file");
    let resolved = resolve(&Some(path.clone()), None).expect("default profile");
    assert_eq!(resolved.profile_name, "zeta");
    assert_eq!(resolved.configuration_path, path);
    assert!(resolved.profile.allows(&resolved.request));
    assert_eq!(resolve(&Some(path.clone()), Some("alpha")).expect("named profile").profile_name, "alpha");
    assert!(resolve(&Some(path), Some("missing")).is_err());
    assert!(resolve(&Some(directory.path().join("missing.json")), None).is_err());
}
#[test]
fn test_resolve_auth_source_uses_its_named_profile() {
    let directory = tempfile::tempdir().expect("temporary ACRE directory");
    let path = directory.path().join("custom.json");
    let fixture = include_str!("../../../../../tests/fixtures/oauth2/config/ordered.json");
    let source = r#""sources": [{"id":"combined","provider":"sharepoint","profile":"alpha","resource":"graph","host":"example.sharepoint.com","sitePath":"/sites/Example","libraryPath":"/Library","itemPath":"logbook.jsonl","scopes":["Files.Read"]}]"#;
    write(&path, fixture.replace("\"sources\": []", source)).expect("source fixture");
    let resolved = resolve_source(&Some(path), "https://example.sharepoint.com/sites/Example/Library/logbook.jsonl").expect("configured source");
    assert_eq!(resolved.profile_name, "alpha");
    assert_eq!(resolved.request.resource.scopes(), &["Files.Read"]);
}
