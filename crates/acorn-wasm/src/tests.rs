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
const CFF_FIXTURE: &str = include_str!("../../../tests/fixtures/CITATION.cff");
const DATACITE_FIXTURE: &str = include_str!("../../../tests/fixtures/schema/datacite.json");
const DUBLIN_CORE_FIXTURE: &str = include_str!("../../../tests/fixtures/schema/dublin-core-format.json");
const INVENIO_FIXTURE: &str = include_str!("../../../tests/fixtures/schema/invenio-format.json");
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
fn test_assess_fair_skips_principles_before_scoring() {
    let result = assess_fair_with_skip("{}", "datacite", None, &[FairPrinciple::Findable, FairPrinciple::Reusable]).unwrap();
    assert!(result
        .findings
        .iter()
        .all(|finding| matches!(finding.indicator.principle, FairPrinciple::Accessible | FairPrinciple::Interoperable)));
    assert_eq!(
        result.profile.total.indeterminate + result.profile.total.not_applicable + result.profile.total.supported + result.profile.total.unsupported,
        result.findings.len()
    );
    let empty = assess_fair_with_skip(
        "{}",
        "datacite",
        None,
        &[
            FairPrinciple::Findable,
            FairPrinciple::Accessible,
            FairPrinciple::Interoperable,
            FairPrinciple::Reusable,
        ],
    )
    .unwrap();
    assert!(empty.findings.is_empty());
}
#[test]
fn test_check_readability_accepts_every_metric() {
    assert!(["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]
        .into_iter()
        .all(|metric| analyze_readability("This is a short sentence.", metric, None, None).is_ok()));
}
#[test]
fn test_check_readability_applies_threshold() {
    let result = analyze_readability("This is a short sentence.", "fres", None, Some(70.0)).unwrap();
    assert_eq!(result.checks.first().map(|check| check.threshold), Some(70.0));
}
#[test]
fn test_check_readability_rejects_unknown_metrics() {
    assert!(analyze_readability("Readable text.", "unknown", None, None).is_err());
}
#[test]
fn test_check_readability_returns_checks_and_summary() {
    let result = analyze_readability("This is a short sentence.", "fkgl", Some("memory://sample".to_string()), None).unwrap();
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
fn test_discover_identifiers_accepts_escaped_slashes() {
    let result = discover_identifiers(r#""https:\/\/doi.org\/10.15760\/etd.6294""#);
    assert_eq!(result.count, 1);
    assert!(!result.truncated);
    assert_eq!(
        result
            .identifiers
            .first()
            .map(|identifier| (&identifier.kind, identifier.normalized.as_str())),
        Some((&PID::DOI, "10.15760/etd.6294"))
    );
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
fn test_format_metadata_normalizes_cff_syntaxes_idempotently() {
    let source = Cff::decode(CFF_FIXTURE, &MimeType::Cff).unwrap();
    let expected = serde_json::to_value(&source).unwrap();
    for (format, mime) in [("cff", MimeType::Cff), ("json", MimeType::Json), ("yaml", MimeType::Yaml)] {
        let input = if format == "cff" {
            CFF_FIXTURE.to_string()
        } else {
            source.encode(&mime).unwrap()
        };
        let first = format_metadata_input(&input, "citation-file-format", format, None).unwrap();
        assert_eq!(first.standard, "citation-file-format");
        assert_eq!(first.format, format);
        assert_eq!(first.vocabulary_fuzzy, VocabularyFuzzyMode::Strict);
        assert_eq!(serde_json::to_value(Cff::decode(&first.content, &mime).unwrap()).unwrap(), expected);
        let repeated = format_metadata_input(&first.content, "citation-file-format", format, None).unwrap();
        assert_eq!(repeated.content, first.content);
        assert!(!repeated.changed);
    }
    assert!(format_metadata_input(":", "citation-file-format", "cff", None).is_err());
    assert!(format_metadata_input("{", "citation-file-format", "json", None).is_err());
    assert!(format_metadata_input(":", "citation-file-format", "yaml", None).is_err());
    assert!(format_metadata_input("{}", "citation-file-format", "markdown", None).is_err());
}
#[test]
fn test_format_metadata_normalizes_datacite_json_and_yaml() {
    let records: Value = serde_json::from_str(DATACITE_FIXTURE).unwrap();
    let source = serde_json::to_string(records.as_array().and_then(|records| records.first()).unwrap()).unwrap();
    let record = datacite::Record::decode(&source, &MimeType::Json).unwrap();
    let expected = serde_json::to_value(&record).unwrap();
    for (format, mime) in [("json", MimeType::Json), ("yaml", MimeType::Yaml)] {
        let input = if format == "json" {
            source.clone()
        } else {
            record.encode(&mime).unwrap()
        };
        let first = format_metadata_input(&input, "datacite", format, None).unwrap();
        assert_eq!(first.standard, "datacite");
        assert_eq!(first.format, format);
        assert_eq!(
            serde_json::to_value(datacite::Record::decode(&first.content, &mime).unwrap()).unwrap(),
            expected
        );
        let repeated = format_metadata_input(&first.content, "datacite", format, None).unwrap();
        assert_eq!(repeated.content, first.content);
        assert!(!repeated.changed);
    }
    let one = format_metadata_input(&format!("[{source}]"), "datacite", "json", None).unwrap();
    assert!(one.content.trim_start().starts_with('{'));
    assert!(format_metadata_input(&format!("[{source},{source}]"), "datacite", "json", None).is_err());
    assert!(format_metadata_input("{", "datacite", "json", None).is_err());
    assert!(format_metadata_input(":", "datacite", "yaml", None).is_err());
    assert!(format_metadata_input("{}", "datacite", "jsonc", None).is_err());
}
#[test]
fn test_format_metadata_normalizes_invenio_json_and_yaml() {
    let record = invenio::Record::decode(INVENIO_FIXTURE, &MimeType::Json).unwrap();
    let expected = serde_json::to_value(&record).unwrap();
    for (format, mime) in [("json", MimeType::Json), ("yaml", MimeType::Yaml)] {
        let input = if format == "json" {
            INVENIO_FIXTURE.to_string()
        } else {
            record.encode(&mime).unwrap()
        };
        let first = format_metadata_input(&input, "invenio", format, None).unwrap();
        assert_eq!(first.standard, "invenio");
        assert_eq!(first.format, format);
        assert_eq!(
            serde_json::to_value(invenio::Record::decode(&first.content, &mime).unwrap()).unwrap(),
            expected
        );
        let repeated = format_metadata_input(&first.content, "invenio", format, None).unwrap();
        assert_eq!(repeated.content, first.content);
        assert!(!repeated.changed);
    }
    let one = format_metadata_input(&format!("[{INVENIO_FIXTURE}]"), "invenio", "json", None).unwrap();
    assert!(one.content.trim_start().starts_with('{'));
    assert!(format_metadata_input(&format!("[{INVENIO_FIXTURE},{INVENIO_FIXTURE}]"), "invenio", "json", None).is_err());
    assert!(format_metadata_input("{", "invenio", "json", None).is_err());
    assert!(format_metadata_input(":", "invenio", "yaml", None).is_err());
    assert!(format_metadata_input("{}", "invenio", "jsonc", None).is_err());
}
#[test]
fn test_format_metadata_normalizes_rad_json_idempotently() {
    let first = format_metadata_input(VALID_RESEARCH_ACTIVITY, "research-activity-data", "json", None).unwrap();
    assert_eq!(first.standard, "research-activity-data");
    assert_eq!(first.format, "json");
    assert_eq!(first.vocabulary_fuzzy, VocabularyFuzzyMode::Strict);
    assert!(ResearchActivity::decode(&first.content, &MimeType::Json).is_ok());
    let second = format_metadata_input(&first.content, "research-activity-data", "json", Some("strict")).unwrap();
    assert_eq!(second.content, first.content);
    assert!(!second.changed);
}
#[test]
fn test_format_metadata_normalizes_rad_jsonc_and_yaml_idempotently() {
    let jsonc = format!("// Browser comment removed during normalization\n{VALID_RESEARCH_ACTIVITY}");
    let formatted_jsonc = format_metadata_input(&jsonc, "research-activity-data", "jsonc", None).unwrap();
    assert_eq!(formatted_jsonc.format, "jsonc");
    assert!(!formatted_jsonc.content.contains("Browser comment"));
    assert!(ResearchActivity::decode(&formatted_jsonc.content, &MimeType::Jsonc).is_ok());
    let repeated_jsonc = format_metadata_input(&formatted_jsonc.content, "research-activity-data", "jsonc", None).unwrap();
    assert!(!repeated_jsonc.changed);
    assert_eq!(repeated_jsonc.content, formatted_jsonc.content);

    let activity = ResearchActivity::decode(VALID_RESEARCH_ACTIVITY, &MimeType::Json).unwrap();
    assert!(ResearchActivity::decode(VALID_RESEARCH_ACTIVITY, &MimeType::Yaml).is_ok());
    let yaml = activity.encode(&MimeType::Yaml).unwrap();
    let formatted_yaml = format_metadata_input(&yaml, "research-activity-data", "yaml", None).unwrap();
    assert_eq!(formatted_yaml.format, "yaml");
    assert!(ResearchActivity::decode(&formatted_yaml.content, &MimeType::Yaml).is_ok());
    let repeated_yaml = format_metadata_input(&formatted_yaml.content, "research-activity-data", "yaml", None).unwrap();
    assert!(!repeated_yaml.changed);
    assert_eq!(repeated_yaml.content, formatted_yaml.content);
    assert!(format_metadata_input("meta: [", "research-activity-data", "yaml", None).is_err());
    assert!(format_metadata_input("/*", "research-activity-data", "jsonc", None).is_err());
}
#[test]
fn test_format_metadata_normalizes_rad_markdown_and_zon_idempotently() {
    let source = ResearchActivity::decode(VALID_RESEARCH_ACTIVITY, &MimeType::Json).unwrap();
    for (format, mime) in [("markdown", MimeType::Markdown), ("zon", MimeType::Zon)] {
        let encoded = source.encode(&mime).unwrap();
        let first = format_metadata_input(&encoded, "research-activity-data", format, None).unwrap();
        assert_eq!(first.format, format);
        assert!(ResearchActivity::decode(&first.content, &mime).is_ok());
        let repeated = format_metadata_input(&first.content, "research-activity-data", format, None).unwrap();
        assert_eq!(repeated.content, first.content);
        assert!(!repeated.changed);
    }
    assert!(format_metadata_input("# Missing frontmatter", "research-activity-data", "markdown", None).is_err());
    assert!(format_metadata_input("{", "research-activity-data", "zon", None).is_err());
}
#[test]
fn test_format_metadata_normalizes_rdf_dublin_core() {
    let document = DublinCoreDocument::decode(DUBLIN_CORE_FIXTURE, &MimeType::Json).unwrap();
    for (format, mime) in [("rdf-xml", MimeType::RdfXml), ("turtle", MimeType::Turtle)] {
        let input = document.encode(&mime).unwrap();
        let first = format_metadata_input(&input, "dublin-core", format, None).unwrap();
        assert_eq!(first.standard, "dublin-core");
        assert_eq!(first.format, format);
        assert_eq!(DublinCoreDocument::decode(&first.content, &mime).unwrap(), document);
        let repeated = format_metadata_input(&first.content, "dublin-core", format, None).unwrap();
        assert_eq!(repeated.content, first.content);
        assert!(!repeated.changed);
    }
    assert!(format_metadata_input("<rdf:RDF>", "dublin-core", "rdf-xml", None).is_err());
    assert!(format_metadata_input("@prefix", "dublin-core", "turtle", None).is_err());
    assert!(format_metadata_input("{}", "research-activity-data", "turtle", None).is_err());
}
#[test]
fn test_format_metadata_normalizes_structured_dublin_core() {
    let document = DublinCoreDocument::decode(DUBLIN_CORE_FIXTURE, &MimeType::Json).unwrap();
    for (format, mime) in [("json", MimeType::Json), ("json-ld", MimeType::LdJson), ("yaml", MimeType::Yaml)] {
        let input = if format == "json" {
            DUBLIN_CORE_FIXTURE.to_string()
        } else {
            document.encode(&mime).unwrap()
        };
        let first = format_metadata_input(&input, "dublin-core", format, None).unwrap();
        assert_eq!(first.standard, "dublin-core");
        assert_eq!(first.format, format);
        assert_eq!(DublinCoreDocument::decode(&first.content, &mime).unwrap(), document);
        let repeated = format_metadata_input(&first.content, "dublin-core", format, None).unwrap();
        assert_eq!(repeated.content, first.content);
        assert!(!repeated.changed);
    }
    assert!(format_metadata_input("{", "dublin-core", "json", None).is_err());
    assert!(format_metadata_input("{", "dublin-core", "json-ld", None).is_err());
    assert!(format_metadata_input(":", "dublin-core", "yaml", None).is_err());
    assert!(format_metadata_input("{}", "dublin-core", "jsonc", None).is_err());
}
#[test]
fn test_format_metadata_rejects_invalid_and_oversized_input() {
    assert!(format_metadata_input("{", "research-activity-data", "json", None).is_err());
    assert_eq!(
        format_metadata_input(&" ".repeat(FORMAT_CONTENT_LIMIT + 1), "research-activity-data", "json", None).unwrap_err(),
        "Format input exceeds 256 KiB"
    );
    assert!(format_metadata_input("{}", "research-activity-data", "cff", None).is_err());
    assert!(format_metadata_input("{}", "datacite", "json", None).is_err());
}
#[test]
fn test_format_metadata_supports_explicit_vocabulary_policies() {
    for (name, expected) in [
        ("off", VocabularyFuzzyMode::Off),
        ("relaxed", VocabularyFuzzyMode::Relaxed),
        ("strict", VocabularyFuzzyMode::Strict),
    ] {
        let result = format_metadata_input(VALID_RESEARCH_ACTIVITY, "research-activity-data", "json", Some(name)).unwrap();
        assert_eq!(result.vocabulary_fuzzy, expected);
    }
    assert!(format_metadata_input(VALID_RESEARCH_ACTIVITY, "research-activity-data", "json", Some("unknown")).is_err());
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
fn test_parse_activity_metric_ignores_invalid_value_only_when_readability_is_skipped() {
    let skip = [CheckCategory::Readability];
    assert_eq!(parse_activity_metric("not-a-metric", &skip), Ok(ReadabilityType::default()));
    assert!(parse_activity_metric("not-a-metric", &[]).is_err());
    let result = validate_json_with_readability(
        VALID_RESEARCH_ACTIVITY,
        VocabularyFuzzyMode::Off,
        &skip,
        parse_activity_metric("not-a-metric", &skip).unwrap(),
        None,
    );
    assert!(result.diagnostics.iter().all(|item| item.category != CheckCategory::Readability));
}
#[test]
fn test_parse_czml_standard() {
    assert_eq!(parse_standard("czml"), Ok(Standard::Czml));
}
#[test]
fn test_rad_markdown_and_zon_codecs_preserve_fixture_meaning() {
    let source = ResearchActivity::decode(VALID_RESEARCH_ACTIVITY, &MimeType::Json).unwrap();
    let expected = serde_json::to_value(&source).unwrap();
    for mime in [MimeType::Markdown, MimeType::Zon] {
        let encoded = source.encode(&mime).unwrap();
        let decoded = ResearchActivity::decode(&encoded, &mime).unwrap();
        assert_eq!(serde_json::to_value(&decoded).unwrap(), expected, "{mime:?} changed RAD meaning");
        let repeated = decoded.encode(&mime).unwrap();
        assert_eq!(repeated, encoded, "{mime:?} is not idempotent");
    }
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
    assert!(off
        .diagnostics
        .iter()
        .all(|item| matches!(item.category, CheckCategory::Fair | CheckCategory::Readability)));
    assert!(off.diagnostics.iter().any(|item| item.category == CheckCategory::Readability));
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
