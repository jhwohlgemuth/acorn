use acorn_macros::bigo;

#[test]
fn test_bigo_compiles_in_no_std_consumer() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/big-o-no-std.rs");
}
#[test]
fn test_bigo_preserves_literal_and_static_type() {
    const TIME: &str = bigo!("O(n log n)");
    const SPACE: &str = bigo!("O(n)");
    let _: &'static str = TIME;
    assert_eq!(TIME, "O(n log n)");
    assert_eq!(SPACE, "O(n)");
}
#[test]
fn test_bigo_rejects_invalid_inputs() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/big-o-invalid-syntax.rs");
    cases.compile_fail("tests/ui/big-o-nonliteral.rs");
}
