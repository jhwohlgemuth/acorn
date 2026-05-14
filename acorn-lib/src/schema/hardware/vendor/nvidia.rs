//! NVIDIA-specific hardware architectures
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// NVIDIA GPU architectures
#[derive(Clone, Debug, Display, Deserialize, PartialEq, Serialize, JsonSchema)]
pub enum NvidiaGpuArchitecture {
    /// NVIDIA Ada Lovelace microarchitecture (RTX 40xx)
    #[display("Ada Lovelace")]
    AdaLovelace,
    /// NVIDIA Ampere microarchitecture (A100, RTX 30xx)
    #[display("Ampere")]
    Ampere,
    /// NVIDIA Blackwell microarchitecture (RTX 50xx, B100)
    #[display("Blackwell")]
    Blackwell,
    /// NVIDIA Hopper microarchitecture (H100, H200)
    #[display("Hopper")]
    Hopper,
    /// NVIDIA Turing microarchitecture (RTX 20xx)
    #[display("Turing")]
    Turing,
    /// NVIDIA Volta microarchitecture (V100)
    #[display("Volta")]
    Volta,
    /// Other or unspecified NVIDIA GPU architecture
    #[display("{}", _0)]
    Other(String),
}
