use crate::standard::ocp::{Contribution, ContributionId, ManagementProfileClaim, Project, Recognition, RecognitionProgram};
use crate::standard::{ClaimAssessment, ConformanceResult, ContributionKind, StandardReference, StandardsClaim, StandardsRelationship};
use crate::validation::Validate;

fn reference(identifier: &str) -> StandardReference {
    StandardReference::init()
        .authority("Open Compute Project")
        .identifier(identifier)
        .canonical_uri(format!("https://www.opencompute.org/contributions/{identifier}"))
        .build()
}
#[test]
fn test_management_profile_claim_requires_test_evidence() {
    [ConformanceResult::Fail, ConformanceResult::Partial, ConformanceResult::Pass]
        .into_iter()
        .for_each(|result| {
            let claim = ManagementProfileClaim::init()
                .profile(reference("hardware-management-profile"))
                .underlying_standard(reference("redfish"))
                .result(result)
                .build();
            let report = claim.validate().expect_err("tested result should require a timestamp and evidence");
            let paths = report.iter().map(|issue| issue.path.as_str()).collect::<Vec<_>>();
            assert_eq!(paths, ["tested_at", "evidence_uri"]);
        });
    let not_tested = ManagementProfileClaim::init()
        .profile(reference("hardware-management-profile"))
        .underlying_standard(reference("redfish"))
        .result(ConformanceResult::NotTested)
        .build();
    assert!(not_tested.validate().is_ok());
    let passed = ManagementProfileClaim::init()
        .profile(reference("hardware-management-profile"))
        .underlying_standard(reference("redfish"))
        .tested_at("2026-09-18T12:30:00-04:00")
        .result(ConformanceResult::Pass)
        .evidence_uri("https://example.org/reports/ocp-profile")
        .build();
    assert!(passed.validate().is_ok());
}
#[test]
fn test_ocp_contribution_fixture_projects_reference_without_claim_inference() {
    let content = include_str!("../../../../../tests/fixtures/schema/ocp/contribution.json");
    let contribution: Contribution = serde_json::from_str(content).expect("OCP contribution fixture should deserialize");
    assert!(contribution.validate().is_ok());
    let reference = StandardReference::from(&contribution);
    assert_eq!(reference.authority, "Open Compute Project");
    assert_eq!(reference.identifier, "S0806");
    assert_eq!(reference.title.as_deref(), Some("Hardware Management Profile"));
    assert_eq!(reference.version.as_deref(), Some("1.0"));
    assert_eq!(reference.canonical_uri, contribution.canonical_uri);
    let yaml = serde_norway::to_string(&contribution).expect("OCP contribution should serialize as YAML");
    let round_trip: Contribution = serde_norway::from_str(&yaml).expect("OCP contribution YAML should deserialize");
    assert_eq!(round_trip, contribution);
}
#[test]
fn test_ocp_contribution_identifier_and_open_values_validate() {
    assert!(ContributionId::from("S0806").validate().is_ok());
    assert!(ContributionId::from("0806").validate().is_err());
    assert!(ContributionId::from("S").validate().is_err());
    assert!(Project::from("Hardware Management").validate().is_ok());
    assert!(Project::from("  ").validate().is_err());
    assert!(ContributionKind::Other("Firmware bundle".to_string()).validate().is_ok());
    assert!(ContributionKind::Other("".to_string()).validate().is_err());
    assert!(RecognitionProgram::Other("Future program".to_string()).validate().is_ok());
    assert!(RecognitionProgram::Other("".to_string()).validate().is_err());
    assert_eq!(
        serde_json::from_str::<ContributionKind>(r#""firmwareBundle""#).expect("unknown contribution kind should deserialize"),
        ContributionKind::Other("firmwareBundle".to_string())
    );
    assert_eq!(
        serde_json::from_str::<RecognitionProgram>(r#""futureProgram""#).expect("unknown recognition program should deserialize"),
        RecognitionProgram::Other("futureProgram".to_string())
    );
}
#[test]
fn test_ocp_recognition_validates_source_and_timestamp_order() {
    let valid = Recognition::init()
        .program(RecognitionProgram::Accepted)
        .subject_identifier("product-1")
        .source_uri("https://www.opencompute.org/products/product-1")
        .awarded_at("2026-01-01T00:00:00Z")
        .expires_at("2027-01-01T00:00:00Z")
        .build();
    assert!(valid.validate().is_ok());
    let invalid_source = Recognition {
        source_uri: "urn:ocp:product-1".to_string(),
        ..valid.clone()
    };
    assert!(invalid_source.validate().is_err());
    let invalid_order = Recognition {
        expires_at: Some("2025-01-01T00:00:00Z".to_string()),
        ..valid
    };
    let report = invalid_order.validate().expect_err("expiration must follow award");
    assert!(report
        .iter()
        .any(|issue| issue.path == "expires_at" && issue.error.code == "timestamp_order"));
}
#[test]
fn test_standard_reference_and_verified_claim_require_http_evidence() {
    let invalid = StandardReference {
        canonical_uri: "urn:ocp:S0806".to_string(),
        ..reference("S0806")
    };
    assert!(invalid.validate().is_err());
    let claim = StandardsClaim::init()
        .standard(reference("S0806"))
        .relationship(StandardsRelationship::ConformsTo)
        .assessment(ClaimAssessment::Verified)
        .build();
    let report = claim.validate().expect_err("verified claim should require evidence");
    assert!(report
        .iter()
        .any(|issue| issue.path == "evidence_uri" && issue.error.code == "evidence_required"));
    let claim = StandardsClaim {
        evidence_uri: Some("https://example.org/evidence/S0806".to_string()),
        ..claim
    };
    assert!(claim.validate().is_ok());
}
