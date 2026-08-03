#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::io::api::{TreeEntry, TreeEntryType};
use crate::io::bagit::{Bag, BagInfo, Save};
use crate::io::config::{ApplicationConfiguration, Bucket, FilterSet};
use crate::io::http::policy::{http_timeout_layer, shared_http_policy};
use crate::io::http::{should_retry, HttpMethod};
use crate::io::{
    archive, create_rsa_keypair, current_date, env_var_is_truthy, file_checksum, files_all, files_from_git_branch, files_from_git_commit,
    files_from_gitlab_merge_request, filter_git_command_result, filter_ignored, first_env_var, image_paths, jsonc_parse_value, read_file,
    read_large_file, unique_file_extensions, uri_to_path, write_file_bytes, write_rsa_keypair, Executor, InputOutput, Remote, Source, SourceAction,
};
use crate::prelude::OsString;
use crate::util::constants::env::NO_LOCAL_DATABASE;
use crate::{args, Location, Repository, Scheme};
use color_eyre::Report;
use fluent_uri::Uri as UriParse;
use indicatif::{InMemoryTerm, MultiProgress, ProgressDrawTarget, ProgressStyle, TermLike};
use rsa::traits::PublicKeyParts;
use std::env::temp_dir;
use std::fs::{create_dir_all, read_to_string, remove_dir_all, remove_file, write, File};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

