use super::*;
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
    assert_eq!(schema.pointer("/minimum"), Some(&json!(0.0)), "{schema}");
    assert_eq!(schema.pointer("/maximum"), Some(&json!(1.0)), "{schema}");
    assert_eq!(
        serde_json::to_value(Noul::new(0.5).expect("valid unit value")).expect("value should serialize"),
        json!(0.5)
    );
}
