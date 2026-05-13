//! Intel-specific hardware architectures

use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Intel CPU architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum IntelCpuArchitecture {
    /// Intel Itanium architecture (IA-64)
    #[display("IA-64")]
    #[serde(alias = "IA-64", alias = "ia-64", alias = "IA64")]
    Ia64,
    /// Other or unspecified Intel CPU architecture
    #[display("{}", _0)]
    Other(String),
}

/// Intel FPGA architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum IntelFpgaArchitecture {
    /// Intel Agilex FPGA
    #[display("Agilex")]
    Agilex,
    /// Intel/Altera Cyclone V FPGA
    #[display("Cyclone V")]
    CycloneV,
    /// Intel/Altera Stratix V FPGA
    #[display("Stratix V")]
    StratixV,
    /// Other or unspecified Intel FPGA architecture
    #[display("{}", _0)]
    Other(String),
}

/// Intel GPU architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum IntelGpuArchitecture {
    /// Intel Alchemist graphics architecture (Xe-HPG)
    #[display("Alchemist")]
    Alchemist,
    /// Intel Xe graphics architecture
    #[display("Xe")]
    Xe,
    /// Intel Xe2 graphics architecture (Battlemage)
    #[display("Xe2")]
    Xe2,
    /// Intel Xe3 graphics architecture (Celestial)
    #[display("Xe3")]
    Xe3,
    /// Other or unspecified Intel GPU architecture
    #[display("{}", _0)]
    Other(String),
}
