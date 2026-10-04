use super::{Device, Inventory, Reading, RequirementAssessment, RequirementState, Sample};
use crate::hardware::{Backend, GpuArchitecture, Memory, Resource, Vendor};
use crate::validation::Validate;
use acorn_core::time::{Milliseconds, UnitOfTime};
use serde_json::{from_str, to_string};

fn device(identifier: &str, index: Option<u32>) -> Device {
    Device {
        identifier: identifier.to_string(),
        index,
        name: format!("{identifier} name"),
        memory_total_mib: Some(81_920),
    }
}
fn gpu_resource(count: Option<u32>, memory: Option<Memory>) -> Resource {
    Resource::GPU {
        architecture: None,
        backend: None,
        compute_capability: None,
        count,
        memory,
        name: None,
        required: None,
        vendor: None,
    }
}
fn issues<T: Validate>(value: &T) -> Vec<(String, String)> {
    value
        .validate()
        .expect_err("value should be invalid")
        .into_issues()
        .into_iter()
        .map(|issue| (issue.path, issue.error.code))
        .collect()
}
fn reading(identifier: &str, utilization_percent: Option<u8>) -> Reading {
    Reading {
        identifier: identifier.to_string(),
        memory_used_mib: Some(1_024),
        utilization_percent,
    }
}

