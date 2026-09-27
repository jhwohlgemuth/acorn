//! Native filesystem primitives
use acorn_core::util::{Checksum, ChecksumAlgorithm};
use core::future::Future;
use data_encoding::HEXLOWER;
use ring::digest::{Algorithm, Context, SHA256};
#[cfg(unix)]
use std::fs::set_permissions;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{self, create_dir_all, read_to_string, remove_dir_all, File, OpenOptions},
    io::{BufReader, Read, Write},
    path::{Component, Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;

/// Compatibility name for consumers that prefer the explicit type name.
pub type TemporaryDirectory = TempDir;
/// Failure from a native filesystem operation.
#[derive(Debug, Error)]
pub enum FsError {
    /// A path is empty after lexical normalization.
    #[error("safe path cannot be empty")]
    EmptySafePath,
    /// A file URI cannot be converted to a local path.
    #[error("invalid file URI: {0}")]
    InvalidFileUri(String),
    /// A filesystem operation failed.
    #[error("filesystem operation failed for {path:?}: {source}")]
    Io {
        /// Path involved in the operation.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// A filesystem write lock was poisoned.
    #[error("filesystem write lock is poisoned")]
    PoisonedLock,
    /// A relative path can escape its trusted root.
    #[error("unsafe relative path: {0:?}")]
    UnsafePath(PathBuf),
    /// An existing destination component is a link or is not a directory.
    #[error("unsafe destination component: {0:?}")]
    UnsafeDestination(PathBuf),
}
/// A normalized, non-empty relative path without traversal components.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SafePath(PathBuf);
/// A uniquely named temporary directory removed when dropped.
#[derive(Debug)]
pub struct TempDir(PathBuf);
impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}
impl SafePath {
    /// Normalize and validate a relative path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, FsError> {
        let path = path.as_ref();
        let bytes = path.as_os_str().as_encoded_bytes();
        let foreign_absolute = matches!(bytes, [b'\\', ..]) || matches!(bytes, [drive, b':', ..] if drive.is_ascii_alphabetic());
        let foreign_traversal = bytes.split(|byte| matches!(byte, b'/' | b'\\')).any(|component| component == b"..");
        match foreign_absolute || foreign_traversal {
            | true => Err(FsError::UnsafePath(path.to_path_buf())),
            | false => path
                .components()
                .try_fold(PathBuf::new(), |normalized, component| match component {
                    | Component::CurDir => Ok(normalized),
                    | Component::Normal(value) => Ok(normalized.join(value)),
                    | Component::ParentDir | Component::Prefix(_) | Component::RootDir => Err(FsError::UnsafePath(path.to_path_buf())),
                })
                .and_then(|normalized| match normalized.as_os_str().is_empty() {
                    | true => Err(FsError::EmptySafePath),
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
    /// Resolve this path beneath `root` after rejecting unsafe existing components.
    pub fn materialize_under(&self, root: &Path) -> Result<PathBuf, FsError> {
        self.0
            .ancestors()
            .skip(1)
            .map(|parent| root.join(parent))
            .try_for_each(|parent| match parent.symlink_metadata() {
                | Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => Err(FsError::UnsafeDestination(parent)),
                | Ok(_) => Ok(()),
                | Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
                | Err(source) => Err(FsError::Io { path: parent, source }),
            })
            .map(|()| root.join(&self.0))
    }
    /// Return the directory that contains this path, resolving a bare relative name to `.`.
    ///
    /// Staging files and directories are written beside their final destination, and a name without a parent component needs the working directory to keep the sibling next to it.
    pub fn parent_or_current(path: &Path) -> PathBuf {
        path.parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    }
    /// Prepare this path beneath `root`, replacing conflicting destinations without following links.
    pub fn prepare(&self, root: &Path) -> Result<PathBuf, FsError> {
        let remove = |path: &Path| match path.symlink_metadata() {
            | Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => fs::remove_file(path).map_err(|source| FsError::Io {
                path: path.to_path_buf(),
                source,
            }),
            | Ok(_) => remove_dir_all(path).map_err(|source| FsError::Io {
                path: path.to_path_buf(),
                source,
            }),
            | Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
            | Err(source) => Err(FsError::Io {
                path: path.to_path_buf(),
                source,
            }),
        };
        let target = root.join(&self.0);
        self.0
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|parent| root.join(parent))
            .try_for_each(|parent| match parent.symlink_metadata() {
                | Ok(metadata) if metadata.is_dir() => Ok(()),
                | Ok(_) => remove(&parent).and_then(|()| create_dir_all(&parent).map_err(|source| FsError::Io { path: parent, source })),
                | Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                    create_dir_all(&parent).map_err(|source| FsError::Io { path: parent, source })
                }
                | Err(source) => Err(FsError::Io { path: parent, source }),
            })
            .and_then(|()| remove(&target))
            .map(|()| target)
    }
    /// Write bytes to this path beneath `root`, optionally replacing an existing destination.
    pub async fn write_file<F, Fut>(&self, root: &Path, clobber: bool, write_lock: &Mutex<()>, get_bytes: F) -> Result<(), FsError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, FsError>>,
    {
        match clobber {
            | false => match self.materialize_under(root) {
                | Ok(target) => {
                    let file = target.parent().map_or_else(
                        || Err(FsError::UnsafeDestination(target.clone())),
                        |parent| {
                            create_dir_all(parent)
                                .map_err(|source| FsError::Io {
                                    path: parent.to_path_buf(),
                                    source,
                                })
                                .and_then(|()| {
                                    OpenOptions::new()
                                        .write(true)
                                        .create_new(true)
                                        .open(&target)
                                        .map_err(|source| FsError::Io {
                                            path: target.clone(),
                                            source,
                                        })
                                })
                        },
                    );
                    match file {
                        | Ok(mut file) => match get_bytes().await {
                            | Ok(bytes) => file.write_all(&bytes).map_err(|source| FsError::Io { path: target, source }),
                            | Err(why) => Err(why),
                        },
                        | Err(why) => Err(why),
                    }
                }
                | Err(why) => Err(why),
            },
            | true => match get_bytes().await {
                | Ok(bytes) => match write_lock.lock() {
                    | Ok(_guard) => match self.prepare(root) {
                        | Ok(target) => fs::write(&target, bytes).map_err(|source| FsError::Io { path: target, source }),
                        | Err(why) => Err(why),
                    },
                    | Err(_) => Err(FsError::PoisonedLock),
                },
                | Err(why) => Err(why),
            },
        }
    }
}
impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.0);
    }
}
impl TempDir {
    /// Create a temporary directory with `namespace` in its name.
    pub fn create(namespace: &str) -> Result<Self, FsError> {
        let path = std::env::temp_dir().join(format!("{namespace}-{}", nanoid::nanoid!()));
        create_dir_all(&path)
            .map(|()| Self(path.clone()))
            .map_err(|source| FsError::Io { path, source })
    }
    /// Return the temporary directory path.
    pub fn path(&self) -> &Path {
        &self.0
    }
}
/// Calculate a file checksum, using SHA-256 when no algorithm is supplied.
pub fn file_checksum(path: impl AsRef<Path>, algorithm: Option<&'static Algorithm>) -> Result<Checksum, FsError> {
    let path = path.as_ref();
    let algorithm = algorithm.unwrap_or(&SHA256);
    File::open(path)
        .map_err(|source| FsError::Io {
            path: path.to_path_buf(),
            source,
        })
        .and_then(|file| {
            let mut buffer = [0_u8; 8192];
            let mut context = Context::new(algorithm);
            let mut reader = BufReader::new(file);
            loop {
                match reader.read(&mut buffer) {
                    | Ok(0) => {
                        break Ok(Checksum {
                            algorithm: ChecksumAlgorithm::from(algorithm),
                            checksum_value: HEXLOWER.encode(context.finish().as_ref()),
                        });
                    }
                    | Ok(count) => context.update(buffer.get(..count).unwrap_or_default()),
                    | Err(source) => {
                        break Err(FsError::Io {
                            path: path.to_path_buf(),
                            source,
                        });
                    }
                }
            }
        })
}
/// Convert a file URI to a local path, leaving ordinary paths unchanged.
pub fn file_uri_to_path(value: impl AsRef<Path>) -> Result<PathBuf, FsError> {
    let path = value.as_ref();
    let value = path.to_string_lossy();
    match (value.starts_with("file:"), value.starts_with("file://")) {
        | (false, _) => Ok(path.to_path_buf()),
        | (true, false) => Ok(PathBuf::from(value.trim_start_matches("file:"))),
        | (true, true) => {
            let normalized = value.replacen("file://localhost/", "file:///", 1);
            url::Url::parse(&normalized)
                .map_err(|_| FsError::InvalidFileUri(value.to_string()))
                .and_then(|uri| uri.to_file_path().map_err(|()| FsError::InvalidFileUri(value.to_string())))
        }
    }
}
/// Set executable permissions on a file.
#[cfg(unix)]
pub fn make_executable(path: impl AsRef<Path>) -> Result<(), FsError> {
    let path = path.as_ref();
    fs::metadata(path)
        .map_err(|source| FsError::Io {
            path: path.to_path_buf(),
            source,
        })
        .and_then(|metadata| {
            let mut permissions = metadata.permissions();
            match permissions.mode() & 0o100 {
                | mode if mode != 0 => Ok(()),
                | _ => {
                    permissions.set_mode(permissions.mode() | 0o111);
                    set_permissions(path, permissions).map_err(|source| FsError::Io {
                        path: path.to_path_buf(),
                        source,
                    })
                }
            }
        })
}
/// Confirm that a Windows executable path exists.
#[cfg(windows)]
pub fn make_executable(path: impl AsRef<Path>) -> Result<(), FsError> {
    let path = path.as_ref();
    path.is_file().then_some(()).ok_or_else(|| FsError::Io {
        path: path.to_path_buf(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "executable does not exist"),
    })
}
/// Read a UTF-8 file completely.
pub fn read_text(path: impl AsRef<Path>) -> Result<String, FsError> {
    let path = path.as_ref();
    read_to_string(path).map_err(|source| FsError::Io {
        path: path.to_path_buf(),
        source,
    })
}
