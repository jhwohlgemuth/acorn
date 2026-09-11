#[test]
fn with_invalid_attributes_fail_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/with-fail-*.rs");
}
