use crate::*;
use std::path::PathBuf;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_bucket_config_json() {
    let path = PathBuf::from(FIXTURES).join("buckets_with_github.json");
    let config = BucketsConfig::read_json(path).unwrap();
    assert_eq!(config.buckets.len(), 2);
}
#[test]
fn test_bucket_config_yaml() {
    let path = PathBuf::from(FIXTURES).join("buckets.yaml");
    let config = BucketsConfig::read_yaml(path).unwrap();
    assert_eq!(config.buckets.len(), 3);
}
#[test]
fn test_bucket() {
    let bucket: Bucket = Bucket {
        name: "nssd".to_string(),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitLab {
            id: Some(1234_u64),
            uri: "https://code.ornl.gov/research-enablement/buckets/nssd".to_string(),
        },
    };
    assert_eq!(bucket.domain(), "code.ornl.gov".to_string());
    let bucket: Bucket = Bucket {
        name: "nssd".to_string(),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitHub {
            uri: "https://code.ornl.gov/research-enablement/buckets/nssd".to_string(),
        },
    };
    assert_eq!(bucket.domain(), "code.ornl.gov".to_string());
}
#[test]
fn test_gitlab_tree_entry() {
    let entry = GitlabTreeEntry {
        id: 1234.to_string(),
        name: "acorn".to_string(),
        entry_type: EntryType::Tree,
        path: "acorn".to_string(),
        mode: "example".to_string(),
    };
    assert!(!entry.is_blob());
    assert_eq!(entry.path(), "acorn".to_string());
}
#[test]
fn test_repository_gitlab() {
    let repository = Repository::GitLab {
        // id: "research-enablement%2Fvale-package",
        id: Some(18243_u64),
        uri: "https://code.ornl.gov/research-enablement/vale-package".to_string(),
    };
    let release = repository.latest_release();
    assert!(release.is_some());
}
#[ignore]
#[test]
fn test_repository_github() {
    let repository = Repository::GitHub {
        uri: "https://github.com/jhwohlgemuth/voxelcss".to_string(),
    };
    let release = repository.latest_release();
    println!("{release:#?}");
    assert!(release.is_some());
}
