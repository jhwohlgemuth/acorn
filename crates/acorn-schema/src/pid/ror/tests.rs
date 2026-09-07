#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::pid::{PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "https://ror.org/01qz5mb56";
const CONTRACT_INVALID: &str = "01qz5mb55";
const CONTRACT_VALID: &str = "01qz5mb56";
const MOD_97_10_CASES: [(&str, [char; 2]); 4] = [
    ("1wass4", ['8', '0']),
    ("1qz5mb", ['5', '6']),
    ("2xn1ny", ['0', '6']),
    ("0q6v6102", ['0', '2']),
];

#[test]
fn test_idutils_ror_cases() {
    ["03yrm5c26", "http://ror.org/03yrm5c26"]
        .into_iter()
        .for_each(|value| assert!(ROR::is_valid(value), "expected valid ROR: {value}"));
}
#[test]
fn test_mod_97_10_check_digit() {
    MOD_97_10_CASES
        .into_iter()
        .for_each(|(value, expected)| assert_eq!(mod_97_10_check_digit(value), Some(expected.to_vec())));
}
#[test]
fn test_ror() {
    let valid = "https://ror.org/01qz5mb56";
    assert!(valid.is_pid(PID::ROR));
    assert_eq!(valid.to_pid(PID::ROR).to_ror().to_string(), valid);
    assert!(valid.is_ror());
    assert!(valid.format_as(PID::ROR).is_ror());
    assert_eq!(valid.format_as(PID::ROR), valid);
    assert_eq!(valid.format_as(PID::ROR), valid);
}
#[test]
fn test_ror_metadatatools_cases() {
    [
        "https://ror.org/05dxps055",
        "https://ror.org/040ty4453",
        "https://ror.org/03taz7m60",
        "https://ror.org/05td03w87",
        "https://ror.org/024e4df17",
    ]
    .into_iter()
    .for_each(|value| assert!(ROR::is_valid(value), "expected valid ROR: {value}"));
}
#[test]
fn test_ror_struct() {
    let suffix = "01qz5mb56";
    let valid = format!("https://ror.org/{suffix}");
    assert!(ROR::is_valid(&valid));
    assert!(ROR::is_valid("01qz5mb56"));
    assert!(!ROR::is_valid("invalid ROR URL"));
    let pid = ROR::from_string(suffix);
    assert_eq!(pid.check_digit(), Some(vec!['5', '6']));
    let pid = ROR::from_string(&valid);
    assert_eq!(pid.check_digit(), Some(vec!['5', '6']));
    assert_eq!(pid.to_string(), valid);
    assert_eq!(&ROR::from_string(suffix).to_string(), &valid);
    assert_eq!(ROR::from_string("invalid ROR URL").to_string(), "");
    assert_eq!(ROR::from_string(valid.clone()).to_string(), valid.clone());
    assert_eq!(ROR::from_string(suffix).to_string(), valid);
    assert_eq!(ROR::from_string("invalid ROR value").to_string(), "");
    assert_eq!(<ROR as PersistentIdentifierParse>::format("invalid ROR value"), "");
    let value = "03ebg0v16-";
    assert!(!ROR::is_valid(value));
    assert!(ROR::find_all("023064908 0|524|255 x01qz5mb56").is_empty());
    assert_eq!(
        ROR::find_all("ROR: https://ror.org/01qz5mb56.")
            .into_iter()
            .map(|ror| ror.to_string())
            .collect::<Vec<_>>(),
        ["https://ror.org/01qz5mb56"]
    );
}
#[test]
fn test_ror_trait_contract() {
    assert_eq!(ROR::new().to_string(), ROR::default().to_string());
    assert!(ROR::is_valid(CONTRACT_VALID));
    assert!(!ROR::is_valid(CONTRACT_INVALID));
    assert_eq!(ROR::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = ROR::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "https://ror.org/");
    assert_eq!(pid.identifier(), CONTRACT_VALID);
    assert!(pid.prefix().is_none());
    assert_eq!(pid.suffix(), Some(CONTRACT_VALID.to_string()));
    assert_eq!(pid.check_digit(), Some(vec!['5', '6']));
    assert!(pid.url().is_empty());
    let found = <ROR as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::ROR));
    assert_eq!(CONTRACT_VALID.format_as(PID::ROR), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::ROR).to_ror().to_string(), CONTRACT_CANONICAL);
}
