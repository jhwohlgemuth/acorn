use super::*;
use acorn_core::big_o::{Dominance, SymbolGrowth};
use acorn_schema::research_activity::{
    ActivityDateType, ActivityOrganization, DatedOrganizationRole, ResearchActivityMetadata, TypedActivityDate, TypedIdentifier,
};
use acorn_schema::OneOrMany;
use alloc::collections::BTreeMap;

const BIG_O_INVALID_REFERENCE: &str = include_str!("../../../tests/fixtures/data/big_o/invalid-reference/index.json");
const BIG_O_INVALID_SYNTAX: &str = include_str!("../../../tests/fixtures/data/big_o/invalid-syntax/index.json");
const BIG_O_VALID: &str = include_str!("../../../tests/fixtures/data/big_o/valid/index.json");
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
const VALID_RESEARCH_ACTIVITY: &str = include_str!("../../../tests/fixtures/data/valid_project/index.json");
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
fn test_assess_fair_metadata_rejects_invalid_json_and_unknown_standards() {
    assert!(assess_fair("not-json", "datacite", None).is_err());
    assert!(assess_fair("{}", "dcat", None).is_err());
}
#[test]
fn test_assess_fair_metadata_returns_all_indicators_and_profile() {
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
fn test_check_readability_accepts_every_metric() {
    assert!(["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]
        .into_iter()
        .all(|metric| analyze_readability("This is a short sentence.", metric, None).is_ok()));
}
#[test]
fn test_check_readability_rejects_unknown_metrics() {
    assert!(analyze_readability("Readable text.", "unknown", None).is_err());
}
#[test]
fn test_check_readability_returns_checks_and_summary() {
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
fn test_compare_big_o_matches_portable_contract() {
    let canonical = compare_big_o_input(ComparisonInput {
        left: "O(n^2)".to_string(),
        right: "O(n log n)".to_string(),
        assumptions: BTreeMap::new(),
    })
    .expect("canonical Big-O comparison");
    assert_eq!(canonical.dominance, Dominance::Left);
    assert_eq!(canonical.assumptions, BTreeMap::from([("n".to_string(), SymbolGrowth::Growing)]));

    let explicit = compare_big_o_input(ComparisonInput {
        left: "O(n)".to_string(),
        right: "O(1)".to_string(),
        assumptions: BTreeMap::from([("n".to_string(), SymbolGrowth::FixedPositive)]),
    })
    .expect("explicit growth assumption");
    assert_eq!(explicit.dominance, Dominance::Equivalent);
    assert_eq!(explicit.assumptions.get("n"), Some(&SymbolGrowth::FixedPositive));
}
#[test]
fn test_compare_big_o_rejects_invalid_expressions_and_unknown_assumptions() {
    for (left, right, expected) in [
        ("O(n - m)", "O(n)", "invalid left Big-O expression"),
        ("O(n)", "O(n - m)", "invalid right Big-O expression"),
    ] {
        let error = compare_big_o_input(ComparisonInput {
            left: left.to_string(),
            right: right.to_string(),
            assumptions: BTreeMap::new(),
        })
        .expect_err("invalid expression");
        assert!(error.contains(expected));
    }
    let error = compare_big_o_input(ComparisonInput {
        left: "O(n)".to_string(),
        right: "O(n^2)".to_string(),
        assumptions: BTreeMap::from([("unknown".to_string(), SymbolGrowth::Growing)]),
    })
    .expect_err("unknown assumption");
    assert_eq!(error, "growth assumption `unknown` does not reference either expression");
}
#[test]
fn test_crosswalk_rejects_unknown_pairs_and_requires_raid_contact() {
    assert!(crosswalk("{}", "json", "datacite", "raid", None, None).is_err());
    assert!(crosswalk("{}", "json", "raid", "research-activity-data", None, None)
        .unwrap_err()
        .contains("contact seed"));
}
#[test]
fn test_crosswalk_returns_typed_datacite_json() {
    let result = crosswalk(&datacite_project_json(), "json", "research-activity-data", "datacite", None, None).unwrap();
    let value = serde_json::from_str::<Value>(&result.content).unwrap();
    assert_eq!(result.format, "json");
    assert_eq!(result.source, "research-activity-data");
    assert_eq!(result.target, "datacite");
    assert_eq!(value.pointer("/attributes/doi").and_then(Value::as_str), Some("10.5072/acorn-project"));
}
#[test]
fn test_diff_content_supports_unified_and_json_patch_modes() {
    let unified = diff("before\n", "after\n", "unified").unwrap();
    assert!(matches!(unified, ContentDiffResult::Unified { changed: true, .. }));
    let patch = diff(r#"{"title":"Before"}"#, r#"{"title":"After"}"#, "json-patch").unwrap();
    assert!(matches!(patch, ContentDiffResult::JsonPatch { changed: true, .. }));
    assert!(diff("{}", "{}", "unknown").is_err());
}
#[test]
fn test_discover_identifiers_filters_urls_and_normalizes_supported_pids() {
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
fn test_dublin_core_crosswalk_supports_turtle_and_requires_rad_context() {
    let turtle = r#"@prefix dcterms: <http://purl.org/dc/terms/> .
<https://example.org/dataset> dcterms:title "Imported dataset" .
"#;
    assert!(crosswalk(turtle, "turtle", "dublin-core", "research-activity-data", None, None)
        .unwrap_err()
        .contains("contact seed"));
    let contact = serde_json::to_string(&ContactPoint::default()).unwrap();
    let result = crosswalk(
        turtle,
        "turtle",
        "dublin-core",
        "research-activity-data",
        Some(&contact),
        Some("browser-import"),
    )
    .unwrap();
    let activity = serde_json::from_str::<ResearchActivity>(&result.content).unwrap();
    assert_eq!(activity.meta.identifier, "browser-import");
    assert_eq!(activity.title, "Imported dataset");
}
#[test]
fn test_dublin_core_crosswalk_writes_turtle() {
    let result = crosswalk_as(
        &datacite_project_json(),
        "json",
        "research-activity-data",
        "dublin-core",
        "turtle",
        None,
        None,
    )
    .unwrap();
    assert_eq!(result.format, "turtle");
    assert!(DublinCoreDocument::decode(&result.content, &MimeType::Turtle).is_ok());
}
#[test]
fn test_dublin_core_validation_accepts_turtle() {
    let turtle = r#"@prefix dcterms: <http://purl.org/dc/terms/> .
<https://example.org/dataset> dcterms:title "Dataset" .
"#;
    let result = validate_metadata(turtle, "dublin-core", "turtle").unwrap();
    assert!(result.valid);
}
#[test]
fn test_inspect_identifier_accepts_every_supported_kind() {
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
fn test_inspect_identifier_infers_unambiguous_values() {
    let result = inspect_identifier("https://doi.org/10.1000/182", None).unwrap();
    assert_eq!(result.status, IdentifierInspectionStatus::Valid);
    assert_eq!(result.kind, Some(PID::DOI));
    assert_eq!(result.normalized.as_deref(), Some("10.1000/182"));
}
#[test]
fn test_inspect_identifier_rejects_invalid_mismatched_and_unsupported_values() {
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
fn test_inspect_identifier_reports_name_identifier_ambiguity() {
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
fn test_validate_metadata_standard_accepts_portable_text_and_rejects_unknown_options() {
    let result = validate_metadata("A plain metadata document.", "text", "text").unwrap();
    assert!(result.valid);
    assert!(validate_metadata("{}", "dcat", "json").is_err());
    assert!(validate_metadata("{}", "datacite", "xml").is_err());
}
#[test]
fn test_validate_research_activity_collects_parse_diagnostics() {
    let result = validate_json(r#"{"title": 1, "unknown": true}"#);
    assert!(!result.valid);
    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.category == CheckCategory::Schema && diagnostic.severity == CheckSeverity::Error));
    assert!(result.value.is_none());
}
#[test]
fn test_validate_research_activity_preserves_big_o_diagnostics() {
    let valid = validate_json(BIG_O_VALID);
    assert!(valid.valid);
    assert!(valid.diagnostics.iter().all(|item| item.category != CheckCategory::Schema));

    for (json, code, message) in [
        (
            BIG_O_INVALID_SYNTAX,
            "syntax",
            "Invalid complexity expression: unexpected token in Big-O expression at byte 4",
        ),
        (BIG_O_INVALID_REFERENCE, "reference", "Complexity variable `n` is not defined"),
    ] {
        let invalid = validate_json(json);
        assert!(!invalid.valid);
        assert_eq!(
            invalid.diagnostics.iter().filter(|item| item.category == CheckCategory::Schema).count(),
            1
        );
        let diagnostic = invalid.diagnostics.first().expect("Big-O diagnostic");
        assert_eq!(diagnostic.code.as_deref(), Some(code));
        assert_eq!(diagnostic.message, message);
        assert_eq!(diagnostic.path.as_deref(), Some("aspect.computation[0].stages[0].complexity.bounds[0]"));
    }
}
#[test]
fn test_validate_research_activity_preserves_validation_order() {
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
    assert!(result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.category == CheckCategory::Schema)
        .all(|diagnostic| diagnostic.severity == CheckSeverity::Error));
    assert_eq!(paths.first().copied(), Some("title"));
    assert!(paths.contains(&"contact.email"));
}
#[test]
fn test_validate_research_activity_returns_valid_value() {
    let result = validate_json(VALID_RESEARCH_ACTIVITY);
    assert!(result.valid);
    assert!(result.value.is_some());
    assert!(!result.diagnostics.is_empty());
    for severity in [CheckSeverity::Info, CheckSeverity::Suggestion] {
        assert!(result.diagnostics.iter().any(|item| item.severity == severity));
    }
    assert_eq!(result.vocabulary_fuzzy, Some(VocabularyFuzzyMode::Relaxed));
    let off = validate_json_with_fuzzy(VALID_RESEARCH_ACTIVITY, VocabularyFuzzyMode::Off);
    assert!(!off.diagnostics.is_empty());
    assert!(off.diagnostics.iter().all(|item| item.category == CheckCategory::Fair));
    assert_eq!(off.vocabulary_fuzzy, Some(VocabularyFuzzyMode::Off));
}
#[test]
fn test_validate_research_activity_unknown_vocabulary_mode_is_rejected() {
    assert!(parse_vocabulary_fuzzy(Some("unknown")).is_err());
}
#[test]
fn test_validation_adapter_preserves_category_and_severity() {
    for category in [
        CheckCategory::Schema,
        CheckCategory::Link,
        CheckCategory::Prose,
        CheckCategory::Quality,
        CheckCategory::Compute,
        CheckCategory::Fair,
        CheckCategory::Readability,
        CheckCategory::Crosswalk,
    ] {
        for severity in [
            CheckSeverity::Error,
            CheckSeverity::Info,
            CheckSeverity::Suggestion,
            CheckSeverity::Warning,
        ] {
            let source = acorn_service::Diagnostic {
                category: category.clone(),
                code: "test.rule".into(),
                message: "Details".into(),
                path: Some("title".into()),
                severity: severity.clone(),
            };
            let diagnostic = Diagnostic::from(source);
            let value = serde_json::to_value(&diagnostic).unwrap();
            assert_eq!(value.get("category").and_then(Value::as_str).unwrap(), category.to_string());
            assert_eq!(value.get("severity").and_then(Value::as_str).unwrap(), severity.to_string());
            assert_eq!(value.get("message").unwrap(), "Details");
            assert_eq!(value.get("path").unwrap(), "title");
        }
    }
}
