//! Utilities grouped by their minimum runtime capability
//!
pub mod alloc;
pub mod assets;
pub mod constants;
pub mod core;
pub mod portable;
pub use self::alloc::*;
pub use self::core::*;

#[cfg(feature = "std")]
impl From<&'static ring::digest::Algorithm> for ChecksumAlgorithm {
    fn from(algorithm: &'static ring::digest::Algorithm) -> Self {
        match ::core::ptr::eq(algorithm, &ring::digest::SHA512) {
            | true => Self::Sha512,
            | false => Self::Sha256,
        }
    }
}
#[cfg(feature = "std")]
impl From<ChecksumAlgorithm> for &'static ring::digest::Algorithm {
    fn from(value: ChecksumAlgorithm) -> Self {
        match value {
            | ChecksumAlgorithm::Sha512 => &ring::digest::SHA512,
            | _ => &ring::digest::SHA256,
        }
    }
}
