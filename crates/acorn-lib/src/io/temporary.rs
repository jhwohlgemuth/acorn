//! Automatically cleaned temporary directories.
use crate::io::ApiResult;
use crate::prelude::{create_dir_all, remove_dir_all, temp_dir, Path, PathBuf};
use color_eyre::eyre::eyre;
use nanoid::nanoid;

/// A uniquely named temporary directory removed when dropped
#[derive(Debug)]
pub struct TemporaryDirectory(PathBuf);
impl TemporaryDirectory {
    /// Creates a temporary directory using `namespace` in its name
    pub fn create(namespace: &str) -> ApiResult<Self> {
        let path = temp_dir().join(format!("{namespace}-{}", nanoid!()));
        create_dir_all(&path)
            .map(|()| Self(path))
            .map_err(|why| eyre!("Failed to create temporary directory — {why}"))
    }
    /// Returns the temporary directory path
    pub fn path(&self) -> &Path {
        &self.0
    }
}
impl AsRef<Path> for TemporaryDirectory {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}
impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.0);
    }
}
