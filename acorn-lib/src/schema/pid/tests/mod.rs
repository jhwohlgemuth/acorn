use crate::prelude::PathBuf;
use crate::schema::pid::patent::KindCode;
use crate::schema::pid::{
    iso7064_check_digit, noid_check_digit, raid, Betanumeric, Patent, PersistentIdentifier, PersistentIdentifierConvert, ARK, DOI, ORCID, PID, RAID,
};
use crate::schema::Validate;

const FIXTURES: &str = "../tests/fixtures";
const VALID_PATENT_IDENTIFIERS: [&str; 5] = [
    "US1234567B2",
    "US 1234567 B2",
    "US20250123456A2",
    "US2025/0123456A2",
    "US 2025/0123456 A2",
];
const INVALID_PATENT_IDENTIFIERS: [&str; 5] = [
    "1234567",            // No country or kind code
    "US1234567",          // No kind code
    "US ABC123 B2",       // non-numeric serial number
    "US 2025/0123456 ZZ", // invalid kind code
    "Totally just not correct",
];

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
    assert_eq!(pid.format(), valid);
    assert_eq!(pid.prefix(), Some("ark:12148/btv1b8449691v".to_string()));
    assert_eq!(pid.suffix(), Some("f29".to_string()));
    assert_eq!(pid.identifier(), "ark:12148/btv1b8449691v/f29");
    let pid = ARK::from_string("ark:13030/xf93gt2q");
    assert_eq!(pid.check_digit(), Some('q'));
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
fn test_betanumeric() {
    assert!('9'.is_betanumeric());
    assert!(!'a'.is_betanumeric());
    assert!(!'/'.is_betanumeric());
    assert_eq!('0'.to_betanumeric_ordinal(), Some(0));
    assert_eq!('z'.to_betanumeric_ordinal(), Some(28));
}
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
fn test_iso7064_check_digit() {
    let value = "000000031415926";
    let expected = Some('9');
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0002-2057-911";
    let expected = Some('5');
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0002-2057-9115";
    let expected = Some('5');
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0002-1823-1234";
    let expected = Some('2');
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0001-9034-3389";
    let expected = Some('9');
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0002-2816-415X";
    let expected = Some('X');
    assert_eq!(iso7064_check_digit(value), expected);
}
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
fn test_noid_check_digit() {
    let value = "13030/xf93gt2";
    let expected = Some('q');
    assert_eq!(noid_check_digit(value), expected);
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
fn test_orcid_struct() {
    let valid = "https://orcid.org/0000-0002-2057-9115";
    assert!(ORCID::is_valid(valid));
    assert!(!ORCID::is_valid("Invalid ORCID value"));
    let pid = ORCID::from_string(valid);
    assert_eq!(pid.format(), valid);
    assert_eq!(pid.to_string(), valid);
    assert_eq!(ORCID::from_string("0000-0002-2057-9115").to_string(), valid);
    assert_eq!(ORCID::from_string("invalid ORCID value").to_string(), "");
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
    assert_eq!(RAID::format("10.1000/182"), "10.1000/182");
    assert_eq!(RAID::format("https://raid.org/10.1000/182"), "10.1000/182");
    assert_eq!(valid.format_as(PID::RAID), "10.83962/fb5be317");
}
#[test]
fn test_raid_read() {
    let path = PathBuf::from(FIXTURES).join("raid/response_01.json");
    let data = raid::Metadata::read(path);
    assert!(data.is_some());
    let _ = data.unwrap().validate();
    let path = PathBuf::from(FIXTURES).join("raid/response_02.json");
    let data = raid::Metadata::read(path);
    assert!(data.is_some());
    let _ = data.unwrap().validate();
}
