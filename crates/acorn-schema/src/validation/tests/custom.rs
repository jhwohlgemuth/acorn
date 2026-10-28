#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
use super::super::*;
use crate::research_activity::ResearchActivityMetadata;
use crate::validation::{rules, Validate};

const VALID_ARK_VALUES: [&str; 8] = [
    "https://n2t.net/ark:99166/w66d60p2",
    "https://n2t.net/ark:/99166/w66d60p2",
    "ark:1234/w5678",
    "https://n2t.net/ark:12148/btv1b8449691v/f29",
    "https://n2t.net/ark:12148/btv1b8449691v/f29/abc.png",
    "ark:12148/btv1b8449691v/f29/abc.TIFF",
    "https://example.org/ark:12345/x6np1wh8k/c3/s5.v7.xsl",
    "ark:/12148/btv1b8449691v",
];
const VALID_DOI_VALUES: [&str; 31] = [
    "10.11578/dc.20250604.1",
    "10.1000/182",
    "10.97812345/99990",
    "https://doi.org/10.11578/dc.20250604.1",
    "https://doi.org/10.1000/182",
    "https://doi.org/10.97812345/99990",
    "10.1038/s41562-018-0399-z",
    "10.1038/533452a",
    "10.1109/eScience51609.2021.00010",
    "10.1109/ACCESS.2025.3542334",
    "10.1016/j.jbi.2008.04.010",
    "10.48550/arXiv.2312.10997",
    "10.1098/rsos.171511",
    "10.3233/ISU-2010-0613",
    "10.48550/arXiv.2411.06237",
    "10.2139/ssrn.4900122",
    "10.18653/v1/2025.naacl-long.243",
    "10.1007/978-3-642-38288-8_33",
    "10.48550/arXiv.2501.07391",
    "10.48550/arXiv.2504.01990",
    "10.1109/I-SMAC61858.2024.10714814",
    "10.48550/arXiv.2506.06576",
    "10.1186/s13326-025-00327-4",
    "10.1162/dint_a_00186",
    "10.1609/aaai.v39i24.34743",
    "10.48550/arXiv.2504.16736",
    "10.1002/asi.22636",
    "10.1093/gigascience/giy023",
    "10.1162/99608f92.e1f349c2",
    "10.5479/10088/113528",
    "10.1038/s41597-020-0486-7",
];
const VALID_ORCID_VALUES: [&str; 14] = [
    "https://orcid.org/0000-0002-2057-9115",
    "https://orcid.org/0009-0005-5568-6526",
    "0000-0003-1485-2741",
    "0009-0005-8526-4332",
    "0009-0001-1431-2393",
    "0009-0007-2591-8394",
    "0009-0006-8870-3625",
    "0009-0004-4438-9406",
    "0009-0003-1201-0767",
    "0000-0002-2845-8668",
    "0000-0002-0014-1319",
    "0000-0001-9034-3389",
    "0000-0002-0065-494X",
    "0000-0002-2816-415X",
];
const VALID_ROR_VALUES: [&str; 28] = [
    "https://ror.org/01qz5mb56",
    "01qz5mb56",
    "https://ror.org/04ttjf776",
    "https://ror.org/01rxfrp27",
    "https://ror.org/023q4bk22",
    "https://ror.org/006jxzx88",
    "https://ror.org/00wfvh315",
    "https://ror.org/05ktbsm52",
    "https://ror.org/00nx6aa03",
    "https://ror.org/02k3cxs74",
    "https://ror.org/046fa4y88",
    "https://ror.org/02d439m40",
    "https://ror.org/04h08p482",
    "https://ror.org/01zctcs90",
    "https://ror.org/05m7zw681",
    "https://ror.org/03awtex73",
    "https://ror.org/00kv9pj15",
    "https://ror.org/03mjtdk61",
    "https://ror.org/04yyp8h20",
    "https://ror.org/022rkxt86",
    "https://ror.org/01wddqe20",
    "https://ror.org/03kwrfk72",
    "https://ror.org/037mpqg03",
    "https://ror.org/05cggb038",
    "https://ror.org/050b31k83",
    "https://ror.org/01q8f6705",
    "https://ror.org/03t40cz74",
    "https://ror.org/03ebg0v16",
];

