use super::*;
use crate::pid::{PersistentIdentifierConvert, PID};
use crate::validation::Validate;
use proptest::prelude::*;
use serde::Deserialize;

const CONTRACT: &str = include_str!("../../../../../tests/fixtures/rrid/portable.json");
const JATS: &str = include_str!("../../../../../tests/fixtures/rrid/jats-1.4.xml");

#[derive(Deserialize)]
struct Contract {
    invalid: Vec<String>,
    valid: Vec<ValidCase>,
}
#[derive(Deserialize)]
struct ValidCase {
    expected: String,
    input: String,
}

#[test]
fn test_bare_payloads_are_not_discovered() {
    let found = RRID::find_all("SCR_003512 CVCL_3588 Addgene_12260");
    assert!(found.is_empty());
}
#[test]
fn test_contract_fixture() {
    let contract: Contract = serde_json::from_str(CONTRACT).expect("portable RRID contract should deserialize");
    contract.valid.into_iter().for_each(|case| {
        let parsed: RRID = case.input.parse().expect("valid RRID fixture should parse");
        assert_eq!(parsed.to_string(), case.expected, "{}", case.input);
        assert_eq!(RRID::format(&case.input), case.expected, "{}", case.input);
    });
    contract
        .invalid
        .into_iter()
        .for_each(|value| assert!(!RRID::is_valid(&value), "invalid RRID fixture parsed: {value}"));
}
#[test]
fn test_direct_validation_rejects_invalid_payload() {
    let rrid = RRID {
        payload: "SCR 014641".to_string(),
    };
    assert!(rrid.validate().is_err());
}
#[test]
fn test_discovery_ignores_arbitrary_url_context() {
    let found = RRID::find_all("https://example.org/search?q=RRID:SCR_003512 and https://example.org/RRID:CVCL_3588");
    assert!(found.is_empty());
}
#[test]
fn test_discovery_trims_prose_and_deduplicates_aliases() {
    let found = RRID::find_all("Use (RRID:SCR_003512), rrid : SCR_003512, and https://n2t.net/RRID:SCR_003512.");
    assert_eq!(found.len(), 3);
    assert!(found.iter().all(|identifier| identifier.to_string() == "RRID:SCR_003512"));
}
#[test]
fn test_idutils_rrid_case() {
    let (value, schemes, normalized, url) = (
        "RRID:SCR_014641",
        ["rrid"],
        "RRID:SCR_014641",
        "http://scicrunch.org/resolver/RRID:SCR_014641",
    );
    assert_eq!(schemes.map(PID::from), [PID::RRID]);
    assert_eq!(RRID::format(value), normalized);
    assert_eq!(RRID::format(url), normalized);
}
#[test]
fn test_jats_14_fixture_preserves_explicit_rrid() {
    assert!(JATS.contains("resource-id-type=\"RRID\""));
    let found = RRID::find_all(JATS);
    assert_eq!(
        found.into_iter().map(|identifier| identifier.to_string()).collect::<Vec<_>>(),
        ["RRID:SCR_003512"]
    );
}
#[test]
fn test_normalization_is_idempotent() {
    let once = RRID::format("rrid: IMSR_JAX:000664");
    assert_eq!(RRID::format(&once), once);
}
#[test]
fn test_rrid_trait_contract() {
    let value = "RRID:SCR_003512";
    let rrid = RRID::from_string(value);
    assert_eq!(rrid.identifier(), value);
    assert_eq!(rrid.prefix().as_deref(), Some("RRID"));
    assert_eq!(rrid.suffix().as_deref(), Some("SCR_003512"));
    assert_eq!(rrid.url(), "https://n2t.net/RRID:SCR_003512");
    assert!(rrid.check_digit().is_none());
    assert!(value.is_pid(PID::RRID));
    assert_eq!(value.format_as(PID::RRID), value);
    assert_eq!(value.to_pid(PID::RRID).to_rrid(), rrid);
}
#[test]
fn test_serde_round_trip() {
    let rrid: RRID = "RRID:CVCL_3588".parse().expect("RRID should parse");
    let json = serde_json::to_string(&rrid).expect("RRID should serialize");
    assert_eq!(serde_json::from_str::<RRID>(&json).expect("RRID should deserialize"), rrid);
}

proptest! {
    #[test]
    fn test_parse_is_total_and_canonicalization_is_idempotent(value in any::<String>()) {
        if let Ok(parsed) = value.parse::<RRID>() {
            let canonical = parsed.to_string();
            prop_assert_eq!(canonical.parse::<RRID>().map(|rrid| rrid.to_string()), Ok(canonical));
        }
    }
}
