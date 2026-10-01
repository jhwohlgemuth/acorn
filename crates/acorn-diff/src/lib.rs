//! Portable unified-text and RFC 6902 JSON Patch diff primitives
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

mod json_patch;
mod unified;

pub use json_patch::{DiffValue, JsonPatch, Operation, OperationKind, PointerError, Rfc6902};
pub use unified::unified_diff;
