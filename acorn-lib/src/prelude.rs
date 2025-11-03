//! Wrapper module isolate `std` usage and enable `no_std` support

/// Module that provides `std` support
#[allow(unused_imports)]
mod std {
    pub use std::collections::HashMap;
    pub use std::fs::{create_dir_all, remove_file, File};
    pub use std::io::prelude::*;
    pub use std::io::{self, copy, BufRead, BufReader, Cursor, Write};
    pub use std::os::unix::fs::PermissionsExt;
    pub use std::path::{absolute, Path, PathBuf};
}
/// Module that provides `no-std` support
#[allow(unused_imports)]
mod no_std {
    pub use hashbrown::HashMap;
    pub use std::fs::{create_dir_all, remove_file, File};
    pub use std::io::prelude::*;
    pub use std::io::{self, copy, BufRead, BufReader, Cursor, Write};
    pub use std::os::unix::fs::PermissionsExt;
    pub use std::path::{self, absolute, Path, PathBuf};
}

#[cfg(not(feature = "std"))]
pub use no_std::*;
#[cfg(feature = "std")]
pub use std::*;
