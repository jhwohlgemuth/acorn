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
mod tests {
    use super::Bytes;
    use acorn_core::validation;

    #[test]
    fn digest_is_a_valid_sha256_identity() {
        let source = String::from("abc");
        let digest = Bytes::from(&source).digest();
        assert_eq!(digest, "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(Bytes::from(b"abc").digest(), digest);
        assert!(validation::digest(&digest).is_ok());
    }
}
