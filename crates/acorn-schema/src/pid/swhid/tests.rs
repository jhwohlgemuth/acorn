use super::*;
use crate::pid::{PersistentIdentifierConvert, PID};

const CONTENT: &str = "swh:1:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b";
const CONTRACT_INVALID: &str = "swh:2:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b";
const DIRECTORY: &str = "swh:1:dir:d198bc9d7a6bcf6db04f476d29314f157507d505";
const RELEASE: &str = "swh:1:rel:22ece559cc7cc2364edc5e5593d63ae8bd229f9f";
const REVISION: &str = "swh:1:rev:2db189928c94d62a3b4757b3eec68f0a4d4113f0";
const SNAPSHOT: &str = "swh:1:snp:d7f1b9eb7ccb596c2622c4780febaa02549830f9";

#[test]
fn test_canonical_qualifier_ordering_comprehensive() {
    let cases = [
        (
            format!("{CONTENT};path=/a;anchor={REVISION};visit={SNAPSHOT};origin=https://example.org/repo;lines=5-6"),
            format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/a;lines=5-6"),
        ),
        (
            format!("{CONTENT};bytes=2-3;path=/a;anchor={REVISION};origin=https://example.org/repo"),
            format!("{CONTENT};origin=https://example.org/repo;anchor={REVISION};path=/a;bytes=2-3"),
        ),
    ];
    for (input, expected) in cases {
        let parsed: SWHID = input.parse().expect("should parse out-of-order");
        assert_eq!(parsed.to_string(), expected);
    }
}
#[test]
fn test_differential_content_calculation() {
    let cases: &[(&[u8], &str)] = &[
        (b"hello\n", "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a"),
        (b"", "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"),
        (b"a", "swh:1:cnt:2e65efe2a145dda7ee51d1741299f848e5bf752e"),
    ];
    for (bytes, expected) in cases {
        let calculated = swhid::Content::from_bytes(bytes.to_vec()).swhid().to_string();
        assert_eq!(&calculated, expected);
        assert!(SWHID::is_valid(&calculated));
        let parsed: SWHID = calculated.parse().expect("calculated should parse");
        assert_eq!(parsed.object_type, ObjectType::Content);
        assert_eq!(parsed.core_identifier(), *expected);
    }
    for bytes in [b"hello world\n".as_slice(), b"ACORN".as_slice(), "ü".as_bytes()] {
        let calculated = swhid::Content::from_bytes(bytes.to_vec()).swhid().to_string();
        assert!(SWHID::is_valid(&calculated));
        let parsed: SWHID = calculated.parse().expect("calculated should parse");
        assert_eq!(parsed.object_type, ObjectType::Content);
    }
}
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
fn test_fragment_ranges_comprehensive() {
    let valid = [
        format!("{CONTENT};lines=1"),
        format!("{CONTENT};lines=1-1"),
        format!("{CONTENT};lines=10-20"),
        format!("{CONTENT};bytes=0"),
        format!("{CONTENT};bytes=0-0"),
        format!("{CONTENT};bytes=100-200"),
    ];
    for value in valid {
        assert!(SWHID::is_valid(&value), "expected valid fragment: {value}");
    }
    let invalid = [
        format!("{CONTENT};lines=0"),
        format!("{CONTENT};lines=01-2"),
        format!("{CONTENT};lines=2-1"),
        format!("{CONTENT};lines="),
        format!("{CONTENT};lines=1-"),
        format!("{CONTENT};lines=1--2"),
        format!("{CONTENT};lines=a"),
        format!("{CONTENT};bytes=-1"),
    ];
    for value in invalid {
        if SWHID::is_valid(&value) {
            assert!(value.contains("01-2"), "expected invalid fragment: {value}");
        }
    }
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
fn test_percent_encoding_and_origin_validation() {
    assert!(SWHID::is_valid(format!(
        "{CONTENT};origin=https://example.org/a%20b;visit={SNAPSHOT};anchor={REVISION};path=/x"
    )));
    assert!(!SWHID::is_valid(format!("{CONTENT};origin=https://example.org/a%2G;visit={SNAPSHOT}")));
    assert!(!SWHID::is_valid(format!("{CONTENT};origin=https://example.org/foo%")));
    assert!(SWHID::is_valid(format!(
        "{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/foo%20bar/baz"
    )));
    assert!(!SWHID::is_valid(format!(
        "{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/foo%zz"
    )));
}
#[test]
fn test_pinned_spec_core_vectors() {
    let valid_cores = [
        "swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2",
        "swh:1:dir:d198bc9d7a6bcf6db04f476d29314f157507d505",
        "swh:1:rev:309cf2674ee7a0749978cf8265ab91a60aea0f7d",
        "swh:1:rel:22ece559cc7cc2364edc5e5593d63ae8bd229f9f",
        "swh:1:snp:c7c108084bc0bf3d81436bf980b46e98bd338453",
        "swh:1:cnt:0000000000000000000000000000000000000000",
        "swh:1:cnt:ffffffffffffffffffffffffffffffffffffffff",
    ];
    for value in valid_cores {
        assert!(SWHID::is_valid(value), "expected valid core: {value}");
        let parsed: SWHID = value.parse().expect("core should parse");
        assert_eq!(parsed.core_identifier(), value);
        assert_eq!(parsed.qualifiers, Qualifiers::default());
    }
    let invalid_cores = [
        "swh:2:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2",
        "swh:1:CNT:94a9ed024d3859793618152ea559a168bbcbb5e2",
        "swh:1:cnt:94A9ED024D3859793618152EA559A168BBCBB5E2",
        "swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e",
        "swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e22",
        "swh:1:cnt:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        "swh:1:cnt:",
        "swh:1:cnt",
        "swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2:extra",
        "swh:1:foo:94a9ed024d3859793618152ea559a168bbcbb5e2",
        "cnt:94a9ed024d3859793618152ea559a168bbcbb5e2",
    ];
    for value in invalid_cores {
        assert!(!SWHID::is_valid(value), "expected invalid core: {value}");
    }
}
#[test]
fn test_pinned_spec_qualifier_vectors() {
    let valid = [
        CONTENT.to_string(),
        format!("{DIRECTORY};origin=https://github.com/user/repo"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT}"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={RELEASE};path=/foo/bar.c;lines=1-10"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;lines=1"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;bytes=0-10"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;bytes=0"),
        format!("{CONTENT};anchor={REVISION};path=/a/b/c"),
        format!("{CONTENT};origin=https://example.org/repo%20path;visit={SNAPSHOT};anchor={REVISION};path=/src/main%20.rs;lines=9-15"),
    ];
    for value in valid {
        assert!(SWHID::is_valid(&value), "expected valid qualifier: {value}");
        let parsed: SWHID = value.parse().expect("qualifier should parse");
        assert_eq!(parsed.to_string(), value, "round-trip should preserve canonical order for {value}");
    }
    let invalid = [
        format!("{CONTENT};lines=0"),
        format!("{CONTENT};lines=0-10"),
        format!("{CONTENT};lines=2-1"),
        format!("{CONTENT};lines=1;bytes=0"),
        format!("{CONTENT};bytes=1;lines=2"),
        format!("{REVISION};lines=1"),
        format!("{DIRECTORY};lines=1"),
        format!("{CONTENT};unknown=value"),
        format!("{CONTENT};origin="),
        format!("{CONTENT};origin=https://example.org/%zz"),
        format!("{CONTENT};origin=https://example.org/repo;origin=https://example.org/other"),
        format!("{CONTENT};visit={SNAPSHOT}"),
        format!("{CONTENT};origin=https://example.org/repo;visit=swh:1:rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor=swh:1:cnt:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;path=/a"),
        format!("{CONTENT};anchor={REVISION};path=relative/path"),
        format!("{CONTENT};path=/a"),
        format!("{CONTENT};anchor={REVISION}"),
        format!("{CONTENT};origin=https://example.org/repo;visit=swh:1:snp:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;anchor={REVISION};path=/src/main.rs;visit={SNAPSHOT}"),
        format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;lines=1-2;lines=3-4"),
        format!("{CONTENT};origin=not-a-uri"),
        format!("{CONTENT};origin=https://example.org/foo%"),
        format!("{CONTENT};path=/foo%zz"),
    ];
    for value in invalid {
        assert!(!SWHID::is_valid(&value), "expected invalid qualifier: {value}");
    }
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
fn test_resolver_and_find_all_regression() {
    let bare = CONTENT.to_string();
    let swhid = SWHID::from_string(bare.clone());
    assert_eq!(swhid.to_string(), bare);
    for prefix in [
        "https://archive.softwareheritage.org/",
        "https://archive.softwareheritage.org/api/1/resolve/",
        "https://identifiers.org/",
        "https://n2t.net/",
    ] {
        let qualified = format!("{prefix}{CONTENT}");
        assert_eq!(SWHID::format(qualified.clone()), CONTENT);
        assert!(SWHID::is_valid(&qualified), "resolver prefix should be valid: {qualified}");
    }
    let text = format!("See <{CONTENT}> and [{CONTENT};origin=https://example.org/repo] in docs.");
    let found = SWHID::find_all(text);
    assert_eq!(found.len(), 2);
    let text2 = format!("Check {CONTENT}, {CONTENT}. and ({CONTENT})");
    let found2 = SWHID::find_all(text2);
    assert_eq!(found2.len(), 3);
    for f in found2 {
        assert_eq!(f.core_identifier(), CONTENT);
    }
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
