use super::*;
use crate::schema::pid::{PersistentIdentifierConvert, PID};

const CONTENT: &str = "swh:1:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b";
const CONTRACT_INVALID: &str = "swh:2:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b";
const REVISION: &str = "swh:1:rev:2db189928c94d62a3b4757b3eec68f0a4d4113f0";
const SNAPSHOT: &str = "swh:1:snp:d7f1b9eb7ccb596c2622c4780febaa02549830f9";

#[test]
fn test_find_all_preserves_qualified_identifier() {
    let value = format!("See {CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;lines=9-15.");
    let found = SWHID::find_all(value);
    assert_eq!(found.len(), 1);
    assert_eq!(
        found.first().and_then(|identifier| identifier.qualifiers.lines),
        Some(FragmentRange { start: 9, end: Some(15) })
    );
}
#[test]
fn test_idutils_swhid_cases() {
    // Adapted from idutils's BSD-3-Clause test suite: https://github.com/inveniosoftware/idutils/blob/master/tests/test_idutils.py
    [
        "swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2",
        "swh:1:dir:d198bc9d7a6bcf6db04f476d29314f157507d505",
        "swh:1:rev:309cf2674ee7a0749978cf8265ab91a60aea0f7d",
        "swh:1:rel:22ece559cc7cc2364edc5e5593d63ae8bd229f9f",
        "swh:1:snp:c7c108084bc0bf3d81436bf980b46e98bd338453",
        "swh:1:dir:d198bc9d7a6bcf6db04f476d29314f157507d505;origin=https://github.com/user/repo",
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/python/cpython",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";path=/Programs/python.c;lines=12-16"
        ),
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";path=/Programs/python.c;lines=12-16",
            ";origin=https://github.com/python/cpython"
        ),
    ]
    .into_iter()
    .for_each(|value| assert!(SWHID::is_valid(value), "expected valid SWHID: {value}"));
    [
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/python/cpython",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";path=Programs/python.c;lines=12-16"
        ),
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/py%thon/cpython",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";path=/Programs/python.c;lines=12-16"
        ),
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/python/cpython",
            ";visit=https://github.com/python/cpython",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";path=/Programs/python.c;lines=1-2"
        ),
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/python/cpython",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";anchor=https://github.com/python/cpython",
            ";path=/Programs/python.c;lines=1-2"
        ),
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/python/cpython",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";path=/Programs/python.c;lines=12--16"
        ),
        concat!(
            "swh:1:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/python/cpython",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";path=/Programs/python.c;lines="
        ),
        concat!(
            "swh:2:cnt:78e48f800c950530e36d3712d9e2e89673f23562",
            ";origin=https://github.com/python/cpython",
            ";visit=swh:1:snp:cd510e99a42139ed36f15a5774301c113c3e494b",
            ";anchor=swh:1:rel:ae1f6af15f3e4110616801e235873e47fd7d1977",
            ";path=/Programs/python.c;lines=1-2"
        ),
    ]
    .into_iter()
    .for_each(|value| assert!(!SWHID::is_valid(value), "expected invalid SWHID: {value}"));
}
#[test]
fn test_parse_core_object_types() {
    ["cnt", "dir", "rel", "rev", "snp"].into_iter().for_each(|kind| {
        let value = format!("swh:1:{kind}:0123456789abcdef0123456789abcdef01234567");
        assert!(SWHID::is_valid(value));
    });
}
#[test]
fn test_parse_qualified_identifier_canonicalizes_order() {
    let value = format!("{CONTENT};lines=9-15;path=/src/main.rs;anchor={REVISION};origin=https://example.org/repo;visit={SNAPSHOT}");
    let parsed: SWHID = value.parse().expect("qualified SWHID should parse");
    assert_eq!(
        parsed.to_string(),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;lines=9-15")
    );
    assert_eq!(parsed.core_identifier(), CONTENT);
}
#[test]
fn test_rejects_invalid_core_and_qualifiers() {
    let invalid = [
        "swh:2:cnt:0123456789abcdef0123456789abcdef01234567",
        "swh:1:CNT:0123456789abcdef0123456789abcdef01234567",
        "swh:1:cnt:0123456789ABCDEF0123456789abcdef01234567",
        "swh:1:cnt:0123456789abcdef",
        "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;lines=0",
        "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;lines=1;bytes=0",
        "swh:1:rev:0123456789abcdef0123456789abcdef01234567;lines=1",
        "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;unknown=value",
        "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;origin=https://example.org/%zz",
    ];
    invalid.into_iter().for_each(|value| assert!(!SWHID::is_valid(value), "{value}"));
}
#[test]
fn test_resolver_url_normalizes_to_bare_identifier() {
    let value = format!("https://archive.softwareheritage.org/{CONTENT}");
    assert_eq!(SWHID::format(value), CONTENT);
}
#[test]
fn test_swhid_trait_contract() {
    assert_eq!(SWHID::new().to_string(), SWHID::default().to_string());
    assert!(SWHID::is_valid(CONTENT));
    assert!(!SWHID::is_valid(CONTRACT_INVALID));
    assert_eq!(SWHID::format(CONTENT), CONTENT);
    let pid = SWHID::from_string(CONTENT);
    assert_eq!(pid.to_string(), CONTENT);
    assert_eq!(pid.schema_uri(), "https://www.swhid.org/specification/v1.2");
    assert_eq!(pid.identifier(), CONTENT);
    assert_eq!(pid.prefix(), Some("swh:1:cnt".to_string()));
    assert_eq!(pid.suffix(), Some("4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b".to_string()));
    assert!(pid.check_digit().is_none());
    assert_eq!(pid.url(), format!("https://archive.softwareheritage.org/{CONTENT}"));
    let found = <SWHID as PersistentIdentifierParse>::find_all(format!("Identifier: {CONTENT}"));
    assert_eq!(found.len(), 1);
    assert_eq!(found.first().map(ToString::to_string).as_deref(), Some(CONTENT));
    assert!(CONTENT.is_pid(PID::SWHID));
    assert_eq!(CONTENT.format_as(PID::SWHID), CONTENT);
    assert_eq!(CONTENT.to_pid(PID::SWHID).to_swhid().to_string(), CONTENT);
}
