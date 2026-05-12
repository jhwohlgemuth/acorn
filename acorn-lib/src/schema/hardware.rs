//! Hardware resource, architecture, and vendor types
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Hardware architecture or microarchitecture identifier
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
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
    #[serde(alias = "x86_64", alias = "x86-64", alias = "amd64")]
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
/// Enumeration for hardware resources used by technology
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub enum Resource {
    /// Application-specific integrated circuit
    ///
    /// A custom-designed chip built to perform one particular function or a narrow set of functions, rather than acting as a general-purpose processor like a CPU or GPU
    ASIC {
        /// ASIC architecture or design family
        #[serde(alias = "arch")]
        architecture: Option<Architecture>,
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
        architecture: Option<Architecture>,
        /// Number of physical cores required (if applicable)
        cores: Option<u32>,
        /// Number of CPUs required (if applicable)
        count: Option<u32>,
        /// Minimum RAM in GB (if applicable)
        #[serde(alias = "ram")]
        memory: Option<u32>,
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
        architecture: Option<Architecture>,
        /// Number of DPUs required (if applicable)
        count: Option<u32>,
        /// On-chip memory in GB (if applicable)
        #[serde(alias = "mem")]
        memory: Option<u32>,
        /// Hardware vendor (e.g., NVIDIA, Intel, AMD)
        vendor: Option<Vendor>,
    },
    /// Digital signal processor
    ///
    /// A specialized microprocessor optimized to perform fast mathematical operations on digital signals in real time
    DSP {
        /// DSP architecture
        #[serde(alias = "arch")]
        architecture: Option<Architecture>,
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
        architecture: Option<Architecture>,
        /// Number of FPGAs required (if applicable)
        count: Option<u32>,
        /// Number of logic elements or LUTs available
        logic_elements: Option<u32>,
        /// Block RAM in KB (if applicable)
        #[serde(alias = "mem")]
        memory: Option<u32>,
        /// Hardware vendor (e.g., Intel, AMD)
        vendor: Option<Vendor>,
    },
    /// Graphics processing unit
    GPU {
        /// GPU microarchitecture
        #[serde(alias = "arch")]
        architecture: Option<Architecture>,
        /// Compute backend or API
        backend: Option<Backend>,
        /// NVIDIA CUDA compute capability (e.g., 8.0 for A100)
        compute_capability: Option<f32>,
        /// Number of GPUs required (if applicable)
        count: Option<u32>,
        /// VRAM in GB (if applicable)
        #[serde(alias = "vram")]
        memory: Option<u32>,
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
        architecture: Option<Architecture>,
        /// Number of NPUs required (if applicable)
        count: Option<u32>,
        /// On-chip SRAM in MB (if applicable)
        #[serde(alias = "mem")]
        memory: Option<u32>,
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
        architecture: Option<Architecture>,
        /// Number of TPU chips required (if applicable)
        count: Option<u32>,
        /// High-bandwidth memory in GB (if applicable)
        #[serde(alias = "mem")]
        memory: Option<u32>,
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
                                architecture: Option<Architecture>,
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
                                architecture: Option<Architecture>,
                                cores: Option<u32>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<u32>,
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
                                architecture: Option<Architecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<u32>,
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
                                architecture: Option<Architecture>,
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
                                architecture: Option<Architecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                logic_elements: Option<u32>,
                                memory: Option<u32>,
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
                                architecture: Option<Architecture>,
                                backend: Option<Backend>,
                                compute_capability: Option<f32>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<u32>,
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
                                architecture: Option<Architecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<u32>,
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
                                architecture: Option<Architecture>,
                                #[serde(default = "default_count")]
                                count: Option<u32>,
                                memory: Option<u32>,
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
