#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
define_required_fn!(1, required1_test, A, a);
define_required_fn!(3, required3_test, A, a, B, b, C, c);
define_required_fn!(4, required4_test, A, a, B, b, C, c, D, d);

#[test]
fn required1_returns_error() {
    let result = required1_test::<i32>(None);
    let message = result.err().map(|why| why.to_string());
    assert_eq!(message.as_deref(), Some("Failed to insert: a is required"));
}
#[test]
fn required1_returns_value_tuple() {
    let result = required1_test::<i32>(Some(7));
    assert_eq!(result.ok(), Some((7,)));
}
#[test]
fn required3_returns_first_error() {
    let result = required3_test::<i32, i32, i32>(None, None, None);
    let message = result.err().map(|why| why.to_string());
    assert_eq!(message.as_deref(), Some("Failed to insert: a is required"));
}
#[test]
fn required3_returns_value_tuple() {
    let result = required3_test::<i32, i32, i32>(Some(1), Some(2), Some(3));
    assert_eq!(result.ok(), Some((1, 2, 3)));
}
#[test]
fn required4_returns_first_error() {
    let result = required4_test::<i32, i32, i32, i32>(Some(1), None, None, None);
    let message = result.err().map(|why| why.to_string());
    assert_eq!(message.as_deref(), Some("Failed to insert: b is required"));
}
#[test]
fn required4_returns_value_tuple() {
    let result = required4_test::<i32, i32, i32, i32>(Some(1), Some(2), Some(3), Some(4));
    assert_eq!(result.ok(), Some((1, 2, 3, 4)));
}
