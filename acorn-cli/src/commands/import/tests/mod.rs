#![allow(clippy::unwrap_used)]
use super::spec::endpoint_from_import;

const SPEC: &str = r#"
openapi: 3.1.0
paths:
  /widgets:
    get:
      operationId: listWidgets
"#;

#[test]
fn test_endpoint_from_import_applies_metadata() {
    let endpoint = endpoint_from_import(
        &Some("widgets::api".to_string()),
        &Some("api.example.com".to_string()),
        &Some("v1".to_string()),
        &Some("token".to_string()),
        SPEC,
    )
    .unwrap();
    assert_eq!(endpoint.name, "widgets::api");
    assert_eq!(endpoint.domain, "api.example.com");
    assert_eq!(endpoint.root, Some("v1".to_string()));
    assert_eq!(endpoint.resources.len(), 1);
    assert!(endpoint.authentication.is_some());
}
#[test]
fn test_endpoint_from_import_requires_name() {
    let result = endpoint_from_import(&None, &Some("api.example.com".to_string()), &None, &None, SPEC);
    assert!(result.is_err());
}
#[test]
fn test_endpoint_from_import_requires_domain() {
    let result = endpoint_from_import(&Some("widgets::api".to_string()), &None, &None, &None, SPEC);
    assert!(result.is_err());
}
