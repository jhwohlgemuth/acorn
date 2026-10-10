//! Portable foundation types and utilities for ACORN
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
pub mod error;
mod location;
mod noul;
pub mod options;
pub mod prelude;
#[cfg(feature = "alloc")]
pub mod similarity;
pub mod time;
pub mod util;
pub mod validation;
pub use error::{AcornError, AcornResult};
pub use location::{Location, Release, Repository, Scheme, SimpleOrDetailed};
pub use noul::{Noul, NoulError};
