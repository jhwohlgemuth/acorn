use super::*;
use acorn_schema::research_activity::{
    ActivityDateType, ActivityOrganization, DatedOrganizationRole, ResearchActivityMetadata, TypedActivityDate, TypedIdentifier,
};
use acorn_schema::OneOrMany;

const VALID_RESEARCH_ACTIVITY: &str = include_str!("../../../tests/fixtures/data/valid_project/index.json");
const VALID_IDENTIFIERS: [(&str, &str); 12] = [
    ("ark", "https://n2t.net/ark:/13030/xf93gt2q"),
    ("arxiv", "https://arxiv.org/abs/2106.09685"),
    ("doi", "https://doi.org/10.1000/182"),
    ("handle", "https://hdl.handle.net/10013/epic.10033"),
    ("isbn", "978-0-306-40627-0"),
    ("isni", "https://isni.org/isni/0000000492299539"),
    ("orcid", "https://orcid.org/0000-0002-2057-9115"),
    ("patent", "US20250123456A2"),
    ("raid", "RAID:10.83962/fb5be317"),
    ("ror", "https://ror.org/01qz5mb56"),
    ("rrid", "RRID:SCR_003512"),
    ("swhid", "swh:1:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b"),
];

fn datacite_project_json() -> String {
    let publisher_identifier = TypedIdentifier::init()
        .value("https://ror.org/01qz5mb56".to_string())
        .scheme("ROR".to_string())
        .scheme_uri("https://ror.org".to_string())
        .build();
    let publisher = ActivityOrganization::init()
        .name("Example Repository".to_string())
        .identifiers(vec![publisher_identifier.clone()])
        .roles(vec![DatedOrganizationRole::init()
            .organization(publisher_identifier)
            .role("Publisher".to_string())
            .build()])
        .build();
    let activity = ResearchActivity {
        contact: ContactPoint::init().given_name("Jane".to_string()).family_name("Doe".to_string()).build(),
        meta: ResearchActivityMetadata {
            doi: Some(OneOrMany::One("https://doi.org/10.5072/acorn-project".to_string())),
            identifier: "datacite-project".to_string(),
            organizations: Some(OneOrMany::One(publisher)),
            typed_dates: Some(OneOrMany::One(
                TypedActivityDate::init()
                    .date_type(ActivityDateType::Issued)
                    .date("2026-09-17T12:30:00-04:00".to_string())
                    .build(),
            )),
            ..ResearchActivityMetadata::default()
        },
        title: "DataCite Project activity".to_string(),
        ..ResearchActivity::default()
    };
    serde_json::to_string(&activity).unwrap()
}

