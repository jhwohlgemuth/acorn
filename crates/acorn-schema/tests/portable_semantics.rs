//! Cross-feature contract tests for portable schema behavior
use acorn_core::util::MimeType;
use acorn_schema::{
    agent::{AuthenticationScheme, ProviderDetails},
    validation::{format_timestamp, rules},
    vocabulary::{VocabularyKind, VocabularyOptions, VocabularyOutcome, VocabularyResolver},
    ContactPoint, OneOrMany, OneOrManyCodec,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct CrosswalkRecord {
    id: u8,
    title: String,
}

#[test]
fn test_contact_normalization_is_portable() {
    let contact = ContactPoint::init()
        .organization("ORNL".to_string())
        .telephone("(555) 123-4567".to_string())
        .build()
        .format();
    assert_eq!(contact.organization, "Oak Ridge National Laboratory");
    assert_eq!(contact.telephone, "555.123.4567");
}
#[test]
fn test_one_or_many_crosswalk_data_round_trips_zon() {
    let values: OneOrMany<CrosswalkRecord> = vec![
        CrosswalkRecord {
            id: 1,
            title: "First".to_string(),
        },
        CrosswalkRecord {
            id: 2,
            title: "Second".to_string(),
        },
    ]
    .into();
    let encoded = values.encode(MimeType::Zon).expect("crosswalk data should serialize as ZON");
    let decoded = OneOrMany::<CrosswalkRecord>::parse(&encoded, MimeType::Zon).expect("crosswalk ZON should parse");
    assert_eq!(decoded, values);
}
#[test]
fn test_provider_authentication_keeps_its_typed_wire_shape() {
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
fn test_timestamp_and_year_rules_are_portable() {
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
#[test]
fn test_vocabulary_resolution_is_portable() {
    let report = VocabularyResolver::embedded(VocabularyKind::Technology)
        .and_then(|resolver| resolver.resolve(["rs", "typescirpt", "not-a-technology"], VocabularyOptions::default()))
        .expect("technology vocabulary should resolve");
    let rust = report.resolutions.first().expect("rust resolution");
    let typescript = report.resolutions.get(1).expect("typescript resolution");
    let unknown = report.resolutions.get(2).expect("unknown resolution");
    assert_eq!(rust.canonical.as_deref(), Some("rust"));
    assert_eq!(typescript.canonical.as_deref(), Some("typescript"));
    assert_eq!(unknown.outcome, VocabularyOutcome::Unmatched);
    assert_eq!(report.lossless_values(), vec!["rust", "typescript", "not-a-technology"]);
}
