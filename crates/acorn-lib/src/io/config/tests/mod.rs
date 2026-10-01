#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use crate::{
    io::{
        api::{Authenticator, Credential},
        config::{
            bucket::source::{BucketSourceItem, DownloadOptions, RepositoryBucketSource},
            transfer::{count_image_files, count_json_files, has_image_extension, operations_complete_message, TransferItem},
        },
        read_file, InputOutput, Parse,
    },
    util::constants::app::{ACRE_SCHEMA_VERSION, DEFAULT_CONFIG_FILENAMES, GITLAB_CREDENTIAL_VARIABLE_PAIRS},
};
use acorn_core::{options::CommonRuntime, Location, Repository};
use acorn_host::fs::{FsError, SafePath};
use fancy_regex::Regex;
use secrecy::ExposeSecret;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::{
    fs::{create_dir_all, read, read_to_string, remove_dir_all, write},
    path::{Path, PathBuf},
    sync::Mutex,
};
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
                },
                {
                    "name": "languages",
                    "method": "get",
                    "template": "{{ base }}/repos/{{ path }}/languages"
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
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}
fn source_items(paths: &[&str]) -> Vec<BucketSourceItem> {
    paths
        .iter()
        .map(|path| BucketSourceItem::new((*path).to_string(), path, None, None).unwrap())
        .collect()
}
fn temp_resolve_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(core::time::Duration::from_nanos(0))
        .as_nanos();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test_artifacts")
        .join(format!("{name}-{nanos}"))
}
#[test]
fn test_bucket() {
    let bucket = Bucket {
        name: Some("nssd".to_string()),
        description: Some("Bucket for NSSD".to_string()),
        source: Repository::GitLab {
            id: Some(1234_u64),
            location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string())),
        }
        .into(),
    };
    assert_eq!(bucket.domain().unwrap(), "code.ornl.gov".to_string());
    let bucket = Bucket {
        name: Some("nssd".to_string()),
        description: Some("Bucket for NSSD".to_string()),
        source: Repository::GitHub {
            location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string())),
        }
        .into(),
    };
    assert_eq!(bucket.domain().unwrap(), "code.ornl.gov".to_string());
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
fn test_bucket_config_jsonc() {
    let path = fixtures_dir().join("config").join("with_buckets.jsonc");
    let buckets = ApplicationConfiguration::read_jsonc(path).unwrap().buckets.unwrap();
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].name.as_deref(), Some("test-bucket"));
}
#[test]
fn test_bucket_config_yaml() {
    let path = fixtures_dir().join("config").join("with_remote_buckets.yaml");
    let config = ApplicationConfiguration::read_yaml(path).unwrap();
    assert_eq!(config.buckets.as_ref().map_or(0, |b| b.len()), 3);
}
#[test]
fn test_acre_schema_version_defaults_and_serializes() {
    let configuration = ApplicationConfiguration::parse("{}").expect("parse legacy configuration");
    assert_eq!(configuration.schema_version, AcreSchemaVersion::V1_0_0);
    assert_eq!(serde_json::to_value(configuration).unwrap()["schemaVersion"], ACRE_SCHEMA_VERSION);
}
#[test]
fn test_acre_rejects_unsupported_schema_versions_and_unknown_fields() {
    [
        r#"{"schemaVersion":"v1.0.0"}"#,
        r#"{"schemaVersion":"2.0.0"}"#,
        r#"{"schemaVersion":1}"#,
        r#"{"unknown":true}"#,
    ]
    .into_iter()
    .for_each(|content| assert!(ApplicationConfiguration::parse(content).is_err(), "accepted {content}"));
    let message = ApplicationConfiguration::parse(r#"{"schemaVersion":"2.0.0"}"#)
        .expect_err("future ACRE version should fail")
        .to_string();
    assert!(message.contains("unknown variant `2.0.0`"));
    assert!(!message.contains("missing field"));
}
#[test]
fn test_acre_schema_has_human_title_and_feature_independent_fields() {
    let schema = serde_json::to_value(schemars::schema_for!(ApplicationConfiguration)).unwrap();
    assert_eq!(schema["title"], "Application Configuration for Research Enablement");
    let properties = schema["properties"].as_object().expect("schema properties");
    ["powerAutomate", "schemaVersion", "scriptManifest", "workflow"]
        .into_iter()
        .for_each(|name| assert!(properties.contains_key(name), "missing {name}"));
}
#[test]
fn test_bucket_legacy_aliases_normalize_and_serialize_canonically() {
    let repository = r#"{"provider":"git","location":"file:./science"}"#;
    let legacy = ApplicationConfiguration::parse(format!(r#"{{"buckets":[{{"repository":{repository}}}]}}"#).as_str()).unwrap();
    let canonical = ApplicationConfiguration::parse(format!(r#"{{"buckets":[{{"codeRepository":{repository}}}]}}"#).as_str()).unwrap();
    let legacy_bucket = &legacy.buckets.unwrap()[0];
    let canonical_bucket = &canonical.buckets.unwrap()[0];
    assert_eq!(legacy_bucket.source, canonical_bucket.source);
    assert_eq!(
        serde_json::to_value(legacy_bucket).unwrap(),
        serde_json::json!({
            "codeRepository": {
                "provider": "git",
                "location": "file:./science"
            }
        })
    );
}
#[test]
fn test_bucket_shorthand_classification_is_exact_and_fallible() {
    let local = Bucket::try_from("../science").unwrap();
    let github = Bucket::try_from("https://www.github.com/acme/science").unwrap();
    let lookalike = Bucket::try_from("https://github.com.example/acme/science").unwrap();
    assert!(Repository::parse("https://github.com/acme/science").unwrap().is_git_hub());
    assert!(matches!(local.source, BucketSource::Repository(Repository::Git { .. })));
    assert!(matches!(github.source, BucketSource::Repository(Repository::GitHub { .. })));
    assert!(matches!(lookalike.source, BucketSource::Repository(Repository::GitLab { id: None, .. })));
    [
        "",
        " https://github.com/acme/science",
        "http://github.com/acme/science",
        "ssh://git@github.com/acme/science",
        "https://user:secret@github.com/acme/science",
        "https://",
        "oci://registry.example/acme/science:latest",
    ]
    .into_iter()
    .for_each(|value| assert!(Bucket::try_from(value).is_err(), "{value} should be rejected"));
}
#[test]
fn test_bucket_shorthand_normalizes_across_json_jsonc_and_yaml() {
    let json = ApplicationConfiguration::parse(r#"{"buckets":["./science",{"name":"remote","source":"https://github.com/acme/science"}]}"#).unwrap();
    let jsonc = ApplicationConfiguration::parse(
        r#"{
            // Concise bucket definitions use the same normalization path.
            "buckets": ["./science", {"name": "remote", "source": "https://github.com/acme/science"}],
        }"#,
    )
    .unwrap();
    let yaml = serde_norway::from_str::<ApplicationConfiguration>(
        "buckets:\n  - ./science\n  - name: remote\n    source: https://github.com/acme/science\n",
    )
    .unwrap();
    let expected = json.buckets.unwrap();
    [jsonc, yaml].into_iter().for_each(|configuration| {
        let buckets = configuration.buckets.unwrap();
        assert_eq!(buckets.len(), expected.len());
        assert_eq!(buckets[0].source, expected[0].source);
        assert_eq!(buckets[1].source, expected[1].source);
        assert_eq!(buckets[1].name, expected[1].name);
    });
    assert_eq!(
        serde_json::to_value(&expected[1]).unwrap(),
        serde_json::json!({
            "name": "remote",
            "codeRepository": {
                "provider": "github",
                "location": "https://github.com/acme/science"
            }
        })
    );
}
#[test]
fn test_bucket_source_aliases_conflict_instead_of_using_precedence() {
    let content = r#"{
        "buckets": [{
            "source": "./science",
            "repository": {"provider": "git", "location": "file:./other"}
        }]
    }"#;
    assert!(ApplicationConfiguration::parse(content).is_err());
}
#[test]
fn test_bucket_source_items_validate_paths_duplicates_and_order() {
    ["", "/absolute.json", "../outside.json", "C:\\outside.json"]
        .into_iter()
        .for_each(|path| assert!(BucketSourceItem::new("key".to_string(), path, None, None).is_err()));
    let ordered = source_items(&["second.json", "first.json"]);
    assert_eq!(
        BucketSourceItem::validate(ordered)
            .unwrap()
            .into_iter()
            .map(|item| item.path.into_path_buf())
            .collect::<Vec<_>>(),
        vec![PathBuf::from("second.json"), PathBuf::from("first.json")]
    );
    let duplicates = vec![
        BucketSourceItem::new("first".to_string(), "docs/./index.json", None, None).unwrap(),
        BucketSourceItem::new("second".to_string(), "docs/index.json", None, None).unwrap(),
    ];
    assert!(BucketSourceItem::validate(duplicates).unwrap_err().to_string().contains("duplicate path"));
}
#[test]
fn test_builtin_endpoints() {
    let config = ApplicationConfiguration::parse(ENDPOINTS).unwrap();
    insta::assert_snapshot!("builtin_endpoints", format!("{:#?}", config.endpoints));
}
#[tokio::test]
async fn test_clobber_preserves_destination_when_source_read_fails() {
    let output = temp_resolve_dir("clobber-source-failure-output");
    create_dir_all(&output).unwrap();
    write(output.join("index.json"), "old").unwrap();
    let write_lock = Mutex::new(());
    let destination = SafePath::new("index.json").unwrap();
    let source = output.join("missing.json");
    let result = destination
        .write_file(&output, true, &write_lock, || async move {
            read(&source).map_err(|error| FsError::Io { path: source, source: error })
        })
        .await;
    assert!(result.is_err());
    assert_eq!(read_to_string(output.join("index.json")).unwrap(), "old");
    let _ = remove_dir_all(output);
}
#[test]
fn test_config_with_endpoints() {
    let path = fixtures_dir().join("config").join("with_endpoints.json");
    let endpoints = ApplicationConfiguration::read_json(path).unwrap().endpoints.unwrap();
    assert_eq!(endpoints[0].resources.len(), 2);
}
#[tokio::test]
async fn test_copy_files_clobbers_selected_path_conflicts_only() {
    let source = temp_resolve_dir("clobber-enabled-source");
    let output = temp_resolve_dir("clobber-enabled-output");
    create_dir_all(source.join("nested")).unwrap();
    create_dir_all(output.join("directory.json")).unwrap();
    write(source.join("existing.json"), "new file").unwrap();
    write(source.join("directory.json"), "new directory replacement").unwrap();
    write(source.join("nested/index.json"), "new nested file").unwrap();
    write(source.join("nested/other.json"), "new sibling file").unwrap();
    write(output.join("existing.json"), "old file").unwrap();
    write(output.join("directory.json/old.json"), "old directory content").unwrap();
    write(output.join("nested"), "old parent file").unwrap();
    write(output.join("unrelated.json"), "keep").unwrap();
    let bucket = Bucket::init()
        .source(
            Repository::Git {
                location: Box::new(Location::Simple(format!("file:{}", source.display()))),
            }
            .into(),
        )
        .build();
    let options = BucketOptions::init()
        .common(CommonRuntime {
            quiet: true,
            ..CommonRuntime::default()
        })
        .extension(BucketExtension {
            clobber: true,
            ..BucketExtension::default()
        })
        .output(output.clone())
        .build();
    assert!(bucket.copy_files(&options).await.is_ok());
    assert_eq!(read_to_string(output.join("existing.json")).unwrap(), "new file");
    assert_eq!(read_to_string(output.join("directory.json")).unwrap(), "new directory replacement");
    assert_eq!(read_to_string(output.join("nested/index.json")).unwrap(), "new nested file");
    assert_eq!(read_to_string(output.join("nested/other.json")).unwrap(), "new sibling file");
    assert_eq!(read_to_string(output.join("unrelated.json")).unwrap(), "keep");
    let _ = remove_dir_all(source);
    let _ = remove_dir_all(output);
}
#[tokio::test]
async fn test_copy_files_flattens_destinations_and_manifest() {
    let source = temp_resolve_dir("flatten-source");
    let output = temp_resolve_dir("flatten-output");
    create_dir_all(source.join("docs/quest")).unwrap();
    write(source.join("docs/quest/index.json"), "{}").unwrap();
    write(source.join("docs/quest/image.png"), "image").unwrap();
    let bucket = Bucket::init()
        .source(
            Repository::Git {
                location: Box::new(Location::Simple(format!("file:{}", source.display()))),
            }
            .into(),
        )
        .build();
    let options = BucketOptions::init()
        .common(CommonRuntime {
            quiet: true,
            ..CommonRuntime::default()
        })
        .extension(BucketExtension {
            flatten: true,
            ..BucketExtension::default()
        })
        .output(output.clone())
        .build();
    let manifest = bucket.copy_files(&options).await.unwrap();
    assert_eq!(manifest.files.len(), 2);
    assert!(manifest.files.contains(&PathBuf::from("index.json")));
    assert!(manifest.files.contains(&PathBuf::from("image.png")));
    assert!(output.join("index.json").is_file());
    assert!(output.join("image.png").is_file());
    assert!(!output.join("docs").exists());
    let _ = remove_dir_all(source);
    let _ = remove_dir_all(output);
}
#[tokio::test]
async fn test_copy_files_preserves_existing_destination_without_clobber() {
    let source = temp_resolve_dir("clobber-disabled-source");
    let output = temp_resolve_dir("clobber-disabled-output");
    create_dir_all(&source).unwrap();
    create_dir_all(&output).unwrap();
    write(source.join("index.json"), "new").unwrap();
    write(output.join("index.json"), "old").unwrap();
    let bucket = Bucket::init()
        .source(
            Repository::Git {
                location: Box::new(Location::Simple(format!("file:{}", source.display()))),
            }
            .into(),
        )
        .build();
    let options = BucketOptions::init()
        .common(CommonRuntime {
            quiet: true,
            ..CommonRuntime::default()
        })
        .output(output.clone())
        .build();
    assert!(bucket.copy_files(&options).await.is_err());
    assert_eq!(read_to_string(output.join("index.json")).unwrap(), "old");
    let _ = remove_dir_all(source);
    let _ = remove_dir_all(output);
}
#[test]
fn test_count_image_files_counts_supported_extensions() {
    let paths = vec![
        "content/plot.png".to_string(),
        "content/photo.jpg".to_string(),
        "content/photo.jpeg".to_string(),
        "content/index.json".to_string(),
    ];
    assert_eq!(count_image_files(&paths), 2);
}
#[test]
fn test_count_json_files_counts_case_insensitive_json_paths() {
    let paths = vec![
        "content/index.json".to_string(),
        "content/README.md".to_string(),
        "content/data.JSON".to_string(),
    ];
    assert_eq!(count_json_files(&paths), 2);
}
#[test]
fn test_default_config_filenames_require_supported_extensions() {
    assert_eq!(DEFAULT_CONFIG_FILENAMES, [".acorn.json", ".acorn.jsonc", ".acorn.yaml", ".acorn.yml"]);
}
#[tokio::test]
async fn test_download_file_skips_existing_file_without_clobber() {
    let output = temp_resolve_dir("download-file-transfer");
    create_dir_all(&output).expect("temporary output should exist");
    write(output.join("data.txt"), "existing").expect("existing file should be writable");
    let repository = Repository::GitHub {
        location: Box::new(Location::Simple("https://github.com/acme/quest".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: &output,
        source: Path::new("data.txt"),
        destination: Path::new("data.txt"),
        branch: None,
        size: None,
    };
    RepositoryBucketSource::download_file(&options)
        .await
        .expect("existing file should be skipped without network transfer");
    assert_eq!(
        read_to_string(output.join("data.txt")).expect("skipped destination should be readable"),
        "existing"
    );
    let _ = remove_dir_all(output);
}
#[test]
fn test_download_item_github_uses_raw_url() {
    let repository = Repository::GitHub {
        location: Box::new(Location::Simple("https://github.com/acme/quest".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: Path::new(""),
        source: Path::new("index.json"),
        destination: Path::new("index.json"),
        branch: None,
        size: None,
    };
    let item = RepositoryBucketSource::download_item(&options).expect("GitHub raw URL should build");
    assert_eq!(item.url, "https://raw.githubusercontent.com/acme/quest/refs/heads/main/index.json");
    assert!(item.headers.is_none());
}
#[test]
fn test_download_item_gitlab_defaults_to_head_and_percent_encodes_path() {
    let repository = Repository::GitLab {
        id: None,
        location: Box::new(Location::Simple("https://code.ornl.gov/group/project".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: Path::new(""),
        source: Path::new("content/data set/index.json"),
        destination: Path::new("index.json"),
        branch: None,
        size: None,
    };
    let item = RepositoryBucketSource::download_item(&options).expect("GitLab raw URL should build");
    assert_eq!(
        item.url,
        "https://code.ornl.gov/api/v4/projects/group%2Fproject/repository/files/content%2Fdata%20set%2Findex.json/raw?ref=HEAD"
    );
}
#[tokio::test]
async fn test_download_item_gitlab_raw_url_uses_resolved_branch_and_encodes_path() {
    let repository = Repository::GitLab {
        id: None,
        location: Box::new(Location::Simple("https://code.ornl.gov/group/project".to_string())),
    };
    let options = DownloadOptions {
        repository: &repository,
        output: Path::new(""),
        source: Path::new("content/quest/index.json"),
        destination: Path::new("index.json"),
        branch: Some("develop"),
        size: None,
    };
    let item = RepositoryBucketSource::download_item(&options).expect("GitLab raw URL should build");
    assert_eq!(
        item.url,
        "https://code.ornl.gov/api/v4/projects/group%2Fproject/repository/files/content%2Fquest%2Findex.json/raw?ref=develop"
    );
    assert_eq!(item.path, "index.json");
    assert!(item.size.is_none());
    match Credential::from_env(&GITLAB_CREDENTIAL_VARIABLE_PAIRS) {
        | Some(credential) => assert_eq!(
            item.headers
                .as_ref()
                .and_then(|headers| headers.get(credential.header))
                .and_then(|value| value.to_str().ok()),
            Some(ExposeSecret::expose_secret(&credential.token))
        ),
        | None => assert!(item.headers.is_none()),
    }
}
#[test]
fn test_filter_set_filter() {
    let selectors = vec![crate::io::Source::from("alpha.gguf"), crate::io::Source::from("beta.safetensors")];
    let filtered = FilterSet::filter(
        selectors,
        &["gguf$".to_string()],
        &["alpha".to_string()],
        crate::io::Source::identifier,
        |_| true,
    )
    .unwrap();
    assert!(filtered.is_empty());
}
#[test]
fn test_has_image_extension_matches_png_and_jpg() {
    assert!(has_image_extension(&"image.png".to_string()));
    assert!(has_image_extension(&"photo.JPG".to_string()));
    assert!(!has_image_extension(&"graphic.jpeg".to_string()));
}
#[test]
fn test_is_filtered_path() {
    let patterns = [r"\.json$".to_string(), r"img/".to_string()];
    let filter = FilterSet::try_from((&[][..], patterns.as_slice())).unwrap().filter;
    assert!(is_filtered_path("/tmp/data.json", &filter));
    assert!(is_filtered_path("/tmp/img/photo.jpg", &filter));
    assert!(!is_filtered_path("/tmp/README.md", &filter));
    let invalid = ["[".to_string()];
    assert!(FilterSet::try_from((&[][..], invalid.as_slice())).is_err());
    assert!(is_filtered_path("/tmp/README.md", &Vec::<Regex>::new()));
}
#[test]
fn test_is_ignored_path() {
    let patterns = [r"\.jpeg$".to_string(), r"notes\.txt$".to_string()];
    let ignore = FilterSet::try_from((patterns.as_slice(), &[][..])).unwrap().ignore;
    assert!(is_ignored_path("/tmp/photo.jpeg", &ignore));
    assert!(is_ignored_path("/tmp/notes.txt", &ignore));
    assert!(!is_ignored_path("/tmp/index.json", &ignore));
    let invalid = ["[".to_string()];
    assert!(FilterSet::try_from((invalid.as_slice(), &[][..])).is_err());
    assert!(is_ignored_path("/tmp/README.md", &Vec::<Regex>::new()));
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
fn test_jsonc_parse_with_block_comments() {
    let content = r#"{
        /* Block comment */
        "buckets": []
    }"#;
    assert!(ApplicationConfiguration::parse(content).unwrap().buckets.unwrap().is_empty());
}
#[test]
fn test_jsonc_parse_with_line_comments() {
    let content = r#"{
        // This is a comment
        "buckets": [{"name": "test", "repository": {"provider": "git", "location": "file:./"}}]
    }"#;
    let buckets = ApplicationConfiguration::parse(content).unwrap().buckets.unwrap();
    assert_eq!(buckets.len(), 1);
}
#[test]
fn test_jsonc_parse_with_trailing_commas() {
    let content = r#"{
        "buckets": [],
    }"#;
    assert!(ApplicationConfiguration::parse(content).unwrap().buckets.unwrap().is_empty());
}
#[test]
fn test_load_rejects_missing_explicit_path() {
    let missing = temp_resolve_dir("load-missing").join("missing.json");
    let result = ApplicationConfiguration::load(&Some(missing.clone()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        format!("Configuration file does not exist — {}", missing.display())
    );
}
#[test]
fn test_model_update_preserves_jsonc_comments_and_dry_run() {
    let directory = temp_resolve_dir("sync-acorn-config");
    create_dir_all(&directory).unwrap();
    let path = directory.join("config.jsonc");
    let before = "{\n  // Keep this comment\n  \"models\": [\"acme/existing\"]\n}\n";
    write(&path, before).unwrap();
    let configuration = ApplicationConfiguration::read(path.clone()).unwrap();
    let entries = vec![ModelEntry::Selector("acme/added".to_string())];
    let updated = configuration.with_models(&entries).unwrap();
    updated.write_or_preview(&path, true, true).unwrap();
    assert_eq!(read_file(&path).unwrap(), before);
    updated.write_or_preview(&path, false, true).unwrap();
    let content = read_file(&path).unwrap();
    assert_eq!(
        content,
        "{\n  // Keep this comment\n  \"models\": [\n    \"acme/existing\",\n    \"acme/added\"\n  ]\n}\n"
    );
    let _ = remove_dir_all(directory);
}
#[test]
fn test_oci_registry_profile_and_model_source_parse() {
    let configuration = ApplicationConfiguration::parse(
        r#"{
            "registries": {
                "internal": {
                    "kind": "harbor",
                    "endpoint": "https://harbor.example.org",
                    "credentialEnv": "ACORN_HARBOR_TOKEN"
                }
            },
            "models": [{
                "name": "nemotron-oci",
                "source": {
                    "provider": "oci",
                    "location": "oci://harbor.example.org/ai/models/nemotron:v1",
                    "registry": "internal"
                }
            }]
        }"#,
    )
    .unwrap();
    let profiles = configuration.registry_lookup().unwrap();
    assert_eq!(profiles.get("internal").map(|profile| &profile.kind), Some(&RegistryKind::Harbor));
    assert!(matches!(
        configuration.models.as_deref(),
        Some([ModelEntry::Entry(ModelEntryOptions {
            source: Repository::OCI { registry: Some(name), .. },
            ..
        })]) if name == "internal"
    ));
}
#[test]
fn test_oci_registry_profile_rejects_plaintext_secret_and_insecure_endpoint() {
    assert!(ApplicationConfiguration::parse(r#"{"registries":{"bad":{"endpoint":"https://registry.example.org","password":"secret"}}}"#).is_err());
    let configuration =
        ApplicationConfiguration::parse(r#"{"registries":{"bad":{"endpoint":"http://registry.example.org","credentialEnv":"bad-name"}}}"#).unwrap();
    assert!(configuration.registry_lookup().is_err());
}
#[test]
fn test_operations_complete_message_includes_bucket_name_and_guidance() {
    let message = operations_complete_message(Some("acorn".to_string()), 2, 1);
    assert!(message.contains("Obtained"));
    assert!(message.contains("ACORN"));
    assert!(message.contains(" bucket"));
    assert!(message.contains("data file"));
    assert!(message.contains("image"));
    assert!(message.contains("Do you need to add some images?"));
}
#[test]
fn test_operations_complete_message_uses_url_placeholder_without_name() {
    let message = operations_complete_message(None, 0, 0);
    assert!(message.contains("Obtained"));
    assert!(message.contains("<URL>"));
}
#[test]
fn test_parse_supports_yaml_flow_mapping_when_json_detection_fails() {
    assert!(ApplicationConfiguration::parse("{endpoints: []}").is_ok());
}
#[cfg(unix)]
#[test]
fn test_prepare_destination_replaces_parent_symlink_without_following_it() {
    let output = temp_resolve_dir("clobber-symlink-output");
    let external = temp_resolve_dir("clobber-symlink-external");
    create_dir_all(&output).unwrap();
    create_dir_all(&external).unwrap();
    write(external.join("index.json"), "outside").unwrap();
    symlink(&external, output.join("linked")).unwrap();
    let destination = SafePath::new("linked/index.json").unwrap();
    let target = destination.prepare(&output).unwrap();
    assert_eq!(target, output.join("linked/index.json"));
    assert!(output.join("linked").is_dir());
    assert!(!output.join("linked").symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(read_to_string(external.join("index.json")).unwrap(), "outside");
    let _ = remove_dir_all(output);
    let _ = remove_dir_all(external);
}
#[test]
fn test_resolution_failure_reason_reports_fallback_lookup_status() {
    assert_eq!(
        ModelEntry::resolution_failure_reason(&"missing", true, &[]),
        "(missing; no fallback repositories found in the local model database)"
    );
    assert_eq!(ModelEntry::resolution_failure_reason(&"missing", false, &[]), "(missing)");
    assert_eq!(
        ModelEntry::resolution_failure_reason(&"missing", true, &["fallback/model".to_string()]),
        "(missing)"
    );
}
#[test]
fn test_resolve_falls_back_to_default_when_provided_path_missing() {
    let directory = temp_resolve_dir("resolve-fallback");
    create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let default = directory.join(".acorn.yml");
    write(&default, "{}\n").unwrap();
    let resolved = ApplicationConfiguration::resolve_in(&Some(directory.join("missing.json")), &directory);
    assert_eq!(resolved, Some(default));
    let _ = remove_dir_all(directory);
}
#[test]
fn test_resolve_returns_explicit_existing_path() {
    let directory = temp_resolve_dir("resolve-explicit");
    create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let provided = directory.join("config.yaml");
    write(&provided, "{}\n").unwrap();
    write(directory.join(".acorn.json"), "{}\n").unwrap();
    assert_eq!(ApplicationConfiguration::resolve(&Some(provided.clone())), Some(provided));
    let _ = remove_dir_all(directory);
}
#[test]
fn test_runner_target_accepts_string_or_object() {
    let alias = serde_json::from_str::<RunnerTarget>(r#""oklahoma""#).unwrap();
    let object = serde_json::from_str::<RunnerTarget>(r#"{"host": "oklahoma"}"#).unwrap();
    assert_eq!(alias, object);
    assert_eq!(alias.host.as_str(), "ssh://oklahoma");
    assert_eq!(serde_json::to_value(alias).unwrap(), serde_json::json!({"host": "ssh://oklahoma"}));
}
#[test]
fn test_runner_target_rejects_invalid_or_unknown_values() {
    let invalid_host = r#"{
        "runners": [{
            "repository": {"provider": "gitlab", "id": 42, "location": "https://gitlab.com/example/project"},
            "type": "project",
            "target": {"host": "http://builder"}
        }]
    }"#;
    let unknown_target = r#"{
        "runners": [{
            "repository": {"provider": "gitlab", "id": 42, "location": "https://gitlab.com/example/project"},
            "type": "project",
            "target": {"host": "builder", "context": "prod"}
        }]
    }"#;
    assert!(serde_json::from_str::<ApplicationConfiguration>(invalid_host)
        .unwrap_err()
        .to_string()
        .contains("invalid remote"));
    assert!(serde_json::from_str::<ApplicationConfiguration>(unknown_target)
        .unwrap_err()
        .to_string()
        .contains("unknown field"));
}
#[test]
fn test_runners() {
    let path = fixtures_dir().join("config").join("with_runners.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    insta::assert_snapshot!("runners", format!("{:#?}", config.runners));
}
#[test]
fn test_transfer_item_collect_preserves_or_flattens_paths() {
    let preserved = TransferItem::collect(source_items(&["docs/quest/index.json", "docs/quest/image.png"]), false).unwrap();
    let flattened = TransferItem::collect(source_items(&["docs/quest/index.json", "docs/quest/image.png"]), true).unwrap();
    assert_eq!(
        preserved.iter().map(|item| item.destination.clone().into_path_buf()).collect::<Vec<_>>(),
        vec![PathBuf::from("docs/quest/index.json"), PathBuf::from("docs/quest/image.png")]
    );
    assert_eq!(
        flattened.iter().map(|item| item.destination.clone().into_path_buf()).collect::<Vec<_>>(),
        vec![PathBuf::from("index.json"), PathBuf::from("image.png")]
    );
}
#[test]
fn test_transfer_item_collect_rejects_flattened_filename_collisions() {
    let message = TransferItem::collect(source_items(&["one/index.json", "two/index.json"]), true)
        .unwrap_err()
        .to_string()
        .replace('\\', "/");
    assert!(message.contains("index.json"));
    assert!(message.contains("one/index.json"));
    assert!(message.contains("two/index.json"));
}
#[test]
fn test_transfer_item_collect_rejects_unsafe_destination() {
    let result = BucketSourceItem::new("unsafe".to_string(), "../outside.json", None, None);
    assert!(result.unwrap_err().to_string().contains("unsafe"));
}
#[test]
fn test_with_models_keeps_unique_identifiers() {
    let configuration = ApplicationConfiguration::parse(r#"{"models":["acme/existing","acme/existing"]}"#).unwrap();
    let entries = vec![
        ModelEntry::Selector("acme/existing".to_string()),
        ModelEntry::Selector("acme/added".to_string()),
        ModelEntry::Selector("acme/added".to_string()),
    ];
    let identifiers = configuration
        .with_models(&entries)
        .unwrap()
        .models
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| match entry {
            | ModelEntry::Selector(identifier) => Some(identifier),
            | ModelEntry::Entry(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(identifiers, vec!["acme/existing", "acme/added"]);
}
