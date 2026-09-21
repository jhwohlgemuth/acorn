#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
//! Shared test utilities for `acorn-lib` unit tests.
use core::time::Duration;
#[cfg(feature = "agentic")]
use data_encoding::BASE64;
use jiff::Timestamp;
#[cfg(feature = "agentic")]
use ring::hmac as ring_hmac;
use std::{path::PathBuf, time::SystemTime};
/// Computes a Standard Webhooks HMAC-SHA256 signature for the given message components.
#[cfg(feature = "agentic")]
pub(crate) fn compute_standard_webhook_signature(signing_token_base64: &str, webhook_id: &str, timestamp: &str, body: &[u8]) -> String {
    let key_bytes = BASE64.decode(signing_token_base64.strip_prefix("whsec_").unwrap().as_bytes()).unwrap();
    let key = ring_hmac::Key::new(ring_hmac::HMAC_SHA256, &key_bytes);
    let mut message = format!("{webhook_id}.{timestamp}.").into_bytes();
    message.extend_from_slice(body);
    format!("v1,{}", BASE64.encode(ring_hmac::sign(&key, &message).as_ref()))
}
/// Returns the current UTC time as a Unix timestamp string.
#[cfg(feature = "agentic")]
pub(crate) fn current_unix_timestamp() -> String {
    Timestamp::now().as_second().to_string()
}
/// Returns a Unix timestamp string that is `stale_by_secs` seconds in the past.
#[cfg(feature = "agentic")]
pub(crate) fn stale_unix_timestamp(stale_by_secs: u64) -> String {
    (Timestamp::now().as_second() - stale_by_secs as i64).to_string()
}
/// Get path to test fixtures relative to workspace root
pub fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(path)
}
/// Build a unique temporary directory path under `target/test_artifacts`.
pub(crate) fn temp_dir(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_nanos(0))
        .as_nanos();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test_artifacts")
        .join(format!("{name}-{timestamp}"))
}
/// Build a unique path under `target/test_artifacts`.
pub(crate) fn unique_path(name: &str, extension: &str) -> PathBuf {
    let timestamp = Timestamp::now().as_nanosecond();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test_artifacts")
        .join(format!("{name}-{timestamp}.{extension}"))
}
