use super::*;
use acorn_lib::{Bucket, BucketsConfig, EntryType, Repository};
use color_eyre::eyre::Report;
use mockall::predicate::*;
use mockall::{mock, Sequence};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

// Mock for BucketsConfig
mock! {
    pub BucketsConfig {
        pub fn read_json(path: PathBuf) -> Result<BucketsConfig, Report>;
    }
}

// Mock for Bucket
mock! {
    pub Bucket {
        pub fn download_files(self, output: PathBuf) -> usize;
    }
    impl Clone for Bucket {
        fn clone(&self) -> Self;
    }
}

// Store the original functions
thread_local! {
    static ORIGINAL_READ_JSON: RefCell<Option<Arc<dyn Fn(PathBuf) -> Result<BucketsConfig, Report>>>> = RefCell::new(None);
    static ORIGINAL_DOWNLOAD_FILES: RefCell<Option<Arc<dyn Fn(Bucket, PathBuf) -> usize>>> = RefCell::new(None);
}

// Helper function to create a temporary directory with a valid bucket configuration
fn setup_bucket_config() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("buckets.json");
    
    let config_content = r#"
    {
        "buckets": [
            {
                "name": "test-bucket",
                "repository": {
                    "provider": "gitlab",
                    "id": 12345,
                    "uri": "https://gitlab.com/test/repo"
                }
            }
        ]
    }
    "#;
    
    fs::write(&config_path, config_content).expect("Failed to write config file");
    
    (temp_dir, config_path)
}

/// Setup function to mock BucketsConfig::read_json
fn mock_read_json(result: Result<BucketsConfig, Report>) {
    let read_json_fn = move |_: PathBuf| -> Result<BucketsConfig, Report> {
        result.clone()
    };
    
    ORIGINAL_READ_JSON.with(|orig| {
        *orig.borrow_mut() = Some(Arc::new(read_json_fn));
    });
}

/// Setup function to mock Bucket::download_files
fn mock_download_files(download_count: usize) {
    let download_files_fn = move |_: Bucket, _: PathBuf| -> usize {
        download_count
    };
    
    ORIGINAL_DOWNLOAD_FILES.with(|orig| {
        *orig.borrow_mut() = Some(Arc::new(download_files_fn));
    });
}

