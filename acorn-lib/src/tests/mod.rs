use crate::*;
use std::path::PathBuf;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_bucket_config() {
    let path = PathBuf::from(FIXTURES).join("buckets_with_github.json");
    let config = BucketsConfig::read_json(path).unwrap();
    assert_eq!(config.buckets.len(), 4);
}
#[test]
fn test_bucket() {
    let bucket: Bucket = Bucket {
        name: "nssd".to_string(),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitLab {
            id: 1234,
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
