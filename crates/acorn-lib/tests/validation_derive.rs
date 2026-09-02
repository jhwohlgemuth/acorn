//! Downstream validation derive coverage.
use acorn_schema::validation::Validate;

#[derive(Validate)]
struct ExternalModel {
    #[validate(email)]
    email: String,
}
#[test]
fn derive_resolves_acorn_schema_from_a_downstream_crate() {
    assert!(ExternalModel {
        email: "researcher@example.org".to_string(),
    }
    .validate()
    .is_ok());
}
