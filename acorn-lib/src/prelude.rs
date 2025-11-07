//! Wrapper module isolate `std` usage and enable `no_std` support

/// Module that provides `std` support
#[allow(unused_imports)]
mod std {
    pub use std::collections::HashMap;
    pub use std::env::{consts, var};
    pub use std::ffi::OsStr;
    pub use std::fs::{canonicalize, copy, create_dir_all, read, remove_file, set_permissions, File, Permissions};
    pub use std::io::{self, BufRead, BufReader, Cursor, Error, Read, Write};
    pub use std::os::unix::fs::PermissionsExt;
    pub use std::path::{absolute, Path, PathBuf};
    pub use std::process::exit;
}
/// Module that provides `no-std` support
#[allow(unused_imports)]
mod no_std {
    pub use hashbrown::HashMap;
}

#[cfg(not(feature = "std"))]
pub use no_std::*;
#[cfg(feature = "std")]
pub use std::*;
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
