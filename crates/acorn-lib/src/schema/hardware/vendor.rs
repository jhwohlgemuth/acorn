//! Hardware vendor types, the [`Vendored`] trait, and vendor validation
use super::{AcceleratorArchitecture, Architecture, CpuArchitecture, DspArchitecture, FpgaArchitecture, GpuArchitecture, Model, Resource};
use crate::prelude::*;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{ValidationError, ValidationErrors};

/// Trait for types that can report their associated hardware vendor.
pub trait Vendored {
    /// Returns architecture
    fn architecture(&self) -> Option<Architecture> {
        None
    }
    /// Returns the vendor associated with this architecture, if one exists.
    ///
    /// Multi-vendor architectures (e.g., x86, RISC-V) and `Other` variants return `None`.
    fn vendor(&self) -> Option<Vendor>;
}
/// Hardware vendor or manufacturer
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum Vendor {
    /// Advanced Micro Devices
    #[display("AMD")]
    AMD,
    /// Amazon Web Services
    #[display("Amazon")]
    Amazon,
    /// Apple Inc.
    #[display("Apple")]
    Apple,
    /// Arm Holdings
    #[display("ARM")]
    ARM,
    /// BrainChip Holdings (neuromorphic AI)
    #[display("BrainChip")]
    BrainChip,
    /// Broadcom Inc.
    #[display("Broadcom")]
    Broadcom,
    /// Bosch (sensors and embedded systems)
    #[display("Bosch")]
    Bosch,
    /// D-Wave Systems (quantum annealing)
    #[display("D-Wave")]
    DWave,
    /// FLIR Systems (thermal and imaging sensors)
    #[display("FLIR")]
    FLIR,
    /// Google LLC
    #[display("Google")]
    Google,
    /// Groq Inc.
    #[display("Groq")]
    Groq,
    /// Huawei Technologies
    #[display("Huawei")]
    Huawei,
    /// International Business Machines Corporation
    #[display("IBM")]
    IBM,
    /// Intel Corporation
    #[display("Intel")]
    Intel,
    /// IonQ Inc. (trapped ion quantum computing)
    #[display("IonQ")]
    IonQ,
    /// MediaTek Inc.
    #[display("MediaTek")]
    MediaTek,
    /// Microsoft Corporation
    #[display("Microsoft")]
    Microsoft,
    /// NVIDIA Corporation
    #[display("NVIDIA")]
    #[serde(alias = "NVIDIA")]
    Nvidia,
    /// Qualcomm Incorporated
    #[display("Qualcomm")]
    Qualcomm,
    /// Quantinuum (trapped ion quantum computing, Honeywell spin-off)
    #[display("Quantinuum")]
    Quantinuum,
    /// Rigetti Computing (superconducting gate-based quantum)
    #[display("Rigetti")]
    Rigetti,
    /// Samsung Electronics
    #[display("Samsung")]
    Samsung,
    /// Tenstorrent Inc.
    #[display("Tenstorrent")]
    Tenstorrent,
    /// Texas Instruments
    #[display("Texas Instruments")]
    TexasInstruments,
    /// Velodyne Lidar (LiDAR sensors)
    #[display("Velodyne")]
    Velodyne,
    /// Other or unspecified vendor
    #[display("{}", _0)]
    Other(String),
}
impl Vendored for AcceleratorArchitecture {
    fn vendor(&self) -> Option<Vendor> {
        match self {
            | AcceleratorArchitecture::Ascend910 => Some(Vendor::Huawei),
            | AcceleratorArchitecture::Blackhole | AcceleratorArchitecture::Wormhole => Some(Vendor::Tenstorrent),
            | AcceleratorArchitecture::Gaudi2 | AcceleratorArchitecture::Nervana => Some(Vendor::Intel),
            | AcceleratorArchitecture::GraphcoreGc200 => Some(Vendor::Other("Graphcore".to_string())),
            | AcceleratorArchitecture::Inferentia | AcceleratorArchitecture::Trainium => Some(Vendor::Amazon),
            | AcceleratorArchitecture::Lpu => Some(Vendor::Groq),
            | AcceleratorArchitecture::Maia => Some(Vendor::Microsoft),
            | AcceleratorArchitecture::SambaNovaRdu => Some(Vendor::Other("SambaNova".to_string())),
            | AcceleratorArchitecture::TpuV4
            | AcceleratorArchitecture::TpuV5e
            | AcceleratorArchitecture::TpuV5p
            | AcceleratorArchitecture::TpuV6e => Some(Vendor::Google),
            | AcceleratorArchitecture::Wse2 | AcceleratorArchitecture::Wse3 => Some(Vendor::Other("Cerebras".to_string())),
            | AcceleratorArchitecture::Other(_) => None,
        }
    }
}
impl Vendored for Architecture {
    fn vendor(&self) -> Option<Vendor> {
        match self {
            | Architecture::Cdna1
            | Architecture::Cdna2
            | Architecture::Cdna3
            | Architecture::Cdna4
            | Architecture::Rdna1
            | Architecture::Rdna2
            | Architecture::Rdna3
            | Architecture::Versal
            | Architecture::VirtexUltraScalePlus
            | Architecture::Zynq
            | Architecture::Artix
            | Architecture::Kintex
            | Architecture::Spartan => Some(Vendor::AMD),
            | Architecture::NeuralEngine
            | Architecture::ApplePerformanceCore
            | Architecture::AppleEfficiencyCore
            | Architecture::AppleA14
            | Architecture::AppleA16
            | Architecture::AppleA18
            | Architecture::AppleM1
            | Architecture::AppleM2
            | Architecture::AppleM3
            | Architecture::AppleM4
            | Architecture::AppleM5 => Some(Vendor::Apple),
            | Architecture::Ia64
            | Architecture::Alchemist
            | Architecture::Xe
            | Architecture::Xe2
            | Architecture::Xe3
            | Architecture::Agilex
            | Architecture::CycloneV
            | Architecture::StratixV
            | Architecture::Nervana
            | Architecture::Gaudi2
            | Architecture::Stratix10 => Some(Vendor::Intel),
            | Architecture::AdaLovelace
            | Architecture::Ampere
            | Architecture::Blackwell
            | Architecture::Carmel
            | Architecture::Fermi
            | Architecture::Grace
            | Architecture::Hopper
            | Architecture::Kepler
            | Architecture::Maxwell
            | Architecture::Orin
            | Architecture::Pascal
            | Architecture::Tesla
            | Architecture::Thor
            | Architecture::Turing
            | Architecture::Volta
            | Architecture::Vera => Some(Vendor::Nvidia),
            | Architecture::AArch64 | Architecture::Ethos => Some(Vendor::ARM),
            | Architecture::TpuV4 | Architecture::TpuV5e | Architecture::TpuV5p | Architecture::TpuV6e => Some(Vendor::Google),
            | Architecture::Inferentia | Architecture::Trainium => Some(Vendor::Amazon),
            | Architecture::Ascend910 => Some(Vendor::Huawei),
            | Architecture::C66x => Some(Vendor::TexasInstruments),
            | Architecture::Lpu => Some(Vendor::Groq),
            | Architecture::Maia => Some(Vendor::Microsoft),
            | Architecture::Hexagon => Some(Vendor::Qualcomm),
            | Architecture::PowerPc | Architecture::ZArch => Some(Vendor::IBM),
            | Architecture::Blackhole | Architecture::Wormhole => Some(Vendor::Tenstorrent),
            | Architecture::Wse2 | Architecture::Wse3 => Some(Vendor::Other("Cerebras".to_string())),
            | Architecture::LoongArch => Some(Vendor::Other("Loongson".to_string())),
            | Architecture::Mips => Some(Vendor::Other("MIPS".to_string())),
            | Architecture::SambaNovaRdu => Some(Vendor::Other("SambaNova".to_string())),
            | Architecture::GraphcoreGc200 => Some(Vendor::Other("Graphcore".to_string())),
            | Architecture::CrossLink | Architecture::Ecp5 | Architecture::Ice40 => Some(Vendor::Other("Lattice".to_string())),
            | Architecture::Xtensa => Some(Vendor::Other("Cadence".to_string())),
            | Architecture::X86 | Architecture::X86_64 | Architecture::RiscV | Architecture::Other(_) => None,
        }
    }
}
impl Vendored for CpuArchitecture {
    fn vendor(&self) -> Option<Vendor> {
        match self {
            | CpuArchitecture::AArch64 | CpuArchitecture::Ethos => Some(Vendor::ARM),
            | CpuArchitecture::Carmel | CpuArchitecture::Grace | CpuArchitecture::Vera => Some(Vendor::Nvidia),
            | CpuArchitecture::Ia64 => Some(Vendor::Intel),
            | CpuArchitecture::NeuralEngine | CpuArchitecture::PerformanceCore | CpuArchitecture::EfficiencyCore => Some(Vendor::Apple),
            | CpuArchitecture::PowerPc | CpuArchitecture::ZArch => Some(Vendor::IBM),
            | CpuArchitecture::Hexagon => Some(Vendor::Qualcomm),
            | CpuArchitecture::LoongArch => Some(Vendor::Other("Loongson".to_string())),
            | CpuArchitecture::Mips => Some(Vendor::Other("MIPS".to_string())),
            | CpuArchitecture::X86 | CpuArchitecture::X86_64 | CpuArchitecture::RiscV | CpuArchitecture::Other(_) => None,
        }
    }
}
impl Vendored for DspArchitecture {
    fn vendor(&self) -> Option<Vendor> {
        match self {
            | DspArchitecture::C66x => Some(Vendor::TexasInstruments),
            | DspArchitecture::Xtensa => Some(Vendor::Other("Cadence".to_string())),
            | DspArchitecture::Other(_) => None,
        }
    }
}
impl Vendored for FpgaArchitecture {
    fn vendor(&self) -> Option<Vendor> {
        match self {
            | FpgaArchitecture::Agilex | FpgaArchitecture::CycloneV | FpgaArchitecture::StratixV | FpgaArchitecture::Stratix10 => Some(Vendor::Intel),
            | FpgaArchitecture::Artix
            | FpgaArchitecture::Kintex
            | FpgaArchitecture::Spartan
            | FpgaArchitecture::Versal
            | FpgaArchitecture::VirtexUltraScalePlus
            | FpgaArchitecture::Zynq => Some(Vendor::AMD),
            | FpgaArchitecture::CrossLink | FpgaArchitecture::Ecp5 | FpgaArchitecture::Ice40 => Some(Vendor::Other("Lattice".to_string())),
            | FpgaArchitecture::Other(_) => None,
        }
    }
}
impl Vendored for GpuArchitecture {
    fn vendor(&self) -> Option<Vendor> {
        match self {
            | GpuArchitecture::AdaLovelace
            | GpuArchitecture::Ampere
            | GpuArchitecture::Blackwell
            | GpuArchitecture::Fermi
            | GpuArchitecture::Hopper
            | GpuArchitecture::Kepler
            | GpuArchitecture::Maxwell
            | GpuArchitecture::Orin
            | GpuArchitecture::Pascal
            | GpuArchitecture::Tesla
            | GpuArchitecture::Thor
            | GpuArchitecture::Turing
            | GpuArchitecture::Volta => Some(Vendor::Nvidia),
            | GpuArchitecture::Cdna1
            | GpuArchitecture::Cdna2
            | GpuArchitecture::Cdna3
            | GpuArchitecture::Cdna4
            | GpuArchitecture::Rdna1
            | GpuArchitecture::Rdna2
            | GpuArchitecture::Rdna3 => Some(Vendor::AMD),
            | GpuArchitecture::Alchemist | GpuArchitecture::Xe | GpuArchitecture::Xe2 | GpuArchitecture::Xe3 => Some(Vendor::Intel),
            | GpuArchitecture::AppleA14
            | GpuArchitecture::AppleA16
            | GpuArchitecture::AppleA18
            | GpuArchitecture::AppleM1
            | GpuArchitecture::AppleM2
            | GpuArchitecture::AppleM3
            | GpuArchitecture::AppleM4
            | GpuArchitecture::AppleM5 => Some(Vendor::Apple),
            | GpuArchitecture::Other(_) => None,
        }
    }
}
impl Vendored for Model {
    fn vendor(&self) -> Option<Vendor> {
        match self {
            | Model::Advantage | Model::Advantage2 | Model::TwoThousandQ => Some(Vendor::DWave),
            | Model::Ankaa | Model::Aspen => Some(Vendor::Rigetti),
            | Model::Bristlecone | Model::Sycamore | Model::Willow => Some(Vendor::Google),
            | Model::Condor | Model::Eagle | Model::Falcon | Model::Flamingo | Model::Heron | Model::Hummingbird | Model::Osprey => Some(Vendor::IBM),
            | Model::H1 | Model::H2 => Some(Vendor::Quantinuum),
            | Model::IonQAria | Model::IonQForte | Model::IonQHarmony | Model::IonQTempo => Some(Vendor::IonQ),
            | Model::Other(_) => None,
        }
    }
}
impl Vendored for Resource {
    fn architecture(&self) -> Option<Architecture> {
        match self {
            | Resource::ASIC { architecture, .. } | Resource::CPU { architecture, .. } | Resource::DPU { architecture, .. } => {
                architecture.as_ref().map(Architecture::from)
            }
            | Resource::DSP { architecture, .. } => architecture.as_ref().map(Architecture::from),
            | Resource::FPGA { architecture, .. } => architecture.as_ref().map(Architecture::from),
            | Resource::GPU { architecture, .. } => architecture.as_ref().map(Architecture::from),
            | Resource::NPU { architecture, .. } | Resource::TPU { architecture, .. } => architecture.as_ref().map(Architecture::from),
            | Resource::Neuromorphic { .. } | Resource::Quantum { .. } | Resource::Sensor { .. } | Resource::Other(_) => None,
        }
    }
    fn vendor(&self) -> Option<Vendor> {
        let explicit_vendor = || match self {
            | Resource::ASIC { vendor, .. }
            | Resource::CPU { vendor, .. }
            | Resource::DPU { vendor, .. }
            | Resource::DSP { vendor, .. }
            | Resource::FPGA { vendor, .. }
            | Resource::GPU { vendor, .. }
            | Resource::NPU { vendor, .. }
            | Resource::Neuromorphic { vendor, .. }
            | Resource::Sensor { vendor, .. }
            | Resource::TPU { vendor, .. }
            | Resource::Quantum { vendor, .. } => vendor.clone(),
            | Resource::Other(_) => None,
        };
        match self {
            | Resource::Quantum { model, .. } => model.as_ref().and_then(Vendored::vendor).or_else(explicit_vendor),
            | _ => self
                .architecture()
                .and_then(|architecture| architecture.vendor())
                .or_else(explicit_vendor),
        }
    }
}
pub(super) fn validate_vendor(resource: &Resource) -> Result<(), ValidationErrors> {
    let explicit_vendor = match resource {
        | Resource::ASIC { vendor, .. }
        | Resource::CPU { vendor, .. }
        | Resource::DPU { vendor, .. }
        | Resource::DSP { vendor, .. }
        | Resource::FPGA { vendor, .. }
        | Resource::GPU { vendor, .. }
        | Resource::NPU { vendor, .. }
        | Resource::Neuromorphic { vendor, .. }
        | Resource::Sensor { vendor, .. }
        | Resource::TPU { vendor, .. }
        | Resource::Quantum { vendor, .. } => vendor.as_ref(),
        | Resource::Other(_) => None,
    };
    match resource {
        | Resource::Neuromorphic { .. } | Resource::Quantum { .. } | Resource::Sensor { .. } | Resource::Other(_) => Ok(()),
        | value => match (value.architecture(), explicit_vendor) {
            | (Some(arch), Some(vendor)) => match arch.vendor() {
                | Some(arch_vendor) if arch_vendor != *vendor => {
                    let msg = format!("Architecture is not produced by vendor {vendor}");
                    let error = ValidationError::new("vendor_mismatch").with_message(msg.into());
                    let mut errors = ValidationErrors::new();
                    errors.add("vendor", error);
                    Err(errors)
                }
                | _ => Ok(()),
            },
            | _ => Ok(()),
        },
    }
}
