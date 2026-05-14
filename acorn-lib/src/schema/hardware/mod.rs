//! Hardware resource, architecture, and vendor types
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Memory types and (de)serialization for hardware resources
pub mod memory;
/// Vendor-specific hardware architecture types
pub mod vendor;
pub use memory::{Memory, MemoryUnit};
use vendor::{
    AmdCpuArchitecture, AmdGpuArchitecture, AppleCpuArchitecture, AppleGpuArchitecture, IntelCpuArchitecture, IntelGpuArchitecture,
    NvidiaGpuArchitecture,
};

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
    /// Xilinx Artix [`Resource::FPGA`] family
    /// > Cost and transceiver optimized
    ///
    /// See <https://www.amd.com/en/products/adaptive-socs-and-fpgas/fpga/artix-7.html> for more information
    #[display("Artix")]
    Artix,
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
    /// Intel Habana Gaudi2 AI accelerator
    ///
    /// See <[The Habana Gaudi2 Processor for Deep Learning](https://www.intel.com/content/www/us/en/developer/articles/technical/habana-gaudi2-processor-for-deep-learning.html) for more information
    #[display("Gaudi2")]
    Gaudi2,
    /// Graphcore Colossus GC200 / IPU architecture (2018-2019)
    #[display("Graphcore GC200")]
    GraphcoreGc200,
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
    /// Apple Neural Engine
    ///
    /// See [Apple Machine Learning Research - The Apple Neural Engine](https://machinelearning.apple.com/research/neural-engine-transformers#the-apple-neuralengine) for more information
    #[display("Neural Engine")]
    NeuralEngine,
    /// Intel Nervana neural processor (NNP-I / NNP-T)
    ///
    /// See <https://www.intel.com/content/www/us/en/homepage.html> for more information
    #[display("Nervana")]
    Nervana,
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
/// CPU instruction set architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum CpuArchitecture {
    /// 64-bit ARM instruction set
    #[display("AArch64")]
    #[serde(alias = "aarch64", alias = "arm64")]
    AArch64,
    /// ARM Ethos NPU (integrated in some ARM CPUs)
    #[display("Ethos")]
    #[serde(alias = "ethos")]
    Ethos,
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
    /// IBM PowerPC architecture
    #[display("PowerPC")]
    #[serde(alias = "powerpc")]
    PowerPc,
    /// Qualcomm Hexagon processor (integrated in Snapdragon)
    #[display("Hexagon")]
    #[serde(alias = "hexagon")]
    Hexagon,
    /// RISC-V open instruction set architecture
    #[display("RISC-V")]
    #[serde(alias = "riscv", alias = "risc-v")]
    RiscV,
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
/// Quantum computing paradigm or modality
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum QuantumParadigm {
    /// Gate-based quantum computing (e.g., IBM, Google)
    #[display("Gate-based")]
    #[serde(alias = "gate-based", alias = "gate_based", alias = "GateBased")]
    GateBased,
    /// Quantum annealing (e.g., D-Wave)
    #[display("Annealing")]
    Annealing,
    /// Photonic quantum computing (e.g., PsiQuantum, Xanadu)
    #[display("Photonic")]
    Photonic,
    /// Neutral atom quantum computing (e.g., QuEra, Pasqal)
    #[display("Neutral Atom")]
    #[serde(alias = "Neutral Atom", alias = "neutral_atom", alias = "NeutralAtom")]
    NeutralAtom,
    /// Trapped ion quantum computing (e.g., IonQ, Honeywell)
    #[display("Trapped Ion")]
    #[serde(alias = "Trapped Ion", alias = "trapped_ion", alias = "TrappedIon")]
    TrappedIon,
    /// Other or unspecified quantum paradigm
    #[display("{}", _0)]
    Other(String),
}
/// GPU graphics microarchitectures
///
/// Vendor-specific GPU architectures are defined in the `vendor` module:
/// - [`vendor::NvidiaGpuArchitecture`] for NVIDIA GPUs
/// - [`vendor::AmdGpuArchitecture`] for AMD GPUs
/// - [`vendor::IntelGpuArchitecture`] for Intel GPUs
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
        model: Option<String>,
        /// Computing paradigm or modality
        paradigm: Option<QuantumParadigm>,
        /// Number of qubits (physical or logical, as applicable)
        qubits: Option<u32>,
        /// Qubit connectivity topology (e.g., "heavy-hex", "all-to-all")
        topology: Option<String>,
        /// Hardware vendor (e.g., IBM, Google, IonQ)
        vendor: Option<Vendor>,
    },
    /// Tensor processing unit (NPU)
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
    /// BrainChip Holdings (neuromorphic AI)
    #[display("BrainChip")]
    BrainChip,
    /// Broadcom Inc.
    #[display("Broadcom")]
    Broadcom,
    /// D-Wave Systems (quantum annealing)
    #[display("D-Wave")]
    DWave,
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
    NVIDIA,
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
    /// Other or unspecified vendor
    #[display("{}", _0)]
    Other(String),
}
impl Architecture {
    fn from_str(s: &str) -> Self {
        let normalized = s.trim();
        let quoted = format!("\"{normalized}\"");
        [
            serde_json::from_str::<CpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<AmdCpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<AppleCpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<IntelCpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<GpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<AmdGpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<AppleGpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<IntelGpuArchitecture>(&quoted).ok().map(Architecture::from),
            serde_json::from_str::<NvidiaGpuArchitecture>(&quoted).ok().map(Architecture::from),
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
            | GpuArchitecture::Hopper => Architecture::Hopper,
            | GpuArchitecture::Kepler => Architecture::Kepler,
            | GpuArchitecture::Maxwell => Architecture::Maxwell,
            | GpuArchitecture::Pascal => Architecture::Pascal,
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
                    | "Neuromorphic" | "neuromorphic" => Ok(Resource::Neuromorphic {
                        count: Some(1),
                        memory: None,
                        model: None,
                        neurons: None,
                        synapses: None,
                        vendor: None,
                    }),
                    | "Quantum" | "quantum" => Ok(Resource::Quantum {
                        count: Some(1),
                        model: None,
                        paradigm: None,
                        qubits: None,
                        topology: None,
                        vendor: None,
                    }),
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
                        | "Neuromorphic" => {
                            #[derive(Deserialize)]
                            struct F {
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                #[serde(alias = "mem")]
                                memory: Option<Memory>,
                                model: Option<String>,
                                neurons: Option<u64>,
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
                                model: Option<String>,
                                paradigm: Option<QuantumParadigm>,
                                qubits: Option<u32>,
                                topology: Option<String>,
                                vendor: Option<Vendor>,
                            }
                            let next_value_result: Result<F, M::Error> = map.next_value();
                            match next_value_result {
                                | Ok(f) => Ok(Resource::Quantum {
                                    count: f.count,
                                    model: f.model,
                                    paradigm: f.paradigm,
                                    qubits: f.qubits,
                                    topology: f.topology,
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
fn default_count() -> Option<u32> {
    Some(1)
}

#[cfg(test)]
mod tests;
