#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use acorn::prelude::{create_dir_all, read_dir, remove_dir_all, PathBuf};
use core::sync::atomic::{AtomicUsize, Ordering};

static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Get path to test fixtures relative to workspace root
pub(crate) fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(path)
}
/// Test helper to recursively check if a directory contains any files
pub(crate) fn has_any_file(path: &PathBuf) -> bool {
    match read_dir(path) {
        | Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .any(|entry_path| match entry_path.is_file() {
                | true => true,
                | false => has_any_file(&entry_path),
            }),
        | Err(_) => false,
    }
}
/// Create a temporary test directory under target/test_artifacts
pub(crate) fn temp_test_dir(test_name: &str) -> PathBuf {
    let unique = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("test_artifacts")
        .join(format!("{test_name}-{}-{unique}", std::process::id()));
    let _ = remove_dir_all(&temp);
    let _ = create_dir_all(&temp);
    temp
}
/// RAII cleanup helper for test directories
pub(crate) struct TestCleanup {
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