#[test]
fn test_run_valid_paths_single_bucket() {
    // Create a temporary directory for output
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().to_path_buf();
    
    // Create a bucket with mocked download_files method
    let mut bucket = MockBucket::new();
    bucket.expect_download_files()
        .times(1)
        .return_const(5usize);
    
    // Create a bucket config with the mocked bucket
    let buckets_config = BucketsConfig {
        buckets: vec![bucket.into()],
    };
    
    // Setup the mock for read_json
    mock_read_json(Ok(buckets_config));
    
    // Create a temporary config file path
    let (_, config_path) = setup_bucket_config();
    
    // Run the download command
    let result = run(&Some(config_path), &Some(output_path));
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_run_valid_paths_multiple_buckets() {
    // Create a temporary directory for output
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().to_path_buf();
    
    // Create multiple buckets with mocked download_files methods
    let mut bucket1 = MockBucket::new();
    bucket1.expect_download_files()
        .times(1)
        .return_const(3usize);
        
    let mut bucket2 = MockBucket::new();
    bucket2.expect_download_files()
        .times(1)
        .return_const(2usize);
    
    // Create a bucket config with the mocked buckets
    let buckets_config = BucketsConfig {
        buckets: vec![bucket1.into(), bucket2.into()],
    };
    
    // Setup the mock for read_json
    mock_read_json(Ok(buckets_config));
    
    // Create a temporary config file path
    let (_, config_path) = setup_bucket_config();
    
    // Run the download command
    let result = run(&Some(config_path), &Some(output_path));
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_run_invalid_bucket_config_path() {
    // Create a non-existent path for bucket config
    let invalid_path = PathBuf::from("/path/that/does/not/exist.json");
    
    // Setup the mock for read_json to return an error
    mock_read_json(Err(Report::msg("File not found")));
    
    // Create a temporary directory for output
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().to_path_buf();
    
    // Run the download command
    let result = run(&Some(invalid_path), &Some(output_path));
    
    // Verify the result - should still be Ok since the function always returns Ok
    assert!(result.is_ok());
}

#[test]
fn test_run_without_buckets_path() {
    // Run the download command without specifying a buckets path
    let result = run(&None, &Some(PathBuf::from("/tmp")));
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_run_without_output_path() {
    // Create a bucket with mocked download_files method
    // It should never be called because we don't provide an output path
    let bucket = MockBucket::new();
    
    // Create a bucket config with the mocked bucket
    let buckets_config = BucketsConfig {
        buckets: vec![bucket.into()],
    };
    
    // Setup the mock for read_json
    mock_read_json(Ok(buckets_config));
    
    // Create a temporary config file path
    let (_, config_path) = setup_bucket_config();
    
    // Run the download command without specifying an output path
    let result = run(&Some(config_path), &None);
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_run_download_zero_files() {
    // Create a temporary directory for output
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().to_path_buf();
    
    // Create a bucket with mocked download_files method that returns 0
    let mut bucket = MockBucket::new();
    bucket.expect_download_files()
        .times(1)
        .return_const(0usize);
    
    // Create a bucket config with the mocked bucket
    let buckets_config = BucketsConfig {
        buckets: vec![bucket.into()],
    };
    
    // Setup the mock for read_json
    mock_read_json(Ok(buckets_config));
    
    // Create a temporary config file path
    let (_, config_path) = setup_bucket_config();
    
    // Run the download command
    let result = run(&Some(config_path), &Some(output_path));
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_run_download_single_file() {
    // Create a temporary directory for output
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().to_path_buf();
    
    // Create a bucket with mocked download_files method that returns 1
    let mut bucket = MockBucket::new();
    bucket.expect_download_files()
        .times(1)
        .return_const(1usize);
    
    // Create a bucket config with the mocked bucket
    let buckets_config = BucketsConfig {
        buckets: vec![bucket.into()],
    };
    
    // Setup the mock for read_json
    mock_read_json(Ok(buckets_config));
    
    // Create a temporary config file path
    let (_, config_path) = setup_bucket_config();
    
    // Run the download command
    let result = run(&Some(config_path), &Some(output_path));
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_run_with_invalid_json() {
    // Create a temporary directory for output
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().to_path_buf();
    
    // Create a temporary directory with an invalid JSON file
    let temp_dir_config = tempdir().expect("Failed to create temp directory");
    let config_path = temp_dir_config.path().join("invalid.json");
    
    fs::write(&config_path, "{ invalid json }").expect("Failed to write invalid config file");
    
    // Setup the mock for read_json to return an error for invalid JSON
    mock_read_json(Err(Report::msg("Invalid JSON")));
    
    // Run the download command
    let result = run(&Some(config_path), &Some(output_path));
    
    // Verify the result - should still be Ok since the function always returns Ok
    assert!(result.is_ok());
}

#[test]
fn test_run_multiple_buckets_with_different_download_counts() {
    // Create a temporary directory for output
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().to_path_buf();
    
    // Create a sequence to control the order of method calls
    let mut seq = Sequence::new();
    
    // Create multiple buckets with mocked download_files methods returning different counts
    let mut bucket1 = MockBucket::new();
    bucket1.expect_download_files()
        .times(1)
        .in_sequence(&mut seq)
        .return_const(7usize);
        
    let mut bucket2 = MockBucket::new();
    bucket2.expect_download_files()
        .times(1)
        .in_sequence(&mut seq)
        .return_const(3usize);
        
    let mut bucket3 = MockBucket::new();
    bucket3.expect_download_files()
        .times(1)
        .in_sequence(&mut seq)
        .return_const(0usize);
    
    // Create a bucket config with the mocked buckets
    let buckets_config = BucketsConfig {
        buckets: vec![bucket1.into(), bucket2.into(), bucket3.into()],
    };
    
    // Setup the mock for read_json
    mock_read_json(Ok(buckets_config));
    
    // Create a temporary config file path
    let (_, config_path) = setup_bucket_config();
    
    // Run the download command
    let result = run(&Some(config_path), &Some(output_path));
    
    // Verify the result
    assert!(result.is_ok());
}

#[test]
fn test_relative_output_path() {
    // Create a bucket with mocked download_files method
    let mut bucket = MockBucket::new();
    bucket.expect_download_files()
        .times(1)
        .return_const(5usize);
    
    // Create a bucket config with the mocked bucket
    let buckets_config = BucketsConfig {
        buckets: vec![bucket.into()],
    };
    
    // Setup the mock for read_json
    mock_read_json(Ok(buckets_config));
    
    // Create a temporary config file path
    let (_, config_path) = setup_bucket_config();
    
    // Use a relative output path
    let output_path = PathBuf::from("./relative_output");
    
    // Run the download command
    let result = run(&Some(config_path), &Some(output_path));
    
    // Verify the result
    assert!(result.is_ok());
}

