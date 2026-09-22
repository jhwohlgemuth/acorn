//! Cross-feature contract tests for portable schema behavior
use acorn_schema::{
    agent::{AuthenticationScheme, ProviderDetails},
    validation::{format_timestamp, rules},
    ContactPoint,
};

#[test]
fn contact_normalization_is_portable() {
    let contact = ContactPoint::init()
        .organization("ORNL".to_string())
        .telephone("(555) 123-4567".to_string())
        .build()
        .format();
    assert_eq!(contact.organization, "Oak Ridge National Laboratory");
    assert_eq!(contact.telephone, "555.123.4567");
}
#[test]
fn provider_authentication_keeps_its_typed_wire_shape() {
    let details = ProviderDetails::init()
        .authentication(vec![AuthenticationScheme::Basic, AuthenticationScheme::ApiKey])
        .build();
    let value = serde_json::to_value(details).expect("provider details should serialize");
    let expected = serde_json::json!(["Basic", "ApiKey"]);
    assert_eq!(value.get("authentication"), Some(&expected));
    let decoded: ProviderDetails = serde_json::from_value(value).expect("provider details should deserialize");
    assert!(matches!(
        decoded.authentication.as_deref(),
        Some([AuthenticationScheme::Basic, AuthenticationScheme::ApiKey])
    ));
}
#[test]
fn timestamp_and_year_rules_are_portable() {
    assert_eq!(
        format_timestamp("2026-03-10T15:37:24.355614500+00:00"),
        "2026-03-10T15:37:24.355614+00:00"
    );
    #[cfg(feature = "std")]
    assert!(rules::year("3000").is_err());
    #[cfg(not(feature = "std"))]
    assert!(rules::year("3000").is_ok());
    assert!(rules::year("999").is_err());
}
