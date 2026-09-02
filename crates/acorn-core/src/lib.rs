//! Portable foundation types and utilities for ACORN
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod error;
mod location;
pub mod prelude;
pub mod util;
pub mod validation;

pub use error::{AcornError, AcornResult};
pub use location::{Location, Release, Repository, Scheme};
