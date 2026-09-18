#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use std::collections::BTreeSet;

const ALL_SUPPORTED_PID_FIXTURE: &str = include_str!("../../../../tests/fixtures/gather/all-supported-pids.txt");
const ALL_SUPPORTED_PID_CASES: [(PID, &str, &str); 13] = [
    (PID::ARK, "https://n2t.net/ark:/13030/xf93gt2q", "invalid ARK value"),
    (PID::Arxiv, "https://arxiv.org/abs/2106.09685", "arXiv:2100.00000"),
    (PID::DOI, "https://doi.org/10.1000/182", "invalid DOI value"),
    (PID::Handle, "https://hdl.handle.net/10013/epic.10033", "invalid Handle value"),
    (PID::ISBN, "978-0-306-40627-0", "0-306-40615-3"),
    (PID::ISNI, "https://isni.org/isni/0000000492299539", "0000000492299538"),
    (PID::ORCID, "https://orcid.org/0000-0002-2057-9115", "0000-0002-2057-9114"),
    (PID::Patent, "US20250123456A2", "US ABC123 B2"),
    (PID::RAID, "RAID:10.83962/fb5be317", "11.83962/fb5be317"),
    (PID::ROR, "https://ror.org/01qz5mb56", "01qz5mb55"),
    (PID::RRID, "RRID:SCR_003512", "https://example.org/RRID:BAD"),
    (
        PID::SWHID,
        "swh:1:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b",
        "swh:2:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b",
    ),
    (PID::URL, "https://example.org/research/artifact", "ftp://example.org/research/artifact"),
];

const MOD_10_OR_11_CHECK_DIGIT_CASES: [(&str, Option<&[char]>); 6] = [
    ("0-306-40615-2", Some(&['2'])),
    ("0-8044-2957-X", Some(&['X'])),
    ("978-0-306-40627-0", Some(&['0'])),
    ("978-3-16-148410-0", Some(&['0'])),
    ("978-3-16-A48410-0", None),
    ("invalid", None),
];
const MOD_11_2_CHECK_DIGIT_CASES: [(&str, Option<&[char]>); 4] = [
    ("000000031415926", Some(&['9'])),
    ("0000000492299539", Some(&['9'])),
    ("0000 0004 9229 9539", Some(&['9'])),
    ("0000-0002-2816-415X", Some(&['X'])),
];
const MOD_97_10_CHECK_DIGIT_CASES: [(&str, Option<&[char]>); 5] = [
    ("0q6v6102", Some(&['0', '2'])),
    ("1qz5mb", Some(&['5', '6'])),
    ("1wass4", Some(&['8', '0'])),
    ("2xn1ny", Some(&['0', '6'])),
    ("******", None),
];
const NOID_CHECK_DIGIT_CASES: [(&str, Option<&[char]>); 2] = [("13030/xf93gt2", Some(&['q'])), ("c8131/g3js3", Some(&['v']))];

