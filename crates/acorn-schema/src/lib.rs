//! Portable ACORN schema, validation, and codecs.
//!
//! This crate owns RAD, PID, standards, hardware, and agent data models,
//! validation, crosswalks, and string/byte codecs. Path and URL I/O remain
//! in host adapters (`acorn-lib` / `acorn-host`).
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;
macro_rules! schema_error {
    ($($arg:tt)*) => {
        acorn_core::AcornError::new(::alloc::format!($($arg)*))
    };
}
#[cfg(feature = "alloc")]
pub mod agent;
pub mod assets;
pub mod codec;
pub mod computation;
pub mod discovery;
pub mod enrichment;
pub mod geonames;
pub mod graph;
pub mod hardware;
pub mod infrastructure;
pub mod modelkit;
pub mod namespaces;
pub mod oci;
pub mod pid;
pub mod research_activity;
pub mod software;
pub mod standard;
#[macro_use]
pub mod util;
mod types;
pub mod validation;
pub mod vocabulary;
pub use hardware::memory::{Memory, MemoryUnit};
pub use hardware::{
    AcceleratorArchitecture, Architecture, Backend, CpuArchitecture, DspArchitecture, ErrorCorrection, FpgaArchitecture, GpuArchitecture, Model,
    QuantumResource, Qubit, QubitType, Regime, Resource, SensorModality, Technology, Topology, Vendor, Vendored,
};
pub use types::*;
pub use validation::{Each, Validate, ValidationIssue, ValidationReport};
#[cfg(test)]
mod tests;
