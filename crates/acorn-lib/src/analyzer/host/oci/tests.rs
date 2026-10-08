use super::*;
use crate::io::oci::{OciInspectionError, OciInspectionResult, OciReferenceValue};
use acorn_schema::agent::Quantization;
use core::cell::Cell;

enum FakeResult {
    Failure(OciInspectionStage),
    Success,
}
struct FakeInspector {
    calls: Cell<usize>,
    result: FakeResult,
}
impl FakeInspector {
    fn failure(stage: OciInspectionStage) -> Self {
        Self {
            calls: Cell::new(0),
            result: FakeResult::Failure(stage),
        }
    }
    fn success() -> Self {
        Self {
            calls: Cell::new(0),
            result: FakeResult::Success,
        }
    }
}
impl OciInspector for FakeInspector {
    fn inspect(&self, reference: &OciReference, _filter: &[String], _ignore: &[String], _quantization: &[Quantization]) -> OciInspectionResult {
        self.calls.set(self.calls.get().saturating_add(1));
        match self.result {
            | FakeResult::Failure(stage) => Err(OciInspectionError::artifact(stage, eyre!("fixture failure"))),
            | FakeResult::Success => Ok(OciResolution {
                registry: reference.registry().to_string(),
                repository: reference.repository().to_string(),
                requested_reference: reference.to_string(),
                resolved_digest: "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
                artifact_type: Some("application/vnd.acorn.model.v1".to_string()),
                package_format: ModelArtifactKind::Annotated,
                inventory_status: ModelInventoryStatus::Complete,
                total_size: 1,
                transfer_size: 1,
                installed_size: Some(1),
                layers: Vec::new(),
                ignored_layers: Vec::new(),
                files: Vec::new(),
                filter: Vec::new(),
                ignore: Vec::new(),
                quantization: Vec::new(),
            }),
        }
    }
}
fn profile(endpoint: &str) -> RegistryProfile {
    RegistryProfile {
        endpoint: endpoint.to_string(),
        ..RegistryProfile::default()
    }
}
fn reference() -> OciReference {
    OciReference::parse("oci://registry.example.org/models/fixture:latest").expect("valid OCI fixture reference")
}
fn report_category(report: &AnalysisReport, category: CheckCategory) -> &CategoryChecks {
    report
        .batches
        .first()
        .and_then(|batch| batch.categories.iter().find(|result| result.category == category))
        .expect("report category")
}

#[test]
fn test_adapter_maps_package_failure_to_quality_after_schema_success() {
    let inspector = FakeInspector::failure(OciInspectionStage::Package);
    let report = analyze_with(&inspector, &reference(), &AnalysisContext::default()).expect("artifact failure should become findings");
    let schema = report_category(&report, CheckCategory::Schema);
    let quality = report_category(&report, CheckCategory::Quality);
    assert!(schema.checks.iter().all(|check| check.success));
    assert!(quality.checks.iter().any(acorn_analysis::Check::is_failure));
    assert_eq!(inspector.calls.get(), 1);
}
#[test]
fn test_adapter_maps_schema_failure_and_skips_quality() {
    let inspector = FakeInspector::failure(OciInspectionStage::Manifest);
    let report = analyze_with(&inspector, &reference(), &AnalysisContext::default()).expect("artifact failure should become findings");
    let schema = report_category(&report, CheckCategory::Schema);
    let quality = report_category(&report, CheckCategory::Quality);
    assert!(schema.checks.iter().any(acorn_analysis::Check::is_failure));
    assert!(quality.status.is_skipped());
    assert_eq!(inspector.calls.get(), 1);
}
#[test]
fn test_adapter_redacts_invalid_reference_credentials() {
    let report = analyze(
        "oci://user:super-secret@registry.example.org/models/fixture:latest",
        &AnalysisContext::default(),
        &ApplicationConfiguration::default(),
    )
    .expect("invalid reference should become a finding");
    let serialized = format!("{:?}", report.checks());
    assert!(!serialized.contains("super-secret"));
    assert!(serialized.contains("invalid OCI reference"));
}
#[test]
fn test_adapter_reports_resolved_digest_from_one_inspection() {
    let inspector = FakeInspector::success();
    let report = analyze_with(&inspector, &reference(), &AnalysisContext::default()).expect("inspection should succeed");
    assert!(report
        .checks()
        .iter()
        .any(|check| check.category == CheckCategory::Schema && check.message.contains("sha256:0123456789abcdef")));
    assert_eq!(inspector.calls.get(), 1);
}
#[test]
fn test_transport_options_reject_ambiguous_profiles_deterministically() {
    let profiles = HashMap::from([
        ("zeta".to_string(), profile("https://registry.example.org/api")),
        ("alpha".to_string(), profile("https://registry.example.org")),
    ]);
    let error = transport_options(&reference(), &profiles).expect_err("duplicate authority must fail");
    assert!(error.to_string().contains("alpha, zeta"));
}
#[test]
fn test_transport_options_select_public_or_unique_profile() {
    assert_eq!(
        transport_options(&reference(), &HashMap::new()).expect("public options"),
        OciTransportOptions::default()
    );
    let profiles = HashMap::from([
        ("match".to_string(), profile("https://registry.example.org/api")),
        ("other".to_string(), profile("https://other.example.org")),
    ]);
    let selected = transport_options(&reference(), &profiles).expect("unique profile");
    assert_eq!(selected, OciTransportOptions::default());
    assert!(matches!(reference().reference(), OciReferenceValue::Tag(tag) if tag == "latest"));
}