use super::{create_progress_bar_with_renderer, finish_progress_bar, ProgressType};

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
                    "template": "{{ base }}/repos/{{ path }}/git/trees/{{ branch }}{{ query }}"
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
            "name": "languages::gitlab",
            "domain": "gitlab.com",
            "resources": [
                {
                    "name": "languages",
                    "method": "get",
                    "template": "{{ base }}/gitlab-org/gitlab/-/raw/master/vendor/languages.yml"
                }
            ]
        },
        {
            "name": "languages::github",
            "domain": "raw.githubusercontent.com",
            "resources": [
                {
                    "name": "languages",
                    "method": "get",
                    "template": "{{ base }}/github-linguist/linguist/refs/heads/main/lib/linguist/languages.yml"
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
fn test_config_with_endpoints() {
    let path = fixtures_dir().join("config").join("with_endpoints.json");
    let config = ApplicationConfiguration::read_json(path);
    let endpoints = config.unwrap().endpoints.unwrap();
    assert_eq!(endpoints[0].resources.len(), 2);
}
#[test]
fn test_bucket_config_jsonc() {
    let path = fixtures_dir().join("config").join("with_buckets.jsonc");
    let config = ApplicationConfiguration::read_jsonc(path).unwrap();
    let buckets = config.buckets.unwrap();
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].name.as_deref(), Some("test-bucket"));
}
#[test]
fn test_jsonc_parse_with_line_comments() {
    let content = r#"{
        // This is a comment
        "buckets": [{"name": "test", "repository": {"provider": "git", "location": "file:./"}}]
    }"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let buckets = config.buckets.unwrap();
    assert_eq!(buckets.len(), 1);
}
#[test]
fn test_jsonc_parse_with_block_comments() {
    let content = r#"{
        /* Block comment */
        "buckets": []
    }"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    assert!(config.buckets.unwrap().is_empty());
}
#[test]
fn test_jsonc_parse_with_trailing_commas() {
    let content = r#"{
        "buckets": [],
    }"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    assert!(config.buckets.unwrap().is_empty());
}
#[test]
fn test_jsonc_rejects_single_quotes() {
    let content = r#"{'buckets': []}"#;
    assert!(jsonc_parse_value(content).is_err());
}
#[test]
fn test_jsonc_rejects_unquoted_keys() {
    let content = r#"{ buckets: [] }"#;
    assert!(jsonc_parse_value(content).is_err());
}
#[test]
fn test_json_equivalent_to_jsonc() {
    let json = r#"{"buckets":[{"name":"t","repository":{"provider":"git","location":"file:./"}}]}"#;
    let jsonc = r#"{
        // comment
        "buckets": [{"name":"t","repository":{"provider":"git","location":"file:./"}},]
    }"#;
    let a = ApplicationConfiguration::parse(json).unwrap();
    let b = ApplicationConfiguration::parse(jsonc).unwrap();
    assert_eq!(a.buckets.as_ref().unwrap().len(), b.buckets.as_ref().unwrap().len());
}
#[test]
fn test_json_accepts_comments_via_jsonc_fallback() {
    let content = r#"{
        // This comment is valid via JSONC fallback
        "buckets": []
    }"#;
    assert!(ApplicationConfiguration::parse(content).is_ok());
}
#[test]
fn test_create_rsa_keypair_returns_keys() {
    let result = create_rsa_keypair();
    assert!(result.is_ok(), "create_rsa_keypair should produce a keypair");
    let (private_key, public_key) = result.unwrap();
    assert_eq!(private_key.n().bits(), 2048, "private key modulus should be 2048 bits");
    assert_eq!(public_key.n(), private_key.n(), "public and private keys should share the same modulus");
}
#[test]
fn test_write_rsa_keypair_writes_files() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = temp_dir().join(format!("acorn-write-rsa-keypair-{stamp}"));
    create_dir_all(dir.clone()).unwrap();
    let path = dir.join("test_key");
    let keypair = create_rsa_keypair().unwrap();
    let result = write_rsa_keypair(keypair, Some(path.clone()));
    assert!(result.is_ok(), "write_rsa_keypair should succeed");
    let (private_path, public_path) = result.unwrap();
    assert_eq!(private_path, path, "should return the private key path");
    assert_eq!(
        public_path,
        PathBuf::from(format!("{}.pub", path.display())),
        "should return the public key path"
    );
    assert!(private_path.exists(), "private key file should exist");
    assert!(public_path.exists(), "public key file should exist");
    let private_content = read_to_string(private_path).unwrap();
    let public_content = read_to_string(public_path).unwrap();
    assert!(private_content.starts_with("-----BEGIN PRIVATE KEY-----"), "private key should be PEM");
    assert!(
        private_content.trim().ends_with("-----END PRIVATE KEY-----"),
        "private key should end with PEM footer"
    );
    assert!(public_content.starts_with("-----BEGIN PUBLIC KEY-----"), "public key should be PEM");
    assert!(
        public_content.trim().ends_with("-----END PUBLIC KEY-----"),
        "public key should end with PEM footer"
    );
    remove_dir_all(dir).unwrap();
}
#[test]
fn test_write_rsa_keypair_uses_current_dir_when_path_is_none() {
    let keypair = create_rsa_keypair().unwrap();
    let result = write_rsa_keypair(keypair, None::<PathBuf>);
    assert!(result.is_ok(), "write_rsa_keypair should succeed with None path");
    let (private_path, public_path) = result.unwrap();
    let cwd = std::env::current_dir().unwrap().join("id_rsa");
    assert_eq!(private_path, cwd, "should use cwd/id_rsa as private key path");
    assert_eq!(
        public_path,
        PathBuf::from(format!("{}.pub", cwd.display())),
        "should use cwd/id_rsa.pub as public key path"
    );
    assert!(private_path.exists(), "private key file should exist in cwd");
    assert!(public_path.exists(), "public key file should exist in cwd");
    remove_file(private_path).unwrap();
    remove_file(public_path).unwrap();
}
#[test]
fn test_write_rsa_keypair_returns_error_for_invalid_path() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = temp_dir().join(format!("acorn-write-rsa-keypair-invalid-{stamp}"));
    let path = dir.join("subdir").join("test_key");
    let keypair = create_rsa_keypair().unwrap();
    let result = write_rsa_keypair(keypair, Some(path.clone()));
    assert!(result.is_err(), "write_rsa_keypair should fail when parent directory missing");
}
#[test]
fn test_write_rsa_keypair_returns_error_when_public_key_write_fails() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = temp_dir().join(format!("acorn-write-rsa-keypair-pub-fail-{stamp}"));
    create_dir_all(dir.clone()).unwrap();
    let path = dir.join("test_key");

    // Block the public key path by creating a directory in its place
    let public_key_path = PathBuf::from(format!("{}.pub", path.display()));
    create_dir_all(&public_key_path).unwrap();

    let keypair = create_rsa_keypair().unwrap();
    let result = write_rsa_keypair(keypair, Some(path.clone()));

    assert!(
        result.is_err(),
        "write_rsa_keypair should fail when public key path is blocked by a directory"
    );

    remove_dir_all(dir).unwrap();
}
#[test]
fn test_current_date() {
    let date = current_date();
    assert_eq!(date.len(), 10);
    assert_eq!(date.split("-").count(), 3);
}
#[test]
fn test_gitlab_tree_entry() {
    let entry = TreeEntry {
        id: Some(1234.to_string()),
        name: Some("acorn".to_string()),
        entry_type: TreeEntryType::Directory,
        path: "acorn".to_string(),
        mode: Some("example".to_string()),
        sha: None,
        size: None,
        url: None,
    };
    assert!(!entry.is_file());
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
        revision: None,
    };
    assert!(!location.exists().await);
    let location = Location::Detailed {
        scheme: Scheme::File,
        uri: "should fail to parse?".into(),
        revision: None,
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
        | Some(value) => {
            let uri = UriParse::parse(value.as_str()).unwrap();
            assert_eq!(uri.scheme().as_str().to_ascii_lowercase(), "https");
            assert_eq!(uri.path(), "/GSHS/GDS/Common/PIPE/module-a");
        }
        | _ => panic!(),
    }
    let location = Location::Simple("ssh://git@code.ornl.gov/GSHS/GDS/Common/PIPE/module-a.git".into());
    match location.uri() {
        | Some(value) => {
            let uri = UriParse::parse(value.as_str()).unwrap();
            assert_eq!(uri.scheme().as_str().to_ascii_lowercase(), "ssh");
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
    let file = File::open(output_file).unwrap();
    let mut zip = ZipArchive::new(file).unwrap();
    let names = (0..zip.len())
        .filter_map(|index| zip.by_index(index).ok().map(|entry| entry.name().to_string()))
        .collect::<Vec<_>>();
    assert!(names.iter().any(|name| name == "highlight/reference.pptx"));
}
#[test]
fn test_bagit() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temp_dir = temp_dir().join(format!("acorn-bagit-{stamp}"));
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
    assert_eq!(bag.payload.len(), 28);
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
    let dir = temp_dir().join(format!("acorn-baginfo-{stamp}"));
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
    let dir = temp_dir().join(format!("acorn-baginfo-single-{stamp}"));
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
        assert!(result.is_ok());
    }
}
#[test]
fn test_checksum() {
    let calculated = file_checksum(fixtures_dir().join("glob/a.txt"), None).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.checksum_value.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated.checksum_value, expected);
    }
    let calculated = file_checksum("../tests/fixtures/glob/a.txt", None).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.checksum_value.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated.checksum_value, expected);
    }
    let calculated = file_checksum("../tests/fixtures/glob/a.txt", Some(&ring::digest::SHA512)).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.checksum_value.len(), 128);
    } else {
        let expected =
            "d0b8db2e3f9afbcc8baf4cb8189c2fd489abacf232b4d000c54e5eb2b9cc2470163fc3c3b9f2c4fd88d0f2ab52b4075bef9d3ecbda71ad12c5f20cb7934904b4";
        assert_eq!(calculated.checksum_value, expected);
    }
    let result = file_checksum(PathBuf::from("/path/does/not/exist.txt"), None);
    assert!(result.is_none());
}
#[test]
fn test_env_var_is_truthy() {
    temp_env::with_vars([(NO_LOCAL_DATABASE, Some(" true "))], || {
        assert_eq!(env_var_is_truthy(NO_LOCAL_DATABASE), Some(true));
    });
    temp_env::with_vars([(NO_LOCAL_DATABASE, Some("ON"))], || {
        assert_eq!(env_var_is_truthy(NO_LOCAL_DATABASE), Some(true));
    });
    temp_env::with_vars([(NO_LOCAL_DATABASE, Some("1"))], || {
        assert_eq!(env_var_is_truthy(NO_LOCAL_DATABASE), Some(true));
    });
    temp_env::with_vars([(NO_LOCAL_DATABASE, Some("false"))], || {
        assert_eq!(env_var_is_truthy(NO_LOCAL_DATABASE), Some(false));
    });
    temp_env::with_vars([(NO_LOCAL_DATABASE, Some("0"))], || {
        assert_eq!(env_var_is_truthy(NO_LOCAL_DATABASE), Some(false));
    });
    temp_env::with_vars([(NO_LOCAL_DATABASE, None::<&str>)], || {
        assert_eq!(env_var_is_truthy(NO_LOCAL_DATABASE), None);
    });
}
#[test]
fn test_executor_display() {
    assert_eq!(Executor::Apptainer.to_string(), "apptainer");
    assert_eq!(Executor::Docker.to_string(), "docker");
    assert_eq!(Executor::Podman.to_string(), "podman");
    assert_eq!(Executor::Sandbox.to_string(), "sandbox");
    assert_eq!(Executor::Shell.to_string(), "shell");
    assert_eq!(Executor::Ssh.to_string(), "ssh");
    assert_eq!(Executor::Kubernetes.to_string(), "kubernetes");
    assert_eq!(Executor::VirtualMachine.to_string(), "virtual_machine");
    assert_eq!(Executor::from("custom").to_string(), "custom");
}
#[test]
fn test_executor_into_string() {
    let s: String = Executor::Docker.into();
    assert_eq!(s, "docker");
    let os: OsString = Executor::Docker.into();
    assert_eq!(os, OsString::from("docker"));
}
#[test]
fn test_executor_roundtrip() {
    for executor in &[
        Executor::Apptainer,
        Executor::Docker,
        Executor::Podman,
        Executor::Sandbox,
        Executor::Shell,
        Executor::Ssh,
        Executor::Kubernetes,
        Executor::VirtualMachine,
    ] {
        let s = executor.to_string();
        let restored = Executor::from(s.as_str());
        assert_eq!(*executor, restored);
    }
    let executor = Executor::from("custom");
    let s = executor.to_string();
    let restored = Executor::from(s.as_str());
    assert_eq!(executor, restored);
}
#[test]
fn test_executor_command_name() {
    assert_eq!(Executor::Docker.command().unwrap(), "docker");
    assert_eq!(Executor::Podman.command().unwrap(), "podman");
    assert!(Executor::VirtualMachine.command().is_none());
    assert_eq!(Executor::from("gitlab-runner").command().unwrap(), "gitlab-runner");
}
#[test]
fn test_executor_gitlab_name() {
    assert_eq!(Executor::Docker.gitlab_runner_type(), "docker");
    assert_eq!(Executor::Podman.gitlab_runner_type(), "docker");
    assert_eq!(Executor::Apptainer.gitlab_runner_type(), "docker");
    assert_eq!(Executor::Shell.gitlab_runner_type(), "shell");
    assert_eq!(Executor::Ssh.gitlab_runner_type(), "ssh");
    assert_eq!(Executor::Kubernetes.gitlab_runner_type(), "kubernetes");
    assert_eq!(Executor::Sandbox.gitlab_runner_type(), "docker");
    assert_eq!(Executor::from("anything").gitlab_runner_type(), "docker");
    assert!(["virtualbox", "parallels"].contains(&Executor::VirtualMachine.gitlab_runner_type()));
}
#[test]
fn test_remote_docker_args() {
    let command = args!["run", "--detach", "acorn:latest"];
    let remote = "ssh://builder".parse::<Remote>().unwrap();
    assert_eq!(
        remote.docker_args(command),
        args!["--host", "ssh://builder", "run", "--detach", "acorn:latest"]
    );
}
#[test]
fn test_remote_parse_and_docker_executor() {
    let remote = "ssh://deploy@example.org:2222/var/run/docker.sock".parse::<Remote>().unwrap();
    assert_eq!(remote.as_str(), "ssh://deploy@example.org:2222/var/run/docker.sock");
    assert!(Executor::Docker.is_docker());
    assert!(!Executor::Podman.is_docker());
    [
        "http://builder",
        "ssh://",
        "ssh://user:password@builder",
        "ssh://builder?query=value",
        "ssh://builder#fragment",
        "ssh://builder:65536",
    ]
    .into_iter()
    .for_each(|value| assert!(value.parse::<Remote>().is_err()));
}
#[test]
fn test_remote_gpu_template_copy_targets_the_requested_docker_host() {
    let remote = "ssh://builder".parse::<Remote>().unwrap();
    let template = PathBuf::from("/tmp/gpu.template.toml");
    assert_eq!(
        remote.docker_args(args!["cp", &template, "runner-42:/etc/gitlab-runner/gpu.template.toml"]),
        args![
            "--host",
            "ssh://builder",
            "cp",
            template,
            "runner-42:/etc/gitlab-runner/gpu.template.toml"
        ]
    );
}
#[test]
fn test_first_env_var() {
    // Test when none of the variables are set
    assert_eq!(first_env_var(&["NON_EXISTENT_VAR1", "NON_EXISTENT_VAR2"]), None);
    // Test when the first variable is set
    temp_env::with_vars([("FIRST_VAR", Some("first_value"))], || {
        assert_eq!(first_env_var(&["FIRST_VAR", "SECOND_VAR"]), Some("first_value".to_string()));
    });
    // Test when the second variable is set but first is not
    temp_env::with_vars([("MISSING_VAR", None::<&str>), ("SECOND_VAR", Some("second_value"))], || {
        assert_eq!(first_env_var(&["MISSING_VAR", "SECOND_VAR"]), Some("second_value".to_string()));
    });
    // Test with multiple variables where first one wins
    temp_env::with_vars([("WINNING_VAR", Some("winning_value")), ("LOSING_VAR", Some("losing_value"))], || {
        assert_eq!(first_env_var(&["WINNING_VAR", "LOSING_VAR"]), Some("winning_value".to_string()));
    });
    temp_env::with_vars([("EMPTY_VAR", Some("  ")), ("FALLBACK_VAR", Some("fallback_value"))], || {
        assert_eq!(first_env_var(&["EMPTY_VAR", "FALLBACK_VAR"]), Some("fallback_value".to_string()));
    });
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
        let filtered = filter_ignored(files, Some("[/]a.json$".to_string())).unwrap();
        assert_eq!(filtered.len(), 2);
        let files = files_all(fixtures_dir().join("glob"), extensions);
        let filtered = filter_ignored(files, Some("[/](a|b).json$".to_string())).unwrap();
        assert_eq!(filtered.len(), 1);
        let files = files_all(fixtures_dir().join("glob/a.txt"), None);
        let filtered = filter_ignored(files, None).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].file_name().unwrap().to_str().unwrap(), "a.txt");
    }
}
#[test]
fn test_filter_set_filter() {
    use crate::io::Source;
    let selectors = vec![Source::from("alpha.gguf"), Source::from("beta.safetensors")];
    let filtered = FilterSet::filter(selectors, &["gguf$".to_string()], &["alpha".to_string()], Source::identifier, |_| true).unwrap();
    assert!(filtered.is_empty());
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
    let path = temp_dir().join(format!("acorn-read-large-file-{stamp}.txt"));
    let expected = "ACORN ".repeat(500_000);
    write(path.clone(), expected.clone()).unwrap();
    let content = read_large_file(path.clone()).unwrap();
    assert_eq!(content, expected);
    remove_file(path).unwrap();
}
#[test]
fn test_read_file_large_content() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = temp_dir().join(format!("acorn-read-file-large-{stamp}.txt"));
    let expected = "ACORN-LARGE ".repeat(800_000);
    write(path.clone(), expected.clone()).unwrap();
    let content = read_file(path.clone()).unwrap();
    assert_eq!(content, expected);
    remove_file(path).unwrap();
}
#[tokio::test]
async fn test_read_source_from_local_path() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = temp_dir().join(format!("acorn-read-source-local-{stamp}.yaml"));
    let expected = "openapi: 3.1.0\n".to_string();
    write(path.clone(), expected.clone()).unwrap();
    let content = Source::read(path.to_string_lossy().as_ref(), false).await.unwrap();
    assert_eq!(content, expected);
    remove_file(path).unwrap();
}
#[cfg(not(windows))]
#[tokio::test]
async fn test_read_source_from_file_uri_localhost() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = temp_dir().join(format!("acorn-read-source-localhost-{stamp}.yaml"));
    let expected = "openapi: 3.1.0\ninfo:\n  title: Localhost\n".to_string();
    write(path.clone(), expected.clone()).unwrap();
    let source = format!("file://localhost{}", path.to_string_lossy());
    let content = Source::read(&source, false).await.unwrap();
    assert_eq!(content, expected);
    remove_file(path).unwrap();
}
#[tokio::test]
async fn test_read_source_from_file_uri() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = temp_dir().join(format!("acorn-read-source-uri-{stamp}.yaml"));
    let expected = "openapi: 3.1.0\ninfo:\n  title: URI\n".to_string();
    write(path.clone(), expected.clone()).unwrap();
    let source = if cfg!(windows) {
        format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
    } else {
        format!("file://{}", path.to_string_lossy())
    };
    let content = Source::read(&source, false).await.unwrap();
    assert_eq!(content, expected);
    remove_file(path).unwrap();
}
#[tokio::test]
async fn test_read_source_rejects_remote_when_offline() {
    let result = Source::read("https://example.com/openapi.yaml", true).await;
    assert!(result.is_err());
    let why = result.unwrap_err().to_string();
    assert!(why.contains("while offline"));
}
#[tokio::test]
async fn test_read_source_rejects_unsupported_uri_scheme() {
    let result = Source::read("s3://example/openapi.yaml", false).await;
    assert!(result.is_err());
    let why = result.unwrap_err().to_string();
    assert!(why.contains("Unsupported source URI scheme"));
}
#[test]
fn test_uri_to_path_normalizes_file_sources() {
    assert_eq!(uri_to_path("file:./models/qwen.gguf"), PathBuf::from("./models/qwen.gguf"));
    assert_eq!(uri_to_path("./models/qwen.gguf"), PathBuf::from("./models/qwen.gguf"));
    let path = temp_dir().join("acorn-local-source-to-path.gguf");
    let localhost = if cfg!(windows) {
        format!("file://localhost/{}", path.to_string_lossy().replace('\\', "/"))
    } else {
        format!("file://localhost{}", path.to_string_lossy())
    };
    let file_uri = if cfg!(windows) {
        format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
    } else {
        format!("file://{}", path.to_string_lossy())
    };
    assert_eq!(uri_to_path(&localhost), path);
    assert_eq!(uri_to_path(&file_uri), path);
}
#[test]
fn test_location_is_local_detects_local_paths() {
    assert!(Location::from("file:./models/qwen.gguf").is_local());
    assert!(Location::from("./models/qwen.gguf").is_local());
    assert!(Location::from("../models/qwen.gguf").is_local());
    assert!(Location::from(temp_dir().to_string_lossy().as_ref()).is_local());
    assert!(!Location::from("meta-llama/Llama-2-7b-hf").is_local());
    assert!(!Location::from("https://huggingface.co/hf-internal-testing/tiny-random-bert").is_local());
}
#[test]
fn test_source_from_str_detects_local_paths() {
    let source = Source::from("file:./models/qwen.gguf");
    assert!(source.is_local());
    assert_eq!(source.name(), "qwen");
    let source = Source::from("./models/qwen.gguf");
    assert!(source.is_local());
    let source = Source::from("meta-llama/Llama-2-7b-hf");
    assert!(source.is_remote());
    assert_eq!(source.name(), "meta-llama/Llama-2-7b-hf");
}
#[test]
fn test_source_from_repository_uses_hugging_face_identifier() {
    let repository = Repository::HuggingFace {
        location: Location::Simple("https://huggingface.co/hf-internal-testing/tiny-random-bert".to_string()),
    };
    let source = Source::from(&repository).with_name("tiny-bert");
    assert!(source.is_remote());
    assert_eq!(source.name(), "tiny-bert");
    assert_eq!(source.identifier(), "hf-internal-testing/tiny-random-bert");
}
#[test]
fn test_source_local_action_metadata() {
    let source = Source::from("./models/qwen.gguf").with_action(Some(SourceAction::Copy));
    assert_eq!(source.name(), "qwen");
    match source {
        | Source::Local { action, .. } => assert_eq!(action, Some(SourceAction::Copy)),
        | _ => panic!("model selector should be local"),
    }
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
#[test]
fn test_unique_file_extensions() {
    let paths = vec![
        PathBuf::from("project/index.json"),
        PathBuf::from("project/data.yaml"),
        PathBuf::from("project/about.JSON"),
        PathBuf::from("project/readme"),
    ];
    let extensions = unique_file_extensions(&paths);
    assert_eq!(extensions, vec!["json", "yaml"]);
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
#[tokio::test]
async fn test_write_file_bytes_creates_parent_and_writes_content() {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = temp_dir().join(format!("acorn-write-file-bytes-{stamp}"));
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
    let dir = temp_dir().join(format!("acorn-write-file-bytes-error-{stamp}"));
    let path = dir.join("nested").join("output.bin");
    let result = write_file_bytes(path.clone(), || async { Err::<Vec<u8>, Report>(Report::msg("byte source failure")) }).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "byte source failure");
    assert!(path.exists());
    assert_eq!(std::fs::metadata(path.clone()).unwrap().len(), 0);
    remove_dir_all(dir).unwrap();
}
#[test]
fn test_progress_renderer_coordinates_log_and_bar_redraw() {
    let terminal = InMemoryTerm::new(10, 80);
    let renderer = MultiProgress::with_draw_target(ProgressDrawTarget::term_like(Box::new(terminal.clone())));
    let progress = create_progress_bar_with_renderer(2, ProgressType::Bar, &renderer);
    progress.set_style(ProgressStyle::with_template("{pos}/{len} {msg}").unwrap());
    progress.set_message("first item");
    progress.set_position(1);
    progress.tick();
    assert!(terminal.contents().contains("1/2 first item"));
    renderer
        .suspend(|| terminal.write_line("INFO checkpoint"))
        .expect("write coordinated log line");
    progress.set_message("second item");
    progress.set_position(2);
    progress.tick();
    let contents = terminal.contents();
    assert!(contents.contains("INFO checkpoint"));
    assert!(contents.contains("2/2 second item"));
    assert!(!contents.contains("first item"), "redraw should replace the previous progress line");
    finish_progress_bar(&progress, "download complete".to_string());
    assert!(terminal.contents().contains("download complete"));
}
#[test]
fn test_progress_renderer_spinner_and_error_cleanup() {
    let terminal = InMemoryTerm::new(10, 80);
    let renderer = MultiProgress::with_draw_target(ProgressDrawTarget::term_like(Box::new(terminal.clone())));
    let progress = create_progress_bar_with_renderer(0, ProgressType::Spinner, &renderer);
    progress.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
    progress.set_message("waiting");
    progress.tick();
    assert!(terminal.contents().contains("waiting"));
    progress.finish_and_clear();
    assert!(
        !terminal.contents().contains("waiting"),
        "failed operations should be able to clear their indicator"
    );
}
#[test]
fn test_progress_renderer_silent_mode_stays_hidden() {
    let terminal = InMemoryTerm::new(10, 80);
    let renderer = MultiProgress::with_draw_target(ProgressDrawTarget::term_like(Box::new(terminal.clone())));
    let progress = create_progress_bar_with_renderer(1, ProgressType::Silent, &renderer);
    progress.set_message("hidden work");
    progress.inc(1);
    assert!(progress.is_hidden());
    assert!(terminal.contents().is_empty());
}
#[test]
fn test_progress_renderer_coordinates_parallel_bars_and_logs() {
    let terminal = InMemoryTerm::new(40, 80);
    let renderer = MultiProgress::with_draw_target(ProgressDrawTarget::term_like(Box::new(terminal.clone())));
    let first = create_progress_bar_with_renderer(20, ProgressType::Bar, &renderer);
    let second = create_progress_bar_with_renderer(20, ProgressType::Bar, &renderer);
    first.set_style(ProgressStyle::with_template("first {pos}/{len} {msg}").unwrap());
    second.set_style(ProgressStyle::with_template("second {pos}/{len} {msg}").unwrap());
    let first_progress = first.clone();
    let second_progress = second.clone();
    let first_handle = std::thread::spawn(move || {
        (1..=20).for_each(|position| first_progress.set_position(position));
    });
    let second_handle = std::thread::spawn(move || {
        (1..=20).for_each(|position| second_progress.set_position(position));
    });
    let log_renderer = renderer.clone();
    let log_terminal = terminal.clone();
    let log_handle = std::thread::spawn(move || {
        (1..=20).for_each(|position| {
            log_renderer
                .suspend(|| log_terminal.write_line(&format!("log {position}")))
                .expect("write coordinated parallel log line");
        });
    });
    first_handle.join().expect("join first progress thread");
    second_handle.join().expect("join second progress thread");
    log_handle.join().expect("join progress log thread");
    first.tick();
    second.tick();
    let contents = terminal.contents();
    let first_position = contents.find("first 20/20").expect("first completed progress line");
    let second_position = contents.find("second 20/20").expect("second completed progress line");
    assert!(first_position < second_position, "parallel bars should retain registration order");
    assert!(contents.contains("log 20"));
    first.finish_and_clear();
    second.finish_and_clear();
}
