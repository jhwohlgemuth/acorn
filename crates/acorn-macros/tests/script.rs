use acorn_macros::js;

#[test]
fn test_division_accepts_json_bound_divisor() {
    let divisor = 2;
    assert_eq!(js!("return 10 / {divisor}").unwrap(), "return 10 / 2");
}
#[test]
fn test_json_binding_cannot_change_javascript_syntax() {
    let value = "'); window.bad(); //";
    let source = js!("window.good({value})").unwrap();
    assert_eq!(source, "window.good(\"'); window.bad(); //\")");
}
#[test]
fn test_multiple_json_bindings_preserve_order() {
    let left = 2;
    let right = 3;
    assert_eq!(js!("window.sum({left}, {right})").unwrap(), "window.sum(2, 3)");
}
#[test]
fn test_rejected_script_contexts() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/js-*.rs");
}
