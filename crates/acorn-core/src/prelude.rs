//! Imports grouped by their minimum runtime capability
/// Allocation-free language prelude
pub mod core {
    pub use ::core::prelude::rust_2021::*;
}
/// Allocation-backed imports without host services.
pub mod alloc {
    pub use ::alloc::{
        boxed::Box,
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    pub use hashbrown::{DefaultHashBuilder, HashMap, HashSet};
    pub use triomphe::Arc;
}
pub use self::alloc::{format, vec, Arc, Box, DefaultHashBuilder, HashMap, HashSet, String, ToString, Vec};
#[cfg(feature = "std")]
pub use std::{
    fs::{canonicalize, create_dir},
    path::{Component, Path, PathBuf},
};