#[test]
fn assess_fair_metadata_returns_all_indicators_and_profile() {
    let result = assess_fair(
        r#"{"title":"Example dataset","doi":"10.1000/182","license":"https://spdx.org/licenses/MIT.html"}"#,
        "datacite",
        Some("memory://dataset"),
    )
    .unwrap();
    assert_eq!(result.findings.len(), 41);
    assert_eq!(result.profile.resource, "10.1000/182");
    assert_eq!(
        result.profile.total.indeterminate + result.profile.total.not_applicable + result.profile.total.supported + result.profile.total.unsupported,
        41
    );
}
#[test]
fn assess_fair_metadata_rejects_invalid_json_and_unknown_standards() {
    assert!(assess_fair("not-json", "datacite", None).is_err());
    assert!(assess_fair("{}", "dcat", None).is_err());
}
#[test]
fn check_readability_accepts_every_metric() {
    assert!(["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]
        .into_iter()
        .all(|metric| analyze_readability("This is a short sentence.", metric, None).is_ok()));
}
#[test]
fn crosswalk_rejects_unknown_pairs_and_requires_raid_contact() {
    assert!(crosswalk("{}", "json", "datacite", "raid", None, None).is_err());
    assert!(crosswalk("{}", "json", "raid", "research-activity-data", None, None)
        .unwrap_err()
        .contains("contact seed"));
}
#[test]
fn crosswalk_returns_typed_datacite_json() {
    let result = crosswalk(&datacite_project_json(), "json", "research-activity-data", "datacite", None, None).unwrap();
    let value = serde_json::from_str::<Value>(&result.content).unwrap();
    assert_eq!(result.format, "json");
    assert_eq!(result.source, "research-activity-data");
    assert_eq!(result.target, "datacite");
    assert_eq!(value.pointer("/attributes/doi").and_then(Value::as_str), Some("10.5072/acorn-project"));
}
#[test]
fn diff_content_supports_unified_and_json_patch_modes() {
    let unified = diff("before\n", "after\n", "unified").unwrap();
    assert!(matches!(unified, ContentDiffResult::Unified { changed: true, .. }));
    let patch = diff(r#"{"title":"Before"}"#, r#"{"title":"After"}"#, "json-patch").unwrap();
    assert!(matches!(patch, ContentDiffResult::JsonPatch { changed: true, .. }));
    assert!(diff("{}", "{}", "unknown").is_err());
}
#[test]
fn discover_identifiers_filters_urls_and_normalizes_supported_pids() {
    let result = discover_identifiers("See https://example.org and DOI https://doi.org/10.1000/182 plus RRID:SCR_003512.");
    assert_eq!(result.count, 2);
    assert!(!result.truncated);
    assert_eq!(result.identifiers.first().map(|identifier| &identifier.kind), Some(&PID::DOI));
    assert_eq!(
        result.identifiers.first().map(|identifier| identifier.normalized.as_str()),
        Some("10.1000/182")
    );
    assert_eq!(result.identifiers.get(1).map(|identifier| &identifier.kind), Some(&PID::RRID));
}
#[test]
fn check_readability_rejects_unknown_metrics() {
    assert!(analyze_readability("Readable text.", "unknown", None).is_err());
}
#[test]
fn check_readability_returns_checks_and_summary() {
    let result = analyze_readability("This is a short sentence.", "fkgl", Some("memory://sample".to_string())).unwrap();
    assert_eq!(result.checks.len(), 1);
    assert_eq!(result.checks.first().and_then(|check| check.uri.as_deref()), Some("memory://sample"));
    assert_eq!(
        result.summary,
        vec![
            vec!["Schema items found", "0"],
            vec!["Link items found", "0"],
            vec!["Prose items found", "0"],
            vec!["Quality items found", "0"],
            vec!["Compute items found", "0"],
            vec!["Fair items found", "0"],
            vec!["Readability items found", "1"],
            vec!["Crosswalk items found", "0"],
        ]
    );
}
#[test]
fn inspect_identifier_accepts_every_supported_kind() {
    assert!(VALID_IDENTIFIERS.into_iter().all(|(kind, value)| {
        inspect_identifier(value, Some(kind)).is_ok_and(|result| {
            result.status == IdentifierInspectionStatus::Valid
                && result.kind == Some(PID::from(kind))
                && result.normalized.is_some()
                && result.candidates.is_empty()
        })
    }));
}
#[test]
fn inspect_identifier_infers_unambiguous_values() {
    let result = inspect_identifier("https://doi.org/10.1000/182", None).unwrap();
    assert_eq!(result.status, IdentifierInspectionStatus::Valid);
    assert_eq!(result.kind, Some(PID::DOI));
    assert_eq!(result.normalized.as_deref(), Some("10.1000/182"));
}
#[test]
fn inspect_identifier_rejects_invalid_mismatched_and_unsupported_values() {
    assert_eq!(
        inspect_identifier("https://doi.org/10.1000/182", Some("isbn")).unwrap(),
        IdentifierInspectionResult::invalid()
    );
    assert_eq!(
        inspect_identifier("not an identifier", None).unwrap(),
        IdentifierInspectionResult::invalid()
    );
    assert!(inspect_identifier("https://example.org", Some("url")).is_err());
    assert!(inspect_identifier("10.1000/182", Some("pidinst")).is_err());
}
#[test]
fn inspect_identifier_reports_name_identifier_ambiguity() {
    assert_eq!(
        inspect_identifier("0000-0002-2057-9115", None).unwrap(),
        IdentifierInspectionResult::ambiguous()
    );
    assert_eq!(
        inspect_identifier("0000-0002-2057-9115", Some("orcid")).unwrap().status,
        IdentifierInspectionStatus::Valid
    );
}
#[test]
fn validate_research_activity_collects_parse_diagnostics() {
    let result = validate_json(r#"{"title": 1, "unknown": true}"#);
    assert!(!result.valid);
    assert!(!result.diagnostics.is_empty());
    assert!(result.value.is_none());
}
#[test]
fn validate_metadata_standard_accepts_portable_text_and_rejects_unknown_options() {
    let result = validate_metadata("A plain metadata document.", "text", "text").unwrap();
    assert!(result.valid);
    assert!(validate_metadata("{}", "dcat", "json").is_err());
    assert!(validate_metadata("{}", "datacite", "xml").is_err());
}
#[test]
fn validate_research_activity_preserves_validation_order() {
    let invalid = VALID_RESEARCH_ACTIVITY
        .replace(r#""title": "Valid Project""#, r#""title": "x""#)
        .replace("wohlgemuthjh@ornl.gov", "invalid-email");
    let result = validate_json(&invalid);
    let paths = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.path.as_deref())
        .collect::<Vec<_>>();
    assert!(!result.valid);
    assert_eq!(paths.first().copied(), Some("title"));
    assert!(paths.contains(&"contact.email"));
}
#[test]
fn validate_research_activity_returns_valid_value() {
    let result = validate_json(VALID_RESEARCH_ACTIVITY);
    assert!(result.valid);
    assert!(result.diagnostics.is_empty());
    assert!(result.value.is_some());
}
