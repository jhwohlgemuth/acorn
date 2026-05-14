use super::vendor::{
    AmdCpuArchitecture, AmdGpuArchitecture, AppleCpuArchitecture, AppleGpuArchitecture, IntelCpuArchitecture, IntelGpuArchitecture,
    NvidiaGpuArchitecture,
};
use super::{Architecture, CpuArchitecture, GpuArchitecture, Memory, MemoryUnit, Paradigm, Resource, Vendor};
use crate::schema::research_activity::ResearchActivityMetadata;

#[test]
fn test_architecture_from_string() {
    let tests = [
        ("x86_64", Architecture::X86_64),
        ("amd64", Architecture::X86_64),
        ("x86-64", Architecture::X86_64),
        ("AArch64", Architecture::AArch64),
        ("Ampere", Architecture::Ampere),
        ("RDNA1", Architecture::Rdna1),
        ("rdna1", Architecture::Rdna1),
        ("Versal", Architecture::Versal),
        ("TPU v4", Architecture::TpuV4),
        ("C66x", Architecture::C66x),
        ("Neural Engine", Architecture::NeuralEngine),
        ("unknown_arch", Architecture::Other("unknown_arch".to_string())),
    ];
    for (input, expected) in tests {
        let result = Architecture::from(input);
        assert_eq!(result, expected, "Architecture::from(\"{}\") failed", input);
    }
}
#[test]
fn test_architecture_deserialize() {
    let tests = [
        (r#""x86_64""#, Architecture::X86_64),
        (r#""Ampere""#, Architecture::Ampere),
        (r#""RDNA1""#, Architecture::Rdna1),
        (r#""Versal""#, Architecture::Versal),
        (r#""TPU v4""#, Architecture::TpuV4),
        (r#""custom_arch""#, Architecture::Other("custom_arch".to_string())),
    ];
    for (json, expected) in tests {
        let result: Result<Architecture, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Failed to deserialize: {}", json);
        assert_eq!(result.unwrap(), expected, "Deserialization mismatch for: {}", json);
    }
}
#[test]
fn test_vendor_architecture_enums() {
    assert!(matches!(NvidiaGpuArchitecture::Ampere, NvidiaGpuArchitecture::Ampere));
    assert!(matches!(AmdGpuArchitecture::Cdna1, AmdGpuArchitecture::Cdna1));
    assert!(matches!(AmdCpuArchitecture::X86_64, AmdCpuArchitecture::X86_64));
    assert!(matches!(IntelCpuArchitecture::Ia64, IntelCpuArchitecture::Ia64));
    assert!(matches!(
        super::vendor::IntelFpgaArchitecture::Agilex,
        super::vendor::IntelFpgaArchitecture::Agilex
    ));
    assert!(matches!(IntelGpuArchitecture::Alchemist, IntelGpuArchitecture::Alchemist));
    assert!(matches!(AppleCpuArchitecture::NeuralEngine, AppleCpuArchitecture::NeuralEngine));
    assert!(matches!(AppleGpuArchitecture::M3, AppleGpuArchitecture::M3));
}
#[test]
fn test_hardware_resource_dpu_from_string() {
    let data: Resource = serde_json::from_str("\"dpu\"").unwrap();
    match data {
        | Resource::DPU {
            architecture,
            count,
            memory,
            vendor,
        } => {
            assert!(architecture.is_none());
            assert_eq!(count, Some(1));
            assert!(memory.is_none());
            assert!(vendor.is_none());
        }
        | other => panic!("expected DPU resource, got {other:?}"),
    }
}
#[test]
fn test_hardware_resource_dpu_from_map() {
    let data: Resource = serde_json::from_str(r#"{"DPU":{"arch":"x86_64","count":2,"memory":64,"vendor":"Intel"}}"#).unwrap();
    match data {
        | Resource::DPU {
            architecture,
            count,
            memory,
            vendor,
        } => {
            assert_eq!(architecture, Some(CpuArchitecture::X86_64));
            assert_eq!(count, Some(2));
            assert_eq!(memory, Some(Memory::gb(64)));
            match vendor {
                | Some(Vendor::Intel) => {}
                | other => panic!("expected Intel vendor, got {other:?}"),
            }
        }
        | other => panic!("expected DPU resource, got {other:?}"),
    }
}
#[test]
fn test_memory_deserialize() {
    let mem: Memory = serde_json::from_str("\"200GB\"").unwrap();
    assert_eq!(mem, Memory::gb(200));
    let mem: Memory = serde_json::from_str("\"100KB\"").unwrap();
    assert_eq!(mem, Memory::kb(100));
    let mem: Memory = serde_json::from_str("\"512MB\"").unwrap();
    assert_eq!(mem, Memory::mb(512));
    let mem: Memory = serde_json::from_str("\"64 GB\"").unwrap();
    assert_eq!(mem, Memory::gb(64));
    let mem: Memory = serde_json::from_str("80").unwrap();
    assert_eq!(mem, Memory::gb(80));
    let mem: Memory = serde_json::from_str("\"256M\"").unwrap();
    assert_eq!(mem, Memory::mb(256));
    let mem: Memory = serde_json::from_str("\"128kb\"").unwrap();
    assert_eq!(mem, Memory::kb(128));
    let mem = Memory::gb(96);
    let json = serde_json::to_string(&mem).unwrap();
    assert_eq!(json, "\"96GB\"");
    let parsed: Memory = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, mem);
}
#[test]
fn test_memory_deserialize_float() {
    let mem: Memory = serde_json::from_str("\"2.5GB\"").unwrap();
    assert!((mem.amount - 2.5).abs() < f64::EPSILON);
    assert_eq!(mem.unit, MemoryUnit::GB);
    let mem: Memory = serde_json::from_str("\"1.2MB\"").unwrap();
    assert!((mem.amount - 1.2).abs() < f64::EPSILON);
    assert_eq!(mem.unit, MemoryUnit::MB);
    let mem: Memory = serde_json::from_str("\"0.5KB\"").unwrap();
    assert!((mem.amount - 0.5).abs() < f64::EPSILON);
    assert_eq!(mem.unit, MemoryUnit::KB);
    let mem: Memory = serde_json::from_str("\"3.75 MB\"").unwrap();
    assert!((mem.amount - 3.75).abs() < f64::EPSILON);
    assert_eq!(mem.unit, MemoryUnit::MB);
    let mem = Memory::gb(2.5);
    let json = serde_json::to_string(&mem).unwrap();
    assert_eq!(json, "\"2.5GB\"");
    let parsed: Memory = serde_json::from_str(&json).unwrap();
    assert!((parsed.amount - 2.5).abs() < f64::EPSILON);
    assert_eq!(parsed.unit, MemoryUnit::GB);
}
#[test]
fn test_memory_in_resource_with_string() {
    let json = r#"{"CPU":{"memory":"256GB"}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::CPU { memory, .. } => {
            assert_eq!(memory, Some(Memory::gb(256)));
        }
        | other => panic!("expected CPU, got {other:?}"),
    }
    let json = r#"{"GPU":{"memory":"80GB","vendor":"NVIDIA"}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::GPU { memory, .. } => {
            assert_eq!(memory, Some(Memory::gb(80)));
        }
        | other => panic!("expected GPU, got {other:?}"),
    }
    let json = r#"{"FPGA":{"memory":"4MB"}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::FPGA { memory, .. } => {
            assert_eq!(memory, Some(Memory::mb(4)));
        }
        | other => panic!("expected FPGA, got {other:?}"),
    }
    let json = r#"{"NPU":{"memory":"8KB"}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::NPU { memory, .. } => {
            assert_eq!(memory, Some(Memory::kb(8)));
        }
        | other => panic!("expected NPU, got {other:?}"),
    }
    let json = r#"{"TPU":{"memory":"32GB"}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::TPU { memory, .. } => {
            assert_eq!(memory, Some(Memory::gb(32)));
        }
        | other => panic!("expected TPU, got {other:?}"),
    }
    let json = r#"{"TPU":{"memory":"32GB"}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::TPU { memory, .. } => {
            assert_eq!(memory, Some(Memory::gb(32)));
        }
        | other => panic!("expected TPU, got {other:?}"),
    }
    let json = r#"{"DPU":{"memory":64}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::DPU { memory, .. } => {
            assert_eq!(memory, Some(Memory::gb(64)));
        }
        | other => panic!("expected DPU, got {other:?}"),
    }
}
#[test]
fn test_memory_float_in_resource() {
    let json = r#"{"GPU":{"memory":"2.5GB"}}"#;
    let resource: Resource = serde_json::from_str(json).unwrap();
    match resource {
        | Resource::GPU { memory, .. } => {
            let mem = memory.expect("memory should be present");
            assert!((mem.amount - 2.5).abs() < f64::EPSILON);
            assert_eq!(mem.unit, MemoryUnit::GB);
        }
        | other => panic!("expected GPU, got {other:?}"),
    }
}
#[test]
fn test_memory_in_research_activity_string_values() {
    let json = r#"{
		"identifier": "memory-test",
		"archive": false,
		"draft": false,
		"status": "active",
		"keywords": [],
		"technology": [],
		"resources": [
			{"GPU": {"memory": "80GB"}},
			{"CPU": {"memory": "512GB", "cores": 64}},
			{"FPGA": {"memory": "4MB"}},
			{"NPU": {"memory": "8KB"}}
		]
	}"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).unwrap();
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 4);
    assert!(matches!(
        &resources[0],
        Resource::GPU {
            memory: Some(Memory {
                amount: 80.0,
                unit: MemoryUnit::GB
            }),
            ..
        }
    ));
    assert!(matches!(
        &resources[1],
        Resource::CPU {
            memory: Some(Memory {
                amount: 512.0,
                unit: MemoryUnit::GB
            }),
            ..
        }
    ));
    assert!(matches!(
        &resources[2],
        Resource::FPGA {
            memory: Some(Memory {
                amount: 4.0,
                unit: MemoryUnit::MB
            }),
            ..
        }
    ));
    assert!(matches!(
        &resources[3],
        Resource::NPU {
            memory: Some(Memory {
                amount: 8.0,
                unit: MemoryUnit::KB
            }),
            ..
        }
    ));
}
#[test]
fn test_memory_in_research_activity_float() {
    let json = r#"{
		"identifier": "float-mem-test",
		"archive": false,
		"draft": false,
		"status": "active",
		"keywords": [],
		"technology": [],
		"resources": [
			{"GPU": {"memory": "3.5GB"}},
			{"NPU": {"memory": "1.25MB"}}
		]
	}"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).unwrap();
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 2);
    if let Resource::GPU { memory, .. } = &resources[0] {
        let mem = memory.as_ref().expect("GPU memory should be present");
        assert!((mem.amount - 3.5).abs() < f64::EPSILON);
        assert_eq!(mem.unit, MemoryUnit::GB);
    } else {
        panic!("expected GPU");
    }
    if let Resource::NPU { memory, .. } = &resources[1] {
        let mem = memory.as_ref().expect("NPU memory should be present");
        assert!((mem.amount - 1.25).abs() < f64::EPSILON);
        assert_eq!(mem.unit, MemoryUnit::MB);
    } else {
        panic!("expected NPU");
    }
}
#[test]
fn test_quantum_paradigm_deserialize() {
    let tests = [
        (r#""NISQ""#, Paradigm::Nisq),
        (r#""nisq""#, Paradigm::Nisq),
        (r#""Noisy Intermediate-Scale Quantum""#, Paradigm::Nisq),
    ];
    for (json, expected) in tests {
        let actual: Paradigm = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "QuantumParadigm mismatch: {}", json);
    }
    let tests = [
        (r#""Hybrid Quantum-Classical""#, Paradigm::HybridQuantumClassical),
        (r#""hybrid quantum-classical""#, Paradigm::HybridQuantumClassical),
        (r#""hybrid quantum classical""#, Paradigm::HybridQuantumClassical),
    ];
    for (json, expected) in tests {
        let actual: Paradigm = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "QuantumParadigm mismatch: {}", json);
    }
}
#[test]
fn test_vendor_architecture_conversions() {
    let nvidia: GpuArchitecture = NvidiaGpuArchitecture::Ampere.into();
    assert!(matches!(nvidia, GpuArchitecture::Ampere));
    let amd: GpuArchitecture = AmdGpuArchitecture::Cdna1.into();
    assert!(matches!(amd, GpuArchitecture::Cdna1));
    let intel: GpuArchitecture = IntelGpuArchitecture::Alchemist.into();
    assert!(matches!(intel, GpuArchitecture::Alchemist));
    let apple_gpu: GpuArchitecture = AppleGpuArchitecture::M3.into();
    assert!(matches!(apple_gpu, GpuArchitecture::Other(_)));
    let apple_cpu: CpuArchitecture = AppleCpuArchitecture::NeuralEngine.into();
    assert!(matches!(apple_cpu, CpuArchitecture::NeuralEngine));
}
#[test]
fn test_vendor_architecture_to_architecture() {
    let arch: Architecture = NvidiaGpuArchitecture::Hopper.into();
    assert_eq!(arch, Architecture::Hopper);
    let arch: Architecture = AmdGpuArchitecture::Rdna3.into();
    assert_eq!(arch, Architecture::Rdna3);
    let arch: Architecture = AppleGpuArchitecture::M4.into();
    assert!(matches!(arch, Architecture::Other(_)));
}
#[test]
fn test_vendor_architecture_deserialize() {
    let nvidia_tests = [
        (r#""Ampere""#, NvidiaGpuArchitecture::Ampere),
        (r#""Hopper""#, NvidiaGpuArchitecture::Hopper),
        (r#""Blackwell""#, NvidiaGpuArchitecture::Blackwell),
    ];
    for (json, expected) in nvidia_tests {
        let actual: NvidiaGpuArchitecture = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "NvidiaGpuArchitecture mismatch: {}", json);
    }
    let amd_gpu_tests = [(r#""Cdna1""#, AmdGpuArchitecture::Cdna1), (r#""Rdna2""#, AmdGpuArchitecture::Rdna2)];
    for (json, expected) in amd_gpu_tests {
        let actual: AmdGpuArchitecture = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "AmdGpuArchitecture mismatch: {}", json);
    }
    let intel_gpu_tests = [
        (r#""Alchemist""#, IntelGpuArchitecture::Alchemist),
        (r#""Xe2""#, IntelGpuArchitecture::Xe2),
    ];
    for (json, expected) in intel_gpu_tests {
        let actual: IntelGpuArchitecture = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "IntelGpuArchitecture mismatch: {}", json);
    }
    let apple_gpu_tests = [
        (r#""Apple GPU (M3)""#, AppleGpuArchitecture::M3),
        (r#""Apple GPU (M4)""#, AppleGpuArchitecture::M4),
    ];
    for (json, expected) in apple_gpu_tests {
        let actual: AppleGpuArchitecture = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "AppleGpuArchitecture mismatch: {}", json);
    }
    let intel_cpu_tests = [(r#""IA-64""#, IntelCpuArchitecture::Ia64)];
    for (json, expected) in intel_cpu_tests {
        let actual: IntelCpuArchitecture = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "IntelCpuArchitecture mismatch: {}", json);
    }
    let apple_cpu_tests = [(r#""Neural Engine""#, AppleCpuArchitecture::NeuralEngine)];
    for (json, expected) in apple_cpu_tests {
        let actual: AppleCpuArchitecture = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "AppleCpuArchitecture mismatch: {}", json);
    }
    let amd_cpu_tests = [(r#""x86_64""#, AmdCpuArchitecture::X86_64), (r#""X86""#, AmdCpuArchitecture::X86)];
    for (json, expected) in amd_cpu_tests {
        let actual: AmdCpuArchitecture = serde_json::from_str(json).unwrap();
        assert_eq!(actual, expected, "AmdCpuArchitecture mismatch: {}", json);
    }
}
