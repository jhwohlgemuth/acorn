//! Constants used by portable utility implementations
//!
/// Crockford base32 alphabet (excludes I, L, O, U to avoid confusion).
pub const CROCKFORD_BASE32_ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// Cross-platform line separator.
#[cfg(target_os = "windows")]
pub const LINE_SEPARATOR: &str = "\r\n";
/// Cross-platform line separator.
#[cfg(not(target_os = "windows"))]
pub const LINE_SEPARATOR: &str = "\n";
