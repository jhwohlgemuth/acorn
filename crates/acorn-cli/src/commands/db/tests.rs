use super::*;
use crate::test::util::{temp_test_dir, TestCleanup};
use std::fs::{create_dir_all, write};

#[test]
fn test_clear_cache_directory_accepts_missing_cache() {
    let cache = temp_test_dir("clear-missing-chromium-cache");
    let _cleanup = TestCleanup::new(cache.clone());
    remove_dir_all(&cache).expect("remove test cache before exercising missing path");
    assert!(clear_cache_directory(&cache).is_ok());
}
#[test]
fn test_clear_cache_directory_removes_chromium_cache() {
    let cache = temp_test_dir("clear-chromium-cache");
    let _cleanup = TestCleanup::new(cache.clone());
    let browser = cache.join("chrome-version").join("chrome");
    create_dir_all(&browser).expect("create cached Chromium directory");
    write(browser.join("executable"), b"chrome").expect("write cached Chromium executable");
    assert!(clear_cache_directory(&cache).is_ok());
    assert!(!cache.exists());
}
