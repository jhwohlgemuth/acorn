use super::{categories, run, should_run};
use crate::cli::arguments::Diagnostic;
#[test]
fn category_filtering_is_deterministic() {
    assert_eq!(categories(&[Diagnostic::Memory, Diagnostic::System]), ["memory", "system"]);
    assert_eq!(categories(&[Diagnostic::All]), ["gpu", "memory", "network", "software", "system"]);
}
#[test]
fn report_and_interactive_are_rejected() {
    assert!(run(&false, &true, &true, &[Diagnostic::All], false).is_err());
}
#[test]
fn selected_diagnostics_run() {
    assert!(run(&false, &false, &false, &[Diagnostic::System], false).is_ok());
}
#[test]
fn should_run_honors_all_and_explicit_categories() {
    assert!(should_run(&[Diagnostic::All], Diagnostic::Gpu));
    assert!(should_run(&[Diagnostic::Software], Diagnostic::Software));
    assert!(!should_run(&[Diagnostic::Memory], Diagnostic::Network));
}
