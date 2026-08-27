//! Downstream validation derive coverage.
use acorn::validation::Validate;

#[derive(Validate)]
struct ExternalModel {
    #[validate(email)]
    email: String,
}
#[test]
fn derive_resolves_acorn_from_a_downstream_crate() {
    assert!(ExternalModel {
        email: "researcher@example.org".to_string(),
    }
    .validate()
    .is_ok());
}
