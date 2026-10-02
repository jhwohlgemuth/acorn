use super::patterns::*;
use crate::util::constants;
use fancy_regex::Regex;

#[test]
fn compiled_patterns_are_fancy_regex_compatible() {
    [
        ARK_PATTERN,
        ARXIV_DISCOVERY_PATTERN,
        ARXIV_PATTERN,
        DOI_DISCOVERY_PATTERN,
        DOI_PATTERN,
        FAKE_PHONE_PATTERN,
        GND_PATTERN,
        HANDLE_PATTERN,
        HTTP_URL_PREFIX_PATTERN,
        IMAGE_EXTENSION_PATTERN,
        ISBN_10_COMPACT_PATTERN,
        ISBN_10_PATTERN,
        ISBN_13_PATTERN,
        ISBN_PATTERN,
        ISNI_DISCOVERY_PATTERN,
        ISNI_PATTERN,
        ISO_8601_DATE_PATTERN,
        ISO_8601_YEAR_PATTERN,
        ORCID_DISCOVERY_PATTERN,
        ORCID_PATTERN,
        PARTIAL_DATE_PATTERN,
        PATENT_PATTERN,
        PHONE_PATTERN,
        RAID_PATTERN,
        ROR_PATTERN,
        RRID_PATTERN,
        SWHID_PATTERN,
        UNIX_EPOCH_PATTERN,
        URL_PATTERN,
    ]
    .into_iter()
    .for_each(|pattern| assert!(Regex::new(pattern).is_ok(), "failed to compile {pattern}"));
}

#[test]
fn generated_patterns_match_general_values() {
    [
        (FAKE_PHONE_PATTERN, "555.555.5555", "555.123.5555"),
        (HTTP_URL_PREFIX_PATTERN, "https://example.org", "ftp://example.org"),
        (IMAGE_EXTENSION_PATTERN, "figure.webp", "figure.txt"),
        (ISO_8601_DATE_PATTERN, "2025-06-04", "2025-02-30"),
        (ISO_8601_YEAR_PATTERN, "2029", "2030"),
        (PARTIAL_DATE_PATTERN, "2025-09", "2025-13"),
        (PHONE_PATTERN, "+1 (555) 123-4567", "555-123"),
        (UNIX_EPOCH_PATTERN, "1759017645", "175901764"),
        (URL_PATTERN, "https://example.org/path", "ftp://example.org/path"),
    ]
    .into_iter()
    .for_each(|(pattern, valid, invalid)| {
        let regex = Regex::new(pattern).expect("generated pattern should compile");
        assert!(regex.is_match(valid).unwrap_or(false), "expected {valid:?} to match {pattern}");
        assert!(!regex.is_match(invalid).unwrap_or(false), "expected {invalid:?} not to match {pattern}");
    });
}

#[test]
fn legacy_pattern_aliases_are_preserved() {
    [
        (constants::RE_ARK_TEXT, ARK_PATTERN),
        (constants::RE_ARXIV_DISCOVERY_TEXT, ARXIV_DISCOVERY_PATTERN),
        (constants::RE_ARXIV_TEXT, ARXIV_PATTERN),
        (constants::RE_DOI_DISCOVERY_TEXT, DOI_DISCOVERY_PATTERN),
        (constants::RE_DOI_TEXT, DOI_PATTERN),
        (constants::RE_GND_TEXT, GND_PATTERN),
        (constants::RE_HANDLE_TEXT, HANDLE_PATTERN),
        (constants::RE_ISBN_10_COMPACT_TEXT, ISBN_10_COMPACT_PATTERN),
        (constants::RE_ISBN_10_TEXT, ISBN_10_PATTERN),
        (constants::RE_ISBN_13_TEXT, ISBN_13_PATTERN),
        (constants::RE_ISBN_TEXT, ISBN_PATTERN),
        (constants::RE_ISNI_DISCOVERY_TEXT, ISNI_DISCOVERY_PATTERN),
        (constants::RE_ISNI_TEXT, ISNI_PATTERN),
        (constants::RE_ORCID_DISCOVERY_TEXT, ORCID_DISCOVERY_PATTERN),
        (constants::RE_ORCID_TEXT, ORCID_PATTERN),
        (constants::RE_PATENT_TEXT, PATENT_PATTERN),
        (constants::RE_RAID_TEXT, RAID_PATTERN),
        (constants::RE_ROR_TEXT, ROR_PATTERN),
        (constants::RE_RRID_TEXT, RRID_PATTERN),
        (constants::RE_SWHID_TEXT, SWHID_PATTERN),
        (constants::RE_URL_TEXT, URL_PATTERN),
    ]
    .into_iter()
    .for_each(|(legacy, generated)| assert_eq!(legacy, generated));
}

#[test]
fn named_captures_are_preserved() {
    let phone = Regex::new(PHONE_PATTERN)
        .expect("phone pattern should compile")
        .captures("+1 (555) 123-4567")
        .expect("phone matching should succeed")
        .expect("phone should match");
    assert_eq!(phone.name("country").map(|value| value.as_str()), Some("+1 "));
    assert_eq!(phone.name("area").map(|value| value.as_str()), Some("(555)"));
    assert_eq!(phone.name("prefix").map(|value| value.as_str()), Some("123"));
    assert_eq!(phone.name("line").map(|value| value.as_str()), Some("4567"));
    let doi = Regex::new(&format!("^{DOI_PATTERN}$"))
        .expect("DOI pattern should compile")
        .captures("https://doi.org/10.1000/182")
        .expect("DOI matching should succeed")
        .expect("DOI should match");
    assert_eq!(doi.name("schema_uri").map(|value| value.as_str()), Some("https://doi.org/"));
    assert_eq!(doi.name("registrant_code").map(|value| value.as_str()), Some("1000"));
    assert_eq!(doi.name("suffix").map(|value| value.as_str()), Some("182"));
}
