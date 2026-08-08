//! Hardware resource, architecture, and vendor types
use crate::prelude::*;
use crate::schema::research_activity::aspect::Modality;
use crate::util::constants::gpu::{
    NVIDIA_CC_AMPERE_80_PATTERNS, NVIDIA_CC_BLACKWELL_100_PATTERNS, NVIDIA_CC_BLACKWELL_103_PATTERNS, NVIDIA_CC_FERMI_21_GT_SUFFIXES,
    NVIDIA_CC_FERMI_21_PATTERNS, NVIDIA_CC_FERMI_21_QUADRO_SUFFIXES, NVIDIA_CC_KEPLER_32_PATTERNS, NVIDIA_CC_KEPLER_35_PATTERNS,
    NVIDIA_CC_MAXWELL_50_PATTERNS, NVIDIA_CC_MAXWELL_52_PATTERNS, NVIDIA_CC_PASCAL_60_PATTERNS, NVIDIA_CC_TESLA_11_GTS_SUFFIXES,
    NVIDIA_CC_TESLA_11_GT_SUFFIXES, NVIDIA_CC_TESLA_11_PATTERNS, NVIDIA_CC_TESLA_11_QUADRO_SUFFIXES, NVIDIA_CC_TESLA_12_GT_SUFFIXES,
    NVIDIA_CC_TESLA_12_PATTERNS, NVIDIA_CC_TESLA_12_QUADRO_SUFFIXES, NVIDIA_CC_TESLA_13_PATTERNS, NVIDIA_CC_TESLA_13_QUADRO_SUFFIXES,
};
use crate::util::{contains_any, contains_any_with_prefix};
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

/// Memory types and (de)serialization for hardware resources
pub mod memory;
/// Quantum computing paradigms and hardware definitions
pub mod quantum;
/// Vendor types and vendor-aware trait
pub mod vendor;
pub use memory::{Memory, MemoryUnit};
pub use quantum::{Model, Paradigm, Regime, Topology};
use vendor::validate_vendor;
pub use vendor::{Vendor, Vendored};

