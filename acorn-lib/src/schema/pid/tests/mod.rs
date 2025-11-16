use crate::prelude::PathBuf;
use crate::schema::pid::{
    iso7064_check_digit, noid_check_digit, raid, Betanumeric, PersistentIdentifier, PersistentIdentifierConvert, ARK, DOI, ORCID, PID, RAID,
};
use crate::schema::Validate;

const FIXTURES: &str = "../tests/fixtures";

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
fn test_doi() {
    let valid = "https://doi.org/10.11578/dc.20250604.1";
    assert_eq!(valid.to_pid(PID::DOI).to_doi().to_string(), "10.11578/dc.20250604.1");
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
fn test_noid_check_digit() {
    let value = "13030/xf93gt2";
    let expected = Some('q');
    assert_eq!(noid_check_digit(value), expected);
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
