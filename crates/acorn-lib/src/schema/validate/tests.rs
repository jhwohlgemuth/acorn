#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use std::env::temp_dir;

use super::*;
use crate::schema::research_activity::ResearchActivityMetadata;
use crate::util::constants::env::DATABASE_PATH;
use crate::validation::{rules, Validate};
use proptest::prelude::*;

const VALID_ARK_VALUES: [&str; 8] = [
    "https://n2t.net/ark:99166/w66d60p2",
    "https://n2t.net/ark:/99166/w66d60p2",
    "ark:1234/w5678",
    "https://n2t.net/ark:12148/btv1b8449691v/f29",
    "https://n2t.net/ark:12148/btv1b8449691v/f29/abc.png",
    "ark:12148/btv1b8449691v/f29/abc.TIFF",
    "https://example.org/ark:12345/x6np1wh8k/c3/s5.v7.xsl",
    // https://gallica.bnf.fr/ark:/12148/btv1b8449691v
    "ark:/12148/btv1b8449691v",
];
#[test]
fn test_validates_lowercase_sha256_digests() {
    let valid = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    assert!(rules::digest(valid).is_ok());
    assert!(rules::digest("sha256:ABCDEF").is_err());
    assert!(rules::digest("sha512:0123456789abcdef").is_err());
}
const VALID_DOI_VALUES: [&str; 31] = [
    // ACORN DOI
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
    // Jason Wohlgemuth
    "https://orcid.org/0000-0002-2057-9115",
    // Audrey Carson
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
// Downloaded from https://zenodo.org/records/14728473
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
fn test_date_validation() {
    let valid = ["2000-01-01", "2025-06-04", "1950-12-28"];
    let invalid = [
        // "3000-01-01",
        "1949-01-02",
        "01-02-2023",
        "1234",
        "foo",
        "42",
        "2025/06/04",
    ];
    for x in valid.iter() {
        assert!(rules::date(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid ISO 8601 date (YYYY-MM-DD)");
    }
    for x in invalid.iter() {
        assert!(rules::date(x).is_err(), "=> [REASON] \"{x}\" IS valid ISO 8601 date (YYYY-MM-DD)");
    }
    let valid_years = ["2000", "2025", "1950", "0999"];
    let invalid_years = ["3000", "999", "42"];
    for x in valid_years.iter() {
        assert!(rules::year(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid ISO 8601 year (YYYY)");
    }
    for x in invalid_years.iter() {
        assert!(rules::year(x).is_err(), "=> [REASON] \"{x}\" IS valid ISO 8601 year (YYYY)");
    }
}
#[test]
fn test_format_phone_number() {
    let no_country_code = ["555-123-4567", "555.123.4567", "(555) 123-4567"];
    for x in no_country_code.iter() {
        assert_eq!(format_phone_number(x), Ok("555.123.4567".to_string()));
    }
    let with_country_code = ["+1 (555) 123-4567", "+1 (555) 123.4567", "+15551234567"];
    for x in with_country_code.iter() {
        assert_eq!(format_phone_number(x), Ok("+1.555.123.4567".to_string()));
    }
}
#[test]
fn test_format_timestamp() {
    let normalized = format_timestamp("2026-03-10T15:37:24.355614500+00:00");
    assert_eq!(normalized, "2026-03-10T15:37:24.355614+00:00");
    let normalized_zulu = format_timestamp("2023-01-01T12:00:00.123456789Z");
    assert_eq!(normalized_zulu, "2023-01-01T12:00:00.123456+00:00");
    let normalized_offset = format_timestamp("2026-03-10T15:37:24.5-04:30");
    assert_eq!(normalized_offset, "2026-03-10T15:37:24.500000-04:30");
    let passthrough = format_timestamp("not-a-date");
    assert_eq!(passthrough, "not-a-date");
}
#[test]
fn test_has_image_extension() {
    let valid = [
        "foo.png",
        "bar.jpg",
        "baz.jpeg",
        "qux.svg",
        "https://example.com/foo.PNG",
        "https://example.com/bar.JPEG",
    ];
    let invalid = ["foo.pNg", "bar.jpx", "qux.sVG", "https://example.com/fooPNG"];
    for x in valid.iter() {
        assert!(
            has_image_extension(x).is_ok(),
            "=> [REASON] \"{x}\" does NOT HAVE a valid image extension"
        );
    }
    for x in invalid.iter() {
        assert!(has_image_extension(x).is_err(), "=> [REASON] \"{x}\" HAS a valid image extension");
    }
}
#[test]
fn test_is_ark() {
    let valid = VALID_ARK_VALUES;
    let invalid = [
        "978-12345-99990",
        // HTTP not supported
        "http://n2t.net/ark:99166/w66d60p2",
        // Missing ark: label
        "https://n2t.net/99166/w66d60p2",
        // Shoulder starts with number
        "https://n2t.net/99166/9w66d60p2",
        // Shoulder contains letter "l"
        "https://n2t.net/99166/lw66d60p2",
    ];
    for x in valid.iter() {
        assert!(rules::ark(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ARK");
    }
    for x in invalid.iter() {
        assert!(rules::ark(x).is_err(), "=> [REASON] \"{x}\" IS a valid ARK");
    }
    match rules::ark("https://n2t.net/99166/w66d60p2") {
        | Ok(_) => panic!(),
        | Err(err) => assert_eq!(err.to_string(), "Provide valid ARK value"),
    }
}
#[test]
fn test_is_doi() {
    let valid = VALID_DOI_VALUES;
    let invalid = [
        "978-12345-99990",
        // Pretty sure 11 is not a valid DOI directory indicator
        "11.11578/dc.20250604.1",
        "<geo coords=\"10.4515260,51.1656910\"></geo>",
        // 10.5555 is not a DOI prefix, but rather a handle prefix
        "10.5555/182",
    ];
    for x in valid.iter() {
        assert!(rules::doi(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid DOI");
    }
    for x in invalid.iter() {
        assert!(rules::doi(x).is_err(), "=> [REASON] \"{x}\" IS a valid DOI");
    }
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
fn test_is_ip6() {
    let valid = [
        "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        "FE80:0000:0000:0000:0202:B3FF:FE1E:8329",
        "::",
        "::::",
    ];
    let invalid = ["192.168.1.1", "127.0.0.1", "test:test:test:test:test:test:test:test"];
    for x in valid.iter() {
        assert!(is_ip6(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid IP6 address");
    }
    for x in invalid.iter() {
        assert!(is_ip6(x).is_err(), "=> [REASON] \"{x}\" IS a valid IP6 address");
    }
}
#[test]
fn test_is_kebabcase() {
    let valid = ["this-is-valid", "thisisvalid"];
    let invalid = ["this_is_not_valid", "ThisIsNotValid", "this is not valid", "This-Is-Not-Valid"];
    for x in valid.iter() {
        assert!(rules::kebabcase(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid kebab-case");
    }
    for x in invalid.iter() {
        assert!(rules::kebabcase(x).is_err(), "=> [REASON] \"{x}\" IS valid kebab-case");
    }
}
#[test]
fn test_is_commit() {
    let valid = [
        "a1b2c3d",
        "A1B2C3D4",
        "0123456789abcdef0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ];
    let invalid = [
        "",
        "123456",
        "xyzxyzx",
        "g123456",
        "123456 ",
        "12345-678",
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef0",
    ];
    for x in valid.iter() {
        assert!(rules::commit(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid commit hash");
    }
    for x in invalid.iter() {
        assert!(rules::commit(x).is_err(), "=> [REASON] \"{x}\" IS a valid commit hash");
    }
}
#[test]
fn test_is_orcid() {
    let valid = VALID_ORCID_VALUES;
    let invalid = [
        "0000-0002-1823-1234",
        "https://orcid.org/0000-0002-1823-12345",
        "https://orcid.com/0000-0002-1823-1234",
    ];
    for x in valid.iter() {
        assert!(rules::orcid(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ORCiD");
    }
    for x in invalid.iter() {
        assert!(rules::orcid(x).is_err(), "=> [REASON] \"{x}\" IS a valid ORCiD");
    }
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
fn test_is_programming_language() {
    let path = crate::test::utils::temp_database_with_programming_languages().expect("failed to initialize temporary programming-language database");
    let path_string = path.to_string_lossy().to_string();
    temp_env::with_vars([(DATABASE_PATH, Some(path_string.as_str()))], || {
        assert!(is_programming_language("Rust").is_ok());
        assert!(is_programming_language("rust").is_ok());
        assert!(is_programming_language("Not A Language").is_err());
        assert!(is_programming_language("").is_err());
    });
}
#[test]
fn test_is_programming_language_missing_database_path_env() {
    temp_env::with_vars([(DATABASE_PATH, None::<&str>)], || {
        assert!(is_programming_language("Rust").is_err());
    });
}
#[test]
fn test_is_phone_number() {
    let valid = [
        "555-123-4567",
        "555.123.4567",
        "+1 (555) 123-4567",
        "+1 (555) 123.4567",
        "+15551234567",
        "(555) 123-4567",
    ];
    let invalid = [
        // Just a number
        "42",
        // Missing country code sign
        "1 (555) 123-4567",
        // Wrong number of digits
        "1234-567",
        // Missing area code
        "123-4567",
        // Fake
        "555.555.5555",
        // Fake
        "555-555-5555",
    ];
    for x in valid.iter() {
        assert!(rules::phone(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid phone number");
    }
    for x in invalid.iter() {
        assert!(rules::phone(x).is_err(), "=> [REASON] \"{x}\" IS a valid phone number");
    }
}
#[test]
fn test_is_semantic_version() {
    let valid = ["1", "1.2", "1.2.3", "0.0.0", "999.10.42"];
    let invalid = ["v1.2.3", "1.2.3.4", "alpha", "1.2.x", "2.1.0-alpha.1", "1.0.0+build.42"];
    for x in valid.iter() {
        assert!(rules::version(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid semantic version");
    }
    for x in invalid.iter() {
        assert!(rules::version(x).is_err(), "=> [REASON] \"{x}\" IS a valid semantic version");
    }
}
#[test]
fn test_is_ror() {
    let valid = VALID_ROR_VALUES;
    let invalid = ["978-12345-99990", "10.1000/182", "10.97812345/99990", "3t40cz74", "03ebg0v16-"];
    for x in valid.iter() {
        assert!(rules::ror(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ROR");
    }
    for x in invalid.iter() {
        assert!(rules::ror(x).is_err(), "=> [REASON] \"{x}\" IS a valid ROR");
    }
}
#[test]
fn test_metadata_ror_list_validation() {
    let valid = ResearchActivityMetadata::init()
        .ror(VALID_ROR_VALUES.into_iter().map(|x| x.to_string()).collect::<Vec<String>>())
        .build();
    assert!(valid.validate().is_ok());
    let invalid = ResearchActivityMetadata::init()
        .ror(vec!["https://doi.org/10.1000/182".to_string()])
        .build();
    assert!(invalid.validate().is_err());
}
#[test]
fn test_country_rule() {
    let valid_codes = ["US", "FR", "DE", "GB", "JP", "us", "fr", "de", "gb", "jp"];
    for code in valid_codes.iter() {
        assert!(
            rules::country(code).is_ok(),
            "Expected {code} to be valid ISO 3166-1 alpha-2 country code"
        );
    }
    let invalid_codes = [
        "USA",  // 3 letters
        "U",    // 1 letter
        "U.S",  // 3 chars with dot
        "US1",  // contains digit
        "U-S",  // contains dash
        "XX",   // invalid country code
        "ZZ",   // invalid country code
        "",     // empty
        "U",    // single letter
        "USa ", // 4 chars with space
    ];
    for code in invalid_codes.iter() {
        assert!(
            rules::country(code).is_err(),
            "Expected {code} to be invalid ISO 3166-1 alpha-2 country code"
        );
    }
}
#[test]
fn test_is_iso_639_language_code_validation() {
    assert!(is_iso_639_language_code("en").is_ok());
    assert!(is_iso_639_language_code("EN").is_ok());
    assert!(is_iso_639_language_code("en-US").is_ok());
    assert!(is_iso_639_language_code("eng").is_err());
    assert!(is_iso_639_language_code("zz-ZZ").is_err());
    assert!(is_iso_639_1_language_code("en").is_ok());
    assert!(is_iso_639_1_language_code("EN").is_ok());
    assert!(is_iso_639_1_language_code("eng").is_err());
    assert!(is_iso_639_1_language_code("zz").is_err());
}
#[test]
fn test_is_latlon() {
    let valid_latitude = [-90_f64, -45.7_f64, 0., 10_f64, 90_f64];
    for x in valid_latitude.iter() {
        assert!(rules::latitude(*x).is_ok(), "=> [REASON] \"{x}\" IS NOT a valid latitude value");
    }
    let invalid_latitude = [-90.5_f64];
    for x in invalid_latitude.iter() {
        assert!(rules::latitude(*x).is_err(), "=> [REASON] \"{x}\" IS a valid latitude value");
    }

    let valid_longitude = [-180_f64, -90_f64, 0., 90_f64, 180_f64];
    for x in valid_longitude.iter() {
        assert!(rules::longitude(*x).is_ok(), "=> [REASON] \"{x}\" IS NOT a valid longitude value");
    }
    let invalid_longitude = [-180.5_f64, 180.5_f64];
    for x in invalid_longitude.iter() {
        assert!(rules::longitude(*x).is_err(), "=> [REASON] \"{x}\" IS a valid longitude value");
    }
}
proptest! {
    #[test]
    fn prop_is_latitude_accepts_values_in_range(value in -90.0f64..=90.0f64) {
        prop_assert!(rules::latitude(value).is_ok());
    }
    #[test]
    fn prop_is_latitude_rejects_values_out_of_range(
        value in prop_oneof![-1000000.0f64..-90.0000001f64, 90.0000001f64..1000000.0f64]
    ) {
        prop_assert!(rules::latitude(value).is_err());
    }
    #[test]
    fn prop_is_longitude_accepts_values_in_range(value in -180.0f64..=180.0f64) {
        prop_assert!(rules::longitude(value).is_ok());
    }
    #[test]
    fn prop_is_longitude_rejects_values_out_of_range(
        value in prop_oneof![-1000000.0f64..-180.0000001f64, 180.0000001f64..1000000.0f64]
    ) {
        prop_assert!(rules::longitude(value).is_err());
    }
}
#[tokio::test]
async fn test_is_license() {
    let path = crate::test::utils::temp_database_with_licenses()
        .await
        .expect("failed to initialize temporary test database");
    let path_string = path.to_string_lossy().to_string();
    temp_env::with_vars([(DATABASE_PATH, Some(path_string.as_str()))], || {
        assert!(is_license("MIT").is_ok());
        assert!(is_license("mit").is_ok());
        assert!(is_license("MIT License").is_ok());
        assert!(is_license("Not-A-Real-License").is_err());
        assert!(is_license("").is_err());
    });
}
#[test]
#[ignore = "requires isolated process (run with cargo nextest)"]
fn test_is_license_invalid_database_path() {
    let dir = temp_dir().join("acorn_test_invalid_db");
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    let corrupt_db = dir.join("corrupt.db");
    std::fs::write(&corrupt_db, b"not a sqlite database").expect("failed to write corrupt db");
    let path_string = corrupt_db.to_string_lossy().to_string();
    temp_env::with_vars([(DATABASE_PATH, Some(path_string.as_str()))], || {
        let result = is_license("MIT");
        assert!(result.is_err());
        let why = result.unwrap_err();
        assert_eq!(why.message.as_deref(), Some("Failed to access local database for license validation"));
        assert_eq!(why.params.get("value").and_then(|value| value.as_str()), Some("MIT"));
    });
    std::fs::remove_dir_all(&dir).ok();
}
#[test]
fn test_is_license_missing_database_path_env() {
    temp_env::with_vars([(DATABASE_PATH, None::<&str>)], || {
        let result = is_license("MIT");
        assert!(result.is_err());
        let why = result.unwrap_err();
        assert_eq!(why.params.get("value").and_then(|value| value.as_str()), Some("MIT"));
    });
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
    let valid_month_values = [
        MonthValue::Number(1),
        MonthValue::Number(12),
        MonthValue::Text("1".to_string()),
        MonthValue::Text("12".to_string()),
    ];
    valid_month_values
        .iter()
        .for_each(|value| assert!(value.validate().is_ok(), "expected valid month value: {value:?}"));

    let invalid_month_values = [MonthValue::Number(0), MonthValue::Text("13".to_string())];
    invalid_month_values.iter().for_each(|value| {
        let errors = value.validate().expect_err("expected invalid month value to fail validation");
        assert!(errors.iter().any(|issue| issue.path == "month"));
    });
}
#[test]
fn test_validate_postal_code() {
    let valid_values = [
        PostalCode::Number(37830),
        PostalCode::Text("SW1A 1AA".to_string()),
        PostalCode::Text("75008".to_string()),
    ];
    valid_values
        .iter()
        .for_each(|value| assert!(value.validate().is_ok(), "expected postal code to be valid: {value:?}"));
    let invalid_values = [
        PostalCode::Number(-1),
        PostalCode::Text("   ".to_string()),
        PostalCode::Text("123456789012345678901".to_string()),
        PostalCode::Text("12_345".to_string()),
    ];
    invalid_values.iter().for_each(|value| {
        let errors = value.validate().expect_err("expected postal code to fail validation");
        assert!(errors.iter().any(|issue| issue.path == "postal_code"));
    });
}
#[test]
fn test_unix_epoch_rule() {
    let valid = [1759017645];
    let invalid = [42, 123456789];
    for x in valid.iter() {
        assert!(rules::unix_epoch(*x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid Unix epoch timestamp");
    }
    for x in invalid.iter() {
        assert!(rules::unix_epoch(*x).is_err(), "=> [REASON] \"{x}\" IS a valid Unix epoch timestamp");
    }
}