#[test]
fn test_betanumeric() {
    assert!('9'.is_betanumeric());
    assert!(!'a'.is_betanumeric());
    assert!(!'/'.is_betanumeric());
    assert_eq!('0'.to_betanumeric_ordinal(), Some(0));
    assert_eq!('z'.to_betanumeric_ordinal(), Some(28));
}
#[test]
fn test_all_supported_pid_fixture_covers_valid_and_invalid_instances() {
    assert_eq!(ALL_SUPPORTED_PID_FIXTURE.split("\n\n").count(), 4);
    let fixture_kinds = ALL_SUPPORTED_PID_CASES.iter().map(|(kind, _, _)| kind.clone()).collect::<BTreeSet<_>>();
    let supported_kinds = PID::iter().filter(|kind| kind.is_discoverable()).collect::<BTreeSet<_>>();
    assert_eq!(fixture_kinds, supported_kinds);
    ALL_SUPPORTED_PID_CASES.iter().for_each(|(kind, valid, invalid)| {
        assert_eq!(
            ALL_SUPPORTED_PID_FIXTURE.matches(valid).count(),
            1,
            "fixture should contain one valid {kind}"
        );
        assert_eq!(
            ALL_SUPPORTED_PID_FIXTURE.matches(invalid).count(),
            1,
            "fixture should contain one invalid {kind}"
        );
        assert!(
            Identifier {
                kind: kind.clone(),
                value: (*valid).to_string(),
            }
            .normalized()
            .is_some_and(|identifier| identifier.kind == *kind),
            "expected valid {kind}: {valid}"
        );
        assert!(
            Identifier {
                kind: kind.clone(),
                value: (*invalid).to_string(),
            }
            .normalized()
            .is_none(),
            "expected invalid {kind}: {invalid}"
        );
    });
    let discovered_kinds = Identifier::find_all(ALL_SUPPORTED_PID_FIXTURE)
        .into_iter()
        .map(|identifier| identifier.kind)
        .collect::<BTreeSet<_>>();
    assert_eq!(discovered_kinds, supported_kinds);
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
fn test_identifier_unknown_name_identifier_is_ambiguous() {
    assert!(Identifier::new("0000-0002-2057-9115").normalized().is_none());
    assert!(Identifier::new("0000000492299539").normalized().is_none());
    assert_eq!(Identifier::new("ORCID: 0000-0002-2057-9115").normalized().unwrap().kind, PID::ORCID);
    assert_eq!(Identifier::new("ISNI: 0000 0004 9229 9539").normalized().unwrap().kind, PID::ISNI);
}
#[test]
fn test_idutils_url_cases() {
    [
        (
            "http://nbn-resolving.org/urn:nbn:de:bvb:19-146642",
            "http://nbn-resolving.org/urn:nbn:de:bvb:19-146642",
        ),
        ("http://pubmed.ncbi.nlm.nih.gov/12082125/", "http://pubmed.ncbi.nlm.nih.gov/12082125"),
        ("https://example.org/end.),;:!?", "https://example.org/end"),
        ("https://viaf.org/viaf/75121530", "https://viaf.org/viaf/75121530"),
    ]
    .into_iter()
    .for_each(|(value, expected)| {
        let normalized = Identifier::new(value).normalized().expect("expected valid URL");
        assert_eq!(normalized.kind, PID::URL, "{value}");
        assert_eq!(normalized.value, expected, "{value}");
    });
}
#[test]
fn test_mod_10_or_11_check_digit() {
    MOD_10_OR_11_CHECK_DIGIT_CASES.into_iter().for_each(|(value, expected)| {
        assert_eq!(mod_10_or_11_check_digit(value).as_deref(), expected, "unexpected check digit for {value}");
    });
}
#[test]
fn test_mod_11_2_check_digit() {
    MOD_11_2_CHECK_DIGIT_CASES.into_iter().for_each(|(value, expected)| {
        assert_eq!(mod_11_2_check_digit(value).as_deref(), expected, "unexpected check digit for {value}");
    });
}
#[test]
fn test_mod_97_10_check_digit() {
    MOD_97_10_CHECK_DIGIT_CASES.into_iter().for_each(|(value, expected)| {
        assert_eq!(mod_97_10_check_digit(value).as_deref(), expected, "unexpected check digit for {value}");
    });
}
#[test]
fn test_noid_check_digit() {
    NOID_CHECK_DIGIT_CASES.into_iter().for_each(|(value, expected)| {
        assert_eq!(noid_check_digit(value).as_deref(), expected, "unexpected check digit for {value}");
    });
}
#[test]
fn test_normalize_discovery_identifiers() {
    assert_eq!(Identifier::new("https://doi.org/10.1234/ABC").normalized().unwrap().value, "10.1234/ABC");
    assert_eq!(Identifier::new("ISBN 978-0-306-40627-0").normalized().unwrap().kind, PID::ISBN);
    assert_eq!(Identifier::new("ark:/12345/abc").normalized().unwrap().kind, PID::ARK);
    assert_eq!(Identifier::new("RAID:10.12345/xyz").normalized().unwrap().kind, PID::RAID);
    assert_eq!(Identifier::new("RRID:SCR_003512").normalized().unwrap().kind, PID::RRID);
    assert_eq!(Identifier::new("hdl:10013/epic.10033").normalized().unwrap().kind, PID::Handle);
    assert_eq!(Identifier::new("10013/epic.10033").normalized().unwrap().kind, PID::Handle);
    assert_eq!(Identifier::new("US-123456-A1").normalized().unwrap().kind, PID::Patent);
    assert_eq!(
        Identifier::new("swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a")
            .normalized()
            .unwrap()
            .kind,
        PID::SWHID
    );
}
#[test]
fn test_normalize_discovery_url_preserves_case_sensitive_components() {
    let normalized = Identifier::new("https://Example.org/Data/Artifact?Key=Value").normalized().unwrap();
    assert_eq!(normalized.kind, PID::URL);
    assert_eq!(normalized.value, "https://Example.org/Data/Artifact?Key=Value");
}
#[test]
fn test_normalize_pid_values_filters_invalid_values_and_duplicates() {
    assert_eq!(
        normalize::<ISNI>(vec![
            "https://isni.org/isni/0000000492299539".to_string(),
            "0000000492299538".to_string(),
            "0000 0004 9229 9539".to_string(),
        ]),
        ["0000000492299539"]
    );
    assert_eq!(
        normalize::<ORCID>(vec![
            "https://orcid.org/0000-0002-2057-9115".to_string(),
            "0000-0002-2057-9114".to_string(),
        ]),
        ["0000-0002-2057-9115"]
    );
}
#[test]
fn test_pid_display_and_serialization_use_lowercase_wire_names() {
    assert_eq!(PID::Arxiv.to_string(), "arxiv");
    assert_eq!(PID::RRID.to_string(), "rrid");
    assert_eq!(serde_json::to_string(&PID::DOI).unwrap(), r#""doi""#);
    assert_eq!(serde_json::from_str::<PID>(r#""doi""#).unwrap(), PID::DOI);
    assert_eq!(serde_json::from_str::<PID>(r#""DOI""#).unwrap(), PID::DOI);
}
#[test]
fn test_pid_from_str() {
    assert_eq!(PID::from("doi"), PID::DOI);
    assert_eq!(PID::from("ORCID"), PID::ORCID);
    assert_eq!(PID::from(" pidinst "), PID::PIDINST);
    assert_eq!(PID::from("SWHID"), PID::SWHID);
    assert_eq!(PID::from("RRID"), PID::RRID);
    assert_eq!(PID::from("unsupported"), PID::Unknown);
}
#[test]
fn test_pid_infer() {
    assert_eq!(PID::infer("doi"), PID::DOI);
    assert_eq!(PID::infer(String::from("https://orcid.org/")), PID::ORCID);
    assert_eq!(PID::infer("https://isni.org/isni/"), PID::ISNI);
    assert_eq!(PID::infer("unsupported"), PID::Unknown);
}
#[test]
fn test_rrid_is_discoverable_but_not_project_identifying() {
    assert!(PID::RRID.is_discoverable());
    assert!(!PID::RRID.is_project_identifier());
    let identifiers = Identifier::find_all("RRID:SCR_003512 https://n2t.net/RRID:SCR_003512");
    assert_eq!(identifiers, [Identifier::from(RRID::from_string("RRID:SCR_003512"))]);
}
#[test]
fn test_swhid_identity_ignores_qualifiers() {
    let core = Identifier::new("swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a")
        .normalized()
        .unwrap();
    let qualified = Identifier::new("swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a;origin=https://example.org/source;lines=1-2")
        .normalized()
        .unwrap();
    assert_eq!(core.identity_key(), qualified.identity_key());
}
