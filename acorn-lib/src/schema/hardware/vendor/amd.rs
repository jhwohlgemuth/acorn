//! AMD-specific hardware architectures
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// AMD x86 CPU architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum AmdCpuArchitecture {
    /// 32-bit x86 instruction set
    #[display("x86")]
    X86,
    /// x86-64 instruction set
    #[display("x86_64")]
    #[serde(alias = "x86_64", alias = "x86-64", alias = "amd64")]
    X86_64,
    /// Other or unspecified AMD CPU architecture
    #[display("{}", _0)]
    Other(String),
}
/// AMD GPU architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum AmdGpuArchitecture {
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
    /// Other or unspecified AMD GPU architecture
    #[display("{}", _0)]
    Other(String),
}
