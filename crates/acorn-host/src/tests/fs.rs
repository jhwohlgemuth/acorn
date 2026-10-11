use crate::fs::{file_checksum, file_uri_to_path, project_cache_directory_from, project_data_directory_from, FsError, SafePath, TempDir};
use acorn_core::util::ChecksumAlgorithm;
use ring::digest::{Algorithm, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA384, SHA512};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn test_checksum_reads_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("value.txt");
    fs::write(&path, b"acorn").expect("write fixture");
    let checksum = file_checksum(&path, None).expect("checksum");
    assert_eq!(checksum.algorithm, ChecksumAlgorithm::Sha256);
    assert_eq!(checksum.checksum_value.len(), 64);
    let algorithms: [(&'static Algorithm, ChecksumAlgorithm, usize); 4] = [
        (&SHA1_FOR_LEGACY_USE_ONLY, ChecksumAlgorithm::Sha1, 40),
        (&SHA256, ChecksumAlgorithm::Sha256, 64),
        (&SHA384, ChecksumAlgorithm::Sha384, 96),
        (&SHA512, ChecksumAlgorithm::Sha512, 128),
    ];
    algorithms.into_iter().for_each(|(algorithm, expected, length)| {
        let checksum = file_checksum(&path, Some(algorithm)).expect("checksum");
        assert_eq!(checksum.algorithm, expected);
        assert_eq!(checksum.checksum_value.len(), length);
    });
}
#[test]
fn test_file_uri_conversion_supports_localhost_and_escaped_segments() {
    #[cfg(unix)]
    let (uri, expected) = ("file://localhost/tmp/acorn%20host.txt", PathBuf::from("/tmp/acorn host.txt"));
    #[cfg(windows)]
    let (uri, expected) = ("file://localhost/C:/tmp/acorn%20host.txt", PathBuf::from(r"C:\tmp\acorn host.txt"));
    let path = file_uri_to_path(uri).expect("file URI");
    assert_eq!(path, expected);
}
#[test]
fn test_project_cache_directory_creates_namespace() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let namespace = SafePath::new("runs").expect("safe namespace");
    let path = project_cache_directory_from("org", "example", "xylem", &namespace, Some(directory.path())).expect("project cache namespace");
    assert_eq!(path, directory.path().join("runs"));
    assert!(path.is_dir());
}
#[test]
fn test_project_cache_directory_reports_creation_failure() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let root = directory.path().join("file");
    fs::write(&root, "not a directory").expect("write blocking file");
    let namespace = SafePath::new("runs").expect("safe namespace");
    assert!(matches!(
        project_cache_directory_from("org", "example", "xylem", &namespace, Some(&root)),
        Err(FsError::Io { path, .. }) if path == root.join("runs")
    ));
}
#[test]
fn test_project_cache_directory_reports_unavailable_root() {
    let namespace = SafePath::new("runs").expect("safe namespace");
    assert!(matches!(
        project_cache_directory_from("org", "example", "xylem", &namespace, None),
        Err(FsError::ProjectDirectoryUnavailable { qualifier, organization, application })
            if qualifier == "org" && organization == "example" && application == "xylem"
    ));
}
#[test]
fn test_project_data_directory_creates_namespace() {
    let directory = tempfile::tempdir().expect("data root");
    let namespace = SafePath::new("logs/vocabulary").expect("safe namespace");
    let path = project_data_directory_from("org", "example", "xylem", &namespace, Some(directory.path())).expect("project data namespace");
    assert!(path.is_dir());
    assert!(path.ends_with("logs/vocabulary"));
}
#[test]
fn test_safe_path_rejects_empty_traversal_and_absolute_paths() {
    assert!(SafePath::new("").is_err());
    assert!(SafePath::new("../escape").is_err());
    assert!(SafePath::new("/absolute").is_err());
    assert!(SafePath::new(r"C:\absolute").is_err());
    assert_eq!(SafePath::new("./nested/file").expect("safe path").as_path(), Path::new("nested/file"));
}
#[test]
fn test_temporary_directory_is_cleaned_up() {
    let path = {
        let directory = TempDir::create("acorn-host-test").expect("temporary directory");
        directory.path().to_path_buf()
    };
    assert!(!path.exists());
}
