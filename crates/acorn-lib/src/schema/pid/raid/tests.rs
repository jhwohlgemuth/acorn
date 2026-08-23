#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::prelude::PathBuf;
use crate::schema::pid::{PersistentIdentifierConvert, PID};
use crate::schema::Validate;

const CONTRACT_CANONICAL: &str = "10.83962/fb5be317";
const CONTRACT_INVALID: &str = "11.83962/fb5be317";
const CONTRACT_VALID: &str = "https://raid.org/10.83962/fb5be317";
const FIXTURES: &str = "../../tests/fixtures";

#[test]
fn test_idutils_raid_cases() {
    [
        "10.26259/0e59e9a5",
        "10.83962/fb5be317",
        "https://raid.org/10.26259/0e59e9a5",
        "https://raid.org/10.83962/f2a7645d",
        "HTTPS://RAID.ORG/10.26259/0E59E9A5",
    ]
    .into_iter()
    .for_each(|value| assert!(RAID::is_valid(value), "expected valid RAiD: {value}"));
    ["", "not-a-raid", "11.26259/0e59e9a5", "102.26259/0e59e9a5"]
        .into_iter()
        .for_each(|value| assert!(!RAID::is_valid(value), "expected invalid RAiD: {value}"));
}
#[test]
fn test_raid() {
    let valid = "https://raid.org/10.83962/fb5be317";
    let pid = RAID::from_string(valid);
    assert_eq!(pid.to_string(), "10.83962/fb5be317");
    assert_eq!(pid.schema_uri(), "https://raid.org");
    assert_eq!(valid.to_pid(PID::RAID).to_raid().to_string(), "10.83962/fb5be317");
    assert!(valid.is_pid(PID::RAID));
    assert!(RAID::is_valid(valid));
    assert_eq!(pid.to_string(), "10.83962/fb5be317");
    assert_eq!(valid.format_as(PID::RAID), "10.83962/fb5be317");
    assert_eq!(RAID::format("10.1000/182"), "10.1000/182");
    assert_eq!(RAID::format("https://raid.org/10.1000/182"), "10.1000/182");
}
#[test]
fn test_raid_contributor_accepts_orcid_and_isni() {
    let contributor = |id: &str, schema_uri: &str| crate::schema::pid::raid::Contributor {
        id: Some(id.to_string()),
        schema_uri: Some(schema_uri.to_string()),
        status: None,
        status_message: None,
        position: Vec::new(),
        leader: false,
        contact: false,
        email: None,
        role: None,
        uuid: None,
    };
    assert!(contributor("https://orcid.org/0000-0002-2057-9115", "https://orcid.org/")
        .validate()
        .is_ok());
    assert!(contributor("https://isni.org/isni/0000000492299539", "https://isni.org/")
        .validate()
        .is_ok());
    assert!(contributor("https://isni.org/isni/0000000492299538", "https://isni.org/")
        .validate()
        .is_err());
    assert!(contributor("https://isni.org/isni/0000000492299539", "https://orcid.org/")
        .validate()
        .is_err());
    assert!(contributor("https://isni.org/isni/0000000492299539", "https://example.org/")
        .validate()
        .is_err());
}
#[test]
fn test_raid_metadatatools_cases() {
    [
        "10.26259/0e59e9a5",
        "10.83962/fb5be317",
        "https://raid.org/10.26259/0e59e9a5",
        "https://raid.org/10.83962/f2a7645d",
        "HTTPS://RAID.ORG/10.26259/0E59E9A5",
    ]
    .into_iter()
    .for_each(|value| assert!(RAID::is_valid(value), "expected valid RAiD: {value}"));
    ["", "not-a-raid", "10x26259/0e59e9a5", "11.26259/0e59e9a5", "102.26259/0e59e9a5"]
        .into_iter()
        .for_each(|value| assert!(!RAID::is_valid(value), "expected invalid RAiD: {value}"));
    assert_eq!(RAID::format("  HTTPS://RAID.ORG/10.26259/0E59E9A5  "), "10.26259/0E59E9A5");
}
#[test]
fn test_raid_read() {
    let path = PathBuf::from(FIXTURES).join("raid/response_01.json");
    let data = Metadata::read(path);
    assert!(data.is_ok());
    let _ = data.unwrap().validate();
    let path = PathBuf::from(FIXTURES).join("raid/response_02.json");
    let data = Metadata::read(path);
    assert!(data.is_ok());
    let _ = data.unwrap().validate();
}
#[test]
fn test_raid_trait_contract() {
    assert_eq!(RAID::new().to_string(), RAID::default().to_string());
    assert!(RAID::is_valid(CONTRACT_VALID));
    assert!(!RAID::is_valid(CONTRACT_INVALID));
    assert_eq!(RAID::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = RAID::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "https://raid.org");
    assert_eq!(pid.identifier(), CONTRACT_CANONICAL);
    assert_eq!(pid.prefix(), Some("10.83962".to_string()));
    assert_eq!(pid.suffix(), Some("fb5be317".to_string()));
    assert!(pid.check_digit().is_none());
    assert!(pid.url().is_empty());
    let found = <RAID as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::RAID));
    assert_eq!(CONTRACT_VALID.format_as(PID::RAID), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::RAID).to_raid().to_string(), CONTRACT_CANONICAL);
}
