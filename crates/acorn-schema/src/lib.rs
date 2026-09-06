//! Portable ACORN schema, validation, and codecs.
//!
//! This crate owns RAD, PID, standards, hardware, and agent data models,
//! validation, crosswalks, and string/byte codecs. Path and URL I/O remain
//! in host adapters (`acorn-lib` / `acorn-host`).
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

#[cfg(feature = "std")]
macro_rules! schema_error {
    ($($arg:tt)*) => {
        acorn_core::AcornError::new(::alloc::format!($($arg)*))
    };
}

pub mod enrichment;
pub mod oci;
pub mod schema;
#[macro_use]
pub mod util;
pub mod validation;

pub use validation::{Each, Validate, ValidationIssue, ValidationReport};
