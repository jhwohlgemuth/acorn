#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::schema::pid::{Identifier, PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "https://isni.org/isni/0000000492299539";
const CONTRACT_INVALID: &str = "0000000492299538";
const CONTRACT_VALID: &str = "https://isni.org/isni/0000000492299539";

#[test]
fn test_idutils_isni_cases() {
    assert!(ISNI::is_valid("1422-4586-3573-0476"));
}
#[test]
fn test_isni() {
    let canonical = "https://isni.org/isni/0000000492299539";
    assert_eq!(PID::from("ISNI"), PID::ISNI);
    assert_eq!(serde_json::to_string(&PID::ISNI).unwrap(), r#""isni""#);
    assert!(canonical.is_pid(PID::ISNI));
    assert!(canonical.is_isni());
    assert_eq!(canonical.to_pid(PID::ISNI).to_isni().to_string(), canonical);
    assert_eq!("0000 0004 9229 9539".format_as(PID::ISNI), canonical);
    assert_eq!("0000-0004-9229-9539".format_as(PID::ISNI), canonical);
    assert_eq!(ISNI::format("http://isni.org/isni/0000000492299539"), canonical);
    assert_eq!(
        Identifier {
            kind: PID::ISNI,
            value: "0000-0004-9229-9539".to_string(),
        }
        .normalized()
        .expect("explicit ISNI should normalize")
        .value,
        canonical
    );
    assert_eq!(mod_11_2_check_digit("0000000492299539"), Some(vec!['9']));
    assert!(!ISNI::is_valid("0000000492299538"));
}
#[test]
fn test_isni_find_all_and_orcid_precedence() {
    let isni = "https://isni.org/isni/0000000492299539";
    let orcid = "https://orcid.org/0000-0002-2057-9115";
    let values = ISNI::find_all(format!("ISNI {isni}; ORCID {orcid}"));
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].to_string(), isni);
    assert!(ORCID::find_all(isni).is_empty());
    let normalized = Identifier::new(orcid).normalized().expect("ORCID should normalize");
    assert_eq!(normalized.kind, PID::ORCID);
}
#[test]
fn test_isni_metadatatools_cases() {
    ["0000 0001 2096 0218", "0000 0000 8405 6132", "0000 0000 7182 7209"]
        .into_iter()
        .for_each(|value| assert!(ISNI::is_valid(value), "expected valid ISNI: {value}"));
}
#[test]
fn test_isni_trait_contract() {
    assert_eq!(ISNI::new().to_string(), ISNI::default().to_string());
    assert!(ISNI::is_valid(CONTRACT_VALID));
    assert!(!ISNI::is_valid(CONTRACT_INVALID));
    assert_eq!(ISNI::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = ISNI::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "https://isni.org/isni");
    assert_eq!(pid.identifier(), "0000000492299539");
    assert!(pid.prefix().is_none());
    assert_eq!(pid.suffix(), Some("0000000492299539".to_string()));
    assert_eq!(pid.check_digit(), Some(vec!['9']));
    assert!(pid.url().is_empty());
    let found = <ISNI as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::ISNI));
    assert_eq!(CONTRACT_VALID.format_as(PID::ISNI), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::ISNI).to_isni().to_string(), CONTRACT_CANONICAL);
}
