use crate::schema::pid::*;
use crate::schema::*;
use std::path::PathBuf;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_iso7064_check_digit() {
    let value = "000000031415926";
    let expected = 9;
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0002-2057-911";
    let expected = 5;
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0002-2057-9115";
    let expected = 5;
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0002-1823-1234";
    let expected = 2;
    assert_eq!(iso7064_check_digit(value), expected);
    let value = "0000-0001-9034-3389";
    let expected = 9;
    assert_eq!(iso7064_check_digit(value), expected);
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
    assert_eq!(ORCID::from_string(valid).to_string(), valid);
    assert_eq!(ORCID::from_string("0000-0002-2057-9115").to_string(), valid);
    assert_eq!(ORCID::from_string("invalid ORCID value").to_string(), "");
}
#[test]
fn test_doi() {
    let valid = "https://doi.org/10.11578/dc.20250604.1";
    assert_eq!(valid.to_pid(PID::DOI).to_doi().to_string(), "10.11578/dc.20250604.1");
    assert!(valid.is_pid(PID::DOI));
    assert!(valid.is_doi());
}
#[test]
fn test_doi_struct() {
    let valid = "https://doi.org/10.1000/182";
    assert!(DOI::is_valid(valid));
    assert!(DOI::is_valid("10.1000/182"));
    assert!(!DOI::is_valid("invalid DOI URL"));
    assert_eq!(DOI::from_string(valid).to_string(), "10.1000/182");
    assert_eq!(DOI::from_string("10.1000/182").to_string(), "10.1000/182");
    assert_eq!(DOI::from_string("invalid DOI URL").to_string(), "");
    assert_eq!(DOI::format(valid), "10.1000/182");
    assert_eq!(DOI::format("10.1000/182"), "10.1000/182");
    assert_eq!(DOI::format("invalid DOI value"), "");
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
