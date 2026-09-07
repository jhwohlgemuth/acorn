use super::*;
use crate::pid::{Identifier, PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "10013/epic.10033";
const CONTRACT_INVALID: &str = "invalid Handle value";
const CONTRACT_VALID: &str = "https://hdl.handle.net/10013/epic.10033";

#[test]
fn test_handle_find_all_requires_explicit_form() {
    let found = Handle::find_all("See hdl: 10013/epic.10033. Ignore docs/index.md.");
    assert_eq!(found, vec![Handle::from_string("10013/epic.10033")]);
}
#[test]
fn test_handle_pid_integration_and_doi_precedence() {
    let handle = Handle::from_string("https://hdl.handle.net/10013/epic.10033");
    let identifier: Identifier = handle.clone().into();
    assert_eq!(identifier.kind, PID::Handle);
    assert_eq!(identifier.value, "10013/epic.10033");
    assert!(identifier.value.is_handle());
    assert_eq!(identifier.value.to_pid(PID::Handle).to_handle(), handle);
    assert_eq!(serde_json::to_string(&PID::Handle).unwrap(), r#""handle""#);
    assert_eq!(serde_json::from_str::<PID>(r#""HANDLE""#).unwrap(), PID::Handle);
    assert!(PID::Handle.is_discoverable());
    assert!(!PID::Handle.is_project_identifier());
    assert_eq!(Identifier::new("10.1234/ABC").normalized().unwrap().kind, PID::DOI);
}
#[test]
fn test_handle_preserves_case_unicode_and_components() {
    let handle = Handle::from_string("hdl:20.5000/Case/üникóδé");
    assert_eq!(handle.prefix(), Some("20.5000".to_string()));
    assert_eq!(handle.suffix(), Some("Case/üникóδé".to_string()));
    assert_eq!(handle.to_string(), "20.5000/Case/üникóδé");
    assert_eq!(Handle::format(handle.url()), handle.to_string());
}
#[test]
fn test_handle_rejects_invalid_values() {
    [
        "",
        "10013",
        "/suffix",
        "10013/",
        "10..123/name",
        "docs/index.md",
        "https://example.org/10013/name",
        "swh:1/cnt",
    ]
    .into_iter()
    .for_each(|value| assert!(!Handle::is_valid(value), "{value}"));
}
#[test]
fn test_handle_trait_contract() {
    assert_eq!(Handle::new().to_string(), Handle::default().to_string());
    assert!(Handle::is_valid(CONTRACT_VALID));
    assert!(!Handle::is_valid(CONTRACT_INVALID));
    assert_eq!(Handle::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = Handle::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "https://hdl.handle.net");
    assert_eq!(pid.identifier(), CONTRACT_CANONICAL);
    assert_eq!(pid.prefix(), Some("10013".to_string()));
    assert_eq!(pid.suffix(), Some("epic.10033".to_string()));
    assert!(pid.check_digit().is_none());
    assert_eq!(pid.url(), CONTRACT_VALID);
    let found = <Handle as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::Handle));
    assert_eq!(CONTRACT_VALID.format_as(PID::Handle), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::Handle).to_handle().to_string(), CONTRACT_CANONICAL);
}
#[test]
fn test_handle_uri_ignores_query_and_fragment() {
    [
        "hdl:10013/epic.10033?locatt=view:level2",
        "https://hdl.handle.net/10013/epic.10033#metadata",
    ]
    .into_iter()
    .for_each(|value| assert_eq!(Handle::format(value), "10013/epic.10033", "{value}"));
}
#[test]
fn test_idutils_handle_vectors() {
    [
        "10013/epic.10033",
        "hdl:10013/epic.10033",
        "hdl: 10013/epic.10033",
        "HDL:10013/epic.10033",
        "hdl.handle.net/10013/epic.10033",
        "http://hdl.handle.net/10013/epic.10033",
        "https://hdl.handle.net/10013/epic.10033",
    ]
    .into_iter()
    .for_each(|value| assert_eq!(Handle::format(value), "10013/epic.10033", "{value}"));
}
