//! Byte-oriented I/O values
use acorn_core::prelude::String;
use data_encoding::HEXLOWER;
use ring::digest::{digest, SHA256};

/// Borrowed bytes with content-identity operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bytes<'a>(&'a [u8]);
impl Bytes<'_> {
    /// Compute the lowercase SHA-256 content digest.
    pub fn digest(&self) -> String {
        format!("sha256:{}", HEXLOWER.encode(digest(&SHA256, self.0).as_ref()))
    }
}
impl<'a, T> From<&'a T> for Bytes<'a>
where
    T: AsRef<[u8]> + ?Sized,
{
    fn from(value: &'a T) -> Self {
        Self(value.as_ref())
    }
}

#[cfg(test)]
mod tests;
