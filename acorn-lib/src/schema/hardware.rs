//! Hardware resource, architecture, and vendor types
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Unified hardware architecture identifier
///
/// This enum provides a single type that can represent any hardware architecture
/// across all resource types (CPU, GPU, FPGA, accelerators, DSPs). Each variant
/// corresponds to one of the typed architecture enums.
#[derive(Clone, Debug, Display, PartialEq, Serialize, JsonSchema)]
pub enum Architecture {
    /// 64-bit ARM instruction set
    #[display("AArch64")]
    AArch64,
    /// NVIDIA Ada Lovelace microarchitecture (RTX 40xx)
    #[display("Ada Lovelace")]
    AdaLovelace,
    /// Intel Agilex FPGA
    #[display("Agilex")]
    Agilex,
    /// Intel Alchemist graphics architecture (Xe-HPG)
    #[display("Alchemist")]
    Alchemist,
    /// NVIDIA Ampere microarchitecture (A100, RTX 30xx)
    #[display("Ampere")]
    Ampere,
    /// Huawei Ascend 910 AI accelerator
    #[display("Ascend 910")]
    Ascend910,
    /// Tenstorrent Blackhole AI accelerator
    #[display("Blackhole")]
    Blackhole,
    /// NVIDIA Blackwell microarchitecture (RTX 50xx, B100)
    #[display("Blackwell")]
    Blackwell,
    /// Texas Instruments C66x DSP
    #[display("C66x")]
    C66x,
    /// AMD CDNA 1 microarchitecture (MI100)
    #[display("CDNA1")]
    Cdna1,
    /// AMD CDNA 2 microarchitecture (MI250)
    #[display("CDNA2")]
    Cdna2,
    /// AMD CDNA 3 microarchitecture (MI300)
    #[display("CDNA3")]
    Cdna3,
    /// Lattice CrossLink FPGA
    #[display("CrossLink")]
    CrossLink,
    /// Intel/Altera Cyclone V FPGA
    #[display("Cyclone V")]
    CycloneV,
    /// Lattice ECP5 FPGA
    #[display("ECP5")]
    Ecp5,
    /// ARM Ethos NPU
    #[display("Ethos")]
    Ethos,
    /// Intel Habana Gaudi2 AI accelerator
    #[display("Gaudi2")]
    Gaudi2,
    /// Qualcomm Hexagon processor (NPU/DSP)
    #[display("Hexagon")]
    Hexagon,
    /// NVIDIA Hopper microarchitecture (H100, H200)
    #[display("Hopper")]
    Hopper,
    /// Intel Itanium architecture (IA-64)
    #[display("IA-64")]
    Ia64,
    /// Lattice iCE40 FPGA
    #[display("iCE40")]
    Ice40,
    /// Amazon Inferentia AI inference chip
    #[display("Inferentia")]
    Inferentia,
    /// LoongArch (Chinese CPU ISA)
    #[display("LoongArch")]
    LoongArch,
    /// Groq Language Processing Unit
    #[display("LPU")]
    Lpu,
    /// Microsoft Maia AI accelerator
    #[display("Maia")]
    Maia,
    /// MIPS instruction set architecture
    #[display("MIPS")]
    Mips,
    /// Apple Neural Engine
    #[display("Neural Engine")]
    NeuralEngine,
    /// IBM PowerPC architecture
    #[display("PowerPC")]
    PowerPc,
    /// AMD RDNA 1 microarchitecture (RX 5xxx)
    #[display("RDNA1")]
    Rdna1,
    /// AMD RDNA 2 microarchitecture (RX 6xxx)
    #[display("RDNA2")]
    Rdna2,
    /// AMD RDNA 3 microarchitecture (RX 7xxx)
    #[display("RDNA3")]
    Rdna3,
    /// RISC-V open instruction set architecture
    #[display("RISC-V")]
    RiscV,
    /// Intel/Altera Stratix V FPGA
    #[display("Stratix V")]
    StratixV,
    /// Google TPU v4
    #[display("TPU v4")]
    TpuV4,
    /// Google TPU v5e
    #[display("TPU v5e")]
    TpuV5e,
    /// Google TPU v5p
    #[display("TPU v5p")]
    TpuV5p,
    /// Amazon Trainium AI accelerator
    #[display("Trainium")]
    Trainium,
    /// NVIDIA Turing microarchitecture (RTX 20xx)
    #[display("Turing")]
    Turing,
    /// AMD/Xilinx Versal adaptive compute acceleration platform
    #[display("Versal")]
    Versal,
    /// AMD/Xilinx Virtex UltraScale+ FPGA
    #[display("Virtex UltraScale+")]
    VirtexUltraScalePlus,
    /// NVIDIA Volta microarchitecture (V100)
    #[display("Volta")]
    Volta,
    /// Tenstorrent Wormhole AI accelerator
    #[display("Wormhole")]
    Wormhole,
    /// Cerebras Wafer-Scale Engine 2 (WSE-2)
    #[display("WSE-2")]
    Wse2,
    /// Cerebras Wafer-Scale Engine 3 (WSE-3)
    #[display("WSE-3")]
    Wse3,
    /// 32-bit x86 instruction set
    #[display("x86")]
    X86,
    /// x86-64 instruction set (Intel/AMD)
    #[display("x86_64")]
    X86_64,
    /// Intel Xe graphics architecture
    #[display("Xe")]
    Xe,
    /// Intel Xe2 graphics architecture (Battlemage)
    #[display("Xe2")]
    Xe2,
    /// Intel Xe3 graphics architecture (Celestial)
    #[display("Xe3")]
    Xe3,
    /// Cadence Tensilica Xtensa DSP
    #[display("Xtensa")]
    Xtensa,
    /// IBM z/Architecture mainframe
    #[display("z/Architecture")]
    ZArch,
    /// AMD/Xilinx Zynq SoC FPGA
    #[display("Zynq")]
    Zynq,
    /// Other or unspecified architecture
    #[display("{}", _0)]
    Other(String),
}
/// AI accelerator architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum AcceleratorArchitecture {
    /// Huawei Ascend 910 AI accelerator
    #[display("Ascend 910")]
    Ascend910,
    /// Tenstorrent Blackhole AI accelerator
    #[display("Blackhole")]
    Blackhole,
    /// Intel Habana Gaudi2 AI accelerator
    #[display("Gaudi2")]
    Gaudi2,
    /// Amazon Inferentia AI inference chip
    #[display("Inferentia")]
    Inferentia,
    /// Groq Language Processing Unit
    #[display("LPU")]
    Lpu,
    /// Microsoft Maia AI accelerator
    #[display("Maia")]
    Maia,
    /// Google TPU v4
    #[display("TPU v4")]
    #[serde(alias = "TPU v4", alias = "TPUv4", alias = "tpu v4", alias = "tpuv4")]
    TpuV4,
    /// Google TPU v5e
    #[display("TPU v5e")]
    #[serde(alias = "TPU v5e", alias = "TPUv5e", alias = "tpu v5e", alias = "tpuv5e")]
    TpuV5e,
    /// Google TPU v5p
    #[display("TPU v5p")]
    #[serde(alias = "TPU v5p", alias = "TPUv5p", alias = "tpu v5p", alias = "tpuv5p")]
    TpuV5p,
    /// Amazon Trainium AI accelerator
    #[display("Trainium")]
    Trainium,
    /// Tenstorrent Wormhole AI accelerator
    #[display("Wormhole")]
    Wormhole,
    /// Cerebras Wafer-Scale Engine 2 (WSE-2)
    #[display("WSE-2")]
    Wse2,
    /// Cerebras Wafer-Scale Engine 3 (WSE-3)
    #[display("WSE-3")]
    Wse3,
    /// Other or unspecified accelerator architecture
    #[display("{}", _0)]
    Other(String),
}
/// GPU compute backend or API
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum Backend {
    /// NVIDIA CUDA parallel computing platform
    #[display("CUDA")]
    Cuda,
    /// Microsoft DirectML
    #[display("DirectML")]
    DirectMl,
    /// AMD HIP (CUDA-compatible layer)
    #[display("HIP")]
    Hip,
    /// Apple Metal graphics and compute API
    #[display("Metal")]
    Metal,
    /// Intel oneAPI
    #[display("oneAPI")]
    OneApi,
    /// Khronos OpenCL open standard
    #[display("OpenCL")]
    OpenCl,
    /// AMD ROCm open compute platform
    #[display("ROCm")]
    RoCm,
    /// Intel oneAPI SYCL
    #[display("SYCL")]
    Sycl,
    /// Khronos Vulkan graphics and compute API
    #[display("Vulkan")]
    Vulkan,
    /// W3C WebGPU API
    #[display("WebGPU")]
    WebGpu,
    /// Other or unspecified backend
    #[display("{}", _0)]
    Other(String),
}
/// CPU instruction set architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum CpuArchitecture {
    /// 64-bit ARM instruction set
    #[display("AArch64")]
    AArch64,
    /// ARM Ethos NPU (integrated in some ARM CPUs)
    #[display("Ethos")]
    Ethos,
    /// Intel Itanium architecture (IA-64)
    #[display("IA-64")]
    Ia64,
    /// LoongArch (Chinese CPU ISA)
    #[display("LoongArch")]
    LoongArch,
    /// MIPS instruction set architecture
    #[display("MIPS")]
    Mips,
    /// Apple Neural Engine (integrated in Apple silicon)
    #[display("Neural Engine")]
    NeuralEngine,
    /// IBM PowerPC architecture
    #[display("PowerPC")]
    PowerPc,
    /// Qualcomm Hexagon processor (integrated in Snapdragon)
    #[display("Hexagon")]
    Hexagon,
    /// RISC-V open instruction set architecture
    #[display("RISC-V")]
    RiscV,
    /// 32-bit x86 instruction set
    #[display("x86")]
    X86,
    /// x86-64 instruction set (Intel/AMD)
    #[display("x86_64")]
    #[serde(alias = "x86_64", alias = "x86-64", alias = "amd64")]
    X86_64,
    /// IBM z/Architecture mainframe
    #[display("z/Architecture")]
    ZArch,
    /// Other or unspecified CPU architecture
    #[display("{}", _0)]
    Other(String),
}
/// DSP architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum DspArchitecture {
    /// Texas Instruments C66x DSP
    #[display("C66x")]
    C66x,
    /// Cadence Tensilica Xtensa DSP
    #[display("Xtensa")]
    Xtensa,
    /// Other or unspecified DSP architecture
    #[display("{}", _0)]
    Other(String),
}
/// FPGA architecture families
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum FpgaArchitecture {
    /// Intel Agilex FPGA
    #[display("Agilex")]
    Agilex,
    /// Lattice CrossLink FPGA
    #[display("CrossLink")]
    CrossLink,
    /// Intel/Altera Cyclone V FPGA
    #[display("Cyclone V")]
    CycloneV,
    /// Lattice ECP5 FPGA
    #[display("ECP5")]
    Ecp5,
    /// Lattice iCE40 FPGA
    #[display("iCE40")]
    Ice40,
    /// Intel/Altera Stratix V FPGA
    #[display("Stratix V")]
    StratixV,
    /// AMD/Xilinx Versal adaptive compute acceleration platform
    #[display("Versal")]
    Versal,
    /// AMD/Xilinx Virtex UltraScale+ FPGA
    #[display("Virtex UltraScale+")]
    VirtexUltraScalePlus,
    /// AMD/Xilinx Zynq SoC FPGA
    #[display("Zynq")]
    Zynq,
    /// Other or unspecified FPGA architecture
    #[display("{}", _0)]
    Other(String),
}
/// GPU graphics microarchitectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum GpuArchitecture {
    /// NVIDIA Ada Lovelace microarchitecture (RTX 40xx)
    #[display("Ada Lovelace")]
    AdaLovelace,
    /// Intel Alchemist graphics architecture (Xe-HPG)
    #[display("Alchemist")]
    Alchemist,
    /// NVIDIA Ampere microarchitecture (A100, RTX 30xx)
    #[display("Ampere")]
    Ampere,
    /// NVIDIA Blackwell microarchitecture (RTX 50xx, B100)
    #[display("Blackwell")]
    Blackwell,
    /// AMD CDNA 1 microarchitecture (MI100)
    #[display("CDNA1")]
    Cdna1,
    /// AMD CDNA 2 microarchitecture (MI250)
    #[display("CDNA2")]
    Cdna2,
    /// AMD CDNA 3 microarchitecture (MI300)
    #[display("CDNA3")]
    Cdna3,
    /// NVIDIA Hopper microarchitecture (H100, H200)
    #[display("Hopper")]
    Hopper,
    /// AMD RDNA 1 microarchitecture (RX 5xxx)
    #[display("RDNA1")]
    #[serde(alias = "RDNA1", alias = "RDNA 1", alias = "rdna1", alias = "RDNA1", alias = "rdna 1")]
    Rdna1,
    /// AMD RDNA 2 microarchitecture (RX 6xxx)
    #[display("RDNA2")]
    #[serde(alias = "RDNA2", alias = "RDNA 2", alias = "rdna2", alias = "RDNA2", alias = "rdna 2")]
    Rdna2,
    /// AMD RDNA 3 microarchitecture (RX 7xxx)
    #[display("RDNA3")]
    #[serde(alias = "RDNA3", alias = "RDNA 3", alias = "rdna3", alias = "RDNA3", alias = "rdna 3")]
    Rdna3,
    /// NVIDIA Turing microarchitecture (RTX 20xx)
    #[display("Turing")]
    Turing,
    /// NVIDIA Volta microarchitecture (V100)
    #[display("Volta")]
    Volta,
    /// Intel Xe graphics architecture
    #[display("Xe")]
    Xe,
    /// Intel Xe2 graphics architecture (Battlemage)
    #[display("Xe2")]
    Xe2,
    /// Intel Xe3 graphics architecture (Celestial)
    #[display("Xe3")]
    Xe3,
    /// Other or unspecified GPU architecture
    #[display("{}", _0)]
    Other(String),
}
/// Memory unit for hardware resources
#[derive(Clone, Copy, Debug, Display, PartialEq, Serialize)]
pub enum MemoryUnit {
    /// Gigabytes
    GB,
    /// Kilobytes
    KB,
    /// Megabytes
    MB,
}
/// Enumeration for hardware resources used by technology
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub enum Resource {
    /// Application-specific integrated circuit
    ///
    /// A custom-designed chip built to perform one particular function or a narrow set of functions, rather than acting as a general-purpose processor like a CPU or GPU
    ASIC {
        /// ASIC architecture or design family
        #[serde(alias = "arch")]
        architecture: Option<CpuArchitecture>,
        /// Number of ASICs required (if applicable)
        count: Option<u32>,
        /// Intended purpose of the ASIC (e.g., "Bitcoin mining", "video encoding")
        purpose: Option<String>,
        /// Hardware vendor or designer
        vendor: Option<Vendor>,
    },
    /// Central processing unit for "classical" computing
    CPU {
        /// Instruction set architecture
        #[serde(alias = "arch")]
        architecture: Option<CpuArchitecture>,
        /// Number of physical cores required (if applicable)
        cores: Option<u32>,
        /// Number of CPUs required (if applicable)
        count: Option<u32>,
        /// Minimum RAM (if applicable)
        #[serde(alias = "ram")]
        memory: Option<Memory>,
        /// Number of threads required (if applicable)
        threads: Option<u32>,
        /// Hardware vendor (e.g., Intel, AMD, ARM)
        vendor: Option<Vendor>,
    },
    /// Data processing unit
    ///
    /// A programmable processor optimized for data movement, networking, and infrastructure tasks, offloading them from the CPU
    DPU {
        /// DPU architecture or product family
        #[serde(alias = "arch")]
        architecture: Option<CpuArchitecture>,
        /// Number of DPUs required (if applicable)
        count: Option<u32>,
        /// On-chip memory (if applicable)
        #[serde(alias = "mem")]
        memory: Option<Memory>,
        /// Hardware vendor (e.g., NVIDIA, Intel, AMD)
        vendor: Option<Vendor>,
    },
    /// Digital signal processor
    ///
    /// A specialized microprocessor optimized to perform fast mathematical operations on digital signals in real time
    DSP {
        /// DSP architecture
        #[serde(alias = "arch")]
        architecture: Option<DspArchitecture>,
        /// Number of DSPs required (if applicable)
        count: Option<u32>,
        /// Clock frequency in MHz (if applicable)
        #[serde(alias = "freq")]
        frequency: Option<u32>,
        /// Hardware vendor (e.g., Qualcomm, Texas Instruments)
        vendor: Option<Vendor>,
    },
    /// Field-programmable gate array
    ///
    /// A reconfigurable integrated circuit that lets you implement custom digital hardware circuits after manufacturing, rather than having a fixed function like a CPU or ASIC
    FPGA {
        /// FPGA architecture or product family
        #[serde(alias = "arch")]
        architecture: Option<FpgaArchitecture>,
        /// Number of FPGAs required (if applicable)
        count: Option<u32>,
        /// Number of logic elements or LUTs available
        logic_elements: Option<u32>,
        /// Block RAM (if applicable)
        #[serde(alias = "mem")]
        memory: Option<Memory>,
        /// Hardware vendor (e.g., Intel, AMD)
        vendor: Option<Vendor>,
    },
    /// Graphics processing unit
    GPU {
        /// GPU microarchitecture
        #[serde(alias = "arch")]
        architecture: Option<GpuArchitecture>,
        /// Compute backend or API
        backend: Option<Backend>,
        /// NVIDIA CUDA compute capability (e.g., 8.0 for A100)
        compute_capability: Option<f32>,
        /// Number of GPUs required (if applicable)
        count: Option<u32>,
        /// VRAM (if applicable)
        #[serde(alias = "vram")]
        memory: Option<Memory>,
        /// GPU model name (e.g., "H100", "RTX 4090")
        name: Option<String>,
        /// Hardware vendor (e.g., NVIDIA, AMD, Intel)
        vendor: Option<Vendor>,
    },
    /// Neural processing unit
    ///
    /// A specialized chip designed for the matrix multiplication calculations that most AI models rely on
    NPU {
        /// NPU architecture
        #[serde(alias = "arch")]
        architecture: Option<AcceleratorArchitecture>,
        /// Number of NPUs required (if applicable)
        count: Option<u32>,
        /// On-chip SRAM (if applicable)
        #[serde(alias = "mem")]
        memory: Option<Memory>,
        /// Hardware vendor (e.g., Qualcomm, Apple, Intel)
        vendor: Option<Vendor>,
    },
    /// Neuromorphic compute
    Neuromorphic,
    /// Quantum computing (e.g., NISQ, etc.)
    Quantum,
    /// Tensor processing unit
    TPU {
        /// TPU generation
        #[serde(alias = "arch")]
        architecture: Option<AcceleratorArchitecture>,
        /// Number of TPU chips required (if applicable)
        count: Option<u32>,
        /// High-bandwidth memory (if applicable)
        #[serde(alias = "mem")]
        memory: Option<Memory>,
        /// Hardware vendor (e.g., Google)
        vendor: Option<Vendor>,
    },
    /// Unknown, unspecified, or otherwise unclassified resource
    Other(String),
}
/// Hardware vendor or manufacturer
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
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
    /// Broadcom Inc.
    #[display("Broadcom")]
    Broadcom,
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
    Ibm,
    /// Intel Corporation
    #[display("Intel")]
    Intel,
    /// MediaTek Inc.
    #[display("MediaTek")]
    MediaTek,
    /// Microsoft Corporation
    #[display("Microsoft")]
    Microsoft,
    /// NVIDIA Corporation
    #[display("NVIDIA")]
    NVIDIA,
    /// Qualcomm Incorporated
    #[display("Qualcomm")]
    Qualcomm,
    /// Samsung Electronics
    #[display("Samsung")]
    Samsung,
    /// Tenstorrent Inc.
    #[display("Tenstorrent")]
    Tenstorrent,
    /// Texas Instruments
    #[display("Texas Instruments")]
    TexasInstruments,
    /// Other or unspecified vendor
    #[display("{}", _0)]
    Other(String),
}
/// Memory amount with explicit unit
///
/// Supports string formats like `"80GB"`, `"512MB"`, `"64KB"` and plain number (defaults to GB).
#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    /// Numeric amount of memory
    pub amount: u32,
    /// Unit of measurement
    pub unit: MemoryUnit,
}
impl Memory {
    /// Create a Memory value in GB
    pub const fn gb(amount: u32) -> Self {
        Memory {
            amount,
            unit: MemoryUnit::GB,
        }
    }
    /// Create a Memory value in KB
    pub const fn kb(amount: u32) -> Self {
        Memory {
            amount,
            unit: MemoryUnit::KB,
        }
    }
    /// Create a Memory value in MB
    pub const fn mb(amount: u32) -> Self {
        Memory {
            amount,
            unit: MemoryUnit::MB,
        }
    }
}
impl From<String> for Architecture {
    fn from(s: String) -> Self {
        Architecture::from_str(&s)
    }
}
impl From<&str> for Architecture {
    fn from(s: &str) -> Self {
        Architecture::from_str(s)
    }
}
impl Architecture {
    fn from_str(s: &str) -> Self {
        let normalized = s.trim();
        let quoted = format!("\"{normalized}\"");
        [
            serde_json::from_str::<CpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<GpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<FpgaArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<AcceleratorArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<DspArchitecture>(&quoted).ok().map(Architecture::from),
        ]
        .into_iter()
        .flatten()
        .next()
        .unwrap_or_else(|| Architecture::Other(normalized.to_string()))
    }
}
impl From<AcceleratorArchitecture> for Architecture {
    fn from(value: AcceleratorArchitecture) -> Self {
        match value {
            | AcceleratorArchitecture::Ascend910 => Architecture::Ascend910,
            | AcceleratorArchitecture::Blackhole => Architecture::Blackhole,
            | AcceleratorArchitecture::Gaudi2 => Architecture::Gaudi2,
            | AcceleratorArchitecture::Inferentia => Architecture::Inferentia,
            | AcceleratorArchitecture::Lpu => Architecture::Lpu,
            | AcceleratorArchitecture::Maia => Architecture::Maia,
            | AcceleratorArchitecture::TpuV4 => Architecture::TpuV4,
            | AcceleratorArchitecture::TpuV5e => Architecture::TpuV5e,
            | AcceleratorArchitecture::TpuV5p => Architecture::TpuV5p,
            | AcceleratorArchitecture::Trainium => Architecture::Trainium,
            | AcceleratorArchitecture::Wormhole => Architecture::Wormhole,
            | AcceleratorArchitecture::Wse2 => Architecture::Wse2,
            | AcceleratorArchitecture::Wse3 => Architecture::Wse3,
            | AcceleratorArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<CpuArchitecture> for Architecture {
    fn from(value: CpuArchitecture) -> Self {
        match value {
            | CpuArchitecture::AArch64 => Architecture::AArch64,
            | CpuArchitecture::Ethos => Architecture::Ethos,
            | CpuArchitecture::Ia64 => Architecture::Ia64,
            | CpuArchitecture::LoongArch => Architecture::LoongArch,
            | CpuArchitecture::Mips => Architecture::Mips,
            | CpuArchitecture::NeuralEngine => Architecture::NeuralEngine,
            | CpuArchitecture::PowerPc => Architecture::PowerPc,
            | CpuArchitecture::Hexagon => Architecture::Hexagon,
            | CpuArchitecture::RiscV => Architecture::RiscV,
            | CpuArchitecture::X86 => Architecture::X86,
            | CpuArchitecture::X86_64 => Architecture::X86_64,
            | CpuArchitecture::ZArch => Architecture::ZArch,
            | CpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<DspArchitecture> for Architecture {
    fn from(value: DspArchitecture) -> Self {
        match value {
            | DspArchitecture::C66x => Architecture::C66x,
            | DspArchitecture::Xtensa => Architecture::Xtensa,
            | DspArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<FpgaArchitecture> for Architecture {
    fn from(value: FpgaArchitecture) -> Self {
        match value {
            | FpgaArchitecture::Agilex => Architecture::Agilex,
            | FpgaArchitecture::CrossLink => Architecture::CrossLink,
            | FpgaArchitecture::CycloneV => Architecture::CycloneV,
            | FpgaArchitecture::Ecp5 => Architecture::Ecp5,
            | FpgaArchitecture::Ice40 => Architecture::Ice40,
            | FpgaArchitecture::StratixV => Architecture::StratixV,
            | FpgaArchitecture::Versal => Architecture::Versal,
            | FpgaArchitecture::VirtexUltraScalePlus => Architecture::VirtexUltraScalePlus,
            | FpgaArchitecture::Zynq => Architecture::Zynq,
            | FpgaArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<GpuArchitecture> for Architecture {
    fn from(value: GpuArchitecture) -> Self {
        match value {
            | GpuArchitecture::AdaLovelace => Architecture::AdaLovelace,
            | GpuArchitecture::Alchemist => Architecture::Alchemist,
            | GpuArchitecture::Ampere => Architecture::Ampere,
            | GpuArchitecture::Blackwell => Architecture::Blackwell,
            | GpuArchitecture::Cdna1 => Architecture::Cdna1,
            | GpuArchitecture::Cdna2 => Architecture::Cdna2,
            | GpuArchitecture::Cdna3 => Architecture::Cdna3,
            | GpuArchitecture::Hopper => Architecture::Hopper,
            | GpuArchitecture::Rdna1 => Architecture::Rdna1,
            | GpuArchitecture::Rdna2 => Architecture::Rdna2,
            | GpuArchitecture::Rdna3 => Architecture::Rdna3,
            | GpuArchitecture::Turing => Architecture::Turing,
            | GpuArchitecture::Volta => Architecture::Volta,
            | GpuArchitecture::Xe => Architecture::Xe,
            | GpuArchitecture::Xe2 => Architecture::Xe2,
            | GpuArchitecture::Xe3 => Architecture::Xe3,
            | GpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl<'de> Deserialize<'de> for Architecture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct ArchitectureVisitor;
        impl<'de> serde::de::Visitor<'de> for ArchitectureVisitor {
            type Value = Architecture;
            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("an architecture string (e.g. 'x86_64', 'Ampere', 'Versal')")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Architecture, E> {
                Ok(Architecture::from_str(value))
            }
        }
        deserializer.deserialize_str(ArchitectureVisitor)
    }
}
impl<'de> Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct ResourceVisitor;
        impl<'de> serde::de::Visitor<'de> for ResourceVisitor {
            type Value = Resource;
            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(r#"a resource string (e.g. "CPU") or object (e.g. {"CPU": {...}})"#)
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Resource, E> {
                match value {
                    | "ASIC" | "asic" => Ok(Resource::ASIC {
                        architecture: None,
                        count: Some(1),
                        purpose: None,
                        vendor: None,
                    }),
                    | "CPU" | "cpu" => Ok(Resource::CPU {
                        architecture: None,
                        cores: None,
                        count: Some(1),
                        memory: None,
                        threads: None,
                        vendor: None,
                    }),
                    | "DPU" | "dpu" => Ok(Resource::DPU {
                        architecture: None,
                        count: Some(1),
                        memory: None,
                        vendor: None,
                    }),
                    | "DSP" | "dsp" => Ok(Resource::DSP {
                        architecture: None,
                        count: Some(1),
                        frequency: None,
                        vendor: None,
                    }),
                    | "FPGA" | "fpga" => Ok(Resource::FPGA {
                        architecture: None,
                        count: Some(1),
                        logic_elements: None,
                        memory: None,
                        vendor: None,
                    }),
                    | "GPU" | "gpu" => Ok(Resource::GPU {
                        architecture: None,
                        backend: None,
                        compute_capability: None,
                        count: Some(1),
                        memory: None,
                        name: None,
                        vendor: None,
                    }),
                    | "NPU" | "npu" => Ok(Resource::NPU {
                        architecture: None,
                        count: Some(1),
                        memory: None,
                        vendor: None,
                    }),
                    | "Neuromorphic" | "neuromorphic" => Ok(Resource::Neuromorphic),
                    | "Quantum" | "quantum" => Ok(Resource::Quantum),
                    | "TPU" | "tpu" => Ok(Resource::TPU {
                        architecture: None,
                        count: Some(1),
                        memory: None,
                        vendor: None,
                    }),
                    | other => Ok(Resource::Other(other.to_string())),
                }
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(self, mut map: M) -> Result<Resource, M::Error> {
                let next_key_result: Result<Option<String>, M::Error> = map.next_key();
                match next_key_result {
                    | Ok(Some(tag)) => match tag.as_str() {
                        | "ASIC" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<CpuArchitecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                purpose: Option<String>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::ASIC {
                                    architecture: f.architecture,
                                    count: f.count,
                                    purpose: f.purpose,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "CPU" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<CpuArchitecture>,
                                cores: Option<u32>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<Memory>,
                                threads: Option<u32>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::CPU {
                                    architecture: f.architecture,
                                    cores: f.cores,
                                    count: f.count,
                                    memory: f.memory,
                                    threads: f.threads,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "DPU" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<CpuArchitecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<Memory>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::DPU {
                                    architecture: f.architecture,
                                    count: f.count,
                                    memory: f.memory,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "DSP" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<DspArchitecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                frequency: Option<u32>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::DSP {
                                    architecture: f.architecture,
                                    count: f.count,
                                    frequency: f.frequency,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "FPGA" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<FpgaArchitecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                logic_elements: Option<u32>,
                                memory: Option<Memory>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::FPGA {
                                    architecture: f.architecture,
                                    count: f.count,
                                    logic_elements: f.logic_elements,
                                    memory: f.memory,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "GPU" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<GpuArchitecture>,
                                backend: Option<Backend>,
                                compute_capability: Option<f32>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<Memory>,
                                name: Option<String>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::GPU {
                                    architecture: f.architecture,
                                    backend: f.backend,
                                    compute_capability: f.compute_capability,
                                    count: f.count,
                                    memory: f.memory,
                                    name: f.name,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "NPU" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<AcceleratorArchitecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<Memory>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::NPU {
                                    architecture: f.architecture,
                                    count: f.count,
                                    memory: f.memory,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "Neuromorphic" | "Quantum" => {
                            let next_value_result: Result<serde::de::IgnoredAny, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok::<serde::de::IgnoredAny, M::Error>(_) => self.visit_str(&tag),
                                | Err(why) => Err(why),
                            }
                        }
                        | "TPU" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(alias = "arch")]
                                architecture: Option<AcceleratorArchitecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<Memory>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::TPU {
                                    architecture: f.architecture,
                                    count: f.count,
                                    memory: f.memory,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | other => {
                            let next_value_result: Result<serde::de::IgnoredAny, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok::<serde::de::IgnoredAny, M::Error>(_) => Ok(Resource::Other(other.to_string())),
                                | Err(why) => Err(why),
                            }
                        }
                    },
                    | Ok(None) => Err(serde::de::Error::custom("expected resource variant key")),
                    | Err(why) => Err(why),
                }
            }
        }
        deserializer.deserialize_any(ResourceVisitor)
    }
}
impl<'de> Deserialize<'de> for Memory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MemoryVisitor;
        impl<'de> serde::de::Visitor<'de> for MemoryVisitor {
            type Value = Memory;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(r#"a memory string (e.g. "80GB", "512MB") or a number (treated as GB)"#)
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Memory, E> {
                Ok(Memory {
                    amount: value as u32,
                    unit: MemoryUnit::GB,
                })
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Memory, E> {
                if value < 0 {
                    return Err(serde::de::Error::custom("Memory amount cannot be negative"));
                }
                Ok(Memory {
                    amount: value as u32,
                    unit: MemoryUnit::GB,
                })
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Memory, E> {
                Ok(Memory {
                    amount: value as u32,
                    unit: MemoryUnit::GB,
                })
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Memory, E> {
                let s = value.trim();
                let split = s
                    .find(|c: char| !c.is_ascii_digit())
                    .ok_or_else(|| serde::de::Error::custom(format!("Missing unit in memory value: '{s}'")))?;
                let num_str = s[..split].trim();
                let unit_str = s[split..].trim();
                let amount: u32 = num_str
                    .parse()
                    .map_err(|_| serde::de::Error::custom(format!("Invalid memory amount: '{num_str}'")))?;
                let unit = MemoryUnit::deserialize(serde::de::value::StrDeserializer::new(unit_str))?;
                Ok(Memory { amount, unit })
            }
        }
        deserializer.deserialize_any(MemoryVisitor)
    }
}
impl<'de> Deserialize<'de> for MemoryUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MemoryUnitVisitor;
        impl<'de> serde::de::Visitor<'de> for MemoryUnitVisitor {
            type Value = MemoryUnit;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a memory unit string (e.g. 'GB', 'MB', 'KB')")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<MemoryUnit, E> {
                match value.trim().to_uppercase().as_str() {
                    | "GB" | "G" | "GIB" => Ok(MemoryUnit::GB),
                    | "MB" | "M" | "MIB" => Ok(MemoryUnit::MB),
                    | "KB" | "K" | "KIB" => Ok(MemoryUnit::KB),
                    | other => Err(serde::de::Error::custom(format!("Invalid memory unit: '{other}'"))),
                }
            }
        }
        deserializer.deserialize_str(MemoryUnitVisitor)
    }
}
impl Serialize for Memory {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = format!("{}{}", self.amount, self.unit);
        serializer.serialize_str(&s)
    }
}
impl schemars::JsonSchema for Memory {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "Memory".into()
    }
    fn json_schema(_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
        use core::convert::TryInto;
        serde_json::json!({"type": "string", "pattern": "^\\d+\\s*(GB|MB|KB)$"})
            .try_into()
            .unwrap()
    }
    fn inline_schema() -> bool {
        true
    }
}
impl schemars::JsonSchema for MemoryUnit {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "MemoryUnit".into()
    }
    fn json_schema(_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
        use core::convert::TryInto;
        serde_json::json!({"type": "string", "enum": ["GB", "MB", "KB"]}).try_into().unwrap()
    }
    fn inline_schema() -> bool {
        true
    }
}
fn default_count() -> Option<u32> {
    Some(1)
}
