#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::prelude::PathBuf;
use crate::schema::pid::patent::KindCode;
use crate::schema::pid::{
    isbn_check_digit, noid_check_digit, orcid_check_digit, raid, ror_check_digit, Betanumeric, Identifier, Patent, PersistentIdentifier,
    PersistentIdentifierConvert, PersistentIdentifierParse, ARK, DOI, ISBN, ORCID, PID, RAID, ROR,
};
use crate::schema::Validate;

const FIXTURES: &str = "../../tests/fixtures";
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
fn test_normalize_discovery_identifiers() {
    assert_eq!(Identifier::new("https://doi.org/10.1234/ABC").normalize().unwrap().value, "10.1234/ABC");
    assert_eq!(Identifier::new("ISBN 978-0-306-40627-0").normalize().unwrap().kind, PID::ISBN);
    assert_eq!(Identifier::new("ark:/12345/abc").normalize().unwrap().kind, PID::ARK);
    assert_eq!(Identifier::new("RAID:10.12345/xyz").normalize().unwrap().kind, PID::RAID);
    assert_eq!(Identifier::new("US-123456-A1").normalize().unwrap().kind, PID::Patent);
}
#[test]
fn test_normalize_discovery_url_preserves_case_sensitive_components() {
    let normalized = Identifier::new("https://Example.org/Data/Artifact?Key=Value").normalize().unwrap();
    assert_eq!(normalized.kind, PID::URL);
    assert_eq!(normalized.value, "https://Example.org/Data/Artifact?Key=Value");
}
#[test]
fn test_pid_from_str() {
    assert_eq!(PID::from("doi"), PID::DOI);
    assert_eq!(PID::from("ORCID"), PID::ORCID);
    assert_eq!(PID::from(" pidinst "), PID::PIDINST);
    assert_eq!(PID::from("unsupported"), PID::Unknown);
}
#[test]
fn test_identifier_normalize_uses_declared_kind() {
    let doi = Identifier {
        kind: PID::DOI,
        value: "https://doi.org/10.1234/ABC".to_string(),
    };
    let kind: &str = (&doi).into();
    assert_eq!(kind, "doi");
    assert_eq!(doi.normalize().unwrap().value, "10.1234/ABC");
    let mismatched = Identifier {
        kind: PID::ISBN,
        value: "https://doi.org/10.1234/ABC".to_string(),
    };
    assert!(mismatched.normalize().is_none());
}

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
    let valid = "10.11578/dc.20250604.1";
    assert!(valid.is_pid(PID::DOI));
    assert!(valid.is_doi());
    assert_eq!(valid.format_as(PID::DOI), "10.11578/dc.20250604.1");
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
fn test_isbn() {
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
        assert_eq!(isbn_check_digit(value), expected, "{value} check digit is NOT {index}");
    });
    // ISBN-A support
    let isbn = ISBN::from_string("978-0-306-40627-0");
    let doi = DOI::from(isbn);
    assert_eq!(doi.to_string(), "10.978.0306/406270");
    // let result: ISBN = doi.into();
    // assert_eq!(result.to_string(), "978-0-306-40627-0");
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
    let expected = Some(vec!['q']);
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
fn test_orcid_check_digit() {
    let value = "000000031415926";
    let expected = Some(vec!['9']);
    assert_eq!(orcid_check_digit(value), expected);
    let value = "0000-0002-2057-911";
    let expected = Some(vec!['5']);
    assert_eq!(orcid_check_digit(value), expected);
    let value = "0000-0002-2057-9115";
    let expected = Some(vec!['5']);
    assert_eq!(orcid_check_digit(value), expected);
    let value = "0000-0002-1823-1234";
    let expected = Some(vec!['2']);
    assert_eq!(orcid_check_digit(value), expected);
    let value = "0000-0001-9034-3389";
    let expected = Some(vec!['9']);
    assert_eq!(orcid_check_digit(value), expected);
    let value = "0000-0002-2816-415X";
    let expected = Some(vec!['X']);
    assert_eq!(orcid_check_digit(value), expected);
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
    assert_eq!(pid.to_string(), "10.83962/fb5be317");
    assert_eq!(valid.format_as(PID::RAID), "10.83962/fb5be317");
    assert_eq!(RAID::format("10.1000/182"), "10.1000/182");
    assert_eq!(RAID::format("https://raid.org/10.1000/182"), "10.1000/182");
}
#[test]
fn test_raid_read() {
    let path = PathBuf::from(FIXTURES).join("raid/response_01.json");
    let data = raid::Metadata::read(path);
    assert!(data.is_ok());
    let _ = data.unwrap().validate();
    let path = PathBuf::from(FIXTURES).join("raid/response_02.json");
    let data = raid::Metadata::read(path);
    assert!(data.is_ok());
    let _ = data.unwrap().validate();
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
fn test_ror_check_digit() {
    let value = "1wass4";
    let expected = Some(vec!['8', '0']);
    assert_eq!(ror_check_digit(value), expected);
    // ORNL ROR ID
    let value = "1qz5mb";
    let expected = Some(vec!['5', '6']);
    assert_eq!(ror_check_digit(value), expected);
    // Check digit is less than 10
    let value = "2xn1ny";
    let expected = Some(vec!['0', '6']);
    assert_eq!(ror_check_digit(value), expected);
    // Check digit is less than 10, ID starts with 0, and input is longer than 6 characters
    let value = "0q6v6102";
    let expected = Some(vec!['0', '2']);
    assert_eq!(ror_check_digit(value), expected);
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
}
