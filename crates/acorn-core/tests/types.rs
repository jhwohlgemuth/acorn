//! Integration tests for portable value types.
#![allow(clippy::expect_used, missing_docs)]
extern crate alloc;
use acorn_core::{Noul, NoulError, OneOrMany, ThisOrThat};
use alloc::collections::BTreeSet;
use schemars::schema_for;
use serde_json::json;

#[test]
fn test_noul_common_conversions_preserve_values() {
    let value = Noul::try_from(0.25_f32).expect("valid unit value");
    assert_eq!(f32::from(value), 0.25);
    assert_eq!(f64::from(value), 0.25);
    assert_eq!(Noul::from(false), Noul::ZERO);
    assert_eq!(Noul::from(true), Noul::ONE);
    assert_eq!("0.5".parse::<Noul>(), Noul::new(0.5));
}
#[test]
fn test_noul_orders_and_hashes_finite_unit_values() {
    let values = [Noul::ONE, Noul::ZERO, Noul::new(0.5).expect("valid unit value")]
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        values.into_iter().collect::<Vec<_>>(),
        [Noul::ZERO, Noul::new(0.5).expect("valid unit value"), Noul::ONE]
    );
    assert!(Noul::ONE > 0.5_f32);
    assert!(0.5_f32 < Noul::ONE);
    assert_eq!(Noul::new(-0.0), Ok(Noul::ZERO));
}
#[test]
fn test_noul_rejects_invalid_values() {
    [-0.1, 1.1, f64::INFINITY, f64::NEG_INFINITY, f64::NAN]
        .into_iter()
        .for_each(|value| assert_eq!(Noul::new(value), Err(NoulError)));
    assert!(serde_json::from_value::<Noul>(json!(1.1)).is_err());
    assert!(serde_json::from_value::<Noul>(json!(null)).is_err());
}
#[test]
fn test_noul_schema_and_serialization_retain_numeric_bounds() {
    let schema = serde_json::to_value(schema_for!(Noul)).expect("schema should serialize");
    assert_eq!(schema.pointer("/maximum"), Some(&json!(1.0)), "{schema}");
    assert_eq!(schema.pointer("/minimum"), Some(&json!(0.0)), "{schema}");
    assert_eq!(
        serde_json::to_value(Noul::new(0.5).expect("valid unit value")).expect("value should serialize"),
        json!(0.5)
    );
}
#[test]
fn test_one_or_many_preserves_shape_and_iteration_order() {
    let one = OneOrMany::One(1);
    let many = OneOrMany::Many(vec![2, 3]);

    assert_eq!(one.as_slice(), &[1]);
    assert_eq!(many.iter().copied().collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(many.map(|value| value * 2).into_vec(), vec![4, 6]);
}
#[test]
fn test_this_or_that_maps_both_values_in_position_order() {
    let mut positions = Vec::new();
    let values = ThisOrThat::new(2, 3).map(|value| {
        positions.push(value);
        value * 2
    });

    assert_eq!(positions, vec![2, 3]);
    assert_eq!(values.into_parts(), (4, 6));
}
#[test]
fn test_this_or_that_provides_named_access_to_each_value() {
    let values = ThisOrThat::new("left", "right");

    assert_eq!(values.left(), &"left");
    assert_eq!(values.right(), &"right");
}
#[test]
fn test_this_or_that_try_map_with_transforms_each_position_independently() {
    let values = ThisOrThat::new(2, 3).try_map_with(|left| Ok::<_, ()>(left * 2), |right| Ok(right * 3));

    assert_eq!(values, Ok(ThisOrThat::new(4, 9)));
}
#[test]
fn test_this_or_that_zips_values_in_position_order() {
    let values = ThisOrThat::new([1, 2], [3, 4]).zip().collect::<Vec<_>>();

    assert_eq!(values, vec![(1, 3), (2, 4)]);
}
