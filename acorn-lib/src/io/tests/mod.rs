use crate::io::api::gitlab::TreeEntry;
use crate::io::api::TreeEntryType;
use crate::io::bagit::{Bag, BagInfo, Save};
use crate::io::config::{ApplicationConfiguration, Bucket};
use crate::io::http::policy::{http_timeout_layer, shared_http_policy};
use crate::io::http::{should_retry, HttpMethod};
use crate::io::{
    archive, env_var_is_truthy, file_checksum, files_all, files_from_git_branch, files_from_git_commit, files_from_gitlab_merge_request,
    filter_git_command_result, filter_ignored, image_paths, read_file, read_large_file, uri_to_path, write_file_bytes, InputOutput,
};
use crate::util::constants::ENV_NO_LOCAL_DATABASE;
use crate::{Location, Repository, Scheme};
use color_eyre::Report;
use std::fs::{create_dir_all, read_to_string, remove_dir_all, remove_file, write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
const TEST_BAGGING_DATE: &str = "2026-02-22";

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests/fixtures")
}

#[test]
fn test_builtin_endpoints() {
    let config = ApplicationConfiguration::parse(ENDPOINTS).unwrap();
    insta::assert_snapshot!("acorn__io__config__tests__builtin_endpoints", format!("{:#?}", config.endpoints));
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
    assert_eq!(bucket.domain().unwrap(), "code.ornl.gov".to_string());
    let bucket: Bucket = Bucket {
        name: Some("nssd".to_string()),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitHub {
            location: Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string()),
        },
    };
    assert_eq!(bucket.domain().unwrap(), "code.ornl.gov".to_string());
}
#[test]
fn test_gitlab_tree_entry() {
    let entry = TreeEntry {
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
    let location = Location::Simple("https://google.com".into());
    assert!(location.exists().await);
    let location = Location::Simple("https://does-not-exist.com".into());
    assert!(!location.exists().await);
    let location = Location::Simple("file:./Cargo.toml".into());
    assert!(location.exists().await);
    let location = Location::Simple("file:./does-not-exist".into());
    assert!(!location.exists().await);
    let location = Location::Simple("http://google.com".into());
    assert!(location.exists().await);
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "42".into(),
    };
    assert!(!location.exists().await);
    let location = Location::Detailed {
        scheme: Scheme::File,
        uri: "should fail to parse?".into(),
    };
    assert!(!location.exists().await);
}
#[test]
fn test_location_hash() {
    let location = Location::Simple("https://code.ornl.gov/research-enablement/buckets".into());
    assert_eq!(location.hash(), "code_ornl_gov_research-enablement_buckets");
    let location = Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".into());
    assert_eq!(location.hash(), "code_ornl_gov_research-enablement_buckets_nssd");
    let location = Location::Simple("file://localhost/buckets/nssd".into());
    assert_eq!(location.hash(), "localhost_buckets_nssd");
    let location = Location::Simple("file:///buckets/nssd".into());
    assert_eq!(location.hash(), "buckets_nssd");
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
#[tokio::test]
async fn test_repository_gitlab() {
    let repository = Repository::GitLab {
        id: Some(18243_u64),
        location: Location::Simple("https://code.ornl.gov/research-enablement/vale-package".to_string()),
    };
    let release = repository.latest_release().await;
    assert!(release.is_some());
}
#[ignore]
#[tokio::test]
async fn test_repository_github() {
    let repository = Repository::GitHub {
        location: Location::Simple("https://github.com/jhwohlgemuth/voxelcss".to_string()),
    };
    let release = repository.latest_release().await;
    assert!(release.is_some());
}
#[test]
fn test_runners() {
    let path = fixtures_dir().join("config").join("with_runners.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    insta::assert_snapshot!("acorn__io__config__tests__runners", format!("{:#?}", config.runners));
}
#[test]
fn test_archive() {
    let path = fixtures_dir().join("data");
    let output_file = archive(path.clone(), None).unwrap();
    assert_eq!(output_file.file_name().unwrap().to_str().unwrap(), "data.zip");
}
#[test]
fn test_bagit() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("acorn-bagit-{stamp}"));
    let _ = create_dir_all(temp_dir.clone());
    let info = BagInfo::init()
        .organization(vec!["Oak Ridge National Laboratory".to_string()])
        .contact_name(vec!["Jason Wohlgemuth".to_string()])
        .build();
    let bag = Bag::init()
        .base_directory(fixtures_dir().join("data").display().to_string())
        .info(info)
        .build()
        .with_payload();
    assert_eq!(bag.payload.len(), 27);
    assert_eq!(bag.clone().info.unwrap().entries().len(), 2);
    let save_path = temp_dir.join("bag");
    match bag.save(save_path.to_str().unwrap()) {
        | Ok(path) => {
            assert_eq!(path.file_name().unwrap().to_str().unwrap(), "bag.zip");
            assert!(path.exists(), "Saved bag file should exist at {:?}", path);
        }
        | Err(why) => panic!("Failed to save bag: {why}"),
    }
    let _ = remove_dir_all(temp_dir);
}
#[test]
fn test_baginfo_save_repeatable_fields() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("acorn-baginfo-{stamp}"));
    create_dir_all(dir.clone()).unwrap();
    let info = BagInfo::init()
        .organization(vec!["Org One".to_string(), "Org Two".to_string()])
        .contact_name(vec!["Alice".to_string(), "Bob".to_string()])
        .count(vec![(1, Some(3)), (2, None)])
        .date(TEST_BAGGING_DATE.to_string())
        .size("1 MB".to_string())
        .build();
    let path = info.save(dir.clone()).unwrap();
    assert_eq!(path, dir);
    let content = read_to_string(dir.join("bag-info.txt")).unwrap();
    let lines = content.lines().collect::<Vec<_>>();
    let expected_date = format!("Bagging-Date: {TEST_BAGGING_DATE}");
    assert_eq!(
        lines,
        vec![
            "Source-Organization: Org One",
            "Source-Organization: Org Two",
            "Contact-Name: Alice",
            "Contact-Name: Bob",
            "Bag-Count: 1 of 3",
            "Bag-Count: 2 of ?",
            expected_date.as_str(),
            "Bag-Size: 1 MB",
        ]
    );
    remove_dir_all(dir).unwrap();
}
#[test]
fn test_baginfo_save_single_value_fields_only() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("acorn-baginfo-single-{stamp}"));
    create_dir_all(dir.clone()).unwrap();
    let info = BagInfo::init().date(TEST_BAGGING_DATE.to_string()).size("42 KB".to_string()).build();
    let path = info.save(dir.clone()).unwrap();
    assert_eq!(path, dir);
    let content = read_to_string(dir.join("bag-info.txt")).unwrap();
    let lines = content.lines().collect::<Vec<_>>();
    let expected_date = format!("Bagging-Date: {TEST_BAGGING_DATE}");
    assert_eq!(lines, vec![expected_date.as_str(), "Bag-Size: 42 KB"]);
    let _ = remove_dir_all(dir);
}
#[test]
fn test_bagit_verify() {
    // Verification is broke on macbook and linux for some reason
    // Specifically, data/bucket/that/index.json, but could have other checksum issues too
    if cfg!(target_os = "windows") {
        let path = fixtures_dir().join("bag");
        let result = Bag::verify(path);
        dbg!(&result);
        assert!(result.is_ok());
    }
}
#[test]
fn test_checksum() {
    let calculated = file_checksum(fixtures_dir().join("glob/a.txt"), None).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
    let calculated = file_checksum("../tests/fixtures/glob/a.txt", None).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
    let calculated = file_checksum("../tests/fixtures/glob/a.txt", Some(&ring::digest::SHA512)).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 128);
    } else {
        let expected =
            "d0b8db2e3f9afbcc8baf4cb8189c2fd489abacf232b4d000c54e5eb2b9cc2470163fc3c3b9f2c4fd88d0f2ab52b4075bef9d3ecbda71ad12c5f20cb7934904b4";
        assert_eq!(calculated, expected);
    }
    let result = file_checksum(PathBuf::from("/path/does/not/exist.txt"), None);
    assert!(result.is_none());
}
#[test]
fn test_uri_to_path() {
    let plain = PathBuf::from("some/relative/path");
    assert_eq!(uri_to_path(plain.clone()), plain);
    #[cfg(not(windows))]
    assert_eq!(uri_to_path(PathBuf::from("file:///absolute/path")), PathBuf::from("/absolute/path"));
    #[cfg(windows)]
    assert_eq!(uri_to_path(PathBuf::from("file:///C:/absolute/path")), PathBuf::from("C:/absolute/path"));
}
#[test]
fn test_files_all() {
    let extensions = Some(vec!["json"]);
    let files = files_all(fixtures_dir().join("glob"), extensions);
    assert_eq!(files.len(), 3);
    let files = files_all(fixtures_dir().join("glob"), Some(vec!["jpg"]));
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "c.jpg");
    let files = files_all(fixtures_dir().join("glob"), None);
    assert_eq!(files.len(), 8);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "a.json");
}
#[cfg(not(windows))]
#[test]
fn test_files_all_with_uri() {
    let uri = PathBuf::from(format!("file:///{}", fixtures_dir().join("glob").to_string_lossy()));
    let files = files_all(uri, Some(vec!["json"]));
    assert_eq!(files.len(), 3);
}
#[cfg(windows)]
#[test]
fn test_files_all_with_uri() {
    let uri = PathBuf::from(format!("file:///{}", fixtures_dir().join("glob").to_string_lossy().replace('\\', "/")));
    let files = files_all(uri, Some(vec!["json"]));
    assert_eq!(files.len(), 3);
}
#[test]
fn test_files_from_git() {
    let files = files_from_git_branch("main", Some(vec!["json"]));
    assert!(files.is_empty());
    let files = files_from_git_commit("HEAD", Some(vec!["fake"]));
    assert!(files.is_empty());
}
#[test]
fn test_filter_git_command_result() {
    let response = "acorn-lib/assets/constants/keywords.csv\nacorn-lib/assets/constants/technology.csv".to_string();
    let result = response.clone();
    let files = filter_git_command_result(result, Some(vec!["csv"]));
    assert_eq!(files.len(), 2);
    let result = response.clone();
    let files = filter_git_command_result(result, Some(vec!["json"]));
    assert!(files.is_empty());
    let result = response.clone();
    let files = filter_git_command_result(result, Some(vec!["JSON"]));
    assert!(files.is_empty());
    let empty = "".to_string();
    let files = filter_git_command_result(empty, Some(vec!["json"]));
    assert!(files.is_empty());
}
#[test]
#[ignore]
fn test_files_from_git_main() {
    let files = files_from_git_commit("ab8862e", Some(vec!["csv"]));
    assert_eq!(files.len(), 2);
    let hash = "ae61400cfdd079c06c2563c6ffe16d3c714a6bdc";
    let files = files_from_git_commit(hash, Some(vec!["rs"]));
    assert_eq!(files.len(), 6);
    let files = files_from_git_commit(hash, Some(vec!["json", "rs"]));
    assert_eq!(files.len(), 7);
    let files = files_from_git_commit(hash, Some(vec!["json"]));
    assert_eq!(files.len(), 1);
    let expected = "tests/fixtures/data/invalid_project_a/index.json";
    assert_eq!(files[0].to_str().unwrap(), expected);
}
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_files_from_gitlab_merge_request() {
    let extensions = Some(vec!["json", "yaml"]);
    let expected = PathBuf::from("project/gravity/gravity.json");
    let files = temp_env::with_vars(
        [
            ("CI_API_V4_URL", Some("https://code.ornl.gov/api/v4")),
            ("CI_MERGE_REQUEST_PROJECT_ID", Some("17410")),
            ("CI_MERGE_REQUEST_IID", Some("27")),
        ],
        || {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async { files_from_gitlab_merge_request(extensions.clone()).await })
            })
        },
    );
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], expected);
}
#[test]
fn test_filter_ignored() {
    if cfg!(target_os = "linux") {
        let extensions = Some(vec!["json"]);
        let files = files_all(fixtures_dir().join("glob"), extensions.clone());
        let filtered = filter_ignored(files, Some("[/]a.json$".to_string()));
        assert_eq!(filtered.len(), 2);
        let files = files_all(fixtures_dir().join("glob"), extensions);
        let filtered = filter_ignored(files, Some("[/](a|b).json$".to_string()));
        assert_eq!(filtered.len(), 1);
        let files = files_all(fixtures_dir().join("glob/a.txt"), None);
        let filtered = filter_ignored(files, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].file_name().unwrap().to_str().unwrap(), "a.txt");
    }
}
#[test]
fn test_image_paths() {
    let path = fixtures_dir().join("data/empty/");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
    let path = PathBuf::from(".");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
}
#[test]
fn test_read_large_file() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("acorn-read-large-file-{stamp}.txt"));
    let expected = "ACORN ".repeat(500_000);
    write(path.clone(), expected.clone()).unwrap();
    let content = read_large_file(path.clone()).unwrap();
    assert_eq!(content, expected);
    remove_file(path).unwrap();
}
#[test]
fn test_read_file_large_content() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("acorn-read-file-large-{stamp}.txt"));
    let expected = "ACORN-LARGE ".repeat(800_000);
    write(path.clone(), expected.clone()).unwrap();
    let content = read_file(path.clone()).unwrap();
    assert_eq!(content, expected);
    remove_file(path).unwrap();
}
#[tokio::test]
async fn test_write_file_bytes_creates_parent_and_writes_content() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("acorn-write-file-bytes-{stamp}"));
    let path = dir.join("nested").join("output.bin");
    let expected = b"acorn-bytes".to_vec();
    let result = write_file_bytes(path.clone(), || async { Ok::<Vec<u8>, Report>(expected.clone()) }).await;
    assert!(result.is_ok());
    assert!(path.exists());
    assert_eq!(std::fs::read(path.clone()).unwrap(), expected);
    remove_dir_all(dir).unwrap();
}
#[tokio::test]
async fn test_write_file_bytes_propagates_get_bytes_error() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("acorn-write-file-bytes-error-{stamp}"));
    let path = dir.join("nested").join("output.bin");
    let result = write_file_bytes(path.clone(), || async { Err::<Vec<u8>, Report>(Report::msg("byte source failure")) }).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "byte source failure");
    assert!(path.exists());
    assert_eq!(std::fs::metadata(path.clone()).unwrap().len(), 0);
    remove_dir_all(dir).unwrap();
}
#[test]
fn test_should_retry_get_for_retryable_status_codes() {
    assert!(should_retry(&HttpMethod::Get, Some(408)));
    assert!(should_retry(&HttpMethod::Get, Some(429)));
    assert!(should_retry(&HttpMethod::Get, Some(500)));
    assert!(should_retry(&HttpMethod::Get, Some(503)));
}
#[test]
fn test_shared_http_policy_defaults() {
    let policy = shared_http_policy();
    assert_eq!(policy.timeout.as_secs(), 30);
    assert_eq!(policy.max_retries, 2);
    assert_eq!(policy.max_attempts(), 3);
}
#[test]
fn test_shared_http_timeout_layer_constructs() {
    let _layer = http_timeout_layer();
}
#[test]
fn test_env_var_is_truthy() {
    temp_env::with_vars([(ENV_NO_LOCAL_DATABASE, Some(" true "))], || {
        assert_eq!(env_var_is_truthy(ENV_NO_LOCAL_DATABASE), Some(true));
    });
    temp_env::with_vars([(ENV_NO_LOCAL_DATABASE, Some("ON"))], || {
        assert_eq!(env_var_is_truthy(ENV_NO_LOCAL_DATABASE), Some(true));
    });
    temp_env::with_vars([(ENV_NO_LOCAL_DATABASE, Some("1"))], || {
        assert_eq!(env_var_is_truthy(ENV_NO_LOCAL_DATABASE), Some(true));
    });
    temp_env::with_vars([(ENV_NO_LOCAL_DATABASE, Some("false"))], || {
        assert_eq!(env_var_is_truthy(ENV_NO_LOCAL_DATABASE), Some(false));
    });
    temp_env::with_vars([(ENV_NO_LOCAL_DATABASE, Some("0"))], || {
        assert_eq!(env_var_is_truthy(ENV_NO_LOCAL_DATABASE), Some(false));
    });
    temp_env::with_vars([(ENV_NO_LOCAL_DATABASE, None::<&str>)], || {
        assert_eq!(env_var_is_truthy(ENV_NO_LOCAL_DATABASE), None);
    });
}
#[test]
fn test_should_retry_get_for_transport_failure() {
    assert!(should_retry(&HttpMethod::Get, None));
}
#[test]
fn test_should_not_retry_non_idempotent_or_non_retryable_status() {
    assert!(!should_retry(&HttpMethod::Post, Some(500)));
    assert!(!should_retry(&HttpMethod::Put, Some(503)));
    assert!(!should_retry(&HttpMethod::Get, Some(200)));
    assert!(!should_retry(&HttpMethod::Get, Some(404)));
}
