//! Wrapper module isolate `std` usage and enable `no_std` support
pub use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
/// Module that provides `std` support
#[cfg(feature = "std")]
#[allow(unused_imports, clippy::std_instead_of_core)]
mod std {
    pub use hashbrown::{DefaultHashBuilder, HashMap, HashSet};
    pub use std::env::{self, consts, current_dir, temp_dir, var, var_os};
    pub use std::ffi::{OsStr, OsString};
    pub use std::fs::{
        canonicalize, copy, create_dir_all, read, read_dir, read_to_string, remove_dir_all, remove_file, rename, write, File, OpenOptions,
    };
    #[cfg(unix)]
    pub use std::fs::{set_permissions, Permissions};
    pub use std::io::{self, BufRead, BufReader, Cursor, Error, ErrorKind, Read, Write};
    #[cfg(unix)]
    pub use std::os::unix::fs::symlink;
    #[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
    pub use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    #[cfg(windows)]
    pub use std::os::windows::fs::{symlink_dir, symlink_file};
    pub use std::path::{absolute, Path, PathBuf};
    pub use std::process::{exit, Command, ExitStatus, Output, Stdio};
    pub use std::sync::{Mutex, OnceLock};
    pub use triomphe::Arc;
}
/// Module that provides `no-std` support
#[allow(unused_imports)]
mod no_std {
    pub use hashbrown::{DefaultHashBuilder, HashMap, HashSet};
    pub use triomphe::Arc;
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
        match String::from_utf8(self.stdout.clone()) {
            | Ok(value) => value.trim().to_string(),
            | Err(why) => format!("<non-utf8-bytes:{:?}>", why.into_bytes()),
        }
    }
    fn stderr(&self) -> String {
        match String::from_utf8(self.stderr.clone()) {
            | Ok(value) => value.trim().to_string(),
            | Err(why) => format!("<non-utf8-bytes:{:?}>", why.into_bytes()),
        }
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
