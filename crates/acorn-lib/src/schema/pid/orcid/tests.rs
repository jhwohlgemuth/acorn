#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::schema::pid::{PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "https://orcid.org/0000-0002-2057-9115";
const CONTRACT_INVALID: &str = "0000-0002-2057-9114";
const CONTRACT_VALID: &str = "0000-0002-2057-9115";
const MOD_11_2_CASES: [(&str, char); 6] = [
    ("000000031415926", '9'),
    ("0000-0002-2057-911", '5'),
    ("0000-0002-2057-9115", '5'),
    ("0000-0002-1823-1234", '2'),
    ("0000-0001-9034-3389", '9'),
    ("0000-0002-2816-415X", 'X'),
];

#[test]
fn test_idutils_orcid_cases() {
    [
        "0000000218250097",
        "http://orcid.org/0000-0002-1825-0097",
        "https://orcid.org/0000-0002-1825-0097",
        "0000-0002-1694-233X",
        "0009-0005-6000-7479",
        "https://orcid.org/0009-0002-4767-9017",
    ]
    .into_iter()
    .for_each(|value| assert!(ORCID::is_valid(value), "expected valid ORCID: {value}"));
}
#[test]
fn test_mod_11_2_check_digit() {
    MOD_11_2_CASES
        .into_iter()
        .for_each(|(value, expected)| assert_eq!(mod_11_2_check_digit(value), Some(vec![expected])));
}
#[test]
fn test_orcid() {
    let valid = "https://orcid.org/0000-0002-2057-9115";
    assert!(valid.is_pid(PID::ORCID));
    assert_eq!(valid.to_pid(PID::ORCID).to_orcid().to_string(), "https://orcid.org/0000-0002-2057-9115");
    assert!(valid.is_orcid());
    assert!(valid.format_as(PID::ORCID).is_orcid());
    assert_eq!(valid.format_as(PID::ORCID), valid);
    assert_eq!("0000-0002-2057-9115".format_as(PID::ORCID), valid);
    assert_eq!("0000000220579115".format_as(PID::ORCID), valid);
}
#[test]
fn test_orcid_find_all() {
    let valid = "https://orcid.org/0000-0002-2057-9115";
    let text = format!("I made a thing with a person with ORCiD {valid}.");
    let results = ORCID::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), "https://orcid.org/0000-0002-2057-9115");
    let text = format!("The ORCID for Jason is: {valid}.");
    let results = ORCID::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), "https://orcid.org/0000-0002-2057-9115");
    let text = format!("I worked with people with orcids {valid}, {valid} and {valid}. Please check them out.");
    let results = ORCID::find_all(&text);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].to_string(), valid);
    assert_eq!(results[1].to_string(), valid);
    assert_eq!(results[2].to_string(), valid);
    let text = format!(
        r#"I know people with ORCiDs
- {valid},
- 0000000220579115
- {valid} (words about something)

Please check them out!"#
    );
    let results = ORCID::find_all(&text);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].to_string(), valid);
    assert_eq!(results[1].to_string(), valid);
    assert_eq!(results[2].to_string(), valid);
}
#[test]
fn test_orcid_metadatatools_cases() {
    [
        "https://orcid.org/0000-0003-0900-6903",
        "0000-0002-6539-638X",
        "0000-0001-9266-5146",
        "0000-0002-0026-2516",
    ]
    .into_iter()
    .for_each(|value| assert!(ORCID::is_valid(value), "expected valid ORCID: {value}"));
}
#[test]
fn test_orcid_struct() {
    let valid = "https://orcid.org/0000-0002-2057-9115";
    assert!(ORCID::is_valid(valid));
    assert!(!ORCID::is_valid("Invalid ORCID value"));
    let pid = ORCID::from_string(valid);
    assert_eq!(pid.to_string(), valid);
    assert_eq!(ORCID::from_string("0000-0002-2057-9115").to_string(), valid);
    assert_eq!(ORCID::from_string("invalid ORCID value").to_string(), "");
}
#[test]
fn test_orcid_trait_contract() {
    assert_eq!(ORCID::new().to_string(), ORCID::default().to_string());
    assert!(ORCID::is_valid(CONTRACT_VALID));
    assert!(!ORCID::is_valid(CONTRACT_INVALID));
    assert_eq!(ORCID::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = ORCID::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "");
    assert_eq!(pid.identifier(), CONTRACT_VALID);
    assert!(pid.prefix().is_none());
    assert_eq!(pid.suffix(), Some(CONTRACT_VALID.to_string()));
    assert_eq!(pid.check_digit(), Some(vec!['5']));
    assert!(pid.url().is_empty());
    let found = <ORCID as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::ORCID));
    assert_eq!(CONTRACT_VALID.format_as(PID::ORCID), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::ORCID).to_orcid().to_string(), CONTRACT_CANONICAL);
}
