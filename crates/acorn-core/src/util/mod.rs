//! Utilities grouped by their minimum runtime capability
//!
#[cfg(feature = "digest")]
use ring::digest::{Algorithm, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA384, SHA512};

pub mod alloc;
pub mod assets;
pub mod constants;
pub mod core;
pub mod math;
pub mod portable;
pub mod types;
pub use self::alloc::*;
pub use self::core::*;
pub use self::math::*;
pub use self::types::*;

#[cfg(feature = "digest")]
impl From<ChecksumAlgorithm> for &'static Algorithm {
    fn from(value: ChecksumAlgorithm) -> Self {
        match value {
            | ChecksumAlgorithm::Sha1 => &SHA1_FOR_LEGACY_USE_ONLY,
            | ChecksumAlgorithm::Sha384 => &SHA384,
            | ChecksumAlgorithm::Sha512 => &SHA512,
            | _ => &SHA256,
        }
    }
}
#[cfg(feature = "digest")]
impl From<&'static Algorithm> for ChecksumAlgorithm {
    fn from(algorithm: &'static Algorithm) -> Self {
        match (
            ::core::ptr::eq(algorithm, &SHA1_FOR_LEGACY_USE_ONLY),
            ::core::ptr::eq(algorithm, &SHA384),
            ::core::ptr::eq(algorithm, &SHA512),
        ) {
            | (true, _, _) => Self::Sha1,
            | (_, true, _) => Self::Sha384,
            | (_, _, true) => Self::Sha512,
            | _ => Self::Sha256,
        }
    }
}
