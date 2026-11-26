use super::{square_root, Vector};
#[cfg(feature = "alloc")]
use super::{Degree, Monomial, Rational};
use crate::ThisOrThat;
#[cfg(feature = "alloc")]
use core::cmp::Ordering;
#[cfg(feature = "alloc")]
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};

#[cfg(feature = "alloc")]
fn rational() -> impl Strategy<Value = Rational> {
    (any::<i32>(), 1_i32..=1_000_000)
        .prop_map(|(numerator, denominator)| Rational::new(i128::from(numerator), i128::from(denominator)).expect("positive generated denominator"))
}

#[cfg(feature = "alloc")]
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/util/math.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_rational_addition_is_associative_when_defined(a in rational(), b in rational(), c in rational()) {
        if let (Some(left), Some(right)) = (a.add(b).and_then(|sum| sum.add(c)), b.add(c).and_then(|sum| a.add(sum))) {
            prop_assert_eq!(left, right);
        }
    }
    #[test]
    fn test_rational_addition_is_commutative(left in rational(), right in rational()) {
        prop_assert_eq!(left.add(right), right.add(left));
    }
    #[test]
    fn test_rational_construction_is_reduced(numerator in any::<i128>(), denominator in 1_i128..=i128::MAX) {
        let value = Rational::new(numerator, denominator).expect("positive generated denominator");
        prop_assert!(value.denominator > 0);
        prop_assert_eq!(super::greatest_common_divisor(value.numerator, value.denominator), 1);
        if numerator == 0 {
            prop_assert_eq!(value, Rational::ZERO);
        }
    }
    #[test]
    fn test_rational_multiplication_is_associative_when_defined(a in rational(), b in rational(), c in rational()) {
        if let (Some(left), Some(right)) = (a.multiply(b).and_then(|product| product.multiply(c)), b.multiply(c).and_then(|product| a.multiply(product))) {
            prop_assert_eq!(left, right);
        }
    }
    #[test]
    fn test_rational_multiplication_is_commutative(left in rational(), right in rational()) {
        prop_assert_eq!(left.multiply(right), right.multiply(left));
    }
    #[test]
    fn test_rational_order_matches_widened_cross_product(a in any::<i64>(), b in 1_i64..=i64::MAX, c in any::<i64>(), d in 1_i64..=i64::MAX) {
        let left = Rational::new(i128::from(a), i128::from(b)).expect("positive generated denominator");
        let right = Rational::new(i128::from(c), i128::from(d)).expect("positive generated denominator");
        let left_product = i128::from(a).checked_mul(i128::from(d)).expect("i64 products fit in i128");
        let right_product = i128::from(c).checked_mul(i128::from(b)).expect("i64 products fit in i128");
        let oracle = left_product.cmp(&right_product);
        prop_assert_eq!(left.compare(right), oracle);
    }
    #[test]
    fn test_rational_zero_and_one_are_identities(value in rational()) {
        prop_assert_eq!(value.add(Rational::ZERO), Some(value));
        prop_assert_eq!(value.multiply(Rational::ONE), Some(value));
        prop_assert_eq!(value.multiply(Rational::ZERO), Some(Rational::ZERO));
    }
}
#[cfg(feature = "alloc")]
#[test]
fn test_monomial_operations_compose_degrees() {
    let x = Monomial::factor(
        "x",
        Degree {
            power: Rational::ONE,
            ..Degree::default()
        },
    );
    let x_squared = x.clone().multiply(x.clone());
    let square_root = x_squared
        .clone()
        .and_then(|value| Rational::new(1, 2).and_then(|exponent| value.power(exponent)));

    assert_eq!(x_squared.as_ref().and_then(|value| x.direction(value)), Some((true, false)));
    assert_eq!(square_root.as_ref().and_then(|value| value.direction(&x)), Some((false, false)));
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_arithmetic_cancels_factors_before_multiplication() {
    let large = i128::MAX;
    let small = Rational::new(1, large).expect("positive denominator");
    let reciprocal = Rational::new(large, 1).expect("positive denominator");
    assert_eq!(small.multiply(reciprocal), Some(Rational::ONE));
    assert_eq!(small.add(small), Rational::new(2, large));
    let denominator = large - 1;
    let left = Rational::new(1, denominator).expect("positive denominator");
    let right = Rational::new(denominator - 1, denominator).expect("positive denominator");
    assert_eq!(left.add(right), Some(Rational::ONE));
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_comparison_handles_signed_boundaries_without_overflow() {
    let minimum = Rational::new(i128::MIN, 1).expect("minimum numerator");
    let maximum = Rational::new(i128::MAX, 1).expect("maximum numerator");
    let near_one = Rational::new(i128::MAX - 1, i128::MAX).expect("near one");
    let smaller = Rational::new(i128::MAX - 2, i128::MAX - 1).expect("smaller than near one");
    assert_eq!(minimum.compare(maximum), Ordering::Less);
    assert_eq!(minimum.compare(Rational::NEG_ONE), Ordering::Less);
    assert_eq!(near_one.compare(smaller), Ordering::Greater);
    assert_eq!(near_one.compare(near_one), Ordering::Equal);
    for a in -8..=8 {
        for b in 1..=8 {
            for c in -8..=8 {
                for d in 1..=8 {
                    let left = Rational::new(a, b).expect("positive denominator");
                    let right = Rational::new(c, d).expect("positive denominator");
                    assert_eq!(left.compare(right), (a * d).cmp(&(c * b)), "{a}/{b} vs {c}/{d}");
                }
            }
        }
    }
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_construction_reduces_signed_values_and_zero() {
    assert_eq!(Rational::new(i128::MAX, i128::MAX), Some(Rational::ONE));
    assert_eq!(Rational::new(-i128::MAX, i128::MAX), Some(Rational::NEG_ONE));
    assert_eq!(Rational::new(0, i128::MAX), Some(Rational::ZERO));
    assert_eq!(Rational::new(i128::MIN, 2).map(|value| value.numerator), Some(i128::MIN / 2));
    assert_eq!(Rational::new(1, -1), None);
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_operations_use_checked_arithmetic() {
    let half = Rational::new(1, 2);

    assert_eq!(
        half.and_then(|value| value.add(value)).map(|value| value.compare(Rational::ONE)),
        Some(Ordering::Equal)
    );
    assert_eq!(
        half.and_then(|value| value.multiply(value))
            .and_then(|value| Rational::new(1, 4).map(|quarter| value.compare(quarter))),
        Some(Ordering::Equal)
    );
    assert_eq!(half.map(|value| value.compare(Rational::ONE)), Some(Ordering::Less));
    assert_eq!(Rational::new(1, 0), None);
    assert_eq!(Rational::new(i128::MAX, 1).and_then(|value| value.add(Rational::ONE)), None);
}
#[test]
fn test_square_root_handles_zero_and_positive_values() {
    assert_eq!(square_root(0.0), 0.0);
    assert_eq!(square_root(9.0), 3.0);
}
#[test]
fn test_vector_converts_to_and_from_this_or_that() {
    let vector: Vector<[i32; 2]> = ThisOrThat::new(2, 3).into();
    let values: ThisOrThat<i32> = vector.into();

    assert_eq!(values, ThisOrThat::new(2, 3));
}
#[test]
fn test_vector_dot_and_norm_use_slice_backing() {
    let left = Vector::from([3.0, 4.0]);
    let right = Vector::from([2.0, 1.0]);

    assert_eq!(left.dot(&right), Some(10.0));
    assert_eq!(left.norm(), 5.0);
    assert_eq!(left.dot(&Vector::from([1.0])), None);
}
