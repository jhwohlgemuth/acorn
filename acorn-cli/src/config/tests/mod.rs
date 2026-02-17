use crate::config::*;
use acorn::io::api::TreeEntryType;
use acorn::prelude::PathBuf;
use acorn::{Location, Scheme};

const ENDPOINTS: &str = r#"{
    "endpoints": [
        {
            "name": "orcid",
            "domain": "pub.orcid.org",
            "root": "v3.0",
            "resources": [
                {
                    "name": "search",
                    "method": "get",
                    "template": "{{ base }}/expanded-search/{{ query }}"
                },
                {
                    "name": "status",
                    "method": "get",
                    "template": "{{ base }}/pubStatus"
                }
            ]
        },
        {
            "name": "ror",
            "domain": "api.ror.org",
            "resources": [
                {
                    "name": "status",
                    "method": "get",
                    "template": "{{ base }}/heartbeat"
                }
            ]
        },
        {
            "name": "github",
            "domain": "api.github.com",
            "resources": [
                {
                    "name": "tree",
                    "method": "get",
                    "template": "{{ base }}/{{ path }}/git/trees/{{ branch }}?recursive=1"
                }
            ]
        },
        {
            "name": "gitlab",
            "domain": "code.ornl.gov",
            "root": "api/v4",
            "resources": [
                {
                    "name": "tree",
                    "method": "get",
                    "template": "{{ base }}/projects/{{ project_id }}//repository/tree/{{ query }}"
                }
            ]
        },
        {
            "name": "spdx",
            "domain": "spdx.org",
            "resources": [
                {
                    "name": "licenses",
                    "method": "get",
                    "template": "{{ base }}/licenses/licenses.json"
                }
            ]
        }
    ]
}"#;
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests").join("fixtures")
}

#[test]
fn test_builtin_endpoints() {
    let config = ApplicationConfiguration::parse(ENDPOINTS).unwrap();
    insta::assert_snapshot!(format!("{:#?}", config.endpoints));
}
#[test]
fn test_bucket_config_json() {
    let path = fixtures_dir().join("config").join("with_github_buckets.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    assert_eq!(config.buckets.as_ref().map_or(0, |b| b.len()), 2);
    let path = fixtures_dir().join("config").join("with_remote_buckets.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    assert_eq!(config.buckets.as_ref().map_or(0, |b| b.len()), 3);
}
#[test]
fn test_config_with_endpoints() {
    let path = fixtures_dir().join("config").join("with_endpoints.json");
    let config = ApplicationConfiguration::read_json(path);
    let endpoints = config.unwrap().endpoints.unwrap();
    assert_eq!(endpoints[0].resources.len(), 2);
}
#[test]
fn test_bucket_config_yaml() {
    let path = fixtures_dir().join("config").join("with_remote_buckets.yaml");
    let config = ApplicationConfiguration::read_yaml(path).unwrap();
    assert_eq!(config.buckets.as_ref().map_or(0, |b| b.len()), 3);
}
#[test]
fn test_bucket() {
    let bucket: Bucket = Bucket {
        name: Some("nssd".to_string()),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitLab {
            id: Some(1234_u64),
            location: Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string()),
        },
    };
    assert_eq!(bucket.domain(), "code.ornl.gov".to_string());
    let bucket: Bucket = Bucket {
        name: Some("nssd".to_string()),
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
        entry_type: TreeEntryType::Tree,
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
    // HTTP is supported but only advised in local development scenarios
    let location = Location::Simple("http://google.com".into());
    assert!(location.exists().await);
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
#[test]
fn test_runners() {
    let path = fixtures_dir().join("config").join("with_runners.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    insta::assert_snapshot!(format!("{:#?}", config.runners));
}
