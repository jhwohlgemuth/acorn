//! Downstream validation derive coverage.
use acorn::With;
use acorn_schema::validation::Validate;

#[derive(Validate)]
struct ExternalModel {
    #[validate(email)]
    email: String,
}
#[derive(With)]
struct ExternalOptions {
    enabled: bool,
    #[with(public, some)]
    label: Option<String>,
}
#[test]
fn derive_resolves_acorn_schema_from_a_downstream_crate() {
    assert!(ExternalModel {
        email: "researcher@example.org".to_string(),
    }
    .validate()
    .is_ok());
}

#[test]
fn with_derive_is_available_from_the_host_facade() {
    let options = ExternalOptions { enabled: false, label: None }
        .with_enabled(true)
        .with_label("configured");
    assert!(options.enabled);
    assert_eq!(options.label.as_deref(), Some("configured"));
}
