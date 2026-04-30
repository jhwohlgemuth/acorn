//! Wrapper module isolate `std` usage and enable `no_std` support

/// Module that provides `std` support
#[allow(unused_imports)]
mod std {
    pub use alloc::sync::Arc;
    pub use hashbrown::{DefaultHashBuilder, HashMap, HashSet};
    pub use std::env::{self, consts, var};
    pub use std::ffi::OsStr;
    pub use std::fs::{canonicalize, copy, create_dir_all, read, remove_dir_all, remove_file, write, File};
    #[cfg(unix)]
    pub use std::fs::{set_permissions, Permissions};
    pub use std::io::{self, BufRead, BufReader, Cursor, Error, Read, Write};
    #[cfg(unix)]
    pub use std::os::unix::fs::PermissionsExt;
    pub use std::path::{absolute, Path, PathBuf};
    pub use std::process::{exit, Command, ExitStatus, Output, Stdio};
    pub use std::sync::{Mutex, OnceLock};
}
/// Module that provides `no-std` support
#[allow(unused_imports)]
mod no_std {
    pub use alloc::sync::Arc;
    pub use hashbrown::{DefaultHashBuilder, HashMap, HashSet};
}

#[cfg(not(feature = "std"))]
pub use no_std::*;
#[cfg(feature = "std")]
pub use std::*;

/// Extension trait for extracting strings from [`std::process::Output`]
///
/// Eliminates repeated `String::from_utf8_lossy(&output.stdout).trim().to_string()` patterns.
///
/// # Examples
///
/// ```ignore
/// use acorn::cmd;
/// use acorn::prelude::CommandOutput;
///
/// match cmd!("git", ["branch", "--show-current"]) {
///     Ok(output) if output.status.success() => {
///         let branch = output.stdout();
///         println!("{branch}");
///     }
///     Ok(output) => eprintln!("{}", output.stderr()),
///     Err(why) => eprintln!("{why}"),
/// }
/// ```
#[cfg(feature = "std")]
pub trait CommandOutput {
    /// Extract stdout as a trimmed, lossy UTF-8 string
    fn stdout(&self) -> String;
    /// Extract stderr as a trimmed, lossy UTF-8 string
    fn stderr(&self) -> String;
}
#[cfg(feature = "std")]
impl CommandOutput for Output {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).trim().to_string()
    }
    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).trim().to_string()
    }
}
/// Get Vale release filename for a given platform operating system (e.g., linux, windows, macos)
#[cfg(feature = "std")]
pub fn vale_release_filename() -> String {
    // https://doc.rust-lang.org/std/env/consts/constant.OS.html
    let os = std::consts::OS.to_lowercase();
    let platform = match os.as_str() {
        | "linux" => "Linux_64-bit.tar.gz",
        | "macos" | "apple" => "macOS_64-bit.tar.gz",
        | "windows" => "Windows_64-bit.zip",
        | _ => "unknown",
    };
    platform.to_string()
}
