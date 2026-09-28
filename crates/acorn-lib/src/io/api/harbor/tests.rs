use super::*;

#[test]
fn with_reference_accepts_the_borrowed_inner_value() {
    let profile = RegistryProfile::default();
    let options = Options::from_repository(&profile, "ai/model").unwrap().with_reference("sha256:abc");
    assert_eq!(options.reference, Some("sha256:abc"));
}
