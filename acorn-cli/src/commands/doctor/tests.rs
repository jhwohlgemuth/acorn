use crate::commands::doctor::should_run;
use acorn_lib::util::cli::Diagnostic;

#[test]
fn test_should_run() {
    assert!(should_run(&[Diagnostic::Memory], Diagnostic::Memory));
    assert!(!should_run(&[Diagnostic::Memory], Diagnostic::System));
}
