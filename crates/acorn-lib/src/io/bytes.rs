//! Byte-oriented I/O values
use acorn_core::prelude::String;
use data_encoding::HEXLOWER;
use ring::digest::{digest, SHA256};

/// Content-identity operations for anything viewable as bytes.
pub trait ContentDigest {
    /// Compute the lowercase SHA-256 content digest.
    fn digest(&self) -> String;
}
impl<T> ContentDigest for T
where
    T: AsRef<[u8]> + ?Sized,
{
    fn digest(&self) -> String {
        format!("sha256:{}", HEXLOWER.encode(digest(&SHA256, self.as_ref()).as_ref()))
    }
}

#[cfg(test)]
mod tests;
