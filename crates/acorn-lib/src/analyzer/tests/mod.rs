use super::error::ErrorKind;
use super::fair::{indicator, FairCounts, FairEvidence, FairEvidenceItem, FairIndicatorId, FairPrinciple, FairPriority, FairState};
use super::model::ValeOutputItem;
use super::{Check, CheckBatch, CheckCategory, CheckSummary};
use crate::{check_err, check_ok};
use acorn_core::validation::ValidationError;
use acorn_schema::validation::ValidationReport;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};

fn batch(messages: &[&str]) -> CheckBatch {
    messages
        .iter()
        .map(|message| check_ok!(CheckCategory::Quality, message: (*message).to_string()))
        .collect()
}
fn messages(checks: &CheckBatch) -> Vec<String> {
    checks.iter().map(|check| check.message.clone()).collect()
}

#[test]
fn check_batch_collection_flattens_in_insertion_order() {
    let checks = [batch(&["first", "second"]), batch(&["third"])].into_iter().collect::<CheckBatch>();
    assert_eq!(messages(&checks), vec!["first", "second", "third"]);
}
#[test]
fn check_batch_combine_is_associative_and_ordered() {
    let first = batch(&["first"]);
    let second = batch(&["second"]);
    let third = batch(&["third"]);
    let left = first.clone().combine(second.clone()).combine(third.clone());
    let right = first.combine(second.combine(third));
    assert_eq!(messages(&left), vec!["first", "second", "third"]);
    assert_eq!(messages(&left), messages(&right));
}
#[test]
fn check_batch_empty_is_identity_on_both_sides() {
    let checks = batch(&["first", "second"]);
    assert_eq!(messages(&checks.clone().combine(CheckBatch::empty())), messages(&checks));
    assert_eq!(messages(&CheckBatch::empty().combine(checks.clone())), messages(&checks));
}
#[test]
fn check_batch_iterates_and_round_trips_explicitly() {
    let checks = batch(&["first", "second"]);
    let borrowed = (&checks).into_iter().map(|check| check.message.as_str()).collect::<Vec<_>>();
    assert_eq!(borrowed, vec!["first", "second"]);
    let owned = checks.clone().into_iter().map(|check| check.message).collect::<Vec<_>>();
    assert_eq!(owned, vec!["first", "second"]);
    let vector: Vec<Check> = checks.clone().into();
    let round_trip = CheckBatch::from(vector);
    assert_eq!(messages(&round_trip), messages(&checks));
    assert_eq!(round_trip.len(), 2);
    assert!(!round_trip.is_empty());
    assert!(CheckBatch::empty().is_empty());
    assert_eq!(CheckBatch::single(check_ok!(CheckCategory::Quality)).len(), 1);
}
#[test]
fn check_summary_counts_every_category_and_multi_issue_checks() {
    let mut validation = ValidationReport::new();
    validation.add("title", ValidationError::new("length"));
    validation.add("contact.email", ValidationError::new("email"));
    let vale_item = serde_json::from_value::<ValeOutputItem>(serde_json::json!({
        "Action": { "Name": "replace", "Params": null },
        "Check": "Vale.Spelling",
        "Description": "",
        "Line": 1,
        "Link": "",
        "Message": "Prose issue",
        "Severity": "error",
        "Span": [1, 3],
        "Match": "bad"
    }))
    .expect("deserialize Vale fixture");
    let checks = CheckBatch::from(vec![
        check_err!(CheckCategory::Schema, errors: ErrorKind::Validator(validation)),
        check_err!(CheckCategory::Link),
        check_err!(CheckCategory::Prose, errors: ErrorKind::Vale(vec![vale_item.clone(), vale_item])),
        check_err!(CheckCategory::Quality),
        check_err!(CheckCategory::Readability),
        check_err!(CheckCategory::Crosswalk),
    ]);
    assert_eq!(
        CheckSummary::from(checks),
        CheckSummary {
            fair: 0,
            schema: 2,
            link: 1,
            prose: 2,
            quality: 1,
            readability: 1,
            crosswalk: 1,
        }
    );
}
#[test]
fn check_summary_zero_is_identity_on_both_sides() {
    let summary = CheckSummary {
        fair: 7,
        schema: 1,
        link: 2,
        prose: 3,
        quality: 4,
        readability: 5,
        crosswalk: 6,
    };
    assert_eq!(summary.combine(CheckSummary::zero()), summary);
    assert_eq!(CheckSummary::zero().combine(summary), summary);
}
#[test]
fn check_summary_combine_is_associative() {
    let first = CheckSummary {
        schema: 1,
        ..CheckSummary::zero()
    };
    let second = CheckSummary {
        prose: 2,
        ..CheckSummary::zero()
    };
    let third = CheckSummary {
        crosswalk: 3,
        ..CheckSummary::zero()
    };
    assert_eq!(first.combine(second).combine(third), first.combine(second.combine(third)));
}
#[test]
fn combined_batch_summaries_equal_flattened_summary() {
    let first = CheckBatch::from(vec![check_err!(CheckCategory::Schema), check_err!(CheckCategory::Link)]);
    let second = CheckBatch::from(vec![
        check_err!(CheckCategory::Prose, message: "issue"),
        check_err!(CheckCategory::Crosswalk),
    ]);
    let combined_summary = CheckSummary::from(&first).combine(CheckSummary::from(&second));
    assert_eq!(combined_summary, CheckSummary::from(first.combine(second)));
}
#[test]
fn fair_counts_report_support_and_coverage() {
    let counts = FairCounts {
        indeterminate: 2,
        not_applicable: 1,
        supported: 3,
        unsupported: 1,
    };
    assert_eq!(counts.support(), Some(0.75));
    assert_eq!(counts.coverage(), Some(4.0 / 6.0));
    assert_eq!(FairCounts::default().support(), None);
    assert_eq!(FairCounts::default().coverage(), None);
}
#[test]
fn fair_evidence_normalizes_and_assesses_serialized_metadata() {
    let value = serde_json::json!({
        "id": "10.1234/example",
        "attributes": {
            "titles": [{"title": "Example"}],
            "descriptions": [{"description": "Description"}],
            "creators": [{"name": "Researcher"}],
            "publisher": "Repository",
            "subjects": [{"subject": "science", "schemeURI": "https://example.org/vocab"}],
            "url": "https://example.org/data",
            "rightsList": [{"rightsIdentifier": "CC-BY-4.0"}],
            "formats": ["text/csv"]
        }
    });
    let findings = FairEvidence::from_value("10.1234/example", "DataCite 4.7", &value).assess();
    assert_eq!(findings.len(), 41);
    assert!(findings
        .iter()
        .any(|finding| finding.indicator.id == FairIndicatorId::RdaF2_01M && finding.state.is_supported()));
    assert!(findings
        .iter()
        .any(|finding| finding.indicator.id == FairIndicatorId::RdaA2_01M && finding.state.is_indeterminate()));
    assert!(findings
        .iter()
        .any(|finding| finding.indicator.id == FairIndicatorId::RdaA1_03D && finding.state.is_indeterminate()));
}
#[test]
fn fair_identifier_checks_reject_non_pids() {
    ["https://example.org/data", "uuid:123e4567-e89b-12d3-a456-426614174000"]
        .into_iter()
        .for_each(|value| {
            let findings = FairEvidence {
                data_identifier: Some(FairEvidenceItem {
                    locator: "$.identifier".to_string(),
                    value: value.to_string(),
                }),
                resource: value.to_string(),
                ..FairEvidence::default()
            }
            .assess();
            [FairIndicatorId::RdaF1_01D, FairIndicatorId::RdaF1_02D].into_iter().for_each(|id| {
                assert!(findings
                    .iter()
                    .find(|finding| finding.indicator.id == id)
                    .is_some_and(|finding| finding.state.is_indeterminate()));
            });
        });
}
#[test]
fn fair_indicator_uri_selection_uses_evidence_kind() {
    let evidence = FairEvidence {
        direct_uri: Some(FairEvidenceItem {
            locator: "$.downloadURL".to_string(),
            value: "https://example.org/data".to_string(),
        }),
        metadata_identifier: Some(FairEvidenceItem {
            locator: "$.metadataIdentifier".to_string(),
            value: "https://example.org/metadata".to_string(),
        }),
        ..FairEvidence::default()
    };
    assert_eq!(
        FairIndicatorId::RdaA1_02M.uri_from(&evidence).as_deref(),
        Some("https://example.org/metadata")
    );
    assert_eq!(
        FairIndicatorId::RdaA1_05D.uri_from(&evidence).as_deref(),
        Some("https://example.org/data")
    );
    assert_eq!(FairIndicatorId::RdaF1_01D.uri_from(&evidence), None);
}
#[test]
fn fair_protocol_checks_use_scheme_parsing() {
    [
        ("https://example.org/data", FairState::Supported),
        ("httpsish://example.org/data", FairState::Indeterminate),
    ]
    .into_iter()
    .for_each(|(value, expected)| {
        let finding = FairEvidence {
            access_uri: Some(FairEvidenceItem {
                locator: "$.url".to_string(),
                value: value.to_string(),
            }),
            ..FairEvidence::default()
        }
        .assess()
        .into_iter()
        .find(|finding| finding.indicator.id == FairIndicatorId::RdaA1_04D)
        .expect("registry contains RDA-A1-04D");
        assert_eq!(finding.state, expected);
    });
}
#[test]
fn fair_registry_has_expected_principles_and_priorities() {
    let indicators = indicator();
    assert_eq!(indicators.len(), 41);
    assert_eq!(indicators.iter().filter(|value| value.principle == FairPrinciple::Findable).count(), 7);
    assert_eq!(indicators.iter().filter(|value| value.principle == FairPrinciple::Accessible).count(), 12);
    assert_eq!(
        indicators.iter().filter(|value| value.principle == FairPrinciple::Interoperable).count(),
        12
    );
    assert_eq!(indicators.iter().filter(|value| value.principle == FairPrinciple::Reusable).count(), 10);
    assert_eq!(indicators.iter().filter(|value| value.priority == FairPriority::Essential).count(), 20);
    assert_eq!(indicators.iter().filter(|value| value.priority == FairPriority::Important).count(), 14);
    assert_eq!(indicators.iter().filter(|value| value.priority == FairPriority::Useful).count(), 7);
    assert_eq!(indicators.first().map(|value| value.indicator.as_str()), Some("F1"));
}
#[test]
fn fair_terse_output_uses_shared_columns_and_metadata() {
    let value = serde_json::json!({"identifier": "10.1234/example"});
    let output = FairEvidence::from_value("resource", "DataCite 4.7", &value)
        .assess()
        .into_iter()
        .find(|finding| finding.indicator.id == FairIndicatorId::RdaF1_01D)
        .map(Check::from)
        .expect("registry contains RDA-F1-01D")
        .to_string();
    assert!(output.contains("rule=RDA-F1-01D"));
    assert!(output.contains("indicator=F1"));
    assert!(output.contains("state=supported"));
    assert!(output.contains("priority=essential"));
}
