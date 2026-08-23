#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::schema::pid::{PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "978-0-306-40627-0";
const CONTRACT_INVALID: &str = "0-306-40615-3";
const CONTRACT_VALID: &str = "978-0-306-40627-0";
const VALID_ISBN: [&str; 10] = [
    "978-0-306-40627-0",
    "978-0-306-40620-1",
    "978-0-306-40623-2",
    "979-8-345-96320-3",
    "978-0-306-40616-4",
    "978-0-306-40619-5",
    "978-1-625-27449-6",
    "978-0-306-40615-7",
    "978-0-306-40618-8",
    "978-0-306-40624-9",
];

#[test]
fn test_idutils_isbn_cases() {
    [
        "urn:isbn:0451450523",
        "9783468111242",
        "9798847781275",
        "9798991050005",
        "9791370064242",
        "978-65-87773-12-4",
        "978-3-905673-82- 1",
        "978-3-905673-82-1",
        "0-9752298-0-X",
    ]
    .into_iter()
    .for_each(|value| assert!(ISBN::is_valid(value), "expected valid ISBN: {value}"));
}
#[test]
fn test_isbn() {
    let text = r#"I published some books
- 978-0-306-40627-0,
- 978-0-306-40616-4
- 978-0-306-40624-9 (words about something)

Please check them out!"#
        .to_string();
    let results = ISBN::find_all(&text);
    assert_eq!(results.len(), 3);
    VALID_ISBN.iter().enumerate().for_each(|(index, value)| {
        assert!(ISBN::is_valid(value));
        let pid = ISBN::from_string(value);
        assert_eq!(pid.to_string(), *value);
        let expected = char::from_digit(index as u32, 10).map(|c| vec![c]);
        assert_eq!(mod_10_or_11_check_digit(value), expected, "{value} check digit is NOT {index}");
    });
    // ISBN-A support
    let isbn = ISBN::from_string("978-0-306-40627-0");
    let doi = DOI::from(isbn);
    assert_eq!(doi.to_string(), "10.978.0306/406270");
    // let result: ISBN = doi.into();
    // assert_eq!(result.to_string(), "978-0-306-40627-0");
}
#[test]
fn test_isbn_metadatatools_cases() {
    ["0-306-40615-2", "0-8044-2957-X", "978-3-16-148410-0", "978-1606069424", "160606942X"]
        .into_iter()
        .for_each(|value| assert!(ISBN::is_valid(value), "expected valid ISBN: {value}"));
    assert_eq!(ISBN::format("0-306-40615-2"), "0-306-40615-2");
    assert_eq!(ISBN::format("0-8044-2957-X"), "0-8044-2957-X");
    assert_eq!(ISBN::format("160606942X"), "1-606-06942-X");
    assert!(!ISBN::is_valid("0-306-40615-3"));
    assert!(!ISBN::is_valid("0--306-40615-2"));
    assert!(!ISBN::is_valid("100-150-200-0"));
    let discovered = ISBN::find_all("ISBN-10 values: 0-8044-2957-X and 160606942X");
    assert_eq!(
        discovered.into_iter().map(|isbn| isbn.to_string()).collect::<Vec<_>>(),
        ["0-8044-2957-X", "1-606-06942-X"]
    );
}
#[test]
fn test_isbn_shared_conversion() {
    let value = "978-0-306-40627-0";
    assert_eq!(value.format_as(PID::ISBN), value);
    assert_eq!(value.to_pid(PID::ISBN).to_isbn().to_string(), value);
    assert!(value.is_pid(PID::ISBN));
}
#[test]
fn test_isbn_trait_contract() {
    assert_eq!(ISBN::new().to_string(), ISBN::default().to_string());
    assert!(ISBN::is_valid(CONTRACT_VALID));
    assert!(!ISBN::is_valid(CONTRACT_INVALID));
    assert_eq!(ISBN::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = ISBN::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert!(pid.schema_uri().is_empty());
    assert_eq!(pid.identifier(), CONTRACT_CANONICAL);
    assert_eq!(pid.prefix(), Some("978.0306".to_string()));
    assert_eq!(pid.suffix(), Some("406270".to_string()));
    assert_eq!(pid.check_digit(), Some(vec!['0']));
    assert!(pid.url().is_empty());
    let found = <ISBN as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::ISBN));
    assert_eq!(CONTRACT_VALID.format_as(PID::ISBN), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::ISBN).to_isbn().to_string(), CONTRACT_CANONICAL);
}
