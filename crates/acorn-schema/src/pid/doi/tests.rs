#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::pid::{PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "10.1000/182";
const CONTRACT_INVALID: &str = "invalid DOI value";
const CONTRACT_VALID: &str = "https://doi.org/10.1000/182";

#[test]
fn test_doi() {
    let valid = "https://doi.org/10.11578/dc.20250604.1";
    assert_eq!(valid.to_pid(PID::DOI).to_doi().to_string(), "10.11578/dc.20250604.1");
    assert!(valid.is_pid(PID::DOI));
    assert!(valid.is_doi());
    assert_eq!(valid.format_as(PID::DOI), "10.11578/dc.20250604.1");
    let text = format!("I made a thing with doi {valid}.");
    let results = DOI::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), "10.11578/dc.20250604.1");
    let text = format!("The DOI for ACORN is: {valid}.");
    let results = DOI::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), "10.11578/dc.20250604.1");
    let text = format!("I made a thing with dois {valid}, {valid} and {valid}. Please check it out.");
    let results = DOI::find_all(&text);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].to_string(), "10.11578/dc.20250604.1");
    assert_eq!(results[1].to_string(), "10.11578/dc.20250604.1");
    assert_eq!(results[2].to_string(), "10.11578/dc.20250604.1");
    let text = format!(
        r#"I made a thing with dois
- {valid},
- https://doi.org/10.11578/dc.20250604.1
- {valid} (words about something)

Please check it out!"#
    );
    let results = DOI::find_all(&text);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].to_string(), "10.11578/dc.20250604.1");
    assert_eq!(results[1].to_string(), "10.11578/dc.20250604.1");
    assert_eq!(results[2].to_string(), "10.11578/dc.20250604.1");
    let results = DOI::find_all("[DOI](https://doi.org/10.1234/example) 10.1234/(balanced)");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].to_string(), "10.1234/example");
    assert_eq!(results[1].to_string(), "10.1234/(balanced)");
    let valid = "10.11578/dc.20250604.1";
    assert!(valid.is_pid(PID::DOI));
    assert!(valid.is_doi());
    assert_eq!(valid.format_as(PID::DOI), "10.11578/dc.20250604.1");
}
#[test]
fn test_doi_metadatatools_cases() {
    [
        "10.22002/bv2pv-2b295",
        "https://doi.org/10.22002/bv2pv-2b295",
        "HTTPS://DOI.ORG/10.22002/BV2PV-2B295",
        "https://doi.org/10.22002/1gffr-va537",
        "https://doi.org/10.22002/0c391-eeh07",
    ]
    .into_iter()
    .for_each(|value| assert!(DOI::is_valid(value), "expected valid DOI: {value}"));
    ["", "not-a-doi", "10.123/abc"]
        .into_iter()
        .for_each(|value| assert!(!DOI::is_valid(value), "expected invalid DOI: {value}"));
    assert_eq!(DOI::format("  HTTPS://DOI.ORG/10.22002/BV2PV-2B295  "), "10.22002/BV2PV-2B295");
}
#[test]
fn test_doi_struct() {
    let valid = "https://doi.org/10.1000/182";
    assert!(DOI::is_valid(valid));
    assert!(DOI::is_valid("10.1000/182"));
    assert!(!DOI::is_valid("invalid DOI URL"));
    let pid = DOI::from_string(valid);
    assert!(pid.check_digit().is_none());
    assert_eq!(pid.to_string(), "10.1000/182");
    assert_eq!(DOI::from_string("10.1000/182").to_string(), "10.1000/182");
    assert_eq!(DOI::from_string("invalid DOI URL").to_string(), "");
    assert_eq!(DOI::format(valid), "10.1000/182");
    assert_eq!(DOI::format("10.1000/182"), "10.1000/182");
    assert_eq!(DOI::format("invalid DOI value"), "");
}
#[test]
fn test_doi_trait_contract() {
    assert_eq!(DOI::new().to_string(), DOI::default().to_string());
    assert!(DOI::is_valid(CONTRACT_VALID));
    assert!(!DOI::is_valid(CONTRACT_INVALID));
    assert_eq!(DOI::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = DOI::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "https://doi.org");
    assert_eq!(pid.identifier(), CONTRACT_CANONICAL);
    assert_eq!(pid.prefix(), Some("10.1000".to_string()));
    assert_eq!(pid.suffix(), Some("182".to_string()));
    assert!(pid.check_digit().is_none());
    assert_eq!(pid.url(), CONTRACT_VALID);
    let found = <DOI as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::DOI));
    assert_eq!(CONTRACT_VALID.format_as(PID::DOI), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::DOI).to_doi().to_string(), CONTRACT_CANONICAL);
}
#[test]
fn test_doi_url() {
    let doi = DOI::from_string("https://doi.org/10.1000/182");
    assert_eq!(doi.url(), "https://doi.org/10.1000/182");
    let doi = DOI::from_string("10.1000/182");
    assert_eq!(doi.url(), "https://doi.org/10.1000/182");
    let doi = DOI::from_string("https://doi.org/10.11578/dc.20250604.1");
    assert_eq!(doi.url(), "https://doi.org/10.11578/dc.20250604.1");
    let empty_doi = DOI::new();
    assert_eq!(empty_doi.url(), "");
}
#[test]
fn test_idutils_doi_cases() {
    [
        "10.1016/j.epsl.2011.11.037",
        "doi:10.1016/j.epsl.2011.11.037",
        "doi: 10.1016/j.epsl.2011.11.037",
        "DOI:10.1016/j.epsl.2011.11.037",
        "http://dx.doi.org/10.1016/j.epsl.2011.11.037",
        "https://doi.org/10.1016/j.epsl.2011.11.037",
        "doi.org/10.1016/j.epsl.2011.11.037",
        "10.1016/üникóδé-дôΐ",
        "10.1016/སྦ་བཞེད་",
        "10.1002/(SICI)1521-3978(199806)46:4/5<493::AID-PROP493>3.0.CO;2-P",
        "10.1000/123456",
        "10.1038/issn.1476-4687",
        "10.1000.10/123456",
    ]
    .into_iter()
    .for_each(|value| assert!(DOI::is_valid(value), "expected valid DOI: {value}"));
    ["10.1000/", "10.10O0/123456", "10.1.NOTGOOD.0/123456"]
        .into_iter()
        .for_each(|value| assert!(!DOI::is_valid(value), "expected invalid DOI: {value}"));
}
