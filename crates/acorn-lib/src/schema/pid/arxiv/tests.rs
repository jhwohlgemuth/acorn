#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::schema::pid::{PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "arXiv:2106.09685";
const CONTRACT_INVALID: &str = "arXiv:2100.00000";
const CONTRACT_VALID: &str = "https://arxiv.org/abs/2106.09685";

#[test]
fn test_arxiv_datacite_doi_conversions() {
    let versioned = Arxiv::from_string("arXiv:2106.09685v2");
    let doi = DOI::from(versioned);
    assert_eq!(doi.to_string(), "10.48550/arXiv.2106.09685");
    assert_eq!(Arxiv::try_from(doi).unwrap().to_string(), "arXiv:2106.09685");
    let legacy = Arxiv::from_string("arXiv:hep-th/9901001v3");
    let doi = DOI::from(legacy);
    assert_eq!(doi.to_string(), "10.48550/arXiv.hep-th/9901001");
    assert_eq!(Arxiv::try_from(doi).unwrap().to_string(), "arXiv:hep-th/9901001");
    assert!(Arxiv::try_from(DOI::from_string("10.1000/182")).is_err());
    assert!(Arxiv::try_from(DOI::from_string("10.48550/arXiv.invalid")).is_err());
}
#[test]
fn test_arxiv_metadatatools_cases() {
    ["arXiv:2412.03631", "arXiv:2412.03649", "arXiv:2412.04386"]
        .into_iter()
        .for_each(|value| assert!(Arxiv::is_valid(value), "expected valid arXiv identifier: {value}"));
}
#[test]
fn test_arxiv_modern_and_legacy_formats() {
    let modern = Arxiv::from_string("https://arxiv.org/pdf/1501.00001v2.pdf");
    assert_eq!(modern.to_string(), "arXiv:1501.00001v2");
    assert_eq!(modern.work_identifier(), "arXiv:1501.00001");
    assert_eq!(modern.url(), "https://arxiv.org/abs/1501.00001v2");
    assert_eq!(modern.version.as_deref(), Some("v2"));
    let legacy = Arxiv::from_string("arXiv:math.CA/0611800v2");
    assert_eq!(legacy.to_string(), "arXiv:math/0611800v2");
    assert_eq!(legacy.archive.as_deref(), Some("math"));
    assert_eq!(legacy.work_identifier(), "arXiv:math/0611800");
}
#[test]
fn test_arxiv_trait_contract() {
    assert_eq!(Arxiv::new().to_string(), Arxiv::default().to_string());
    assert!(Arxiv::is_valid(CONTRACT_VALID));
    assert!(!Arxiv::is_valid(CONTRACT_INVALID));
    assert_eq!(Arxiv::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = Arxiv::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert_eq!(pid.schema_uri(), "https://arxiv.org");
    assert_eq!(pid.identifier(), CONTRACT_CANONICAL);
    assert_eq!(pid.prefix(), Some("2106".to_string()));
    assert_eq!(pid.suffix(), Some("2106.09685".to_string()));
    assert!(pid.check_digit().is_none());
    assert_eq!(pid.url(), CONTRACT_VALID);
    let found = <Arxiv as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::Arxiv));
    assert_eq!(CONTRACT_VALID.format_as(PID::Arxiv), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::Arxiv).to_arxiv().to_string(), CONTRACT_CANONICAL);
}
#[test]
fn test_arxiv_validation_and_discovery() {
    [
        "arXiv:0704.0001",
        "arXiv:1412.7878v3",
        "arXiv:1501.00001",
        "arXiv:hep-th/9901001v2",
        "https://www.arxiv.org/abs/2106.09685",
        "http://arxiv.org/pdf/hep-th/9901001.pdf",
    ]
    .into_iter()
    .for_each(|value| {
        assert!(
            Arxiv::is_valid(value),
            "expected valid arXiv identifier: {value}; match: {:?}",
            crate::util::constants::RE_ARXIV.find(value)
        )
    });
    [
        "arXiv:0703.0001",
        "arXiv:1412.07878",
        "arXiv:1501.0001",
        "arXiv:2100.00000",
        "arXiv:2106.09685v0",
        "https://example.org/abs/2106.09685",
        "https://arxiv.org/abs/2106.09685.pdf",
        "https://arxiv.org/abs/2106.09685?download=1",
    ]
    .into_iter()
    .for_each(|value| assert!(!Arxiv::is_valid(value), "expected invalid arXiv identifier: {value}"));
    let found = Arxiv::find_all("See arXiv:2106.09685 and https://arxiv.org/abs/2106.09685.");
    assert_eq!(found.len(), 2);
    assert!("arXiv:2106.09685".is_arxiv());
    assert!("arXiv:2106.09685".is_pid(PID::Arxiv));
    assert_eq!("https://arxiv.org/abs/2106.09685".format_as(PID::Arxiv), "arXiv:2106.09685");
    assert_eq!("arXiv:2106.09685".to_pid(PID::Arxiv).to_arxiv().to_string(), "arXiv:2106.09685");
}
#[test]
fn test_idutils_arxiv_cases() {
    [
        "arXiv:1310.2590",
        "arxiv:1310.2590",
        "1310.2590",
        "math.GT/0309136",
        "hep-th/9901001v27",
        "arxiv:math.GT/0309136v2",
        "arXiv:hep-th/9901001v27",
        "9912.12345v2",
        "arXiv:hep-th/1601.07616",
        "hep-th/1601.07616",
    ]
    .into_iter()
    .for_each(|value| assert!(Arxiv::is_valid(value), "expected valid arXiv identifier: {value}"));
}
