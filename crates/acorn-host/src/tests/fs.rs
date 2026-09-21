use crate::fs::{file_checksum, file_uri_to_path, SafePath, TempDir};
use acorn_core::util::ChecksumAlgorithm;
use ring::digest::{Algorithm, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA384, SHA512};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn checksum_reads_file() {
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
fn file_uri_conversion_supports_localhost_and_escaped_segments() {
    let path = file_uri_to_path("file://localhost/tmp/acorn%20host.txt").expect("file URI");
    assert_eq!(path, PathBuf::from("/tmp/acorn host.txt"));
}
#[test]
fn safe_path_rejects_traversal_and_absolute_paths() {
    assert!(SafePath::new("../escape").is_err());
    assert!(SafePath::new("/absolute").is_err());
    assert_eq!(SafePath::new("./nested/file").expect("safe path").as_path(), Path::new("nested/file"));
}
#[test]
fn temporary_directory_is_cleaned_up() {
    let path = {
        let directory = TempDir::create("acorn-host-test").expect("temporary directory");
        directory.path().to_path_buf()
    };
    assert!(!path.exists());
}
