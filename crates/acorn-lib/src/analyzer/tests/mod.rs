use super::error::ErrorKind;
use super::model::ValeOutputItem;
use super::{Check, CheckBatch, CheckCategory, CheckSummary};
use crate::{check_err, check_ok};
use acorn_core::validation::ValidationError;
use acorn_schema::validation::ValidationReport;

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
