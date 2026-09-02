#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use crate::schema::pid::{PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "https://n2t.net/ark:13030/xf93gt2q";
const CONTRACT_INVALID: &str = "invalid ARK value";
const CONTRACT_VALID: &str = "https://n2t.net/ark:/13030/xf93gt2q";

#[test]
fn test_ark() {
    let valid = "https://n2t.net/ark:12148/btv1b8449691v/f29";
    assert!(valid.is_pid(PID::ARK));
    assert_eq!(valid.to_pid(PID::ARK).to_ark().to_string(), "https://n2t.net/ark:12148/btv1b8449691v/f29");
    assert_eq!(
        "ark:12148/btv1b8449691v/f29.pdf".to_pid(PID::ARK).to_ark().to_string(),
        "ark:12148/btv1b8449691v/f29.pdf"
    );
    assert_eq!(
        "ark:12148/btv1b8449691v/f29.pdf.v2".to_pid(PID::ARK).to_ark().to_string(),
        "ark:12148/btv1b8449691v/f29.pdf.v2"
    );
    assert_eq!(
        "ark:12148/btv1b8449691v/f29/abc.pdf.v2".to_pid(PID::ARK).to_ark().to_string(),
        "ark:12148/btv1b8449691v/f29/abc.pdf.v2"
    );
    assert!(valid.is_ark());
    assert_eq!(
        ARK::format("https://n2t.net/ark:12148/btv1b8449691v/f29"),
        "https://n2t.net/ark:12148/btv1b8449691v/f29"
    );
    assert!(valid.format_as(PID::ARK).is_ark());
    assert_eq!(valid.format_as(PID::ARK), valid);
    assert_eq!(valid.to_pid(PID::ARK).to_ark().to_string(), valid);
    let pid = ARK::from_string(valid);
    assert_eq!(pid.schema_uri(), "https://n2t.net");
    assert_eq!(pid.to_string(), valid);
    assert_eq!(pid.prefix(), Some("ark:12148/btv1b8449691v".to_string()));
    assert_eq!(pid.suffix(), Some("f29".to_string()));
    assert_eq!(pid.identifier(), "ark:12148/btv1b8449691v/f29");
    let pid = ARK::from_string("ark:13030/xf93gt2q");
    assert_eq!(pid.check_digit(), Some(vec!['q']));
}
#[test]
fn test_ark_find_all() {
    let valid = "https://n2t.net/ark:12148/btv1b8449691v/f29";
    let text = format!("{valid} is the ARK for the thing I mentioned.");
    let results = ARK::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), valid);
    let text = format!("The ARK for Grande Bible historiale complétée is: {valid}.");
    let results = ARK::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), valid);
    let text = format!("I found things with ARKs {valid}, {valid}, and {valid}. Please check them out.");
    let results = ARK::find_all(&text);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].to_string(), valid);
    assert_eq!(results[1].to_string(), valid);
    assert_eq!(results[2].to_string(), valid);
}
#[test]
fn test_ark_struct() {
    let valid = "https://n2t.net/ark:12148/btv1b8449691v/f29";
    let value = ARK::from_string(valid);
    assert_eq!(value.assigned_name.unwrap(), "btv1b8449691v");
    assert!(ARK::is_valid(valid));
    assert_eq!(ARK::from_string(valid).to_string(), valid);
    assert!(!ARK::is_valid("Invalid ARK value"));
    assert_eq!(ARK::from_string("invalid ARK value").to_string(), "");
}
#[test]
fn test_ark_trait_contract() {
    assert_eq!(ARK::new().to_string(), ARK::default().to_string());
    assert!(ARK::is_valid(CONTRACT_VALID));
    assert!(!ARK::is_valid(CONTRACT_INVALID));
    assert_eq!(ARK::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = ARK::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "https://n2t.net");
    assert_eq!(pid.identifier(), "ark:13030/xf93gt2q");
    assert_eq!(pid.prefix(), Some("ark:13030/xf93gt2q".to_string()));
    assert_eq!(pid.suffix(), Some(String::new()));
    assert_eq!(pid.check_digit(), Some(vec!['q']));
    assert!(pid.url().is_empty());
    let found = <ARK as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::ARK));
    assert_eq!(CONTRACT_VALID.format_as(PID::ARK), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::ARK).to_ark().to_string(), CONTRACT_CANONICAL);
}
#[test]
fn test_idutils_ark_cases() {
    // Adapted from idutils's BSD-3-Clause test suite: https://github.com/inveniosoftware/idutils/blob/master/tests/test_idutils.py
    ["ark:/13030/tqb3kh97gh8w", "ark:/c8131/g3js3v", "ark:c8131/g3js3v.1"]
        .into_iter()
        .for_each(|value| assert!(ARK::is_valid(value), "expected valid ARK: {value}"));
    let http_value = "http://www.example.org/ark:/13030/tqb3kh97gh8w";
    assert_eq!(ARK::from_string(http_value).to_string(), "http://www.example.org/ark:13030/tqb3kh97gh8w");
    assert!(!ARK::is_valid(http_value));
}