#[test]
fn test_gpu_contract_rejects_unknown_and_legacy_fields() {
    assert!(from_str::<Device>(r#"{"identifier":"GPU-a","index":0,"name":"A","memory_total_mib":1,"extra":true}"#).is_err());
    assert!(from_str::<Device>(r#"{"uuid":"GPU-a","index":0,"name":"A","memory_total_mib":1}"#).is_err());
    assert!(from_str::<Reading>(r#"{"identifier":"GPU-a","memory_used_mib":1,"utilization_percent":2,"extra":true}"#).is_err());
    assert!(from_str::<Sample>(r#"{"timestamp":0,"readings":[],"extra":true}"#).is_err());
}
#[test]
fn test_gpu_contract_round_trips_exact_json() {
    let device_json = r#"{"identifier":"GPU-a","index":0,"name":"NVIDIA H100","memory_total_mib":81920}"#;
    let parsed_device = from_str::<Device>(device_json).expect("device should deserialize");
    assert_eq!(to_string(&parsed_device).expect("device should serialize"), device_json);
    let inventory_json = format!("[{device_json}]");
    let parsed_inventory = from_str::<Inventory>(&inventory_json).expect("inventory should deserialize");
    assert_eq!(to_string(&parsed_inventory).expect("inventory should serialize"), inventory_json);
    let reading_json = r#"{"identifier":"GPU-a","memory_used_mib":null,"utilization_percent":null}"#;
    let parsed_reading = from_str::<Reading>(reading_json).expect("reading should deserialize");
    assert_eq!(to_string(&parsed_reading).expect("reading should serialize"), reading_json);
    let sample_json = format!(r#"{{"timestamp":0,"readings":[{reading_json}]}}"#);
    let parsed_sample = from_str::<Sample>(&sample_json).expect("sample should deserialize");
    assert_eq!(parsed_sample.timestamp, Milliseconds::new(0));
    assert_eq!(to_string(&parsed_sample).expect("sample should serialize"), sample_json);
}
#[test]
fn test_gpu_inventory_constructor_and_accessors_preserve_order() {
    let devices = vec![device("GPU-b", Some(1)), device("GPU-a", Some(0))];
    let inventory = Inventory::new(devices.clone()).expect("inventory should be valid");
    assert_eq!(inventory.devices(), devices.as_slice());
    assert_eq!(inventory.into_devices(), devices);
}
#[test]
fn test_gpu_inventory_reports_duplicate_identifiers_and_present_indices_in_order() {
    let inventory = Inventory(vec![device("GPU-a", Some(2)), device("GPU-a", Some(2)), device("GPU-a", None)]);
    assert_eq!(
        issues(&inventory),
        vec![
            ("[1].identifier".to_string(), "unique".to_string()),
            ("[2].identifier".to_string(), "unique".to_string()),
            ("[1].index".to_string(), "unique".to_string()),
        ]
    );
}
#[test]
fn test_gpu_requirement_assessment_checks_count_and_memory_per_device() {
    let inventory = Inventory::new(vec![device("GPU-a", Some(0)), device("GPU-b", Some(1))]).expect("inventory should be valid");
    let assessment = gpu_resource(Some(2), Some(Memory::gb(80)))
        .assess_gpu_inventory(&inventory)
        .expect("GPU resource should be assessed");
    assert_eq!(assessment.count, RequirementState::Satisfied);
    assert_eq!(assessment.memory, RequirementState::Satisfied);
    assert_eq!(assessment.overall(), RequirementState::Satisfied);
    let shortfall = gpu_resource(Some(3), Some(Memory::gb(80)))
        .assess_gpu_inventory(&inventory)
        .expect("GPU resource should be assessed");
    assert_eq!(shortfall.count, RequirementState::Unsatisfied);
    assert_eq!(shortfall.memory, RequirementState::Unsatisfied);
}
#[test]
fn test_gpu_requirement_assessment_preserves_unknown_observations() {
    let mut unknown_memory = device("GPU-a", Some(0));
    unknown_memory.memory_total_mib = None;
    let inventory = Inventory::new(vec![unknown_memory]).expect("inventory should be valid");
    let resource = Resource::GPU {
        architecture: Some(GpuArchitecture::Ampere),
        backend: Some(Backend::Cuda),
        compute_capability: Some(8.0),
        count: None,
        memory: Some(Memory::gb(80)),
        name: Some("NVIDIA H100".to_string()),
        required: None,
        vendor: Some(Vendor::Nvidia),
    };
    let assessment = resource.assess_gpu_inventory(&inventory).expect("GPU resource should be assessed");
    assert_eq!(assessment.count, RequirementState::Satisfied);
    assert_eq!(assessment.memory, RequirementState::Unknown);
    assert_eq!(assessment.architecture, RequirementState::Unknown);
    assert_eq!(assessment.backend, RequirementState::Unknown);
    assert_eq!(assessment.compute_capability, RequirementState::Unknown);
    assert_eq!(assessment.name, RequirementState::Unknown);
    assert_eq!(assessment.vendor, RequirementState::Unknown);
    assert_eq!(assessment.overall(), RequirementState::Unknown);
}
#[test]
fn test_gpu_requirement_assessment_prioritizes_failure_and_skips_other_resources() {
    let assessment = RequirementAssessment {
        count: RequirementState::Unknown,
        memory: RequirementState::Unsatisfied,
        architecture: RequirementState::Satisfied,
        backend: RequirementState::Satisfied,
        compute_capability: RequirementState::Satisfied,
        name: RequirementState::Satisfied,
        vendor: RequirementState::Satisfied,
    };
    assert_eq!(assessment.overall(), RequirementState::Unsatisfied);
    let inventory = Inventory::new(vec![device("GPU-a", Some(0))]).expect("inventory should be valid");
    assert_eq!(Resource::Other("custom".to_string()).assess_gpu_inventory(&inventory), None);
}
#[test]
fn test_gpu_sample_reports_duplicate_identifiers_in_order() {
    let sample = Sample {
        timestamp: Milliseconds::new(1),
        readings: vec![reading("GPU-a", Some(1)), reading("GPU-a", None), reading("GPU-a", Some(100))],
    };
    assert_eq!(
        issues(&sample),
        vec![
            ("readings[1].identifier".to_string(), "unique".to_string()),
            ("readings[2].identifier".to_string(), "unique".to_string()),
        ]
    );
}
#[test]
fn test_gpu_validation_accepts_optional_values_boundaries_and_opaque_identifiers() {
    let inventory = Inventory::new(vec![device("GPU-a", None), device("gpu-a", None)]).expect("absent indices should repeat");
    assert_eq!(inventory.devices().first().map(|device| device.identifier.as_str()), Some("GPU-a"));
    let sample = Sample {
        timestamp: Milliseconds::new(0),
        readings: vec![
            Reading {
                identifier: " GPU-a ".to_string(),
                memory_used_mib: None,
                utilization_percent: Some(0),
            },
            Reading {
                identifier: "unlisted-device".to_string(),
                memory_used_mib: Some(u64::MAX),
                utilization_percent: Some(100),
            },
        ],
    };
    assert!(sample.validate().is_ok());
}
#[test]
fn test_gpu_validation_rejects_empty_collections() {
    assert_eq!(issues(&Inventory(Vec::new())), vec![(String::new(), "nonempty".to_string())]);
    assert_eq!(
        issues(&Sample {
            timestamp: Milliseconds::new(0),
            readings: Vec::new(),
        }),
        vec![("readings".to_string(), "nonempty".to_string())]
    );
    assert!(Inventory::new(Vec::new()).is_err());
}
#[test]
fn test_gpu_validation_reports_local_field_failures() {
    assert_eq!(
        issues(&Device {
            identifier: " \n".to_string(),
            index: None,
            name: "\t".to_string(),
            memory_total_mib: None,
        }),
        vec![
            ("identifier".to_string(), "nonempty".to_string()),
            ("name".to_string(), "nonempty".to_string()),
        ]
    );
    assert_eq!(
        issues(&reading("", Some(101))),
        vec![
            ("identifier".to_string(), "nonempty".to_string()),
            ("utilization_percent".to_string(), "range".to_string()),
        ]
    );
}
