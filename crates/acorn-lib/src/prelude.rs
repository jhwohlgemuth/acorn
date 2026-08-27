//! Imports grouped by their minimum runtime capability
/// Allocation-free language prelude
pub mod core {
    pub use ::core::prelude::rust_2021::*;
}
/// Allocation-backed imports without host services
pub mod alloc {
    pub use ::alloc::{
        boxed::Box,
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    pub use hashbrown::{DefaultHashBuilder, HashMap, HashSet};
    pub use triomphe::Arc;
}
/// Imports that require operating-system or runtime services
#[cfg(feature = "std")]
#[allow(unused_imports, clippy::std_instead_of_core)]
pub mod std {
    pub use super::alloc::{format, vec, Arc, Box, DefaultHashBuilder, HashMap, HashSet, String, ToString, Vec};
    pub use ::std::env::{self, consts, current_dir, temp_dir, var, var_os};
    pub use ::std::ffi::{OsStr, OsString};
    pub use ::std::fs::{
        self, canonicalize, copy, create_dir, create_dir_all, read, read_dir, read_to_string, remove_dir_all, remove_file, rename, write, File,
        OpenOptions,
    };
    #[cfg(unix)]
    pub use ::std::fs::{set_permissions, Permissions};
    pub use ::std::io::{self, stdin, BufRead, BufReader, Cursor, Error, ErrorKind, IsTerminal, Read, Seek, Write};
    #[cfg(unix)]
    pub use ::std::os::unix::fs::symlink;
    #[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
    pub use ::std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    #[cfg(windows)]
    pub use ::std::os::windows::fs::{symlink_dir, symlink_file};
    pub use ::std::path::{absolute, Component, Path, PathBuf, MAIN_SEPARATOR};
    pub use ::std::process::{exit, Command, ExitStatus, Output, Stdio};
    pub use ::std::sync::{Mutex, OnceLock};
    /// Extension trait for extracting strings from [`Output`]
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
    pub trait CommandOutput {
        /// Extract stdout as a trimmed, lossy UTF-8 string
        fn stdout(&self) -> String;
        /// Extract stderr as a trimmed, lossy UTF-8 string
        fn stderr(&self) -> String;
    }
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
    /// Get the Vale release filename for the current operating system (e.g., linux, windows, macos)
    pub fn vale_release_filename() -> String {
        // https://doc.rust-lang.org/std/env/consts/constant.OS.html
        let os = consts::OS.to_lowercase();
        match os.as_str() {
            | "linux" => "Linux_64-bit.tar.gz",
            | "macos" | "apple" => "macOS_64-bit.tar.gz",
            | "windows" => "Windows_64-bit.zip",
            | _ => "unknown",
        }
        .to_string()
    }
}
pub use self::alloc::{format, vec, Arc, Box, DefaultHashBuilder, HashMap, HashSet, String, ToString, Vec};
#[cfg(feature = "std")]
pub use self::std::{
    absolute, canonicalize, consts, copy, create_dir, create_dir_all, current_dir, env, exit, fs, io, read, read_dir, read_to_string, remove_dir_all,
    remove_file, rename, stdin, temp_dir, vale_release_filename, var, var_os, write, BufRead, BufReader, Command, CommandOutput, Component, Cursor,
    Error, ErrorKind, ExitStatus, File, IsTerminal, Mutex, OnceLock, OpenOptions, OsStr, OsString, Output, Path, PathBuf, Read, Seek, Stdio, Write,
    MAIN_SEPARATOR,
};
#[cfg(all(feature = "std", unix))]
pub use self::std::{set_permissions, symlink, OpenOptionsExt, Permissions, PermissionsExt};
#[cfg(all(feature = "std", windows))]
pub use self::std::{symlink_dir, symlink_file};
