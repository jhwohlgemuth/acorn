#[test]
fn invalid_validation_attributes_fail_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/validate-*.rs");
}
