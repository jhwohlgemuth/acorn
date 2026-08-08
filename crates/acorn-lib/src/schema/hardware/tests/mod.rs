#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::memory::{Memory, MemoryUnit};
use super::vendor::Vendored;
use super::{
    AcceleratorArchitecture, Architecture, CpuArchitecture, DspArchitecture, FpgaArchitecture, GpuArchitecture, Paradigm, Regime, Resource,
    SensorModality, Vendor,
};
use crate::schema::research_activity::aspect::Modality as DataModality;
use crate::schema::research_activity::ResearchActivityMetadata;
use validator::Validate;

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
        ("Apple Performance Core", Architecture::ApplePerformanceCore),
        ("Apple GPU (M3)", Architecture::AppleM3),
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
fn test_architecture_vendor() {
    assert_eq!(Architecture::Ampere.vendor(), Some(Vendor::Nvidia));
    assert_eq!(Architecture::Hopper.vendor(), Some(Vendor::Nvidia));
    assert_eq!(Architecture::Cdna3.vendor(), Some(Vendor::AMD));
    assert_eq!(Architecture::Versal.vendor(), Some(Vendor::AMD));
    assert_eq!(Architecture::Alchemist.vendor(), Some(Vendor::Intel));
    assert_eq!(Architecture::Xe2.vendor(), Some(Vendor::Intel));
    assert_eq!(Architecture::AppleM3.vendor(), Some(Vendor::Apple));
    assert_eq!(Architecture::NeuralEngine.vendor(), Some(Vendor::Apple));
    assert_eq!(Architecture::X86_64.vendor(), None);
    assert_eq!(Architecture::RiscV.vendor(), None);
    assert_eq!(Architecture::Other("foo".to_string()).vendor(), None);
    assert_eq!(Architecture::AArch64.vendor(), Some(Vendor::ARM));
    assert_eq!(Architecture::TpuV4.vendor(), Some(Vendor::Google));
    assert_eq!(Architecture::Inferentia.vendor(), Some(Vendor::Amazon));
    // Accelerator architecture
    assert_eq!(AcceleratorArchitecture::TpuV4.vendor(), Some(Vendor::Google));
    assert_eq!(AcceleratorArchitecture::Ascend910.vendor(), Some(Vendor::Huawei));
    assert_eq!(AcceleratorArchitecture::Trainium.vendor(), Some(Vendor::Amazon));
    assert_eq!(AcceleratorArchitecture::Lpu.vendor(), Some(Vendor::Groq));
    assert_eq!(AcceleratorArchitecture::Wse2.vendor(), Some(Vendor::Other("Cerebras".to_string())));
    // CPU architecture
    assert_eq!(CpuArchitecture::X86_64.vendor(), None);
    assert_eq!(CpuArchitecture::AArch64.vendor(), Some(Vendor::ARM));
    assert_eq!(CpuArchitecture::NeuralEngine.vendor(), Some(Vendor::Apple));
    assert_eq!(CpuArchitecture::PerformanceCore.vendor(), Some(Vendor::Apple));
    assert_eq!(CpuArchitecture::Ia64.vendor(), Some(Vendor::Intel));
    assert_eq!(CpuArchitecture::PowerPc.vendor(), Some(Vendor::IBM));
    assert_eq!(CpuArchitecture::Hexagon.vendor(), Some(Vendor::Qualcomm));
    assert_eq!(CpuArchitecture::RiscV.vendor(), None);
    // DSP architecture
    assert_eq!(DspArchitecture::C66x.vendor(), Some(Vendor::TexasInstruments));
    assert_eq!(DspArchitecture::Xtensa.vendor(), Some(Vendor::Other("Cadence".to_string())));
    // GPU architecture
    assert_eq!(GpuArchitecture::Ampere.vendor(), Some(Vendor::Nvidia));
    assert_eq!(GpuArchitecture::Cdna2.vendor(), Some(Vendor::AMD));
    assert_eq!(GpuArchitecture::Alchemist.vendor(), Some(Vendor::Intel));
    assert_eq!(GpuArchitecture::AppleM1.vendor(), Some(Vendor::Apple));
    assert_eq!(GpuArchitecture::AppleM4.vendor(), Some(Vendor::Apple));
    assert_eq!(GpuArchitecture::Other("custom".to_string()).vendor(), None);
    // FPGA architecture
    assert_eq!(FpgaArchitecture::Agilex.vendor(), Some(Vendor::Intel));
    assert_eq!(FpgaArchitecture::Versal.vendor(), Some(Vendor::AMD));
    assert_eq!(FpgaArchitecture::CrossLink.vendor(), Some(Vendor::Other("Lattice".to_string())));
}
#[test]
fn test_resource_vendor() {
    let gpu = Resource::GPU {
        architecture: Some(GpuArchitecture::Hopper),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: None,
    };
    assert_eq!(gpu.vendor(), Some(Vendor::Nvidia));
    let cpu = Resource::CPU {
        architecture: Some(CpuArchitecture::X86_64),
        cores: None,
        count: Some(1),
        memory: None,
        required: None,
        threads: None,
        vendor: Some(Vendor::Intel),
    };
    assert_eq!(cpu.vendor(), Some(Vendor::Intel));
    let quantum = Resource::Quantum {
        count: Some(1),
        model: Some(super::Model::Heron),
        paradigm: None,
        required: None,
        qubits: None,
        topology: None,
        vendor: None,
    };
    assert_eq!(quantum.vendor(), Some(Vendor::IBM));
    let other = Resource::Other("custom".to_string());
    assert_eq!(other.vendor(), None);
}
#[test]
fn test_hardware_resource_dpu_from_string() {
    let data: Resource = serde_json::from_str("\"dpu\"").unwrap();
    match data {
        | Resource::DPU {
            architecture,
            count,
            memory,
            required,
            vendor,
        } => {
            assert!(architecture.is_none());
            assert_eq!(count, Some(1));
            assert!(memory.is_none());
            assert!(required.is_none());
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
            required,
            vendor,
        } => {
            assert_eq!(architecture, Some(CpuArchitecture::X86_64));
            assert_eq!(count, Some(2));
            assert_eq!(memory, Some(Memory::gb(64)));
            assert!(required.is_none());
            match vendor {
                | Some(Vendor::Intel) => {}
                | other => panic!("expected Intel vendor, got {other:?}"),
            }
        }
        | other => panic!("expected DPU resource, got {other:?}"),
    }
}
#[test]
fn test_hardware_resource_sensor_from_string() {
    let data: Resource = serde_json::from_str("\"sensor\"").unwrap();
    match data {
        | Resource::Sensor {
            count,
            modality,
            model,
            quantum_regime,
            sampling_rate: sampling_rate_hz,
            required,
            vendor,
        } => {
            assert_eq!(count, Some(1));
            assert!(modality.is_none());
            assert!(model.is_none());
            assert!(quantum_regime.is_none());
            assert!(sampling_rate_hz.is_none());
            assert!(required.is_none());
            assert!(vendor.is_none());
        }
        | other => panic!("expected Sensor resource, got {other:?}"),
    }
}
#[test]
fn test_hardware_resource_sensor_from_map() {
    let data: Resource = serde_json::from_str(
        r#"{"Sensor":{"count":3,"modality":["lidar"],"model":"VLP-16","samplingRateHz":10.0,"required":true,"vendor":"Intel"}}"#,
    )
    .unwrap();
    match data {
        | Resource::Sensor {
            count,
            modality,
            model,
            quantum_regime,
            sampling_rate: sampling_rate_hz,
            required,
            vendor,
        } => {
            assert_eq!(count, Some(3));
            assert_eq!(modality, Some(vec![SensorModality::Lidar]));
            assert_eq!(model, Some("VLP-16".to_string()));
            assert!(quantum_regime.is_none());
            assert_eq!(sampling_rate_hz, Some(10.0));
            assert_eq!(required, Some(true));
            assert_eq!(vendor, Some(Vendor::Intel));
        }
        | other => panic!("expected Sensor resource, got {other:?}"),
    }
}
#[test]
fn test_hardware_resource_sensor_from_map_with_quantum_regime() {
    let data: Resource =
        serde_json::from_str(r#"{"Sensor":{"count":1,"modality":["lidar"],"quantumRegime":"deep","model":"Q-Sensor","required":true}}"#).unwrap();
    match data {
        | Resource::Sensor {
            modality, quantum_regime, ..
        } => {
            assert_eq!(modality, Some(vec![SensorModality::Lidar]));
            assert_eq!(quantum_regime, Some(Regime::Deep));
        }
        | other => panic!("expected Sensor resource, got {other:?}"),
    }
}
#[test]
fn test_sensor_modality_bridge_to_data_modality() {
    let audio: DataModality = SensorModality::Audio.into();
    assert!(matches!(audio, DataModality::Audio));
    let image: DataModality = SensorModality::Image.into();
    assert!(matches!(image, DataModality::Video));
    let lidar: DataModality = SensorModality::Lidar.into();
    assert!(matches!(lidar, DataModality::Signal));
    let other: DataModality = SensorModality::Other("custom".to_string()).into();
    assert!(matches!(other, DataModality::Signal));
}
#[test]
fn test_memory_deserialize() {
    let mem: Memory = serde_json::from_str("\"200GB\"").unwrap();
    assert_eq!(mem, Memory::gb(200));
    let mem: Memory = serde_json::from_str("\"2TB\"").unwrap();
    assert_eq!(mem, Memory::tb(2));
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
    let mem: Memory = serde_json::from_str("\"1.5TB\"").unwrap();
    assert!((mem.amount - 1.5).abs() < f64::EPSILON);
    assert_eq!(mem.unit, MemoryUnit::TB);
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

    let mem = Memory::tb(3.75);
    let json = serde_json::to_string(&mem).unwrap();
    assert_eq!(json, "\"3.75TB\"");
    let parsed: Memory = serde_json::from_str(&json).unwrap();
    assert!((parsed.amount - 3.75).abs() < f64::EPSILON);
    assert_eq!(parsed.unit, MemoryUnit::TB);
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
fn test_resource_validate_vendor() {
    let gpu = Resource::GPU {
        architecture: Some(GpuArchitecture::Hopper),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: Some(Vendor::Nvidia),
    };
    assert!(gpu.validate().is_ok());
    let cpu = Resource::CPU {
        architecture: Some(CpuArchitecture::X86_64),
        cores: None,
        count: Some(1),
        memory: None,
        required: None,
        threads: None,
        vendor: Some(Vendor::Intel),
    };
    assert!(cpu.validate().is_ok());
    let fpga = Resource::FPGA {
        architecture: Some(FpgaArchitecture::Versal),
        count: Some(1),
        logic_elements: None,
        memory: None,
        required: None,
        vendor: Some(Vendor::AMD),
    };
    assert!(fpga.validate().is_ok());
    let gpu = Resource::GPU {
        architecture: Some(GpuArchitecture::Hopper),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: Some(Vendor::AMD),
    };
    let result = gpu.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.errors().len(), 1);
    assert!(errors.errors().contains_key("vendor"));
    if let Some(validator::ValidationErrorsKind::Field(field_errors)) = errors.errors().get("vendor") {
        assert_eq!(field_errors.len(), 1);
        assert_eq!(field_errors[0].code, "vendor_mismatch");
    } else {
        panic!("expected Field errors for 'vendor'");
    }
    let gpu = Resource::GPU {
        architecture: Some(GpuArchitecture::Hopper),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: None,
    };
    assert!(gpu.validate().is_ok());
    let gpu = Resource::GPU {
        architecture: None,
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: Some(Vendor::Nvidia),
    };
    assert!(gpu.validate().is_ok());
    let cpu = Resource::CPU {
        architecture: Some(CpuArchitecture::X86_64),
        cores: None,
        count: Some(1),
        memory: None,
        required: None,
        threads: None,
        vendor: Some(Vendor::AMD),
    };
    assert!(cpu.validate().is_ok());
    let cpu = Resource::CPU {
        architecture: Some(CpuArchitecture::X86_64),
        cores: None,
        count: Some(1),
        memory: None,
        required: None,
        threads: None,
        vendor: Some(Vendor::Intel),
    };
    assert!(cpu.validate().is_ok());
}
#[test]
fn test_compute_value_blackwell() {
    let spark = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("DGX Spark".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(spark.compute_value(), Some((12, 1)));
    let spark_gb10 = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA GB10".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(spark_gb10.compute_value(), Some((12, 1)));
    let rtx5090 = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce RTX 5090".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(rtx5090.compute_value(), Some((12, 0)));
    let b200 = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA B200".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(b200.compute_value(), Some((10, 0)));
    let gb200 = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA GB200".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gb200.compute_value(), Some((10, 0)));
    let b300 = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA B300".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(b300.compute_value(), Some((10, 3)));
    let gb300 = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA GB300".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gb300.compute_value(), Some((10, 3)));
    let blackwell_no_name = Resource::GPU {
        architecture: Some(GpuArchitecture::Blackwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: None,
    };
    assert_eq!(blackwell_no_name.compute_value(), Some((12, 0)));
}
#[test]
fn test_compute_value_hopper() {
    let h100 = Resource::GPU {
        architecture: Some(GpuArchitecture::Hopper),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("H100".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(h100.compute_value(), Some((9, 0)));
    let h200 = Resource::GPU {
        architecture: Some(GpuArchitecture::Hopper),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("H200".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(h200.compute_value(), Some((9, 0)));
}
#[test]
fn test_compute_value_ada_lovelace() {
    let rtx4090 = Resource::GPU {
        architecture: Some(GpuArchitecture::AdaLovelace),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce RTX 4090".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(rtx4090.compute_value(), Some((8, 9)));
    let l40 = Resource::GPU {
        architecture: Some(GpuArchitecture::AdaLovelace),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA L40".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(l40.compute_value(), Some((8, 9)));
}
#[test]
fn test_compute_value_ampere() {
    let a100 = Resource::GPU {
        architecture: Some(GpuArchitecture::Ampere),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA A100".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(a100.compute_value(), Some((8, 0)));
    let a30 = Resource::GPU {
        architecture: Some(GpuArchitecture::Ampere),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA A30".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(a30.compute_value(), Some((8, 0)));
    let rtx3090 = Resource::GPU {
        architecture: Some(GpuArchitecture::Ampere),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce RTX 3090".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(rtx3090.compute_value(), Some((8, 6)));
    let ampere_no_name = Resource::GPU {
        architecture: Some(GpuArchitecture::Ampere),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: None,
        required: None,
        vendor: None,
    };
    assert_eq!(ampere_no_name.compute_value(), Some((8, 6)));
}
#[test]
fn test_compute_value_turing() {
    let rtx2080 = Resource::GPU {
        architecture: Some(GpuArchitecture::Turing),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce RTX 2080".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(rtx2080.compute_value(), Some((7, 5)));
}
#[test]
fn test_compute_value_new_archs() {
    let thor = Resource::GPU {
        architecture: Some(GpuArchitecture::Thor),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Jetson T5000".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(thor.compute_value(), Some((11, 0)));
    let orin = Resource::GPU {
        architecture: Some(GpuArchitecture::Orin),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Jetson AGX Orin".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(orin.compute_value(), Some((8, 7)));
}
#[test]
fn test_compute_value_volta() {
    let v100 = Resource::GPU {
        architecture: Some(GpuArchitecture::Volta),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("NVIDIA V100".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(v100.compute_value(), Some((7, 0)));
    let xavier = Resource::GPU {
        architecture: Some(GpuArchitecture::Volta),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Jetson AGX Xavier".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(xavier.compute_value(), Some((7, 2)));
}
#[test]
fn test_compute_value_pascal() {
    let gtx1080 = Resource::GPU {
        architecture: Some(GpuArchitecture::Pascal),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GTX 1080".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gtx1080.compute_value(), Some((6, 1)));
    let p100 = Resource::GPU {
        architecture: Some(GpuArchitecture::Pascal),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Tesla P100".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(p100.compute_value(), Some((6, 0)));
    let tx2 = Resource::GPU {
        architecture: Some(GpuArchitecture::Pascal),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Jetson TX2".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(tx2.compute_value(), Some((6, 2)));
}
#[test]
fn test_compute_value_maxwell() {
    let m60 = Resource::GPU {
        architecture: Some(GpuArchitecture::Maxwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Tesla M60".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(m60.compute_value(), Some((5, 2)));
    let nano = Resource::GPU {
        architecture: Some(GpuArchitecture::Maxwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Jetson Nano".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(nano.compute_value(), Some((5, 3)));
    let gtx750ti = Resource::GPU {
        architecture: Some(GpuArchitecture::Maxwell),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GTX 750 Ti".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gtx750ti.compute_value(), Some((5, 0)));
}
#[test]
fn test_compute_value_kepler() {
    let k80 = Resource::GPU {
        architecture: Some(GpuArchitecture::Kepler),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Tesla K80".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(k80.compute_value(), Some((3, 7)));
    let k40 = Resource::GPU {
        architecture: Some(GpuArchitecture::Kepler),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Tesla K40".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(k40.compute_value(), Some((3, 5)));
    let gtx680 = Resource::GPU {
        architecture: Some(GpuArchitecture::Kepler),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GTX 680".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gtx680.compute_value(), Some((3, 0)));
}
#[test]
fn test_compute_value_fermi() {
    let gtx580 = Resource::GPU {
        architecture: Some(GpuArchitecture::Fermi),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GTX 580".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gtx580.compute_value(), Some((2, 0)));
    let gtx560ti = Resource::GPU {
        architecture: Some(GpuArchitecture::Fermi),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GTX 560 Ti".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gtx560ti.compute_value(), Some((2, 1)));
}
#[test]
fn test_compute_value_tesla() {
    let gtx285 = Resource::GPU {
        architecture: Some(GpuArchitecture::Tesla),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GTX 285".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gtx285.compute_value(), Some((1, 3)));
    let gt240 = Resource::GPU {
        architecture: Some(GpuArchitecture::Tesla),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GT 240".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gt240.compute_value(), Some((1, 2)));
    let gtx260 = Resource::GPU {
        architecture: Some(GpuArchitecture::Tesla),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce GTX 260".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(gtx260.compute_value(), Some((1, 3)));
    let c1060 = Resource::GPU {
        architecture: Some(GpuArchitecture::Tesla),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Tesla C1060".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(c1060.compute_value(), Some((1, 3)));
    let ione = Resource::GPU {
        architecture: Some(GpuArchitecture::Tesla),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("GeForce 8800 GT".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(ione.compute_value(), Some((1, 1)));
}
#[test]
fn test_compute_value_non_nvidia() {
    let amd = Resource::GPU {
        architecture: Some(GpuArchitecture::Rdna3),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("AMD RX 7900 XTX".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(amd.compute_value(), None);
    let intel = Resource::GPU {
        architecture: Some(GpuArchitecture::Alchemist),
        backend: None,
        compute_capability: None,
        count: Some(1),
        memory: None,
        name: Some("Intel Arc A770".to_string()),
        required: None,
        vendor: None,
    };
    assert_eq!(intel.compute_value(), None);
    let non_gpu = Resource::CPU {
        architecture: Some(CpuArchitecture::X86_64),
        cores: Some(8),
        count: Some(1),
        memory: None,
        required: None,
        threads: None,
        vendor: Some(Vendor::Intel),
    };
    assert_eq!(non_gpu.compute_value(), None);
}