/// Unified hardware architecture identifier
///
/// This enum provides a single type that can represent any hardware architecture
/// across all resource types (CPU, GPU, FPGA, accelerators, DSPs). Each variant
/// corresponds to one of the typed architecture enums.
#[derive(Clone, Debug, Display, PartialEq, Serialize, JsonSchema)]
pub enum Architecture {
    /// 64-bit ARM instruction set
    ///
    /// See <https://developer.arm.com/documentation/102374/0103/> for more information
    #[display("AArch64")]
    AArch64,
    /// NVIDIA Ada Lovelace architecture (RTX 40xx)
    /// > Designed to provide revolutionary performance for ray tracing and AI-based neural graphics
    ///
    /// See <https://www.nvidia.com/en-us/geforce/ada-lovelace-architecture/> for more information
    #[display("Ada Lovelace")]
    AdaLovelace,
    /// Intel Agilex [`Resource::FPGA`]
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Agilex")]
    Agilex,
    /// Intel Alchemist graphics architecture (Xe-HPG)
    /// > High-performance graphics architecture designed for discrete GPUs
    ///
    /// See [Introduction to the Xe-HPG Architecture](https://www.intel.com/content/www/us/en/developer/articles/technical/introduction-to-the-xe-hpg-architecture.html) for more information
    #[display("Alchemist")]
    Alchemist,
    /// NVIDIA (2020) Ampere architecture (A100, RTX 30xx)
    /// > Designed for AI, HPC, and graphics workloads
    ///
    /// See <https://www.nvidia.com/en-us/data-center/ampere-architecture/> for more information
    #[display("Ampere")]
    Ampere,
    /// Apple GPU architecture (A14/A15 Bionic)
    #[display("Apple GPU (A14/A15)")]
    AppleA14,
    /// Apple GPU architecture (A16/A17 Bionic)
    #[display("Apple GPU (A16/A17)")]
    AppleA16,
    /// Apple GPU architecture (A18/A18 Pro)
    #[display("Apple GPU (A18)")]
    AppleA18,
    /// Apple efficiency core (big/little architecture)
    #[display("Apple Efficiency Core")]
    AppleEfficiencyCore,
    /// Apple GPU architecture (M1 series)
    #[display("Apple GPU (M1)")]
    AppleM1,
    /// Apple GPU architecture (M2 series)
    #[display("Apple GPU (M2)")]
    AppleM2,
    /// Apple GPU architecture (M3 series)
    #[display("Apple GPU (M3)")]
    AppleM3,
    /// Apple GPU architecture (M4 series)
    #[display("Apple GPU (M4)")]
    AppleM4,
    /// Apple GPU architecture (M5 series)
    #[display("Apple GPU (M5)")]
    AppleM5,
    /// Apple performance core (big/little architecture)
    #[display("Apple Performance Core")]
    ApplePerformanceCore,
    /// Xilinx Artix [`Resource::FPGA`] family
    /// > Cost and transceiver optimized
    ///
    /// See <https://www.amd.com/en/products/adaptive-socs-and-fpgas/fpga/artix-7.html> for more information
    #[display("Artix")]
    Artix,
    /// Huawei Ascend 910 AI accelerator
    ///
    /// See <https://www.huawei.com/en/> for more information
    #[display("Ascend 910")]
    Ascend910,
    /// Tenstorrent Blackhole AI accelerator
    /// > PCIe boards that are infinitely scalable and designed for high performance AI processing — features 16 big RISC-V cores and up to 32GB of GDDR6 memory per chip
    ///
    /// See <https://tenstorrent.com/hardware/cards> for more information
    #[display("Blackhole")]
    Blackhole,
    /// NVIDIA Blackwell architecture (RTX 50xx, B100)
    ///
    /// See <https://www.nvidia.com/en-us/data-center/technologies/blackwell-architecture/> for more information
    #[display("Blackwell")]
    Blackwell,
    /// Texas Instruments C66x [`Resource::DSP`]
    ///
    /// See [C66x DSP CPU and Instruction Set](https://www.nvidia.com/en-us/data-center/technologies/blackwell-architecture/) for more information
    #[display("C66x")]
    C66x,
    /// NVIDIA Carmel CPU architecture
    ///
    /// Custom ARMv8.2 CPU core used in Jetson Xavier series (AGX Xavier, Xavier NX)
    ///
    /// See <https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/> for more information
    #[display("Carmel")]
    Carmel,
    /// AMD CDNA 1 microarchitecture (MI100)
    /// > Dedicated compute architecture underlying AMD Instinct™ GPUs and APUs
    ///
    /// See <https://www.amd.com/en/technologies/cdna.html> for more information
    #[display("CDNA1")]
    Cdna1,
    /// AMD CDNA 2 microarchitecture (MI250)
    /// > Dedicated compute architecture underlying AMD Instinct™ GPUs and APUs
    ///
    /// See <https://www.amd.com/en/technologies/cdna.html> for more information
    #[display("CDNA2")]
    Cdna2,
    /// AMD CDNA 3 microarchitecture (MI300)
    /// > Dedicated compute architecture underlying AMD Instinct™ GPUs and APUs
    ///
    /// See <https://www.amd.com/en/technologies/cdna.html> for more information
    #[display("CDNA3")]
    Cdna3,
    /// AMD CDNA 4 microarchitecture (MI350)
    /// > Dedicated compute architecture underlying AMD Instinct™ GPUs and APUs
    ///
    /// See <https://www.amd.com/en/technologies/cdna.html> for more information
    #[display("CDNA4")]
    Cdna4,
    /// Lattice CrossLink [`Resource::FPGA`]
    /// > Low power FPGA featuring hardened MIPI[^mipi] D-PHY[^dphy], LVDS[^lvds], SLVS[^slvs], subLVDS[^sublvds], & Open LDI[^openldi] bridging
    ///
    /// [^mipi]: Mobile Industry Processor Interface (a standards body/spec family)
    /// [^dphy]: A specific MIPI-defined physical layer (often just called "MIPI D-PHY") rather than a word-by-word acronym expansion in practice
    /// [^lvds]: Low-Voltage Differential Signaling
    /// [^slvs]: Scalable Low-Voltage Signaling
    /// [^sublvds]: Sub Low-Voltage Differential Signaling (a lower-swing LVDS variant used in imaging)
    /// [^openldi]: Open LVDS Display Interface (Open LDI / OpenLDI)
    ///
    /// See <https://www.latticesemi.com/products/fpgaandcpld/crosslink> for more information
    #[display("CrossLink")]
    CrossLink,
    /// Intel/Altera Cyclone V [`Resource::FPGA`]
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Cyclone V")]
    CycloneV,
    /// Lattice ECP5 [`Resource::FPGA`]
    /// > Lattice mid-range FPGA family
    ///
    /// See <https://www.latticesemi.com/products/fpgaandcpld/ecp5> for more information
    #[display("ECP5")]
    Ecp5,
    /// ARM Ethos [`Resource::NPU`]
    ///
    /// See <https://www.arm.com/products/silicon-ip-cpu/ethos/ethos-u85> for more information
    #[display("Ethos")]
    Ethos,
    /// NVIDIA Fermi microarchitecture (Tesla 20xx, GeForce 4xx/5xx, 2010)
    ///
    /// Compute capability 2.x
    ///
    /// See <https://developer.nvidia.com/cuda-gpus> for more information
    #[display("Fermi")]
    Fermi,
    /// Intel Habana Gaudi2 AI accelerator
    ///
    /// See <[The Habana Gaudi2 Processor for Deep Learning](https://www.intel.com/content/www/us/en/developer/articles/technical/habana-gaudi2-processor-for-deep-learning.html) for more information
    #[display("Gaudi2")]
    Gaudi2,
    /// Graphcore Colossus GC200 / IPU architecture (2018-2019)
    #[display("Graphcore GC200")]
    GraphcoreGc200,
    /// NVIDIA Grace CPU architecture
    ///
    /// See <https://www.nvidia.com/en-us/data-center/grace-cpu/> for more information
    #[display("Grace")]
    Grace,
    /// Qualcomm Hexagon processor NPU
    /// > Purpose-built for AI, powers advanced gen AI models and cutting-edge gen AI experiences while maintaining best-in-class power efficiency
    ///
    /// See https://www.qualcomm.com/processors/hexagon> for more information
    #[display("Hexagon")]
    Hexagon,
    /// NVIDIA Hopper architecture (H100, H200)
    ///
    /// See <https://www.nvidia.com/en-us/data-center/technologies/hopper-architecture/> for more information
    #[display("Hopper")]
    Hopper,
    /// Intel Itanium architecture (IA-64)
    ///
    /// See [WIkipedia](https://en.wikipedia.org/wiki/Itanium) for more information
    #[display("IA-64")]
    Ia64,
    /// Lattice iCE40 LP/HX/UltraPlus [`Resource::FPGA`]
    /// > Low-Power, High-Performance FPGA with Small [^BGA] package for the thinnest devices
    ///
    /// [^BGA]: Ball Grid Array (BGA) is a type of surface-mount packaging used for integrated circuits, where the connections are made through an array of solder balls on the underside of the package rather than traditional pins. This allows for a smaller footprint and better thermal performance, making it ideal for compact devices.
    ///
    /// See <https://www.latticesemi.com/products/fpgaandcpld/ice40> for more information
    #[display("iCE40")]
    Ice40,
    /// Amazon Inferentia AI inference chip
    ///
    /// See <https://aws.amazon.com/ai/machine-learning/inferentia/> for more information
    #[display("Inferentia")]
    Inferentia,
    /// NVIDIA (2012) Kepler [`Resource::GPU`] architecture (GK-series)
    ///
    /// See [Tuning CUDA Applications for Kepler](https://docs.nvidia.com/cuda/archive/9.2/kepler-tuning-guide/index.html#nvidia-kepler-compute-architecture) for more information
    #[display("Kepler")]
    Kepler,
    /// Xilinx Kintex FPGA family
    #[display("Kintex")]
    Kintex,
    /// LoongArch (Chinese CPU ISA)
    ///
    /// See [Wikipedia](https://en.wikipedia.org/wiki/Loongson#LoongArch) for more information
    #[display("LoongArch")]
    LoongArch,
    /// Groq Language Processing Unit
    ///
    /// See <https://groq.com/lpu-architecture> for more information
    #[display("LPU")]
    Lpu,
    /// Microsoft Maia AI accelerator
    ///
    /// See <https://news.microsoft.com/maia-200/> for more information
    #[display("Maia")]
    Maia,
    /// NVIDIA (2014) Maxwell [`Resource::GPU`] architecture (GM-series)
    ///
    /// See <https://developer.nvidia.com/maxwell-compute-architecture> for more information
    #[display("Maxwell")]
    Maxwell,
    /// [^MIPS] instruction set architecture
    ///
    /// [^MIPS]: Microprocessor without Interlocked Pipeline Stages (MIPS) is a RISC (Reduced Instruction Set Computer) architecture developed in the 1980s.
    ///
    /// See [Wikipedia](https://en.wikipedia.org/wiki/MIPS_architecture) for more information
    #[display("MIPS")]
    Mips,
    /// Intel Nervana neural processor (NNP-I / NNP-T)
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Nervana")]
    Nervana,
    /// Apple Neural Engine
    ///
    /// See [Apple Machine Learning Research - The Apple Neural Engine](https://machinelearning.apple.com/research/neural-engine-transformers#the-apple-neuralengine) for more information
    #[display("Neural Engine")]
    NeuralEngine,
    /// NVIDIA Orin system-on-chip (Jetson AGX Orin, Orin NX, Orin Nano)
    ///
    /// Ampere-based GPU integrated in the Jetson Orin family. Compute capability 8.7
    ///
    /// See <https://developer.nvidia.com/cuda-gpus> for more information
    #[display("Orin")]
    Orin,
    /// NVIDIA (2016) Pascal [`Resource::GPU`] microarchitecture (GP-series)
    ///
    /// See <https://www.nvidia.com/en-us/data-center/pascal-gpu-architecture/> for more information
    #[display("Pascal")]
    Pascal,
    /// IBM PowerPC architecture
    /// > RISC [^ISA] created in 1991
    ///
    /// [^ISA]: Instruction Set Architecture (ISA)
    ///
    /// See [Wikipedia](https://en.wikipedia.org/wiki/PowerPC) for more information
    #[display("PowerPC")]
    PowerPc,
    /// AMD RDNA 1 architecture (RX 5xxx)
    ///
    /// See <https://www.amd.com/en/technologies/rdna.html> for more information
    #[display("RDNA1")]
    Rdna1,
    /// AMD RDNA 2 architecture (RX 6xxx)
    ///
    /// See <https://www.amd.com/en/technologies/rdna.html> for more information
    #[display("RDNA2")]
    Rdna2,
    /// AMD RDNA 3 architecture (RX 7xxx)
    ///
    /// See <https://www.amd.com/en/technologies/rdna.html> for more information
    #[display("RDNA3")]
    Rdna3,
    /// [^RISC]-V open [^ISA]
    ///
    /// [^RISC]: Reduced Instruction Set Computer (RISC)
    /// [^ISA]: Instruction Set Architecture (ISA)
    ///
    /// See <https://riscv.org/> for more information
    #[display("RISC-V")]
    RiscV,
    /// SambaNova Reconfigurable Dataflow Unit (RDU) architecture
    /// > Fifth-generation chip purpose-built for agentic inference
    ///
    /// See <https://sambanova.ai/products/rdu-ai-chips> for more information
    #[display("SambaNova RDU")]
    SambaNovaRdu,
    /// Xilinx Spartan [`Resource::FPGA`] family
    /// > I/O Optimization with High Performance-per-Watt
    ///
    /// See <https://www.amd.com/en/products/adaptive-socs-and-fpgas/fpga/spartan-7.html> for more information
    #[display("Spartan")]
    Spartan,
    /// Intel Stratix 10 [`Resource::FPGA`] family
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Stratix 10")]
    Stratix10,
    /// Intel/Altera Stratix V [`Resource::FPGA`]
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Stratix V")]
    StratixV,
    /// NVIDIA Tesla microarchitecture (G80, GT200, pre-Fermi, 2006-2009)
    ///
    /// Compute capability 1.x. Covers G80, G92, G94, G96, G200, GT21x families
    ///
    /// See <https://developer.nvidia.com/cuda-gpus> for more information
    #[display("Tesla")]
    Tesla,
    /// Jetson Thor system-on-chip (Jetson T5000, T4000)
    ///
    /// Next-generation Jetson embedded GPU. Compute capability 11.0
    ///
    /// See <https://developer.nvidia.com/cuda-gpus> for more information
    #[display("Thor")]
    Thor,
    /// Google [`Resource::TPU`] v4
    ///
    /// See <https://docs.cloud.google.com/tpu/docs/v4> for more information
    #[display("TPU v4")]
    TpuV4,
    /// Google [`Resource::TPU`] v5e
    ///
    /// See <https://docs.cloud.google.com/tpu/docs/v5e> for more information
    #[display("TPU v5e")]
    TpuV5e,
    /// Google [`Resource::TPU`] v5p
    ///
    /// See <https://docs.cloud.google.com/tpu/docs/v5p> for more information
    #[display("TPU v5p")]
    TpuV5p,
    /// Google [`Resource::TPU`] v6e ("Trillium")
    ///
    /// See <https://cloud.google.com/tpu/docs/v6e> for more information
    #[display("TPU v6e")]
    TpuV6e,
    /// Amazon Trainium AI accelerator
    /// > Purpose-built for high-performance, cost-efficient AI at scale
    ///
    /// See <https://aws.amazon.com/ai/machine-learning/trainium/> for more information
    #[display("Trainium")]
    Trainium,
    /// NVIDIA (2018) Turing architecture (RTX 20xx)
    ///
    /// See <https://www.nvidia.com/en-us/geforce/turing/> for more information
    #[display("Turing")]
    Turing,
    /// AMD/Xilinx Versal adaptive compute acceleration platform
    /// > Heterogeneous acceleration from cloud to edge
    ///
    /// See <https://www.amd.com/en/products/adaptive-socs-and-fpgas/versal.html> for more information
    #[display("Versal")]
    Versal,
    /// AMD/Xilinx Virtex UltraScale+ [`Resource::FPGA`]
    ///
    /// See <https://www.amd.com/en/products/adaptive-socs-and-fpgas/fpga/virtex-ultrascale-plus.html> for more information
    #[display("Virtex UltraScale+")]
    VirtexUltraScalePlus,
    /// NVIDIA Volta architecture (V100)
    ///
    /// See <https://www.nvidia.com/en-us/data-center/volta-gpu-architecture/> for more information
    #[display("Volta")]
    Volta,
    /// NVIDIA Vera CPU architecture
    ///
    /// See <https://www.nvidia.com/en-us/data-center/cpu/> for more information
    #[display("Vera")]
    Vera,
    /// Tenstorrent Wormhole AI accelerator
    /// > Flexible, scalable processors built with Tensix Cores™. Each includes a compute unit, network-on-chip, local cache and "baby RISC-V" cores, coalescing in powerful data movement through the chip.
    ///
    /// See <https://tenstorrent.com/hardware/cards#Wormhole> for more information
    #[display("Wormhole")]
    Wormhole,
    /// Cerebras Wafer-Scale Engine 2 (WSE-2)
    ///
    /// See <https://www.cerebras.ai/chip> for more information
    #[display("WSE-2")]
    Wse2,
    /// Cerebras Wafer-Scale Engine 3 (WSE-3)
    ///
    /// See <https://www.cerebras.ai/chip> for more information
    #[display("WSE-3")]
    Wse3,
    /// 32-bit x86 instruction set
    ///
    /// See [Wikipedia](https://en.wikipedia.org/wiki/X86) for more information
    #[display("x86")]
    X86,
    /// x86-64 instruction set (Intel/AMD)
    ///
    /// See [Wikipedia](https://en.wikipedia.org/wiki/X86-64) for more information
    #[display("x86_64")]
    X86_64,
    /// Intel Xe graphics architecture
    #[display("Xe")]
    Xe,
    /// Intel Xe2 graphics architecture (Battlemage)
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Xe2")]
    Xe2,
    /// Intel Xe3 graphics architecture (Celestial)
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Xe3")]
    Xe3,
    /// Cadence Tensilica Xtensa [`Resource::DSP`]
    ///
    /// See <https://www.cadence.com/en_US/home/tools/ip/tensilica-ip.html> for more information
    #[display("Xtensa")]
    Xtensa,
    /// IBM z/Architecture mainframe
    ///
    /// See <https://www.ibm.com/us-en> for more information
    #[display("z/Architecture")]
    ZArch,
    /// AMD/Xilinx Zynq SoC [`Resource::FPGA`]
    ///
    /// See <https://www.amd.com/en/products/adaptive-socs-and-fpgas/soc.html> for more information
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
    #[serde(alias = "ascend 910", alias = "Ascend910", alias = "ascend910")]
    Ascend910,
    /// Tenstorrent Blackhole AI accelerator
    #[display("Blackhole")]
    #[serde(alias = "blackhole")]
    Blackhole,
    /// Intel Habana Gaudi2 AI accelerator
    #[display("Gaudi2")]
    #[serde(alias = "Gaudi 2", alias = "gaudi2", alias = "gaudi 2")]
    Gaudi2,
    /// Graphcore Colossus GC200 / IPU architecture (2018-2019)
    #[display("Graphcore GC200")]
    #[serde(alias = "graphcore gc200", alias = "GC200", alias = "gc200")]
    GraphcoreGc200,
    /// Amazon Inferentia AI inference chip
    #[display("Inferentia")]
    #[serde(alias = "inferentia")]
    Inferentia,
    /// Groq Language Processing Unit
    #[display("LPU")]
    #[serde(alias = "lpu", alias = "Language Processing Unit", alias = "language processing unit")]
    Lpu,
    /// Microsoft Maia AI accelerator
    #[display("Maia")]
    #[serde(alias = "maia")]
    Maia,
    /// Intel Nervana neural processor (NNP-I / NNP-T, 2010s)
    #[display("Nervana")]
    #[serde(alias = "nervana")]
    Nervana,
    /// SambaNova Reconfigurable Dataflow Unit (RDU) architecture (2020s)
    #[display("SambaNova RDU")]
    #[serde(alias = "sambanova rdu", alias = "RDU", alias = "rdu")]
    SambaNovaRdu,
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
    /// Google TPU v6e ("Trillium")
    #[display("TPU v6e")]
    #[serde(
        alias = "TPU v6e",
        alias = "TPUv6e",
        alias = "tpu v6e",
        alias = "tpuv6e",
        alias = "Trillium",
        alias = "trillium"
    )]
    TpuV6e,
    /// Amazon Trainium AI accelerator
    #[display("Trainium")]
    #[serde(alias = "trainium")]
    Trainium,
    /// Tenstorrent Wormhole AI accelerator
    #[display("Wormhole")]
    #[serde(alias = "wormhole")]
    Wormhole,
    /// Cerebras Wafer-Scale Engine 2 (WSE-2)
    #[display("WSE-2")]
    #[serde(alias = "wse2", alias = "WSE2", alias = "wse-2")]
    Wse2,
    /// Cerebras Wafer-Scale Engine 3 (WSE-3)
    #[display("WSE-3")]
    #[serde(alias = "wse3", alias = "WSE3", alias = "wse-3")]
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
/// Sensor modalities for hardware resources
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum SensorModality {
    /// Acoustic sensing such as microphones and hydrophones
    #[display("audio")]
    #[serde(alias = "audio", alias = "acoustic", alias = "microphone", alias = "hydrophone")]
    Audio,
    /// Chemical sensing such as gas composition or pH
    #[display("chemical")]
    #[serde(alias = "chemical", alias = "chem", alias = "gas", alias = "ph")]
    Chemical,
    /// Depth sensing such as time-of-flight and structured light
    #[display("depth")]
    #[serde(alias = "depth", alias = "tof", alias = "time-of-flight", alias = "structured-light")]
    Depth,
    /// Event-based vision sensors
    #[display("event")]
    #[serde(alias = "event", alias = "event-camera", alias = "event camera", alias = "dvs")]
    Event,
    /// Hyperspectral sensing across many narrow wavelength bands
    #[display("hyperspectral")]
    #[serde(alias = "hyperspectral", alias = "hyper-spectral", alias = "hsi")]
    Hyperspectral,
    /// Imaging modalities such as RGB cameras and thermal imagers
    #[display("image")]
    #[serde(alias = "image", alias = "camera", alias = "vision", alias = "thermal")]
    Image,
    /// Inertial measurement sensing (accelerometer/gyroscope)
    #[display("inertial")]
    #[serde(alias = "inertial", alias = "imu", alias = "accel", alias = "gyroscope")]
    Inertial,
    /// Light detection and ranging
    #[display("lidar")]
    #[serde(alias = "lidar", alias = "li-dar")]
    Lidar,
    /// Magnetic field sensing
    #[display("magnetic")]
    #[serde(alias = "magnetic", alias = "magnetometer")]
    Magnetic,
    /// Position and navigation sensing (e.g., GNSS)
    #[display("navigation")]
    #[serde(alias = "navigation", alias = "gnss", alias = "gps")]
    Navigation,
    /// Network traffic telemetry and packet-level sensing
    #[display("network-traffic")]
    #[serde(alias = "network-traffic", alias = "network traffic", alias = "netflow", alias = "packet")]
    NetworkTraffic,
    /// Pressure sensing
    #[display("pressure")]
    #[serde(alias = "pressure", alias = "barometer")]
    Pressure,
    /// Radio detection and ranging
    #[display("radar")]
    #[serde(alias = "radar", alias = "sar")]
    Radar,
    /// General RF sensing and spectrum observation
    #[display("rf")]
    #[serde(alias = "rf", alias = "radio", alias = "spectrum")]
    Radio,
    /// Seismic sensing for ground motion and vibration
    #[display("seismic")]
    #[serde(alias = "seismic", alias = "geophone")]
    Seismic,
    /// Server-side event stream sensing from server push channels
    #[display("server-side-event")]
    #[serde(
        alias = "server-side-event",
        alias = "server side event",
        alias = "server-sent-event",
        alias = "server sent event",
        alias = "sse"
    )]
    ServerSideEvent,
    /// Sonar sensing using reflected acoustic waves
    #[display("sonar")]
    #[serde(alias = "sonar", alias = "acoustic-ranging")]
    Sonar,
    /// Temperature sensing
    #[display("temperature")]
    #[serde(alias = "temperature", alias = "temp", alias = "thermometer")]
    Temperature,
    /// Ultrasonic sensing using high-frequency acoustic waves
    #[display("ultrasonic")]
    #[serde(alias = "ultrasonic", alias = "ultrasound")]
    Ultrasonic,
    /// Webhook event stream sensing from HTTP callbacks
    #[display("webhook")]
    #[serde(alias = "webhook", alias = "web hook", alias = "http-callback", alias = "http callback")]
    Webhook,
    /// Other or unspecified sensor modality
    #[display("{}", _0)]
    Other(String),
}
/// CPU instruction set architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum CpuArchitecture {
    /// 64-bit ARM instruction set
    #[display("AArch64")]
    #[serde(alias = "arm", alias = "aarch64", alias = "arm64")]
    AArch64,
    /// NVIDIA Carmel CPU architecture
    ///
    /// Custom ARMv8.2 CPU core used in Jetson Xavier series (AGX Xavier, Xavier NX)
    ///
    /// See <https://www.nvidia.com/en-us/autonomous-machines/embedded-systems/> for more information
    #[display("Carmel")]
    #[serde(alias = "carmel", alias = "carmel cpu")]
    Carmel,
    /// Apple-specific efficiency core (big/little architecture)
    #[display("Apple Efficiency Core")]
    #[serde(alias = "Apple Efficiency Core", alias = "apple efficiency core", alias = "EfficiencyCore")]
    EfficiencyCore,
    /// ARM Ethos NPU (integrated in some ARM CPUs)
    #[display("Ethos")]
    #[serde(alias = "ethos")]
    Ethos,
    /// NVIDIA Grace architecture
    ///
    /// Used in [DGX Spark](https://www.nvidia.com/en-us/products/workstations/dgx-spark/) machine
    ///
    /// See <https://www.nvidia.com/en-us/data-center/grace-cpu-superchip/> for more information
    #[display("Grace")]
    #[serde(alias = "grace", alias = "grace cpu")]
    Grace,
    /// Qualcomm Hexagon processor (integrated in Snapdragon)
    #[display("Hexagon")]
    #[serde(alias = "hexagon")]
    Hexagon,
    /// Intel Itanium architecture (IA-64)
    #[display("IA-64")]
    #[serde(alias = "ia64", alias = "IA64")]
    Ia64,
    /// LoongArch (Chinese CPU ISA)
    #[display("LoongArch")]
    #[serde(alias = "loongarch")]
    LoongArch,
    /// MIPS instruction set architecture
    #[display("MIPS")]
    #[serde(alias = "mips")]
    Mips,
    /// Apple Neural Engine (integrated in Apple silicon)
    #[display("Neural Engine")]
    #[serde(alias = "Neural Engine")]
    NeuralEngine,
    /// Apple-specific performance core (big/little architecture)
    #[display("Apple Performance Core")]
    #[serde(alias = "Apple Performance Core", alias = "apple performance core", alias = "PerformanceCore")]
    PerformanceCore,
    /// IBM PowerPC architecture
    #[display("PowerPC")]
    #[serde(alias = "powerpc")]
    PowerPc,
    /// RISC-V open instruction set architecture
    #[display("RISC-V")]
    #[serde(alias = "riscv", alias = "risc-v")]
    RiscV,
    /// NVIDIA Vera architecture
    ///
    /// See <https://www.nvidia.com/en-us/data-center/grace-cpu-superchip/> for more information
    #[display("Vera")]
    #[serde(alias = "vera", alias = "vera cpu")]
    Vera,
    /// 32-bit x86 instruction set
    #[display("x86")]
    #[serde(alias = "X86")]
    X86,
    /// x86-64 instruction set (Intel/AMD)
    #[display("x86_64")]
    #[serde(alias = "x86_64", alias = "x86-64", alias = "amd64")]
    X86_64,
    /// IBM z/Architecture mainframe
    #[display("z/Architecture")]
    #[serde(alias = "z architecture", alias = "zarchitecture")]
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
    #[serde(alias = "c66x")]
    C66x,
    /// Cadence Tensilica Xtensa DSP
    #[display("Xtensa")]
    #[serde(alias = "xtensa")]
    Xtensa,
    /// Other or unspecified DSP architecture
    #[display("{}", _0)]
    Other(String),
}
/// FPGA architecture families used by [`Resource::FPGA`] resources
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum FpgaArchitecture {
    /// Intel Agilex FPGA
    #[display("Agilex")]
    #[serde(alias = "agilex")]
    Agilex,
    /// Xilinx Artix FPGA family (2010s)
    #[display("Artix")]
    #[serde(alias = "artix")]
    Artix,
    /// Lattice CrossLink FPGA
    #[display("CrossLink")]
    #[serde(alias = "crosslink", alias = "cross-link")]
    CrossLink,
    /// Intel/Altera Cyclone V FPGA
    #[display("Cyclone V")]
    #[serde(alias = "cyclonev", alias = "cyclone v")]
    CycloneV,
    /// Lattice ECP5 FPGA
    #[display("ECP5")]
    #[serde(alias = "ecp5")]
    Ecp5,
    /// Lattice iCE40 FPGA
    #[display("iCE40")]
    #[serde(alias = "ice40", alias = "iCE40")]
    Ice40,
    /// Xilinx Kintex FPGA family (2010s)
    #[display("Kintex")]
    #[serde(alias = "kintex")]
    Kintex,
    /// Xilinx Spartan FPGA family (modern 2010s+ variants)
    #[display("Spartan")]
    #[serde(alias = "spartan")]
    Spartan,
    /// Intel Stratix 10 FPGA family (2016-2017)
    #[display("Stratix 10")]
    #[serde(alias = "stratix10", alias = "stratix 10")]
    Stratix10,
    /// Intel/Altera Stratix V FPGA
    #[display("Stratix V")]
    #[serde(alias = "stratixv", alias = "stratix v")]
    StratixV,
    /// AMD/Xilinx Versal adaptive compute acceleration platform
    #[display("Versal")]
    #[serde(alias = "versal")]
    Versal,
    /// AMD/Xilinx Virtex UltraScale+ FPGA
    #[display("Virtex UltraScale+")]
    #[serde(alias = "virtex ultrascale+", alias = "virtexultrascaleplus", alias = "virtex ultrascale plus")]
    VirtexUltraScalePlus,
    /// AMD/Xilinx Zynq SoC FPGA
    #[display("Zynq")]
    #[serde(alias = "zynq")]
    Zynq,
    /// Other or unspecified FPGA architecture
    #[display("{}", _0)]
    Other(String),
}
/// GPU graphics microarchitectures
///
/// Vendor-specific GPU architectures are defined in the [`vendor`] module.
///
/// This enum provides backward-compatible JSON deserialization for all GPU
/// architectures. The variants map to their respective vendor enums.
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum GpuArchitecture {
    /// NVIDIA Ada Lovelace microarchitecture (RTX 40xx)
    #[display("Ada Lovelace")]
    #[serde(
        alias = "Ada",
        alias = "ada",
        alias = "Ada Lovelace",
        alias = "ada lovelace",
        alias = "ada-lovelace",
        alias = "ada_lovelace"
    )]
    AdaLovelace,
    /// Intel Alchemist graphics architecture (Xe-HPG)
    #[display("Alchemist")]
    #[serde(alias = "alchemist", alias = "Arc", alias = "arc", alias = "DG2", alias = "dg2")]
    Alchemist,
    /// NVIDIA Ampere microarchitecture (A100, RTX 30xx)
    #[display("Ampere")]
    #[serde(alias = "ampere", alias = "GA100", alias = "ga100", alias = "GA102", alias = "ga102")]
    Ampere,
    /// Apple GPU architecture (A14/A15 Bionic)
    #[display("Apple GPU (A14/A15)")]
    #[serde(alias = "Apple GPU (A14/A15)", alias = "apple gpu (a14/a15)", alias = "A14", alias = "a14")]
    AppleA14,
    /// Apple GPU architecture (A16/A17 Bionic)
    #[display("Apple GPU (A16/A17)")]
    #[serde(alias = "Apple GPU (A16/A17)", alias = "apple gpu (a16/a17)", alias = "A16", alias = "a16")]
    AppleA16,
    /// Apple GPU architecture (A18/A18 Pro)
    #[display("Apple GPU (A18)")]
    #[serde(alias = "Apple GPU (A18)", alias = "apple gpu (a18)", alias = "A18", alias = "a18")]
    AppleA18,
    /// Apple GPU architecture (M1 series)
    #[display("Apple GPU (M1)")]
    #[serde(alias = "Apple GPU (M1)", alias = "apple gpu (m1)", alias = "M1", alias = "m1")]
    AppleM1,
    /// Apple GPU architecture (M2 series)
    #[display("Apple GPU (M2)")]
    #[serde(alias = "Apple GPU (M2)", alias = "apple gpu (m2)", alias = "M2", alias = "m2")]
    AppleM2,
    /// Apple GPU architecture (M3 series)
    #[display("Apple GPU (M3)")]
    #[serde(alias = "Apple GPU (M3)", alias = "apple gpu (m3)", alias = "M3", alias = "m3")]
    AppleM3,
    /// Apple GPU architecture (M4 series)
    #[display("Apple GPU (M4)")]
    #[serde(alias = "Apple GPU (M4)", alias = "apple gpu (m4)", alias = "M4", alias = "m4")]
    AppleM4,
    /// Apple GPU architecture (M5 series)
    #[display("Apple GPU (M5)")]
    #[serde(alias = "Apple GPU (M5)", alias = "apple gpu (m5)", alias = "M5", alias = "m5")]
    AppleM5,
    /// NVIDIA Blackwell microarchitecture (RTX 50xx, B100)
    #[display("Blackwell")]
    #[serde(alias = "blackwell", alias = "GB100", alias = "gb100", alias = "GB200", alias = "gb200")]
    Blackwell,
    /// AMD CDNA 1 microarchitecture (MI100)
    #[display("CDNA1")]
    #[serde(
        alias = "CDNA 1",
        alias = "cdna1",
        alias = "cdna 1",
        alias = "MI100",
        alias = "mi100",
        alias = "Arcturus",
        alias = "arcturus"
    )]
    Cdna1,
    /// AMD CDNA 2 microarchitecture (MI250)
    #[display("CDNA2")]
    #[serde(
        alias = "CDNA 2",
        alias = "cdna2",
        alias = "cdna 2",
        alias = "MI200",
        alias = "mi200",
        alias = "MI250",
        alias = "mi250",
        alias = "Aldebaran",
        alias = "aldebaran"
    )]
    Cdna2,
    /// AMD CDNA 3 microarchitecture (MI300)
    #[display("CDNA3")]
    #[serde(
        alias = "CDNA 3",
        alias = "cdna3",
        alias = "cdna 3",
        alias = "MI300",
        alias = "mi300",
        alias = "Aqua Vanjaram",
        alias = "aqua vanjaram"
    )]
    Cdna3,
    /// AMD CDNA 4 microarchitecture (MI350)
    #[display("CDNA4")]
    #[serde(alias = "CDNA 4", alias = "cdna4", alias = "cdna 4", alias = "MI350", alias = "mi350")]
    Cdna4,
    /// NVIDIA Fermi microarchitecture (Tesla 20xx, GeForce 4xx/5xx, 2010)
    ///
    /// Compute capability 2.x
    #[display("Fermi")]
    #[serde(alias = "fermi", alias = "GF100", alias = "gf100", alias = "GF110", alias = "gf110")]
    Fermi,
    /// NVIDIA Hopper microarchitecture (H100, H200)
    #[display("Hopper")]
    #[serde(alias = "hopper", alias = "GH100", alias = "gh100", alias = "GH200", alias = "gh200")]
    Hopper,
    /// NVIDIA Kepler GPU microarchitecture (GK-series, 2012)
    #[display("Kepler")]
    #[serde(alias = "kepler", alias = "GK110", alias = "gk110")]
    Kepler,
    /// NVIDIA Maxwell GPU microarchitecture (GM-series, 2014)
    #[display("Maxwell")]
    #[serde(alias = "maxwell", alias = "GM200", alias = "gm200")]
    Maxwell,
    /// NVIDIA Orin system-on-chip (Jetson AGX Orin, Orin NX, Orin Nano)
    ///
    /// Ampere-based GPU integrated in the Jetson Orin family. Compute capability 8.7
    #[display("Orin")]
    #[serde(
        alias = "orin",
        alias = "Jetson Orin",
        alias = "jetson orin",
        alias = "AGX Orin",
        alias = "agx orin",
        alias = "Orin NX",
        alias = "orin nx",
        alias = "Orin Nano",
        alias = "orin nano"
    )]
    Orin,
    /// NVIDIA Pascal GPU microarchitecture (GP-series, 2016)
    #[display("Pascal")]
    #[serde(alias = "pascal", alias = "GP100", alias = "gp100")]
    Pascal,
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
    /// NVIDIA Tesla microarchitecture (G80, GT200, pre-Fermi, 2006-2009)
    ///
    /// Compute capability 1.x. Covers G80, G92, G94, G96, G200, GT21x families
    #[display("Tesla")]
    #[serde(
        alias = "tesla",
        alias = "G80",
        alias = "g80",
        alias = "G92",
        alias = "g92",
        alias = "GT200",
        alias = "gt200",
        alias = "GT21x",
        alias = "gt21x"
    )]
    Tesla,
    /// Jetson Thor system-on-chip (Jetson T5000, T4000)
    ///
    /// Next-generation Jetson embedded GPU. Compute capability 11.0
    #[display("Thor")]
    #[serde(
        alias = "thor",
        alias = "Jetson Thor",
        alias = "jetson thor",
        alias = "Jetson T5000",
        alias = "jetson t5000",
        alias = "Jetson T4000",
        alias = "jetson t4000"
    )]
    Thor,
    /// NVIDIA Turing microarchitecture (RTX 20xx)
    #[display("Turing")]
    #[serde(alias = "turing", alias = "TU102", alias = "tu102")]
    Turing,
    /// NVIDIA Volta microarchitecture (V100)
    #[display("Volta")]
    #[serde(alias = "volta", alias = "GV100", alias = "gv100")]
    Volta,
    /// Intel Xe graphics architecture
    #[display("Xe")]
    #[serde(alias = "xe", alias = "Xe HPG", alias = "xe hpg", alias = "Xe-HPG", alias = "xe-hpg")]
    Xe,
    /// Intel Xe2 graphics architecture (Battlemage)
    #[display("Xe2")]
    #[serde(alias = "Battlemage", alias = "battlemage", alias = "xe2")]
    Xe2,
    /// Intel Xe3 graphics architecture (Celestial)
    #[display("Xe3")]
    #[serde(alias = "Celestial", alias = "celestial", alias = "xe3")]
    Xe3,
    /// Other or unspecified GPU architecture
    #[display("{}", _0)]
    Other(String),
}
/// Enumeration for hardware resources used by technology
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub enum Resource {
    /// Application-specific integrated circuit (ASIC)
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
        /// Whether this resource is required
        required: Option<bool>,
        /// Hardware vendor or designer
        vendor: Option<Vendor>,
    },
    /// Central processing unit (CPU) for "classical" computing
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
        /// Whether this resource is required
        required: Option<bool>,
        /// Number of threads required (if applicable)
        threads: Option<u32>,
        /// Hardware vendor (e.g., Intel, AMD, ARM)
        vendor: Option<Vendor>,
    },
    /// Data processing unit (DPU)
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
        /// Whether this resource is required
        required: Option<bool>,
        /// Hardware vendor (e.g., NVIDIA, Intel, AMD)
        vendor: Option<Vendor>,
    },
    /// Digital signal processor (DSP)
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
        /// Whether this resource is required
        required: Option<bool>,
        /// Hardware vendor (e.g., Qualcomm, Texas Instruments)
        vendor: Option<Vendor>,
    },
    /// Field-programmable gate array (FPGA)
    ///
    /// A reconfigurable integrated circuit that lets you implement custom digital hardware circuits after manufacturing, rather than having a fixed function like a CPU or ASIC
    ///
    /// See [`FpgaArchitecture`] for supported architecture families.
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
        /// Whether this resource is required
        required: Option<bool>,
        /// Hardware vendor (e.g., Intel, AMD)
        vendor: Option<Vendor>,
    },
    /// Graphics processing unit (GPU)
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
        /// Whether this resource is required
        required: Option<bool>,
        /// Hardware vendor (e.g., NVIDIA, AMD, Intel)
        vendor: Option<Vendor>,
    },
    /// Neural processing unit (NPU)
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
        /// Whether this resource is required
        required: Option<bool>,
        /// Hardware vendor (e.g., Qualcomm, Apple, Intel)
        vendor: Option<Vendor>,
    },
    /// Neuromorphic compute
    Neuromorphic {
        /// Number of neuromorphic chips required (if applicable)
        count: Option<u32>,
        /// On-chip memory (if applicable)
        #[serde(alias = "mem")]
        memory: Option<Memory>,
        /// Chip model name (e.g., "Loihi 2", "TrueNorth", "SpiNNaker2")
        model: Option<String>,
        /// Neuron capacity per chip (if applicable)
        neurons: Option<u64>,
        /// Whether this resource is required
        required: Option<bool>,
        /// Synapse capacity per chip (if applicable)
        synapses: Option<u64>,
        /// Hardware vendor (e.g., Intel, IBM)
        vendor: Option<Vendor>,
    },
    /// Quantum computing (e.g., NISQ, etc.)
    Quantum {
        /// Number of QPUs required (if applicable)
        count: Option<u32>,
        /// Chip or system model name (e.g., "Eagle", "Heron", "Sycamore")
        model: Option<Model>,
        /// Computing paradigm or modality
        paradigm: Option<Paradigm>,
        /// Number of qubits (physical or logical, as applicable)
        qubits: Option<u32>,
        /// Whether this resource is required
        required: Option<bool>,
        /// Qubit connectivity topology (e.g., "heavy-hex", "all-to-all")
        topology: Option<Topology>,
        /// Hardware vendor (e.g., IBM, Google, IonQ)
        vendor: Option<Vendor>,
    },
    /// Sensor hardware for data acquisition and measurement
    Sensor {
        /// Number of sensors required (if applicable)
        count: Option<u32>,
        /// Sensor modalities (e.g., image, lidar, radar, temperature)
        modality: Option<Vec<SensorModality>>,
        /// Sensor model name (e.g., "FLIR Boson", "Velodyne VLP-16")
        model: Option<String>,
        /// Quantum sensing regime when the sensor relies on quantum effects
        #[serde(alias = "regime", alias = "quantum", alias = "quantumRegime", alias = "quantum regime")]
        quantum_regime: Option<Regime>,
        /// Whether this resource is required
        required: Option<bool>,
        /// Sampling rate in Hz (if applicable)
        #[serde(alias = "samplingRate", alias = "sampling_rate")]
        sampling_rate: Option<f32>,
        /// Hardware vendor (e.g., FLIR, Velodyne, Bosch)
        vendor: Option<Vendor>,
    },
    /// Tensor processing unit (TPU)
    TPU {
        /// TPU generation
        #[serde(alias = "arch")]
        architecture: Option<AcceleratorArchitecture>,
        /// Number of TPU chips required (if applicable)
        count: Option<u32>,
        /// High-bandwidth memory (if applicable)
        #[serde(alias = "mem")]
        memory: Option<Memory>,
        /// Whether this resource is required
        required: Option<bool>,
        /// Hardware vendor (e.g., Google)
        vendor: Option<Vendor>,
    },
    /// Unknown, unspecified, or otherwise unclassified resource
    Other(String),
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
impl From<CpuArchitecture> for Architecture {
    fn from(value: CpuArchitecture) -> Self {
        match value {
            | CpuArchitecture::AArch64 => Architecture::AArch64,
            | CpuArchitecture::Carmel => Architecture::Carmel,
            | CpuArchitecture::Ethos => Architecture::Ethos,
            | CpuArchitecture::Grace => Architecture::Grace,
            | CpuArchitecture::Ia64 => Architecture::Ia64,
            | CpuArchitecture::LoongArch => Architecture::LoongArch,
            | CpuArchitecture::Mips => Architecture::Mips,
            | CpuArchitecture::NeuralEngine => Architecture::NeuralEngine,
            | CpuArchitecture::PerformanceCore => Architecture::ApplePerformanceCore,
            | CpuArchitecture::EfficiencyCore => Architecture::AppleEfficiencyCore,
            | CpuArchitecture::PowerPc => Architecture::PowerPc,
            | CpuArchitecture::Hexagon => Architecture::Hexagon,
            | CpuArchitecture::RiscV => Architecture::RiscV,
            | CpuArchitecture::Vera => Architecture::Vera,
            | CpuArchitecture::X86 => Architecture::X86,
            | CpuArchitecture::X86_64 => Architecture::X86_64,
            | CpuArchitecture::ZArch => Architecture::ZArch,
            | CpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<&CpuArchitecture> for Architecture {
    fn from(value: &CpuArchitecture) -> Self {
        match value {
            | CpuArchitecture::AArch64 => Architecture::AArch64,
            | CpuArchitecture::Carmel => Architecture::Carmel,
            | CpuArchitecture::Ethos => Architecture::Ethos,
            | CpuArchitecture::Grace => Architecture::Grace,
            | CpuArchitecture::Ia64 => Architecture::Ia64,
            | CpuArchitecture::LoongArch => Architecture::LoongArch,
            | CpuArchitecture::Mips => Architecture::Mips,
            | CpuArchitecture::NeuralEngine => Architecture::NeuralEngine,
            | CpuArchitecture::PerformanceCore => Architecture::ApplePerformanceCore,
            | CpuArchitecture::EfficiencyCore => Architecture::AppleEfficiencyCore,
            | CpuArchitecture::PowerPc => Architecture::PowerPc,
            | CpuArchitecture::Hexagon => Architecture::Hexagon,
            | CpuArchitecture::RiscV => Architecture::RiscV,
            | CpuArchitecture::Vera => Architecture::Vera,
            | CpuArchitecture::X86 => Architecture::X86,
            | CpuArchitecture::X86_64 => Architecture::X86_64,
            | CpuArchitecture::ZArch => Architecture::ZArch,
            | CpuArchitecture::Other(s) => Architecture::Other(s.clone()),
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
impl From<&DspArchitecture> for Architecture {
    fn from(value: &DspArchitecture) -> Self {
        match value {
            | DspArchitecture::C66x => Architecture::C66x,
            | DspArchitecture::Xtensa => Architecture::Xtensa,
            | DspArchitecture::Other(s) => Architecture::Other(s.clone()),
        }
    }
}
impl From<AcceleratorArchitecture> for Architecture {
    fn from(value: AcceleratorArchitecture) -> Self {
        match value {
            | AcceleratorArchitecture::Ascend910 => Architecture::Ascend910,
            | AcceleratorArchitecture::Blackhole => Architecture::Blackhole,
            | AcceleratorArchitecture::Gaudi2 => Architecture::Gaudi2,
            | AcceleratorArchitecture::GraphcoreGc200 => Architecture::GraphcoreGc200,
            | AcceleratorArchitecture::Inferentia => Architecture::Inferentia,
            | AcceleratorArchitecture::Lpu => Architecture::Lpu,
            | AcceleratorArchitecture::Maia => Architecture::Maia,
            | AcceleratorArchitecture::Nervana => Architecture::Nervana,
            | AcceleratorArchitecture::SambaNovaRdu => Architecture::SambaNovaRdu,
            | AcceleratorArchitecture::TpuV4 => Architecture::TpuV4,
            | AcceleratorArchitecture::TpuV5e => Architecture::TpuV5e,
            | AcceleratorArchitecture::TpuV5p => Architecture::TpuV5p,
            | AcceleratorArchitecture::TpuV6e => Architecture::TpuV6e,
            | AcceleratorArchitecture::Trainium => Architecture::Trainium,
            | AcceleratorArchitecture::Wormhole => Architecture::Wormhole,
            | AcceleratorArchitecture::Wse2 => Architecture::Wse2,
            | AcceleratorArchitecture::Wse3 => Architecture::Wse3,
            | AcceleratorArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<&AcceleratorArchitecture> for Architecture {
    fn from(value: &AcceleratorArchitecture) -> Self {
        match value {
            | AcceleratorArchitecture::Ascend910 => Architecture::Ascend910,
            | AcceleratorArchitecture::Blackhole => Architecture::Blackhole,
            | AcceleratorArchitecture::Gaudi2 => Architecture::Gaudi2,
            | AcceleratorArchitecture::GraphcoreGc200 => Architecture::GraphcoreGc200,
            | AcceleratorArchitecture::Inferentia => Architecture::Inferentia,
            | AcceleratorArchitecture::Lpu => Architecture::Lpu,
            | AcceleratorArchitecture::Maia => Architecture::Maia,
            | AcceleratorArchitecture::Nervana => Architecture::Nervana,
            | AcceleratorArchitecture::SambaNovaRdu => Architecture::SambaNovaRdu,
            | AcceleratorArchitecture::TpuV4 => Architecture::TpuV4,
            | AcceleratorArchitecture::TpuV5e => Architecture::TpuV5e,
            | AcceleratorArchitecture::TpuV5p => Architecture::TpuV5p,
            | AcceleratorArchitecture::TpuV6e => Architecture::TpuV6e,
            | AcceleratorArchitecture::Trainium => Architecture::Trainium,
            | AcceleratorArchitecture::Wormhole => Architecture::Wormhole,
            | AcceleratorArchitecture::Wse2 => Architecture::Wse2,
            | AcceleratorArchitecture::Wse3 => Architecture::Wse3,
            | AcceleratorArchitecture::Other(s) => Architecture::Other(s.clone()),
        }
    }
}
impl From<FpgaArchitecture> for Architecture {
    fn from(value: FpgaArchitecture) -> Self {
        match value {
            | FpgaArchitecture::Agilex => Architecture::Agilex,
            | FpgaArchitecture::Artix => Architecture::Artix,
            | FpgaArchitecture::CrossLink => Architecture::CrossLink,
            | FpgaArchitecture::CycloneV => Architecture::CycloneV,
            | FpgaArchitecture::Ecp5 => Architecture::Ecp5,
            | FpgaArchitecture::Ice40 => Architecture::Ice40,
            | FpgaArchitecture::Kintex => Architecture::Kintex,
            | FpgaArchitecture::Spartan => Architecture::Spartan,
            | FpgaArchitecture::Stratix10 => Architecture::Stratix10,
            | FpgaArchitecture::StratixV => Architecture::StratixV,
            | FpgaArchitecture::Versal => Architecture::Versal,
            | FpgaArchitecture::VirtexUltraScalePlus => Architecture::VirtexUltraScalePlus,
            | FpgaArchitecture::Zynq => Architecture::Zynq,
            | FpgaArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<&FpgaArchitecture> for Architecture {
    fn from(value: &FpgaArchitecture) -> Self {
        match value {
            | FpgaArchitecture::Agilex => Architecture::Agilex,
            | FpgaArchitecture::Artix => Architecture::Artix,
            | FpgaArchitecture::CrossLink => Architecture::CrossLink,
            | FpgaArchitecture::CycloneV => Architecture::CycloneV,
            | FpgaArchitecture::Ecp5 => Architecture::Ecp5,
            | FpgaArchitecture::Ice40 => Architecture::Ice40,
            | FpgaArchitecture::Kintex => Architecture::Kintex,
            | FpgaArchitecture::Spartan => Architecture::Spartan,
            | FpgaArchitecture::Stratix10 => Architecture::Stratix10,
            | FpgaArchitecture::StratixV => Architecture::StratixV,
            | FpgaArchitecture::Versal => Architecture::Versal,
            | FpgaArchitecture::VirtexUltraScalePlus => Architecture::VirtexUltraScalePlus,
            | FpgaArchitecture::Zynq => Architecture::Zynq,
            | FpgaArchitecture::Other(s) => Architecture::Other(s.clone()),
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
            | GpuArchitecture::Cdna4 => Architecture::Cdna4,
            | GpuArchitecture::Fermi => Architecture::Fermi,
            | GpuArchitecture::Hopper => Architecture::Hopper,
            | GpuArchitecture::Kepler => Architecture::Kepler,
            | GpuArchitecture::Maxwell => Architecture::Maxwell,
            | GpuArchitecture::Orin => Architecture::Orin,
            | GpuArchitecture::Pascal => Architecture::Pascal,
            | GpuArchitecture::Rdna1 => Architecture::Rdna1,
            | GpuArchitecture::Rdna2 => Architecture::Rdna2,
            | GpuArchitecture::Rdna3 => Architecture::Rdna3,
            | GpuArchitecture::Tesla => Architecture::Tesla,
            | GpuArchitecture::Thor => Architecture::Thor,
            | GpuArchitecture::Turing => Architecture::Turing,
            | GpuArchitecture::Volta => Architecture::Volta,
            | GpuArchitecture::Xe => Architecture::Xe,
            | GpuArchitecture::Xe2 => Architecture::Xe2,
            | GpuArchitecture::Xe3 => Architecture::Xe3,
            | GpuArchitecture::AppleA14 => Architecture::AppleA14,
            | GpuArchitecture::AppleA16 => Architecture::AppleA16,
            | GpuArchitecture::AppleA18 => Architecture::AppleA18,
            | GpuArchitecture::AppleM1 => Architecture::AppleM1,
            | GpuArchitecture::AppleM2 => Architecture::AppleM2,
            | GpuArchitecture::AppleM3 => Architecture::AppleM3,
            | GpuArchitecture::AppleM4 => Architecture::AppleM4,
            | GpuArchitecture::AppleM5 => Architecture::AppleM5,
            | GpuArchitecture::Other(s) => Architecture::Other(s),
        }
    }
}
impl From<&GpuArchitecture> for Architecture {
    fn from(value: &GpuArchitecture) -> Self {
        match value {
            | GpuArchitecture::AdaLovelace => Architecture::AdaLovelace,
            | GpuArchitecture::Alchemist => Architecture::Alchemist,
            | GpuArchitecture::Ampere => Architecture::Ampere,
            | GpuArchitecture::Blackwell => Architecture::Blackwell,
            | GpuArchitecture::Cdna1 => Architecture::Cdna1,
            | GpuArchitecture::Cdna2 => Architecture::Cdna2,
            | GpuArchitecture::Cdna3 => Architecture::Cdna3,
            | GpuArchitecture::Cdna4 => Architecture::Cdna4,
            | GpuArchitecture::Fermi => Architecture::Fermi,
            | GpuArchitecture::Hopper => Architecture::Hopper,
            | GpuArchitecture::Kepler => Architecture::Kepler,
            | GpuArchitecture::Maxwell => Architecture::Maxwell,
            | GpuArchitecture::Orin => Architecture::Orin,
            | GpuArchitecture::Pascal => Architecture::Pascal,
            | GpuArchitecture::Rdna1 => Architecture::Rdna1,
            | GpuArchitecture::Rdna2 => Architecture::Rdna2,
            | GpuArchitecture::Rdna3 => Architecture::Rdna3,
            | GpuArchitecture::Tesla => Architecture::Tesla,
            | GpuArchitecture::Thor => Architecture::Thor,
            | GpuArchitecture::Turing => Architecture::Turing,
            | GpuArchitecture::Volta => Architecture::Volta,
            | GpuArchitecture::Xe => Architecture::Xe,
            | GpuArchitecture::Xe2 => Architecture::Xe2,
            | GpuArchitecture::Xe3 => Architecture::Xe3,
            | GpuArchitecture::AppleA14 => Architecture::AppleA14,
            | GpuArchitecture::AppleA16 => Architecture::AppleA16,
            | GpuArchitecture::AppleA18 => Architecture::AppleA18,
            | GpuArchitecture::AppleM1 => Architecture::AppleM1,
            | GpuArchitecture::AppleM2 => Architecture::AppleM2,
            | GpuArchitecture::AppleM3 => Architecture::AppleM3,
            | GpuArchitecture::AppleM4 => Architecture::AppleM4,
            | GpuArchitecture::AppleM5 => Architecture::AppleM5,
            | GpuArchitecture::Other(s) => Architecture::Other(s.clone()),
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
impl From<&SensorModality> for Modality {
    fn from(value: &SensorModality) -> Self {
        match value {
            | SensorModality::Audio => Modality::Audio,
            | SensorModality::Image | SensorModality::Event | SensorModality::Depth => Modality::Video,
            | SensorModality::Lidar
            | SensorModality::Radar
            | SensorModality::Radio
            | SensorModality::NetworkTraffic
            | SensorModality::Webhook
            | SensorModality::Hyperspectral
            | SensorModality::Seismic
            | SensorModality::ServerSideEvent
            | SensorModality::Sonar
            | SensorModality::Inertial
            | SensorModality::Magnetic
            | SensorModality::Navigation
            | SensorModality::Pressure
            | SensorModality::Temperature
            | SensorModality::Ultrasonic
            | SensorModality::Chemical
            | SensorModality::Other(_) => Modality::Signal,
        }
    }
}
impl From<SensorModality> for Modality {
    fn from(value: SensorModality) -> Self {
        (&value).into()
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
                        required: None,
                        vendor: None,
                    }),
                    | "CPU" | "cpu" => Ok(Resource::CPU {
                        architecture: None,
                        cores: None,
                        count: Some(1),
                        memory: None,
                        required: None,
                        threads: None,
                        vendor: None,
                    }),
                    | "DPU" | "dpu" => Ok(Resource::DPU {
                        architecture: None,
                        count: Some(1),
                        memory: None,
                        required: None,
                        vendor: None,
                    }),
                    | "DSP" | "dsp" => Ok(Resource::DSP {
                        architecture: None,
                        count: Some(1),
                        frequency: None,
                        required: None,
                        vendor: None,
                    }),
                    | "FPGA" | "fpga" => Ok(Resource::FPGA {
                        architecture: None,
                        count: Some(1),
                        logic_elements: None,
                        memory: None,
                        required: None,
                        vendor: None,
                    }),
                    | "GPU" | "gpu" => Ok(Resource::GPU {
                        architecture: None,
                        backend: None,
                        compute_capability: None,
                        count: Some(1),
                        memory: None,
                        name: None,
                        required: None,
                        vendor: None,
                    }),
                    | "NPU" | "npu" => Ok(Resource::NPU {
                        architecture: None,
                        count: Some(1),
                        memory: None,
                        required: None,
                        vendor: None,
                    }),
                    | "Neuromorphic" | "neuromorphic" => Ok(Resource::Neuromorphic {
                        count: Some(1),
                        memory: None,
                        model: None,
                        neurons: None,
                        required: None,
                        synapses: None,
                        vendor: None,
                    }),
                    | "Quantum" | "quantum" => Ok(Resource::Quantum {
                        count: Some(1),
                        model: None,
                        paradigm: None,
                        qubits: None,
                        required: None,
                        topology: None,
                        vendor: None,
                    }),
                    | "Sensor" | "sensor" => Ok(Resource::Sensor {
                        count: Some(1),
                        modality: None,
                        model: None,
                        quantum_regime: None,
                        sampling_rate: None,
                        required: None,
                        vendor: None,
                    }),
                    | "TPU" | "tpu" => Ok(Resource::TPU {
                        architecture: None,
                        count: Some(1),
                        memory: None,
                        required: None,
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
                                required: Option<bool>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::ASIC {
                                    architecture: f.architecture,
                                    count: f.count,
                                    purpose: f.purpose,
                                    required: f.required,
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
                                required: Option<bool>,
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
                                    required: f.required,
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
                                required: Option<bool>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::DPU {
                                    architecture: f.architecture,
                                    count: f.count,
                                    memory: f.memory,
                                    required: f.required,
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
                                required: Option<bool>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::DSP {
                                    architecture: f.architecture,
                                    count: f.count,
                                    frequency: f.frequency,
                                    required: f.required,
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
                                required: Option<bool>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::FPGA {
                                    architecture: f.architecture,
                                    count: f.count,
                                    logic_elements: f.logic_elements,
                                    memory: f.memory,
                                    required: f.required,
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
                                required: Option<bool>,
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
                                    required: f.required,
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
                                required: Option<bool>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::NPU {
                                    architecture: f.architecture,
                                    count: f.count,
                                    memory: f.memory,
                                    required: f.required,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "Neuromorphic" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                #[serde(alias = "mem")]
                                memory: Option<Memory>,
                                model: Option<String>,
                                neurons: Option<u64>,
                                required: Option<bool>,
                                synapses: Option<u64>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::Neuromorphic {
                                    count: f.count,
                                    memory: f.memory,
                                    model: f.model,
                                    neurons: f.neurons,
                                    required: f.required,
                                    synapses: f.synapses,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "Quantum" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                model: Option<Model>,
                                paradigm: Option<Paradigm>,
                                qubits: Option<u32>,
                                required: Option<bool>,
                                topology: Option<Topology>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::Quantum {
                                    count: f.count,
                                    model: f.model,
                                    paradigm: f.paradigm,
                                    qubits: f.qubits,
                                    required: f.required,
                                    topology: f.topology,
                                    vendor: f.vendor,
                                }),
                                | Err(why) => Err(why),
                            }
                        }
                        | "Sensor" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                modality: Option<Vec<SensorModality>>,
                                model: Option<String>,
                                #[serde(alias = "quantumRegime", alias = "quantum regime")]
                                quantum_regime: Option<Regime>,
                                #[serde(alias = "samplingRateHz", alias = "sampling_rate")]
                                sampling_rate_hz: Option<f32>,
                                required: Option<bool>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::Sensor {
                                    count: f.count,
                                    modality: f.modality,
                                    model: f.model,
                                    quantum_regime: f.quantum_regime,
                                    sampling_rate: f.sampling_rate_hz,
                                    required: f.required,
                                    vendor: f.vendor,
                                }),
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
                                required: Option<bool>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::TPU {
                                    architecture: f.architecture,
                                    count: f.count,
                                    memory: f.memory,
                                    required: f.required,
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
impl Validate for Resource {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate_vendor(self)
    }
}
impl Resource {
    fn is_spark(&self) -> bool {
        match self {
            | Resource::GPU {
                name: Some(name),
                architecture: Some(architecture),
                ..
            } => {
                let value = name.to_lowercase();
                *architecture == GpuArchitecture::Blackwell && ["spark", "gb10", "dgx spark"].iter().any(|term| value.contains(term))
            }
            | _ => false,
        }
    }
    fn name(&self) -> Option<String> {
        match self {
            | Resource::GPU { name: Some(name), .. } => Some(name.to_lowercase().replace(' ', "")),
            | _ => None,
        }
    }
    /// Returns compute value (major, minor) which defines the hardware features and supported instructions for each NVIDIA GPU architecture
    ///
    /// Values come from the official [NVIDIA CUDA GPU compute capability page](https://developer.nvidia.com/cuda/gpus) (and [this page](https://developer.nvidia.com/cuda/gpus/legacy) for legacy GPUs)
    ///
    /// Returns `None` for non-NVIDIA GPUs or unrecognized models.
    #[allow(dead_code)]
    pub fn compute_value(&self) -> Option<(u8, u8)> {
        match &self.vendor() {
            | Some(Vendor::Nvidia) => match self.architecture() {
                | Some(Architecture::Blackwell) => {
                    if self.is_spark() {
                        Some((12, 1))
                    } else if let Some(v) = self.name() {
                        if contains_any(NVIDIA_CC_BLACKWELL_103_PATTERNS, &v) {
                            Some((10, 3))
                        } else if contains_any(NVIDIA_CC_BLACKWELL_100_PATTERNS, &v) {
                            Some((10, 0))
                        } else {
                            Some((12, 0))
                        }
                    } else {
                        Some((12, 0))
                    }
                }
                | Some(Architecture::Hopper) => Some((9, 0)),
                | Some(Architecture::AdaLovelace) => Some((8, 9)),
                | Some(Architecture::Thor) => Some((11, 0)),
                | Some(Architecture::Orin) => Some((8, 7)),
                | Some(Architecture::Ampere) => {
                    if let Some(v) = self.name() {
                        if contains_any(NVIDIA_CC_AMPERE_80_PATTERNS, &v) {
                            Some((8, 0))
                        } else {
                            Some((8, 6))
                        }
                    } else {
                        Some((8, 6))
                    }
                }
                | Some(Architecture::Turing) => Some((7, 5)),
                | Some(Architecture::Volta) => {
                    if let Some(v) = self.name() {
                        if v.contains("xavier") {
                            Some((7, 2))
                        } else {
                            Some((7, 0))
                        }
                    } else {
                        Some((7, 0))
                    }
                }
                | Some(Architecture::Pascal) => {
                    if let Some(v) = self.name() {
                        if v.contains("tx2") {
                            Some((6, 2))
                        } else if contains_any(NVIDIA_CC_PASCAL_60_PATTERNS, &v) {
                            Some((6, 0))
                        } else {
                            Some((6, 1))
                        }
                    } else {
                        Some((6, 1))
                    }
                }
                | Some(Architecture::Maxwell) => {
                    if let Some(v) = self.name() {
                        if v.contains("jetson") && v.contains("nano") {
                            Some((5, 3))
                        } else if contains_any(NVIDIA_CC_MAXWELL_52_PATTERNS, &v) {
                            Some((5, 2))
                        } else if contains_any(NVIDIA_CC_MAXWELL_50_PATTERNS, &v) || (v.contains("gtx") && v.contains("750")) {
                            Some((5, 0))
                        } else {
                            Some((5, 2))
                        }
                    } else {
                        Some((5, 2))
                    }
                }
                | Some(Architecture::Kepler) => {
                    if let Some(v) = self.name() {
                        if v.contains("k80") {
                            Some((3, 7))
                        } else if contains_any(NVIDIA_CC_KEPLER_35_PATTERNS, &v) {
                            Some((3, 5))
                        } else if contains_any(NVIDIA_CC_KEPLER_32_PATTERNS, &v) {
                            Some((3, 2))
                        } else {
                            Some((3, 0))
                        }
                    } else {
                        Some((3, 0))
                    }
                }
                | Some(Architecture::Fermi) => {
                    if let Some(v) = self.name() {
                        if contains_any(NVIDIA_CC_FERMI_21_PATTERNS, &v)
                            || contains_any_with_prefix(&v, "quadro", NVIDIA_CC_FERMI_21_QUADRO_SUFFIXES)
                            || contains_any_with_prefix(&v, "gt", NVIDIA_CC_FERMI_21_GT_SUFFIXES)
                        {
                            Some((2, 1))
                        } else {
                            Some((2, 0))
                        }
                    } else {
                        Some((2, 0))
                    }
                }
                | Some(Architecture::Tesla) => {
                    if let Some(v) = self.name() {
                        if contains_any(NVIDIA_CC_TESLA_13_PATTERNS, &v) || contains_any_with_prefix(&v, "quadro", NVIDIA_CC_TESLA_13_QUADRO_SUFFIXES)
                        {
                            Some((1, 3))
                        } else if contains_any(NVIDIA_CC_TESLA_12_PATTERNS, &v)
                            || contains_any_with_prefix(&v, "quadro", NVIDIA_CC_TESLA_12_QUADRO_SUFFIXES)
                            || contains_any_with_prefix(&v, "gt", NVIDIA_CC_TESLA_12_GT_SUFFIXES)
                        {
                            Some((1, 2))
                        } else if contains_any(NVIDIA_CC_TESLA_11_PATTERNS, &v)
                            || contains_any_with_prefix(&v, "quadro", NVIDIA_CC_TESLA_11_QUADRO_SUFFIXES)
                            || contains_any_with_prefix(&v, "gts", NVIDIA_CC_TESLA_11_GTS_SUFFIXES)
                            || contains_any_with_prefix(&v, "gt", NVIDIA_CC_TESLA_11_GT_SUFFIXES)
                        {
                            Some((1, 1))
                        } else {
                            Some((1, 0))
                        }
                    } else {
                        Some((1, 0))
                    }
                }
                | _ => None,
            },
            | _ => None,
        }
    }
}
fn default_count() -> Option<u32> {
    Some(1)
}

#[cfg(test)]
mod tests;
