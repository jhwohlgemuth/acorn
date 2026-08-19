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
    isbn_check_digit, isni_check_digit, noid_check_digit, orcid_check_digit, raid, ror_check_digit, Betanumeric, Identifier, Patent,
    PersistentIdentifier, PersistentIdentifierConvert, PersistentIdentifierParse, ARK, ARXIV, DOI, ISBN, ISNI, ORCID, PID, RAID, ROR,
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
    assert_eq!(Identifier::new("https://doi.org/10.1234/ABC").normalized().unwrap().value, "10.1234/ABC");
    assert_eq!(Identifier::new("ISBN 978-0-306-40627-0").normalized().unwrap().kind, PID::ISBN);
    assert_eq!(Identifier::new("ark:/12345/abc").normalized().unwrap().kind, PID::ARK);
    assert_eq!(Identifier::new("RAID:10.12345/xyz").normalized().unwrap().kind, PID::RAID);
    assert_eq!(Identifier::new("US-123456-A1").normalized().unwrap().kind, PID::Patent);
}
#[test]
fn test_normalize_discovery_url_preserves_case_sensitive_components() {
    let normalized = Identifier::new("https://Example.org/Data/Artifact?Key=Value").normalized().unwrap();
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
fn test_pid_display_and_serialization_use_lowercase_wire_names() {
    assert_eq!(PID::ARXIV.to_string(), "arxiv");
    assert_eq!(serde_json::to_string(&PID::DOI).unwrap(), r#""doi""#);
    assert_eq!(serde_json::from_str::<PID>(r#""doi""#).unwrap(), PID::DOI);
    assert_eq!(serde_json::from_str::<PID>(r#""DOI""#).unwrap(), PID::DOI);
}
#[test]
fn test_identifier_normalize_uses_declared_kind() {
    let doi = Identifier {
        kind: PID::DOI,
        value: "https://doi.org/10.1234/ABC".to_string(),
    };
    let kind: &str = (&doi).into();
    assert_eq!(kind, "doi");
    assert_eq!(doi.normalized().unwrap().value, "10.1234/ABC");
    let mismatched = Identifier {
        kind: PID::ISBN,
        value: "https://doi.org/10.1234/ABC".to_string(),
    };
    assert!(mismatched.normalized().is_none());
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
fn test_arxiv_modern_and_legacy_formats() {
    let modern = ARXIV::from_string("https://arxiv.org/pdf/1501.00001v2.pdf");
    assert_eq!(modern.to_string(), "arXiv:1501.00001v2");
    assert_eq!(modern.work_identifier(), "arXiv:1501.00001");
    assert_eq!(modern.url(), "https://arxiv.org/abs/1501.00001v2");
    assert_eq!(modern.version.as_deref(), Some("v2"));
    let legacy = ARXIV::from_string("arXiv:math.CA/0611800v2");
    assert_eq!(legacy.to_string(), "arXiv:math/0611800v2");
    assert_eq!(legacy.archive.as_deref(), Some("math"));
    assert_eq!(legacy.work_identifier(), "arXiv:math/0611800");
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
            ARXIV::is_valid(value),
            "expected valid arXiv identifier: {value}; match: {:?}",
            crate::util::constants::RE_ARXIV.find(value)
        )
    });
    [
        "2106.09685",
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
    .for_each(|value| assert!(!ARXIV::is_valid(value), "expected invalid arXiv identifier: {value}"));
    let found = ARXIV::find_all("See arXiv:2106.09685 and https://arxiv.org/abs/2106.09685.");
    assert_eq!(found.len(), 2);
    assert!("arXiv:2106.09685".is_arxiv());
    assert!("arXiv:2106.09685".is_pid(PID::ARXIV));
    assert_eq!("https://arxiv.org/abs/2106.09685".format_as(PID::ARXIV), "arXiv:2106.09685");
    assert_eq!("arXiv:2106.09685".to_pid(PID::ARXIV).to_arxiv().to_string(), "arXiv:2106.09685");
}
#[test]
fn test_arxiv_metadatatools_cases() {
    ["arXiv:2412.03631", "arXiv:2412.03649", "arXiv:2412.04386"]
        .into_iter()
        .for_each(|value| assert!(ARXIV::is_valid(value), "expected valid arXiv identifier: {value}"));
}
#[test]
fn test_arxiv_datacite_doi_conversions() {
    let versioned = ARXIV::from_string("arXiv:2106.09685v2");
    let doi = DOI::from(versioned);
    assert_eq!(doi.to_string(), "10.48550/arXiv.2106.09685");
    assert_eq!(ARXIV::try_from(doi).unwrap().to_string(), "arXiv:2106.09685");
    let legacy = ARXIV::from_string("arXiv:hep-th/9901001v3");
    let doi = DOI::from(legacy);
    assert_eq!(doi.to_string(), "10.48550/arXiv.hep-th/9901001");
    assert_eq!(ARXIV::try_from(doi).unwrap().to_string(), "arXiv:hep-th/9901001");
    assert!(ARXIV::try_from(DOI::from_string("10.1000/182")).is_err());
    assert!(ARXIV::try_from(DOI::from_string("10.48550/arXiv.invalid")).is_err());
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
fn test_isni() {
    let canonical = "https://isni.org/isni/0000000492299539";
    assert_eq!(PID::from("ISNI"), PID::ISNI);
    assert_eq!(serde_json::to_string(&PID::ISNI).unwrap(), r#""isni""#);
    assert!(canonical.is_pid(PID::ISNI));
    assert!(canonical.is_isni());
    assert_eq!(canonical.to_pid(PID::ISNI).to_isni().to_string(), canonical);
    assert_eq!("0000 0004 9229 9539".format_as(PID::ISNI), canonical);
    assert_eq!("0000-0004-9229-9539".format_as(PID::ISNI), canonical);
    assert_eq!(ISNI::format("http://isni.org/isni/0000000492299539"), canonical);
    assert_eq!(
        Identifier {
            kind: PID::ISNI,
            value: "0000-0004-9229-9539".to_string(),
        }
        .normalized()
        .expect("explicit ISNI should normalize")
        .value,
        canonical
    );
    assert_eq!(isni_check_digit("0000000492299539"), Some(vec!['9']));
    assert!(!ISNI::is_valid("0000000492299538"));
}
#[test]
fn test_isni_metadatatools_cases() {
    ["0000 0001 2096 0218", "0000 0000 8405 6132", "0000 0000 7182 7209"]
        .into_iter()
        .for_each(|value| assert!(ISNI::is_valid(value), "expected valid ISNI: {value}"));
}
#[test]
fn test_isni_find_all_and_orcid_precedence() {
    let isni = "https://isni.org/isni/0000000492299539";
    let orcid = "https://orcid.org/0000-0002-2057-9115";
    let values = ISNI::find_all(format!("ISNI {isni}; ORCID {orcid}"));
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].to_string(), isni);
    assert!(ORCID::find_all(isni).is_empty());
    let normalized = Identifier::new(orcid).normalized().expect("ORCID should normalize");
    assert_eq!(normalized.kind, PID::ORCID);
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
fn test_orcid_metadatatools_cases() {
    [
        "https://orcid.org/0000-0003-0900-6903",
        "0000-0002-6539-638X",
        "0000-0001-9266-5146",
        "0000-0002-0026-2516",
    ]
    .into_iter()
    .for_each(|value| assert!(ORCID::is_valid(value), "expected valid ORCID: {value}"));
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
fn test_raid_contributor_accepts_orcid_and_isni() {
    let contributor = |id: &str, schema_uri: &str| raid::Contributor {
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
    assert!(ROR::find_all("023064908 0|524|255 x01qz5mb56").is_empty());
    assert_eq!(
        ROR::find_all("ROR: https://ror.org/01qz5mb56.")
            .into_iter()
            .map(|ror| ror.to_string())
            .collect::<Vec<_>>(),
        ["https://ror.org/01qz5mb56"]
    );
}
