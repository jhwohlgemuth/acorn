//! Portable foundation types and utilities for ACORN
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[cfg(feature = "alloc")]
pub mod big_o;
pub mod error;
mod location;
pub mod options;
pub mod prelude;
#[cfg(feature = "alloc")]
pub mod similarity;
pub mod time;
pub mod util;
pub mod validation;
pub use error::{AcornError, AcornResult};
pub use location::{Location, Release, Repository, Scheme, SimpleOrDetailed};
#[cfg(feature = "alloc")]
pub use util::types::OneOrMany;
pub use util::types::{Noul, NoulError, ThisOrThat};
