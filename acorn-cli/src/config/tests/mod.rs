use crate::config::*;
use acorn_lib::prelude::PathBuf;
use acorn_lib::{Location, Scheme};

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_bucket_config_json() {
    let path = PathBuf::from(FIXTURES).join("buckets_with_github.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    assert_eq!(config.buckets.len(), 2);
    let path = PathBuf::from(FIXTURES).join("buckets.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    assert_eq!(config.buckets.len(), 3);
}
#[test]
fn test_bucket_config_yaml() {
    let path = PathBuf::from(FIXTURES).join("buckets.yaml");
    let config = ApplicationConfiguration::read_yaml(path).unwrap();
    assert_eq!(config.buckets.len(), 3);
}
#[test]
fn test_bucket() {
    let bucket: Bucket = Bucket {
        name: "nssd".to_string(),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitLab {
            id: Some(1234_u64),
            location: Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string()),
        },
    };
    assert_eq!(bucket.domain(), "code.ornl.gov".to_string());
    let bucket: Bucket = Bucket {
        name: "nssd".to_string(),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitHub {
            location: Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string()),
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
    assert_eq!(entry.path(), "acorn");
}
#[tokio::test]
async fn test_location_exists() {
    // remote URL
    let location = Location::Simple("https://google.com".into());
    assert!(location.exists().await);
    // non-existent remote URL
    let location = Location::Simple("https://does-not-exist.com".into());
    assert!(!location.exists().await);
    // local path
    let location = Location::Simple("file:./Cargo.toml".into());
    assert!(location.exists().await);
    // non-existent local path
    let location = Location::Simple("file:./does-not-exist".into());
    assert!(!location.exists().await);
    // unsupported scheme (HTTP)
    let location = Location::Simple("http://google.com".into());
    assert!(!location.exists().await);
    // failed remote uri
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "42".into(),
    };
    assert!(!location.exists().await);
    // failed local uri
    let location = Location::Detailed {
        scheme: Scheme::File,
        uri: "should fail to parse?".into(),
    };
    assert!(!location.exists().await);
}
#[test]
fn test_location_hash() {
    // remote
    let location = Location::Simple("https://code.ornl.gov/research-enablement/buckets".into());
    assert_eq!(location.hash(), "code_ornl_gov_research-enablement_buckets");
    let location = Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".into());
    assert_eq!(location.hash(), "code_ornl_gov_research-enablement_buckets_nssd");
    // host = localhost
    let location = Location::Simple("file://localhost/buckets/nssd".into());
    assert_eq!(location.hash(), "localhost_buckets_nssd");
    // No host (absolute path)
    let location = Location::Simple("file:///buckets/nssd".into());
    assert_eq!(location.hash(), "buckets_nssd");
    // No host (relative path)
    let location = Location::Simple("file:./buckets/nssd".into());
    assert_eq!(location.hash(), "buckets_nssd");
}
#[test]
fn test_location_uri() {
    let location = Location::Simple("https://code.ornl.gov/GSHS/GDS/Common/PIPE/module-a".into());
    match location.uri() {
        | Some(uri) => {
            assert_eq!(uri.scheme(), "HTTPS");
            assert_eq!(uri.path(), "/GSHS/GDS/Common/PIPE/module-a");
        }
        | _ => panic!(),
    }
    let location = Location::Simple("ssh://git@code.ornl.gov/GSHS/GDS/Common/PIPE/module-a.git".into());
    match location.uri() {
        | Some(uri) => {
            assert_eq!(uri.scheme(), "SSH");
            assert_eq!(uri.path(), "/GSHS/GDS/Common/PIPE/module-a.git");
        }
        | _ => panic!(),
    }
}
#[ignore]
#[test]
fn test_repository_gitlab() {
    let repository = Repository::GitLab {
        // id: "research-enablement%2Fvale-package",
        id: Some(18243_u64),
        location: Location::Simple("https://code.ornl.gov/research-enablement/vale-package".to_string()),
    };
    let release = repository.latest_release();
    assert!(release.is_some());
}
#[ignore]
#[test]
fn test_repository_github() {
    let repository = Repository::GitHub {
        location: Location::Simple("https://github.com/jhwohlgemuth/voxelcss".to_string()),
    };
    let release = repository.latest_release();
    assert!(release.is_some());
}
