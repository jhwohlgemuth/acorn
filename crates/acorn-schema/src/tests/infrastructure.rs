use crate::hardware::{GpuArchitecture, Resource, Vendor};
use crate::infrastructure::{ComputeEnvironment, ComputeFacility, ComputePlatform};
use crate::standard::ocp::{Recognition, RecognitionProgram};
use crate::standard::{ClaimAssessment, StandardReference, StandardsClaim, StandardsRelationship};
use crate::validation::Validate;

fn accepted_recognition() -> Recognition {
    Recognition::init()
        .program(RecognitionProgram::Accepted)
        .subject_identifier("official-product-1")
        .source_uri("https://www.opencompute.org/products/official-product-1")
        .build()
}
fn gpu() -> Resource {
    Resource::GPU {
        architecture: Some(GpuArchitecture::Hopper),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: Some(Vendor::Nvidia),
    }
}
fn standard_claim() -> StandardsClaim {
    StandardsClaim::init()
        .standard(
            StandardReference::init()
                .authority("Open Compute Project")
                .identifier("S0806")
                .canonical_uri("https://www.opencompute.org/contributions?menu%5Bspec_id%5D=S0806")
                .build(),
        )
        .relationship(StandardsRelationship::ConformsTo)
        .assessment(ClaimAssessment::Declared)
        .build()
}
#[test]
fn test_infrastructure_fixture_round_trips_and_validates() {
    let content = include_str!("../../../../tests/fixtures/schema/ocp/infrastructure.json");
    let environment: ComputeEnvironment = serde_json::from_str(content).expect("infrastructure fixture should deserialize");
    assert!(environment.validate().is_ok());
    assert_eq!(environment.platforms.len(), 1);
    assert_eq!(environment.platforms[0].resources.len(), 1);
    assert_eq!(
        environment
            .facility
            .as_ref()
            .and_then(|facility| facility.ror.as_ref())
            .and_then(|rors| rors.first())
            .map(String::as_str),
        Some("https://ror.org/01qz5mb56")
    );
    let json = serde_json::to_string(&environment).expect("infrastructure should serialize as JSON");
    let round_trip: ComputeEnvironment = serde_json::from_str(&json).expect("infrastructure JSON should deserialize");
    assert_eq!(round_trip.identifier, environment.identifier);
    assert_eq!(round_trip.platforms.len(), environment.platforms.len());
    let portable_yaml = ComputeEnvironment::init()
        .identifier("yaml-environment")
        .name("YAML Environment")
        .platforms(vec![ComputePlatform::init()
            .identifier("yaml-platform")
            .name("YAML Platform")
            .resources(Vec::<Resource>::new())
            .build()])
        .build();
    let yaml = serde_norway::to_string(&portable_yaml).expect("infrastructure should serialize as YAML");
    let round_trip: ComputeEnvironment = serde_norway::from_str(&yaml).expect("infrastructure YAML should deserialize");
    assert_eq!(round_trip.identifier, portable_yaml.identifier);
    assert_eq!(round_trip.platforms.len(), portable_yaml.platforms.len());
}
#[test]
fn test_infrastructure_rejects_duplicate_claims() {
    let claim = standard_claim();
    let platform = ComputePlatform::init()
        .identifier("platform-1")
        .name("Platform 1")
        .resources(vec![gpu()])
        .standards_claims(vec![claim.clone(), claim])
        .build();
    let report = platform.validate().expect_err("duplicate claims should fail");
    assert!(report
        .iter()
        .any(|issue| issue.path == "standards_claims[1]" && issue.error.code == "duplicate_standard_claim"));
}
#[test]
fn test_infrastructure_rejects_empty_required_collections() {
    let platform = ComputePlatform::init()
        .identifier("platform-1")
        .name("Platform 1")
        .resources(Vec::<Resource>::new())
        .build();
    let report = platform.validate().expect_err("platform resources should not be empty");
    assert!(report.iter().any(|issue| issue.path == "resources" && issue.error.code == "nonempty"));
    let environment = ComputeEnvironment::init()
        .identifier("environment-1")
        .name("Environment 1")
        .platforms(Vec::<ComputePlatform>::new())
        .build();
    let report = environment.validate().expect_err("environment platforms should not be empty");
    assert!(report.iter().any(|issue| issue.path == "platforms" && issue.error.code == "nonempty"));
}
#[test]
fn test_infrastructure_rejects_recognition_on_wrong_subject_kind() {
    let platform = ComputePlatform::init()
        .identifier("platform-1")
        .name("Platform 1")
        .resources(vec![gpu()])
        .recognitions(vec![Recognition::init()
            .program(RecognitionProgram::Ready)
            .subject_identifier("facility-1")
            .source_uri("https://www.opencompute.org/facilities/facility-1")
            .build()])
        .build();
    let report = platform.validate().expect_err("Ready recognition should not attach to a platform");
    assert!(report
        .iter()
        .any(|issue| issue.path == "recognitions[0].program" && issue.error.code == "recognition_scope"));
    let facility = ComputeFacility::init()
        .identifier("facility-1")
        .name("Facility 1")
        .recognitions(vec![accepted_recognition()])
        .build();
    assert!(facility.validate().is_err());
    let environment = ComputeEnvironment::init()
        .identifier("environment-1")
        .name("Environment 1")
        .platforms(vec![ComputePlatform::init()
            .identifier("platform-1")
            .name("Platform 1")
            .resources(vec![gpu()])
            .build()])
        .recognitions(vec![accepted_recognition()])
        .build();
    assert!(environment.validate().is_err());
}
#[test]
fn test_infrastructure_validates_facility_ror() {
    let facility = ComputeFacility::init()
        .identifier("facility-1")
        .name("Facility 1")
        .ror(vec!["not-a-ror".to_string()])
        .build();
    let report = facility.validate().expect_err("facility ROR identifiers should use canonical ROR syntax");
    assert!(report.iter().any(|issue| issue.path == "ror[0]" && issue.error.code == "ROR"));
}
