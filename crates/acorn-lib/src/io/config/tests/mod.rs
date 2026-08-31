#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use crate::io::config::transfer::{
    count_image_files, count_json_files, has_image_extension, operations_complete_message, DownloadOptions, TransferItem,
};
use crate::io::{first_env_var, read_file, InputOutput};
use crate::prelude::{create_dir_all, read_to_string, remove_dir_all, write, Mutex, Path, PathBuf};
use crate::util::constants::app::DEFAULT_CONFIG_FILENAMES;
use crate::util::constants::env::GITLAB_TOKEN_VARIABLE_NAMES;
use crate::{Location, Repository};
use color_eyre::eyre::eyre;
use color_eyre::Report;
use fancy_regex::Regex;
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
        code_repository: Repository::GitLab {
            id: Some(1234_u64),
            location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string())),
        },
    };
    assert_eq!(bucket.domain().unwrap(), "code.ornl.gov".to_string());
    let bucket = Bucket {
        name: Some("nssd".to_string()),
        description: Some("Bucket for NSSD".to_string()),
        code_repository: Repository::GitHub {
            location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/buckets/nssd".to_string())),
        },
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
    let result = Bucket::write_file(&output, Path::new("index.json"), true, &write_lock, || async {
        Err::<Vec<u8>, Report>(eyre!("source failure"))
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
        .code_repository(Repository::Git {
            location: Box::new(Location::Simple(format!("file:{}", source.display()))),
        })
        .build();
    let options = BucketOptions::init().output(output.clone()).quiet(true).clobber(true).build();
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
        .code_repository(Repository::Git {
            location: Box::new(Location::Simple(format!("file:{}", source.display()))),
        })
        .build();
    let options = BucketOptions::init().output(output.clone()).quiet(true).flatten(true).build();
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
        .code_repository(Repository::Git {
            location: Box::new(Location::Simple(format!("file:{}", source.display()))),
        })
        .build();
    let options = BucketOptions::init().output(output.clone()).quiet(true).build();
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
        clobber: false,
    };
    Bucket::download_file(&options)
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
        clobber: false,
    };
    let item = Bucket::download_item(&options).expect("GitHub raw URL should build");
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
        clobber: false,
    };
    let item = Bucket::download_item(&options).expect("GitLab raw URL should build");
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
        clobber: false,
    };
    let item = Bucket::download_item(&options).expect("GitLab raw URL should build");
    assert_eq!(
        item.url,
        "https://code.ornl.gov/api/v4/projects/group%2Fproject/repository/files/content%2Fquest%2Findex.json/raw?ref=develop"
    );
    assert_eq!(item.path, "index.json");
    assert!(item.size.is_none());
    let expected_headers = first_env_var(&GITLAB_TOKEN_VARIABLE_NAMES)
        .filter(|value| !value.is_empty())
        .map(|_| 1)
        .unwrap_or(0);
    assert_eq!(item.headers.as_ref().map(|headers| headers.len()).unwrap_or(0), expected_headers);
}
#[test]
fn test_extensionless_config_is_last_and_read_as_jsonc() {
    assert_eq!(DEFAULT_CONFIG_FILENAMES.last(), Some(&".acorn"));
    let directory = temp_resolve_dir("extensionless-jsonc");
    create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let extensionless = directory.join(".acorn");
    write(&extensionless, "{\n  // Comment\n}\n").unwrap();
    let config = ApplicationConfiguration::read(extensionless.clone()).unwrap();
    assert!(config.write(extensionless).is_ok());
    let _ = remove_dir_all(directory);
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
    let filter = FilterSet::compile(&[], &[r"\.json$".to_string(), r"img/".to_string()]).unwrap().filter;
    assert!(is_filtered_path("/tmp/data.json", &filter));
    assert!(is_filtered_path("/tmp/img/photo.jpg", &filter));
    assert!(!is_filtered_path("/tmp/README.md", &filter));
    assert!(FilterSet::compile(&[], &["[".to_string()]).is_err());
    assert!(is_filtered_path("/tmp/README.md", &Vec::<Regex>::new()));
}
#[test]
fn test_is_ignored_path() {
    let ignore = FilterSet::compile(&[r"\.jpeg$".to_string(), r"notes\.txt$".to_string()], &[])
        .unwrap()
        .ignore;
    assert!(is_ignored_path("/tmp/photo.jpeg", &ignore));
    assert!(is_ignored_path("/tmp/notes.txt", &ignore));
    assert!(!is_ignored_path("/tmp/index.json", &ignore));
    assert!(FilterSet::compile(&["[".to_string()], &[]).is_err());
    assert!(is_ignored_path("/tmp/README.md", &Vec::<Regex>::new()));
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
    crate::prelude::symlink(&external, output.join("linked")).unwrap();
    let target = Bucket::prepare_destination(&output, Path::new("linked/index.json")).unwrap();
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
fn test_runners() {
    let path = fixtures_dir().join("config").join("with_runners.json");
    let config = ApplicationConfiguration::read_json(path).unwrap();
    insta::assert_snapshot!("runners", format!("{:#?}", config.runners));
}
#[test]
fn test_transfer_item_collect_preserves_or_flattens_paths() {
    let paths = vec!["docs/quest/index.json".to_string(), "docs/quest/image.png".to_string()];
    let preserved = TransferItem::collect(paths.clone(), false).unwrap();
    let flattened = TransferItem::collect(paths, true).unwrap();
    assert_eq!(
        preserved.iter().map(|item| item.destination.clone()).collect::<Vec<_>>(),
        vec![PathBuf::from("docs/quest/index.json"), PathBuf::from("docs/quest/image.png")]
    );
    assert_eq!(
        flattened.iter().map(|item| item.destination.clone()).collect::<Vec<_>>(),
        vec![PathBuf::from("index.json"), PathBuf::from("image.png")]
    );
}
#[test]
fn test_transfer_item_collect_rejects_flattened_filename_collisions() {
    let message = TransferItem::collect(vec!["one/index.json".to_string(), "two/index.json".to_string()], true)
        .unwrap_err()
        .to_string();
    assert!(message.contains("index.json"));
    assert!(message.contains("one/index.json"));
    assert!(message.contains("two/index.json"));
}
#[test]
fn test_transfer_item_collect_rejects_unsafe_destination() {
    let result = TransferItem::collect(vec!["../outside.json".to_string()], false);
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
