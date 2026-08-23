#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::*;
use crate::schema::pid::{PersistentIdentifierConvert, PID};

const CONTRACT_CANONICAL: &str = "US 2025/0123456 A2";
const CONTRACT_INVALID: &str = "US ABC123 B2";
const CONTRACT_VALID: &str = "US20250123456A2";
const INVALID_PATENT_IDENTIFIERS: [&str; 5] = ["1234567", "US1234567", "US ABC123 B2", "US 2025/0123456 ZZ", "Totally just not correct"];
const VALID_PATENT_IDENTIFIERS: [&str; 5] = [
    "US1234567B2",
    "US 1234567 B2",
    "US20250123456A2",
    "US2025/0123456A2",
    "US 2025/0123456 A2",
];

#[test]
fn test_kindcode() {
    assert!(KindCode::B1.is_granted());
    assert!(!KindCode::A1.is_granted());
    // Test individual kind codes
    let json_a1 = r#""A1""#;
    let kind: KindCode = serde_json::from_str(json_a1).expect("Failed to deserialize A1");
    assert_eq!(kind, KindCode::A1);
    let json_b2 = r#""B2""#;
    let kind: KindCode = serde_json::from_str(json_b2).expect("Failed to deserialize B2");
    assert_eq!(kind, KindCode::B2);
    let json_e1 = r#""E1""#;
    let kind: KindCode = serde_json::from_str(json_e1).expect("Failed to deserialize E1");
    assert_eq!(kind, KindCode::E1);
    // E, S, and H are normalized to two-character variants
    let json_e1 = r#""H""#;
    let kind: KindCode = serde_json::from_str(json_e1).expect("Failed to deserialize H");
    assert_eq!(kind, KindCode::H1);
    let json_unknown = r#""Unknown""#;
    let kind: KindCode = serde_json::from_str(json_unknown).expect("Failed to deserialize Unknown");
    assert_eq!(kind, KindCode::Unknown);
    let kind = KindCode::A1;
    let json = serde_json::to_string(&kind).expect("Failed to serialize A1");
    assert_eq!(json, r#""A1""#);
    let kind = KindCode::B2;
    let json = serde_json::to_string(&kind).expect("Failed to serialize B2");
    assert_eq!(json, r#""B2""#);
    let kinds = vec![KindCode::A1, KindCode::B2, KindCode::E1, KindCode::S1, KindCode::Unknown];
    for original in kinds {
        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: KindCode = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(original, deserialized);
    }
}
#[test]
fn test_patent() {
    for value in VALID_PATENT_IDENTIFIERS {
        assert!(Patent::is_valid(value), "=> [REASON] \"{value}\" is NOT a valid patent identifier");
    }
    for value in INVALID_PATENT_IDENTIFIERS {
        assert!(!Patent::is_valid(value), "=> [REASON] \"{value}\" IS a valid patent identifier");
    }
    // Granted patents
    let patent = Patent::parse("US1234567B2");
    if let Some(value) = patent {
        assert_eq!(value.to_string(), "US 1234567 B2");
    }
    let patent = Patent::parse("US 7,654,321 B1");
    if let Some(value) = patent {
        assert_eq!(value.to_string(), "US 7654321 B1");
    }
    // Patent publications
    let patent = Patent::parse("US20250123456A2");
    if let Some(value) = patent {
        assert_eq!(value.to_string(), "US 2025/0123456 A2");
    }
    let patent = Patent::parse("US2025/0123456A2");
    if let Some(value) = patent {
        assert_eq!(value.to_string(), "US 2025/0123456 A2");
    }
    let patent = Patent::parse("US 2018/0123456 A2");
    if let Some(value) = patent {
        assert_eq!(value.to_string(), "US 2018/0123456 A2");
    }
    let patent = Patent::parse("2025/0123456A2");
    if let Some(value) = patent {
        assert_eq!(value.to_string(), "US 2025/0123456 A2");
    }
    let patent = Patent::parse("0123456A2");
    if let Some(value) = patent {
        assert_eq!(value.to_string(), "US 0123456 A2");
    }
}
#[test]
fn test_patent_find_all() {
    let expected = "US 7654321 B1";
    let text = "Patent number: US 7,654,321 B1".to_string();
    let results = Patent::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), expected);
    let text = "I patented a thing - US 7,654,321 B1.".to_string();
    let results = Patent::find_all(&text);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), expected);
    let text = r#"I patented some things for money!
- US20250123456A2,
- US 7,654,321 B1,
- US 0123456 A2 (words about something)

Please check them out!"#;
    let results = Patent::find_all(text);
    assert_eq!(results.len(), 3);
    assert_eq!(results[1].to_string(), expected);
    assert!(Patent::find_all("US 2 Stanford US 21 Experiments").is_empty());
    assert!(Patent::parse("US 12345 S").is_some());
}
#[test]
fn test_patent_persistent_identifier() {
    let patent = Patent::from_string("US20250123456A2");
    assert_eq!(patent.identifier(), "US 2025/0123456 A2");
    assert_eq!(patent.prefix(), Some("US/2025".to_string()));
    assert_eq!(patent.suffix(), Some("0123456".to_string()));
    assert!(patent.schema_uri().is_empty());
    assert!(patent.url().is_empty());
}
#[test]
fn test_patent_shared_conversion() {
    let value = "US20250123456A2";
    assert_eq!(value.format_as(PID::Patent), "US 2025/0123456 A2");
    assert_eq!(value.to_pid(PID::Patent).to_patent().to_string(), "US 2025/0123456 A2");
    assert!(value.is_pid(PID::Patent));
}
#[test]
fn test_patent_trait_contract() {
    assert_eq!(Patent::new().to_string(), Patent::default().to_string());
    assert!(Patent::is_valid(CONTRACT_VALID));
    assert!(!Patent::is_valid(CONTRACT_INVALID));
    assert_eq!(<Patent as PersistentIdentifierParse>::format(CONTRACT_VALID), CONTRACT_CANONICAL);
    let pid = <Patent as PersistentIdentifierParse>::from_string(CONTRACT_VALID);
    assert_eq!(pid.to_string(), CONTRACT_CANONICAL);
    assert!(pid.schema_uri().is_empty());
    assert_eq!(pid.identifier(), CONTRACT_CANONICAL);
    assert_eq!(pid.prefix(), Some("US/2025".to_string()));
    assert_eq!(pid.suffix(), Some("0123456".to_string()));
    assert!(pid.check_digit().is_none());
    assert!(pid.url().is_empty());
    let found = <Patent as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTRACT_VALID}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTRACT_CANONICAL));
    assert!(CONTRACT_VALID.is_pid(PID::Patent));
    assert_eq!(CONTRACT_VALID.format_as(PID::Patent), CONTRACT_CANONICAL);
    assert_eq!(CONTRACT_VALID.to_pid(PID::Patent).to_patent().to_string(), CONTRACT_CANONICAL);
}
