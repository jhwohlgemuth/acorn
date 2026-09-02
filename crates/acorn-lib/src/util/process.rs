//! Process output utilities
use acorn_core::prelude::{format, String, ToString};
use std::process::Output;

/// Extension trait for extracting strings from process output
pub trait CommandOutput {
    /// Extract stderr as a trimmed, lossy UTF-8 string.
    fn stderr(&self) -> String;
    /// Extract stdout as a trimmed, lossy UTF-8 string.
    fn stdout(&self) -> String;
}
impl CommandOutput for Output {
    fn stderr(&self) -> String {
        match String::from_utf8(self.stderr.clone()) {
            | Ok(value) => value.trim().to_string(),
            | Err(why) => format!("<non-utf8-bytes:{:?}>", why.into_bytes()),
        }
    }
    fn stdout(&self) -> String {
        match String::from_utf8(self.stdout.clone()) {
            | Ok(value) => value.trim().to_string(),
            | Err(why) => format!("<non-utf8-bytes:{:?}>", why.into_bytes()),
        }
    }
}
