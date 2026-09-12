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
    assert!(ExternalOptions { enabled: false }.with_enabled(true).enabled);
}
