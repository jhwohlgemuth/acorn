use super::{Check, CheckBatch, CheckCategory};
use crate::check_ok;

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
