//! Shared test utilities for acorn-cli tests
use acorn::prelude::{create_dir_all, remove_dir_all, PathBuf};

/// Get path to test fixtures relative to workspace root
pub fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(path)
}
/// Create a temporary test directory under target/test_artifacts
pub fn temp_test_dir(test_name: &str) -> PathBuf {
    let temp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("test_artifacts")
        .join(test_name);
    let _ = create_dir_all(&temp);
    temp
}
/// RAII cleanup helper for test directories
pub struct TestCleanup {
    path: PathBuf,
}
impl TestCleanup {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
impl Drop for TestCleanup {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = remove_dir_all(&self.path);
        }
    }
}
