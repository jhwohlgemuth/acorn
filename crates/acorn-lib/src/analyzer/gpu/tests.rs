use super::check_gpu_requirements;
use crate::analyzer::{CheckCategory, CheckSeverity};
use acorn_schema::hardware::{gpu, Memory, Resource, Vendor};

fn device(identifier: &str, memory_total_mib: Option<u64>) -> gpu::Device {
    gpu::Device {
        identifier: identifier.to_string(),
        index: None,
        name: identifier.to_string(),
        memory_total_mib,
    }
}
fn gpu_resource(count: Option<u32>, memory: Option<Memory>, required: Option<bool>, vendor: Option<Vendor>) -> Resource {
    Resource::GPU {
        architecture: None,
        backend: None,
        compute_capability: None,
        count,
        memory,
        name: None,
        required,
        vendor,
    }
}

#[test]
fn gpu_checks_preserve_resource_and_dimension_order() {
    let inventory = gpu::Inventory::new(vec![device("GPU-a", Some(81_920))]).expect("inventory should be valid");
    let resources = [
        Resource::Other("custom".to_string()),
        gpu_resource(Some(1), Some(Memory::gb(80)), None, Some(Vendor::Nvidia)),
    ];
    let checks = check_gpu_requirements(&resources, &inventory, Some("activity.json".to_string()));
    let locators = checks.iter().filter_map(|check| check.locator.as_deref()).collect::<Vec<_>>();
    assert_eq!(locators, vec!["resources[1].count", "resources[1].memory", "resources[1].vendor"]);
    assert!(checks.iter().all(|check| check.uri.as_deref() == Some("activity.json")));
    assert!(checks.iter().all(|check| check.category == CheckCategory::Compute));
    assert_eq!(checks.iter().filter(|check| check.success).count(), 2);
    let vendor = checks.iter().find(|check| check.locator.as_deref() == Some("resources[1].vendor"));
    assert_eq!(vendor.map(|check| &check.severity), Some(&CheckSeverity::Suggestion));
    assert!(vendor.is_some_and(|check| !check.is_failure()));
}
#[test]
fn gpu_count_shortfalls_respect_required_flag() {
    let inventory = gpu::Inventory::new(vec![device("GPU-a", Some(1))]).expect("inventory should be valid");
    let resources = [gpu_resource(Some(2), None, None, None), gpu_resource(Some(2), None, Some(false), None)];
    let checks = check_gpu_requirements(&resources, &inventory, None);
    let required = checks.iter().find(|check| check.locator.as_deref() == Some("resources[0].count"));
    assert_eq!(required.map(|check| &check.severity), Some(&CheckSeverity::Error));
    assert!(required.is_some_and(crate::analyzer::Check::is_failure));
    let optional = checks.iter().find(|check| check.locator.as_deref() == Some("resources[1].count"));
    assert_eq!(optional.map(|check| &check.severity), Some(&CheckSeverity::Suggestion));
    assert!(optional.is_some_and(|check| !check.is_failure()));
}
#[test]
fn gpu_missing_memory_is_advisory_unknown() {
    let inventory = gpu::Inventory::new(vec![device("GPU-a", None)]).expect("inventory should be valid");
    let resources = [gpu_resource(Some(1), Some(Memory::gb(80)), None, None)];
    let checks = check_gpu_requirements(&resources, &inventory, None);
    let memory = checks.iter().find(|check| check.locator.as_deref() == Some("resources[0].memory"));
    assert_eq!(memory.map(|check| &check.severity), Some(&CheckSeverity::Suggestion));
    assert!(memory.is_some_and(|check| check.message.contains("cannot assess") && !check.is_failure()));
}
