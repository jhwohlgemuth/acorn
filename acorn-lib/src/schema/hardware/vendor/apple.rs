//! Apple-specific hardware architectures
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Apple CPU architectures
///
/// Note: Apple Silicon uses the AArch64 instruction set, represented in the
/// unified [`super::Architecture`] enum. This type captures Apple-specific
/// integrated components.
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum AppleCpuArchitecture {
    /// Apple Neural Engine (integrated in Apple Silicon)
    #[display("Neural Engine")]
    #[serde(alias = "Neural Engine", alias = "neural engine")]
    NeuralEngine,
    /// Apple-specific performance cores (big/little architecture)
    #[display("Apple Performance Core")]
    PerformanceCore,
    /// Apple-specific efficiency cores
    #[display("Apple Efficiency Core")]
    EfficiencyCore,
    /// Other or unspecified Apple CPU architecture
    #[display("{}", _0)]
    Other(String),
}
/// Apple GPU architectures (integrated in Apple Silicon)
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum AppleGpuArchitecture {
    /// Apple GPU architecture (A14/A15 Bionic)
    #[display("Apple GPU (A14/A15)")]
    #[serde(alias = "Apple GPU (A14/A15)")]
    A14,
    /// Apple GPU architecture (A16/A17 Bionic)
    #[display("Apple GPU (A16/A17)")]
    #[serde(alias = "Apple GPU (A16/A17)")]
    A16,
    /// Apple GPU architecture (A18/A18 Pro)
    #[display("Apple GPU (A18)")]
    #[serde(alias = "Apple GPU (A18)")]
    A18,
    /// Apple GPU architecture (M1 series)
    #[display("Apple GPU (M1)")]
    #[serde(alias = "Apple GPU (M1)")]
    M1,
    /// Apple GPU architecture (M2 series)
    #[display("Apple GPU (M2)")]
    #[serde(alias = "Apple GPU (M2)")]
    M2,
    /// Apple GPU architecture (M3 series)
    #[display("Apple GPU (M3)")]
    #[serde(alias = "Apple GPU (M3)")]
    M3,
    /// Apple GPU architecture (M4 series)
    #[display("Apple GPU (M4)")]
    #[serde(alias = "Apple GPU (M4)")]
    M4,
    /// Apple GPU architecture (M5 series)
    #[display("Apple GPU (M5)")]
    #[serde(alias = "Apple GPU (M5)")]
    M5,
    /// Other or unspecified Apple GPU architecture
    #[display("{}", _0)]
    Other(String),
}
