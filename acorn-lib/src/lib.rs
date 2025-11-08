//! # 🌱 ACORN Library
//! > "Plant an ACORN and grow your research"
//!
//! `acorn-lib` is a one-stop-shop for everything related to building and maintaining research activity data (RAD)-related technology, including the Accessible Content Optimization for Research Needs (ACORN) tool.
//! The modules, structs, enums and constants found here support the ACORN CLI, which checks, analyzes, and exports research activity data into useable formats.
//!
#[cfg(feature = "analyzer")]
pub mod analyzer;
#[cfg_attr(not(feature = "std"), no_std)]
pub mod bucket;
pub mod constants;
#[cfg(feature = "doctor")]
pub mod doctor;
#[cfg(feature = "powerpoint")]
pub mod powerpoint;
pub mod prelude;
pub mod schema;
#[cfg(feature = "util")]
pub mod util;
