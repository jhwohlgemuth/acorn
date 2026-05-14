//! Vendor-specific hardware architecture types and implementations
use crate::schema::hardware::{Architecture, CpuArchitecture, GpuArchitecture};
/// AMD-specific CPU and GPU architectures
pub mod amd;
/// Apple-specific CPU and GPU architectures
pub mod apple;
/// Intel-specific CPU, FPGA, and GPU architectures
pub mod intel;
/// NVIDIA-specific GPU architectures
pub mod nvidia;

pub use amd::{AmdCpuArchitecture, AmdGpuArchitecture};
pub use apple::{AppleCpuArchitecture, AppleGpuArchitecture};
pub use intel::{IntelCpuArchitecture, IntelFpgaArchitecture, IntelGpuArchitecture};
pub use nvidia::NvidiaGpuArchitecture;

impl From<AmdCpuArchitecture> for Architecture {
    fn from(value: AmdCpuArchitecture) -> Self {
        match value {
            | AmdCpuArchitecture::X86 => Architecture::X86,
            | AmdCpuArchitecture::X86_64 => Architecture::X86_64,
            | AmdCpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<AmdGpuArchitecture> for Architecture {
    fn from(value: AmdGpuArchitecture) -> Self {
        match value {
            | AmdGpuArchitecture::Cdna1 => Architecture::Cdna1,
            | AmdGpuArchitecture::Cdna2 => Architecture::Cdna2,
            | AmdGpuArchitecture::Cdna3 => Architecture::Cdna3,
            | AmdGpuArchitecture::Cdna4 => Architecture::Cdna4,
            | AmdGpuArchitecture::Rdna1 => Architecture::Rdna1,
            | AmdGpuArchitecture::Rdna2 => Architecture::Rdna2,
            | AmdGpuArchitecture::Rdna3 => Architecture::Rdna3,
            | AmdGpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<AppleCpuArchitecture> for Architecture {
    fn from(value: AppleCpuArchitecture) -> Self {
        match value {
            | AppleCpuArchitecture::NeuralEngine => Architecture::NeuralEngine,
            | AppleCpuArchitecture::PerformanceCore => Architecture::Other("Apple Performance Core".to_string()),
            | AppleCpuArchitecture::EfficiencyCore => Architecture::Other("Apple Efficiency Core".to_string()),
            | AppleCpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<AppleGpuArchitecture> for Architecture {
    fn from(value: AppleGpuArchitecture) -> Self {
        match value {
            | AppleGpuArchitecture::A14 => Architecture::Other("Apple GPU (A14/A15)".to_string()),
            | AppleGpuArchitecture::A16 => Architecture::Other("Apple GPU (A16/A17)".to_string()),
            | AppleGpuArchitecture::A18 => Architecture::Other("Apple GPU (A18)".to_string()),
            | AppleGpuArchitecture::M1 => Architecture::Other("Apple GPU (M1)".to_string()),
            | AppleGpuArchitecture::M2 => Architecture::Other("Apple GPU (M2)".to_string()),
            | AppleGpuArchitecture::M3 => Architecture::Other("Apple GPU (M3)".to_string()),
            | AppleGpuArchitecture::M4 => Architecture::Other("Apple GPU (M4)".to_string()),
            | AppleGpuArchitecture::M5 => Architecture::Other("Apple GPU (M5)".to_string()),
            | AppleGpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<IntelCpuArchitecture> for Architecture {
    fn from(value: IntelCpuArchitecture) -> Self {
        match value {
            | IntelCpuArchitecture::Ia64 => Architecture::Ia64,
            | IntelCpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<IntelFpgaArchitecture> for Architecture {
    fn from(value: IntelFpgaArchitecture) -> Self {
        match value {
            | IntelFpgaArchitecture::Agilex => Architecture::Agilex,
            | IntelFpgaArchitecture::CycloneV => Architecture::CycloneV,
            | IntelFpgaArchitecture::StratixV => Architecture::StratixV,
            | IntelFpgaArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<IntelGpuArchitecture> for Architecture {
    fn from(value: IntelGpuArchitecture) -> Self {
        match value {
            | IntelGpuArchitecture::Alchemist => Architecture::Alchemist,
            | IntelGpuArchitecture::Xe => Architecture::Xe,
            | IntelGpuArchitecture::Xe2 => Architecture::Xe2,
            | IntelGpuArchitecture::Xe3 => Architecture::Xe3,
            | IntelGpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<NvidiaGpuArchitecture> for Architecture {
    fn from(value: NvidiaGpuArchitecture) -> Self {
        match value {
            | NvidiaGpuArchitecture::AdaLovelace => Architecture::AdaLovelace,
            | NvidiaGpuArchitecture::Ampere => Architecture::Ampere,
            | NvidiaGpuArchitecture::Blackwell => Architecture::Blackwell,
            | NvidiaGpuArchitecture::Hopper => Architecture::Hopper,
            | NvidiaGpuArchitecture::Turing => Architecture::Turing,
            | NvidiaGpuArchitecture::Volta => Architecture::Volta,
            | NvidiaGpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<AppleCpuArchitecture> for CpuArchitecture {
    fn from(value: AppleCpuArchitecture) -> Self {
        match value {
            | AppleCpuArchitecture::NeuralEngine => CpuArchitecture::NeuralEngine,
            | AppleCpuArchitecture::PerformanceCore => CpuArchitecture::Other("Apple Performance Core".to_string()),
            | AppleCpuArchitecture::EfficiencyCore => CpuArchitecture::Other("Apple Efficiency Core".to_string()),
            | AppleCpuArchitecture::Other(s) => CpuArchitecture::Other(s),
        }
    }
}
impl From<AmdGpuArchitecture> for GpuArchitecture {
    fn from(value: AmdGpuArchitecture) -> Self {
        match value {
            | AmdGpuArchitecture::Cdna1 => GpuArchitecture::Cdna1,
            | AmdGpuArchitecture::Cdna2 => GpuArchitecture::Cdna2,
            | AmdGpuArchitecture::Cdna3 => GpuArchitecture::Cdna3,
            | AmdGpuArchitecture::Cdna4 => GpuArchitecture::Cdna4,
            | AmdGpuArchitecture::Rdna1 => GpuArchitecture::Rdna1,
            | AmdGpuArchitecture::Rdna2 => GpuArchitecture::Rdna2,
            | AmdGpuArchitecture::Rdna3 => GpuArchitecture::Rdna3,
            | AmdGpuArchitecture::Other(s) => GpuArchitecture::Other(s),
        }
    }
}
impl From<AppleGpuArchitecture> for GpuArchitecture {
    fn from(value: AppleGpuArchitecture) -> Self {
        match value {
            | AppleGpuArchitecture::A14 => GpuArchitecture::Other("Apple GPU (A14/A15)".to_string()),
            | AppleGpuArchitecture::A16 => GpuArchitecture::Other("Apple GPU (A16/A17)".to_string()),
            | AppleGpuArchitecture::A18 => GpuArchitecture::Other("Apple GPU (A18)".to_string()),
            | AppleGpuArchitecture::M1 => GpuArchitecture::Other("Apple GPU (M1)".to_string()),
            | AppleGpuArchitecture::M2 => GpuArchitecture::Other("Apple GPU (M2)".to_string()),
            | AppleGpuArchitecture::M3 => GpuArchitecture::Other("Apple GPU (M3)".to_string()),
            | AppleGpuArchitecture::M4 => GpuArchitecture::Other("Apple GPU (M4)".to_string()),
            | AppleGpuArchitecture::M5 => GpuArchitecture::Other("Apple GPU (M5)".to_string()),
            | AppleGpuArchitecture::Other(s) => GpuArchitecture::Other(s),
        }
    }
}
impl From<IntelGpuArchitecture> for GpuArchitecture {
    fn from(value: IntelGpuArchitecture) -> Self {
        match value {
            | IntelGpuArchitecture::Alchemist => GpuArchitecture::Alchemist,
            | IntelGpuArchitecture::Xe => GpuArchitecture::Xe,
            | IntelGpuArchitecture::Xe2 => GpuArchitecture::Xe2,
            | IntelGpuArchitecture::Xe3 => GpuArchitecture::Xe3,
            | IntelGpuArchitecture::Other(s) => GpuArchitecture::Other(s),
        }
    }
}
impl From<NvidiaGpuArchitecture> for GpuArchitecture {
    fn from(value: NvidiaGpuArchitecture) -> Self {
        match value {
            | NvidiaGpuArchitecture::AdaLovelace => GpuArchitecture::AdaLovelace,
            | NvidiaGpuArchitecture::Ampere => GpuArchitecture::Ampere,
            | NvidiaGpuArchitecture::Blackwell => GpuArchitecture::Blackwell,
            | NvidiaGpuArchitecture::Hopper => GpuArchitecture::Hopper,
            | NvidiaGpuArchitecture::Turing => GpuArchitecture::Turing,
            | NvidiaGpuArchitecture::Volta => GpuArchitecture::Volta,
            | NvidiaGpuArchitecture::Other(s) => GpuArchitecture::Other(s),
        }
    }
}
