//! Validated relative paths for materialization beneath trusted roots.
use core::fmt;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

/// Failure to validate or safely materialize a relative path.
#[derive(Debug)]
pub enum SafePathError {
    /// The path is empty after lexical normalization.
    Empty,
    /// A destination component could not be inspected.
    InspectDestination {
        /// Component that could not be inspected.
        path: PathBuf,
        /// Filesystem error returned while inspecting the component.
        source: std::io::Error,
    },
    /// The path contains an absolute, parent, or platform-prefix component.
    Unsafe(PathBuf),
    /// An existing destination component is a symbolic link or not a directory.
    UnsafeDestination(PathBuf),
}
/// A normalized, non-empty relative path without traversal components.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SafePath(PathBuf);
impl SafePath {
    /// Normalize and validate a relative path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, SafePathError> {
        let path = path.as_ref();
        let bytes = path.as_os_str().as_encoded_bytes();
        let foreign_absolute = matches!(bytes, [b'\\', ..]) || matches!(bytes, [drive, b':', ..] if drive.is_ascii_alphabetic());
        let foreign_traversal = bytes.split(|byte| matches!(byte, b'/' | b'\\')).any(|component| component == b"..");
        match foreign_absolute || foreign_traversal {
            | true => Err(SafePathError::Unsafe(path.to_path_buf())),
            | false => path
                .components()
                .try_fold(PathBuf::new(), |normalized, component| match component {
                    | Component::CurDir => Ok(normalized),
                    | Component::Normal(value) => Ok(normalized.join(value)),
                    | Component::ParentDir | Component::Prefix(_) | Component::RootDir => Err(SafePathError::Unsafe(path.to_path_buf())),
                })
                .and_then(|normalized| match normalized.as_os_str().is_empty() {
                    | true => Err(SafePathError::Empty),
                    | false => Ok(Self(normalized)),
                }),
        }
    }
    /// Borrow the normalized relative path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
    /// Consume this value and return its normalized relative path.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
    /// Resolve this path beneath `root` after rejecting unsafe existing destination components.
    pub fn materialize_under(&self, root: &Path) -> Result<PathBuf, SafePathError> {
        self.0
            .ancestors()
            .skip(1)
            .map(|parent| root.join(parent))
            .try_for_each(|parent| match parent.symlink_metadata() {
                | Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => Err(SafePathError::UnsafeDestination(parent)),
                | Ok(_) => Ok(()),
                | Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
                | Err(source) => Err(SafePathError::InspectDestination { path: parent, source }),
            })
            .map(|()| root.join(&self.0))
    }
}
impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}
impl core::error::Error for SafePathError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            | Self::InspectDestination { source, .. } => Some(source),
            | Self::Empty | Self::Unsafe(_) | Self::UnsafeDestination(_) => None,
        }
    }
}
impl fmt::Display for SafePathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Empty => write!(formatter, "Safe path cannot be empty"),
            | Self::Unsafe(path) => write!(formatter, "Unsafe relative path: {}", path.display()),
            | Self::UnsafeDestination(path) => write!(formatter, "Unsafe destination component: {}", path.display()),
            | Self::InspectDestination { path, source } => {
                write!(formatter, "Failed to inspect destination component {} — {source}", path.display())
            }
        }
    }
}