#[test]
fn test_country_rule() {
    ["US", "FR", "DE", "GB", "JP", "us", "fr", "de", "gb", "jp"]
        .into_iter()
        .for_each(|value| assert!(rules::country(value).is_ok(), "expected valid country: {value}"));
    ["USA", "U", "U.S", "US1", "U-S", "XX", "ZZ", "", "USa "]
        .into_iter()
        .for_each(|value| assert!(rules::country(value).is_err(), "expected invalid country: {value}"));
}
#[test]
fn test_date_validation() {
    ["1950-01-01", "2000-02-29", "2024-02-29", "2025-06-04", "2029-12-31"]
        .into_iter()
        .for_each(|value| assert!(rules::date(value).is_ok(), "expected valid date: {value}"));
    [
        "1949-01-02",
        "2026-02-29",
        "2026-04-31",
        "2030-01-01",
        "01-02-2023",
        "1234",
        "foo",
        "42",
        "2025/06/04",
    ]
    .into_iter()
    .for_each(|value| assert!(rules::date(value).is_err(), "expected invalid date: {value}"));
    ["2000", "2025", "1950", "0999"]
        .into_iter()
        .for_each(|value| assert!(rules::year(value).is_ok(), "expected valid year: {value}"));
    ["3000", "999", "42"]
        .into_iter()
        .for_each(|value| assert!(rules::year(value).is_err(), "expected invalid year: {value}"));
}
#[test]
fn test_format_phone_number() {
    ["555-123-4567", "555.123.4567", "(555) 123-4567"]
        .into_iter()
        .for_each(|value| assert_eq!(format_phone_number(value), Ok("555.123.4567".to_string())));
    ["+1 (555) 123-4567", "+1 (555) 123.4567", "+15551234567"]
        .into_iter()
        .for_each(|value| assert_eq!(format_phone_number(value), Ok("+1.555.123.4567".to_string())));
}
#[test]
fn test_format_timestamp() {
    assert_eq!(
        format_timestamp("2026-03-10T15:37:24.355614500+00:00"),
        "2026-03-10T15:37:24.355614+00:00"
    );
    assert_eq!(format_timestamp("2023-01-01T12:00:00.123456789Z"), "2023-01-01T12:00:00.123456+00:00");
    assert_eq!(format_timestamp("2026-03-10T15:37:24.5-04:30"), "2026-03-10T15:37:24.500000-04:30");
    assert_eq!(format_timestamp("not-a-date"), "not-a-date");
}
#[test]
fn test_has_image_extension() {
    [
        "foo.png",
        "bar.jpg",
        "baz.jpeg",
        "qux.svg",
        "https://example.com/foo.PNG",
        "https://example.com/bar.JPEG",
    ]
    .into_iter()
    .for_each(|value| assert!(has_image_extension(value).is_ok(), "expected image extension: {value}"));
    ["foo.pNg", "bar.jpx", "qux.sVG", "https://example.com/fooPNG"]
        .into_iter()
        .for_each(|value| assert!(has_image_extension(value).is_err(), "expected invalid image extension: {value}"));
}
#[test]
fn test_is_ark() {
    VALID_ARK_VALUES
        .into_iter()
        .for_each(|value| assert!(rules::ark(value).is_ok(), "expected valid ARK: {value}"));
    [
        "978-12345-99990",
        "http://n2t.net/ark:99166/w66d60p2",
        "https://n2t.net/99166/w66d60p2",
        "https://n2t.net/99166/9w66d60p2",
        "https://n2t.net/99166/lw66d60p2",
    ]
    .into_iter()
    .for_each(|value| assert!(rules::ark(value).is_err(), "expected invalid ARK: {value}"));
    assert_eq!(
        rules::ark("https://n2t.net/99166/w66d60p2")
            .expect_err("invalid ARK should fail")
            .to_string(),
        "Provide valid ARK value"
    );
}
#[test]
fn test_is_commit() {
    [
        "a1b2c3d",
        "A1B2C3D4",
        "0123456789abcdef0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ]
    .into_iter()
    .for_each(|value| assert!(rules::commit(value).is_ok(), "expected valid commit: {value}"));
    [
        "",
        "123456",
        "xyzxyzx",
        "g123456",
        "123456 ",
        "12345-678",
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef0",
    ]
    .into_iter()
    .for_each(|value| assert!(rules::commit(value).is_err(), "expected invalid commit: {value}"));
}
#[test]
fn test_is_doi() {
    VALID_DOI_VALUES
        .into_iter()
        .for_each(|value| assert!(rules::doi(value).is_ok(), "expected valid DOI: {value}"));
    [
        "978-12345-99990",
        "11.11578/dc.20250604.1",
        "<geo coords=\"10.4515260,51.1656910\"></geo>",
        "10.5555/182",
    ]
    .into_iter()
    .for_each(|value| assert!(rules::doi(value).is_err(), "expected invalid DOI: {value}"));
}
#[test]
fn test_is_handle() {
    ["10013/epic.10033", "hdl:20.5000/example", "https://hdl.handle.net/10013/example"]
        .into_iter()
        .for_each(|value| assert!(rules::handle(value).is_ok(), "expected valid Handle: {value}"));
    ["", "docs/index.md", "https://example.org/10013/example"]
        .into_iter()
        .for_each(|value| assert!(rules::handle(value).is_err(), "expected invalid Handle: {value}"));
}
#[test]
fn test_is_iso_639_language_code_validation() {
    ["en", "EN", "en-US"]
        .into_iter()
        .for_each(|value| assert!(is_iso_639_language_code(value).is_ok()));
    ["eng", "zz-ZZ"]
        .into_iter()
        .for_each(|value| assert!(is_iso_639_language_code(value).is_err()));
    ["en", "EN"]
        .into_iter()
        .for_each(|value| assert!(is_iso_639_1_language_code(value).is_ok()));
    ["eng", "zz"]
        .into_iter()
        .for_each(|value| assert!(is_iso_639_1_language_code(value).is_err()));
}
#[test]
fn test_is_kebabcase() {
    ["this-is-valid", "thisisvalid"]
        .into_iter()
        .for_each(|value| assert!(rules::kebabcase(value).is_ok()));
    ["this_is_not_valid", "ThisIsNotValid", "this is not valid", "This-Is-Not-Valid"]
        .into_iter()
        .for_each(|value| assert!(rules::kebabcase(value).is_err()));
}
#[test]
fn test_is_latlon() {
    [-90.0, -45.7, 0.0, 10.0, 90.0]
        .into_iter()
        .for_each(|value| assert!(rules::latitude(value).is_ok()));
    [-90.5, 90.5].into_iter().for_each(|value| assert!(rules::latitude(value).is_err()));
    [-180.0, -90.0, 0.0, 90.0, 180.0]
        .into_iter()
        .for_each(|value| assert!(rules::longitude(value).is_ok()));
    [-180.5, 180.5].into_iter().for_each(|value| assert!(rules::longitude(value).is_err()));
}
#[test]
fn test_is_license() {
    ["MIT", "MIT License", "Not-A-Real-License"]
        .into_iter()
        .for_each(|value| assert!(is_license(value).is_ok()));
    assert!(is_license("").is_err());
}
#[test]
fn test_is_license_invalid_database_path() {
    assert!(is_license("MIT").is_ok());
    assert_eq!(is_license(" ").expect_err("empty license should fail").code, "license");
}
#[test]
fn test_is_license_missing_database_path_env() {
    assert!(is_license("MIT").is_ok());
}
#[test]
fn test_is_orcid() {
    VALID_ORCID_VALUES
        .into_iter()
        .for_each(|value| assert!(rules::orcid(value).is_ok(), "expected valid ORCID: {value}"));
    [
        "0000-0002-1823-1234",
        "https://orcid.org/0000-0002-1823-12345",
        "https://orcid.com/0000-0002-1823-1234",
    ]
    .into_iter()
    .for_each(|value| assert!(rules::orcid(value).is_err(), "expected invalid ORCID: {value}"));
}
#[test]
fn test_is_phone_number() {
    [
        "555-123-4567",
        "555.123.4567",
        "+1 (555) 123-4567",
        "+1 (555) 123.4567",
        "+15551234567",
        "(555) 123-4567",
    ]
    .into_iter()
    .for_each(|value| assert!(rules::phone(value).is_ok(), "expected valid phone: {value}"));
    ["42", "1 (555) 123-4567", "1234-567", "123-4567", "555.555.5555", "555-555-5555"]
        .into_iter()
        .for_each(|value| assert!(rules::phone(value).is_err(), "expected invalid phone: {value}"));
}
#[test]
fn test_is_programming_language() {
    ["Rust", "Not A Language"]
        .into_iter()
        .for_each(|value| assert!(is_programming_language(value).is_ok()));
    assert!(is_programming_language("").is_err());
}
#[test]
fn test_is_programming_language_missing_database_path_env() {
    assert!(is_programming_language("Rust").is_ok());
}
#[test]
fn test_is_ror() {
    VALID_ROR_VALUES
        .into_iter()
        .for_each(|value| assert!(rules::ror(value).is_ok(), "expected valid ROR: {value}"));
    ["978-12345-99990", "10.1000/182", "3t40cz74", "03ebg0v16-"]
        .into_iter()
        .for_each(|value| assert!(rules::ror(value).is_err(), "expected invalid ROR: {value}"));
}
#[test]
fn test_is_semantic_version() {
    ["1", "1.2", "1.2.3", "0.0.0", "999.10.42"]
        .into_iter()
        .for_each(|value| assert!(rules::version(value).is_ok()));
    ["v1.2.3", "1.2.3.4", "alpha", "1.2.x", "2.1.0-alpha.1", "1.0.0+build.42"]
        .into_iter()
        .for_each(|value| assert!(rules::version(value).is_err()));
}
#[test]
fn test_isni_and_gnd_validation() {
    assert!(rules::isni("https://isni.org/isni/0000000492299539").is_ok());
    assert!(rules::isni("0000 0004 9229 9539").is_ok());
    assert!(rules::isni("0000000492299538").is_err());
    assert!(is_gnd("118540238").is_ok());
    assert!(is_gnd("gnd:118540238").is_ok());
    assert!(is_gnd("not-a-gnd").is_err());
}
#[test]
fn test_metadata_ror_list_validation() {
    let valid = ResearchActivityMetadata::init()
        .ror(VALID_ROR_VALUES.into_iter().map(str::to_string).collect::<Vec<String>>())
        .build();
    assert!(valid.validate().is_ok());
    let invalid = ResearchActivityMetadata::init()
        .ror(vec!["https://doi.org/10.1000/182".to_string()])
        .build();
    assert!(invalid.validate().is_err());
}
#[test]
fn test_scalar_enum_validation_helpers() {
    assert!(IntegerOrString::Number(42).validate().is_ok());
    assert!(IntegerOrString::Text("42".to_string()).validate().is_ok());
    assert!(IntegerOrString::Text("  ".to_string()).validate().is_err());
    assert!(NumberOrString::Number(42.5).validate().is_ok());
    assert!(NumberOrString::Text("42.5".to_string()).validate().is_ok());
    assert!(NumberOrString::Number(f64::NAN).validate().is_err());
    assert!(NumberOrString::Text(String::new()).validate().is_err());
    assert!(YearValue::Number(2025).validate().is_ok());
    assert!(YearValue::Text("2025".to_string()).validate().is_ok());
    [
        MonthValue::Number(1),
        MonthValue::Number(12),
        MonthValue::Text("1".to_string()),
        MonthValue::Text("12".to_string()),
    ]
    .iter()
    .for_each(|value| assert!(value.validate().is_ok()));
    [MonthValue::Number(0), MonthValue::Text("13".to_string())]
        .iter()
        .for_each(|value| assert!(value.validate().is_err()));
}
#[test]
fn test_unix_epoch_rule() {
    assert!(rules::unix_epoch(1_759_017_645).is_ok());
    [42, 123_456_789].into_iter().for_each(|value| assert!(rules::unix_epoch(value).is_err()));
}
#[test]
fn test_validate_postal_code() {
    [
        PostalCode::Number(37830),
        PostalCode::Text("SW1A 1AA".to_string()),
        PostalCode::Text("75008".to_string()),
    ]
    .iter()
    .for_each(|value| assert!(value.validate().is_ok()));
    [
        PostalCode::Number(-1),
        PostalCode::Text("   ".to_string()),
        PostalCode::Text("123456789012345678901".to_string()),
        PostalCode::Text("12_345".to_string()),
    ]
    .iter()
    .for_each(|value| assert!(value.validate().is_err()));
}
#[test]
fn test_validates_lowercase_sha256_digests() {
    assert!(rules::digest("sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").is_ok());
    assert!(rules::digest("sha256:ABCDEF").is_err());
    assert!(rules::digest("sha512:0123456789abcdef").is_err());
}
