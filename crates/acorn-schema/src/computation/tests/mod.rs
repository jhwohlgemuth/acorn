use super::*;
use crate::hardware::Resource;
use crate::research_activity::ResearchActivity;
use serde_json::json;

const COMPUTATION_ID: &str = "00000000-0000-4000-8000-000000000001";
const PREPARE_ID: &str = "00000000-0000-4000-8000-000000000002";
const PROBLEM_ID: &str = "00000000-0000-4000-8000-000000000003";
const PROFILE_ID: &str = "00000000-0000-4000-8000-000000000004";
const SAMPLE_ID: &str = "00000000-0000-4000-8000-000000000005";

#[test]
fn test_complexity_bounds_round_trip_and_validate() {
    let value = json!({
        "case": "worst",
        "bounds": [{"time": "O(n log n)"}, {"space": "O(n)"}],
        "variables": {"n": "number of records"}
    });
    let complexity: AlgorithmComplexity = serde_json::from_value(value.clone()).expect("parse complexity");
    complexity.validate().expect("validate complexity");
    assert_eq!(serde_json::to_value(complexity).expect("serialize complexity"), value);

    let without_variables: AlgorithmComplexity =
        serde_json::from_value(json!({"bounds": {"time": "O(n)"}})).expect("parse complexity without variables");
    without_variables.validate().expect("allow omitted variables map");
}
#[test]
fn test_complexity_validation_rejects_notation_references_empty_and_duplicates() {
    let complexity: AlgorithmComplexity = serde_json::from_value(json!({
        "bounds": [
            {"time": "O(n / m)"},
            {"space": "O(n)"},
            {"time": ""}
        ],
        "variables": {"m": "number of batches"}
    }))
    .expect("parse invalid complexity");
    let report = complexity.validate().expect_err("reject invalid complexity");
    let issues: Vec<_> = report.iter().map(|issue| (issue.path.as_str(), issue.error.code.as_str())).collect();
    assert_eq!(
        issues,
        [
            ("bounds[0]", "syntax"),
            ("bounds[1]", "reference"),
            ("bounds[2]", "nonempty"),
            ("bounds[2]", "duplicate")
        ]
    );
}
#[test]
fn test_duplicate_stages_and_profiles_are_rejected() {
    let computation: Computation = serde_json::from_value(json!({
        "identifier": COMPUTATION_ID,
        "stages": [
            {"identifier": PREPARE_ID, "operation": "classical"},
            {"identifier": PREPARE_ID, "operation": "classical"}
        ],
        "profiles": [
            {"identifier": PROFILE_ID, "environment": "lab", "stages": [
                {"stage": PREPARE_ID, "platform": "host"},
                {"stage": PREPARE_ID, "platform": "host"}
            ]},
            {"identifier": PROFILE_ID, "environment": "lab", "stages": {"stage": PREPARE_ID, "platform": "host"}}
        ]
    }))
    .expect("parse duplicate references");
    let report = computation.validate().expect_err("reject duplicate references");
    let paths: Vec<_> = report.iter().map(|issue| issue.path.as_str()).collect();
    assert!(paths.contains(&"stages[1].identifier"));
    assert!(paths.contains(&"profiles[0].stages[1].stage"));
    assert!(paths.contains(&"profiles[1].identifier"));
}
#[test]
fn test_hybrid_computation_round_trip_and_validation() {
    let value = json!({
        "identifier": COMPUTATION_ID,
        "stages": [
            {"identifier": PREPARE_ID, "operation": "classical", "outputs": {"name": PROBLEM_ID, "data": {"Synthetic": {"modality": "text"}}}},
            {"identifier": SAMPLE_ID, "operation": {"quantum": {"circuit": {"logicalQubits": 8, "logicalDepth": 12, "gateBasis": "Clifford+T"}}}, "dependsOn": [PREPARE_ID], "inputs": {"name": PROBLEM_ID, "data": {"Synthetic": {"modality": "text"}}}}
        ],
        "profiles": {"identifier": PROFILE_ID, "environment": "lab", "stages": [
            {"stage": PREPARE_ID, "platform": "host", "duration": 2},
            {"stage": SAMPLE_ID, "platform": "qpu", "duration": 1, "repetitions": 100, "compiledCircuit": {"depth": 40, "physicalQubits": 24, "gateBasis": "native"}}
        ], "impedance": {"kind": "inputTransfer", "basis": "measured", "scope": "workload", "stage": SAMPLE_ID, "port": PROBLEM_ID, "duration": 60000, "volume": "1MB", "bandwidth": "16KB", "latency": 0}, "observedEndToEnd": 60003}
    });
    let computation: Computation = serde_json::from_value(value.clone()).expect("valid computation JSON");
    computation.validate().expect("valid computation");
    let serialized = serde_json::to_value(&computation).expect("serialize computation");
    assert_eq!(serialized.pointer("/stages/1/operation/quantum/circuit/logicalDepth"), Some(&json!(12)));
    assert_eq!(serialized.pointer("/profiles/impedance/duration"), Some(&json!(60000)));
    assert_eq!(serialized.pointer("/profiles/impedance/volume"), Some(&json!("1MB")));
    assert_eq!(serialized.pointer("/profiles/impedance/bandwidth"), Some(&json!("16KB")));
    let environment = serde_json::from_value(json!({
        "identifier": "lab", "name": "Lab", "platforms": [
            {"identifier": "host", "name": "Host", "resources": "CPU"},
            {"identifier": "qpu", "name": "QPU", "resources": "Quantum"}
        ]
    }))
    .expect("parse compute environment");
    let activity = ResearchActivity {
        meta: crate::research_activity::ResearchActivityMetadata {
            infrastructure: Some(vec![environment].into()),
            ..Default::default()
        },
        aspect: Some(
            crate::research_activity::aspect::AspectFramework::init()
                .computation(vec![computation])
                .build(),
        ),
        ..Default::default()
    };
    assert!(activity.validate().is_ok());
}
#[test]
fn test_invalid_dependencies_and_profile_references_are_rejected() {
    let value = json!({
        "identifier": "invalid",
        "stages": [{"identifier": "first", "operation": "classical", "dependsOn": ["later"]}, {"identifier": "later", "operation": "classical"}],
        "profiles": {"identifier": "run", "environment": "missing", "stages": {"stage": "unknown", "platform": "missing"},
            "impedance": {"kind": "inputTransfer", "basis": "estimated", "scope": "workload", "stage": "first", "port": "unknown"}}
    });
    let computation: Computation = serde_json::from_value(value.clone()).expect("parse invalid references");
    let report = computation.validate().expect_err("invalid stage references");
    assert!(report.to_string().contains("dependsOn"));
    assert!(report.to_string().contains("stage"));
    assert!(report.to_string().contains("port"));
    let activity = ResearchActivity {
        aspect: Some(
            crate::research_activity::aspect::AspectFramework::init()
                .computation(vec![computation])
                .build(),
        ),
        ..Default::default()
    };
    let report = activity.validate().expect_err("missing environment");
    assert!(report.to_string().contains("environment"));
}
#[test]
fn test_quantum_hardware_dimensions_are_independent() {
    let value = json!({"Quantum": {
        "model": "Heron", "technology": "superconducting", "qubits": [
            {"type": "physical", "count": 133}, {"type": "logical", "count": 2}
        ], "fault_tolerance": {"code": "surface", "physicalToLogicalRatio": 50}
    }});
    let resource: Resource = serde_json::from_value(value).expect("parse quantum hardware");
    let serialized = serde_json::to_value(resource).expect("serialize quantum hardware");
    assert_eq!(serialized.pointer("/Quantum/technology"), Some(&json!("superconducting")));
    assert_eq!(serialized.pointer("/Quantum/qubits/0/count"), Some(&json!(133)));
    assert_eq!(serialized.pointer("/Quantum/qubits/1/count"), Some(&json!(2)));
    assert_eq!(serialized.pointer("/Quantum/fault_tolerance/code"), Some(&json!("surface")));
}
#[test]
fn test_zero_bandwidth_is_rejected() {
    let cost: ImpedanceCost = serde_json::from_value(json!({
        "kind": "inputTransfer", "basis": "estimated", "scope": "invocation", "bandwidth": "0KB"
    }))
    .expect("parse cost");
    let report = cost.validate().expect_err("reject zero bandwidth");
    assert!(report.to_string().contains("bandwidth"));
}
