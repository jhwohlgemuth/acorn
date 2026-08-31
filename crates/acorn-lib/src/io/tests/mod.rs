#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::{create_progress_bar_with_renderer, finish_progress_bar, ProgressType};
use crate::enrichment::Provider;
use crate::io::api::{TreeEntry, TreeEntryType};
use crate::io::bagit::{Bag, BagInfo, Save};
use crate::io::document::{SourceDocument, SourceDocuments};
use crate::io::enrichment::Enrich;
use crate::io::http::policy::{http_timeout_layer, retry_delay};
use crate::io::http::{should_retry, HttpMethod};
use crate::io::oci::{
    Descriptor, Manifest, OciExtraction, OciFile, OciFiles, OciLayer, OciReference, OciReferenceValue, OciTransport, OciTransportOptions,
    OrasTransport,
};
use crate::io::{
    archive, create_rsa_keypair, current_date, env_var_is_truthy, extract, file_checksum, files_all, files_from_git_branch, files_from_git_commit,
    files_from_gitlab_merge_request, filter_git_command_result, filter_ignored, first_env_var, image_paths, jsonc_parse_value, read_file,
    read_large_file, remove_fields, slice_path, unique_file_extensions, uri_to_path, walk, write_file_bytes, write_rsa_keypair, Executor,
    Fingerprint, PathConversion, PathExt, Remote, Source, SourceAction,
};
use crate::prelude::{OsString, MAIN_SEPARATOR};
use crate::schema::agent::Quantization;
use crate::util::constants::app::{
    ACORN_MODEL_LAYER, KITOPS_MODELKIT_CONFIG, KITOPS_MODELKIT_MODEL_GZIP, MINIMUM_ORAS_VERSION, MODELPACK_CONFIG, MODELPACK_MANIFEST,
    MODELPACK_WEIGHT_RAW, MODELPACK_WEIGHT_TAR, MODELPACK_WEIGHT_TAR_GZIP, MODELPACK_WEIGHT_TAR_ZSTD, OCI_IMAGE_CONFIG, OCI_IMAGE_INDEX,
    OCI_IMAGE_MANIFEST, OCI_LAYER, OCI_LAYER_GZIP,
};
use crate::util::constants::env::NO_LOCAL_DATABASE;
use crate::util::{MimeType, SemanticVersion, StringConversion};
use crate::validation::Validate;
use crate::{args, Location, Repository, Scheme};
use axum::http::header::RETRY_AFTER;
use axum::http::{HeaderMap, HeaderValue};
use color_eyre::Report;
use fluent_uri::Uri as UriParse;
use indicatif::{InMemoryTerm, MultiProgress, ProgressDrawTarget, ProgressStyle, TermLike};
use jiff::{SignedDuration, Timestamp};
use rsa::traits::PublicKeyParts;
use std::env::temp_dir;
use std::fs::{create_dir_all, metadata, read, read_to_string, remove_dir_all, remove_file, set_permissions, write, File, Permissions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use strum::IntoEnumIterator;
use zip::ZipArchive;

const OCI_TEST_DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
fn minimal_gguf() -> Vec<u8> {
    let mut bytes = b"GGUF".to_vec();
    bytes.extend(3u32.to_le_bytes());
    bytes.extend(1u64.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    bytes.extend(6u64.to_le_bytes());
    bytes.extend(b"weight");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(1u64.to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    let padding = (32 - bytes.len() % 32) % 32;
    bytes.extend(vec![0; padding + 4]);
    bytes
}
const TEST_BAGGING_DATE: &str = "2026-02-22";

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

#[test]
fn test_enrichment_fields_and_provider_order_are_deterministic() {
    assert_eq!(
        crate::io::enrichment::ENRICHMENT_FIELDS,
        ["title", "subtitle", "contact.identifier", "meta.doi", "meta.outputs", "meta.ror"]
    );
    assert_eq!(
        Provider::iter().collect::<Vec<_>>(),
        &[Provider::Osti, Provider::OpenAlex, Provider::CiteAs, Provider::Orcid, Provider::Ror]
    );
    assert_eq!("OPENALEX".parse::<Provider>(), Ok(Provider::OpenAlex));
    assert!("unknown".parse::<Provider>().is_err());
    let before = serde_json::json!({"title": "Existing", "meta": {}});
    let after = serde_json::json!({"title": "Existing", "meta": {"ror": ["https://ror.org/01qz5mb56"]}});
    let fields = crate::io::enrichment::ENRICHMENT_FIELDS
        .into_iter()
        .filter(|path| {
            path.split('.').try_fold(&before, |value, component| value.get(component))
                != path.split('.').try_fold(&after, |value, component| value.get(component))
        })
        .collect::<Vec<_>>();
    assert_eq!(fields, ["meta.ror"]);
}
#[tokio::test]
async fn test_source_document_enrichment_preserves_unsupported_content() {
    let document = SourceDocument::init().content("plain text").format("text").source("<text:1>").build();
    let (enriched, content, failures) = document.enrich(()).await;
    assert_eq!(enriched.content, "plain text");
    assert!(content.is_none());
    assert!(failures.is_empty());
}
#[tokio::test]
async fn test_source_documents_enrichment_can_be_disabled() {
    let document = SourceDocument::init().content("plain text").format("text").source("<text:1>").build();
    let (documents, content, failures) = SourceDocuments::from(vec![document]).enrich(false).await;
    assert_eq!(documents.0.len(), 1);
    assert!(content.is_empty());
    assert!(failures.is_empty());
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
        location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/vale-package".to_string())),
    };
    let release = repository.latest_release().await;
    assert!(release.is_some());
}
#[ignore]
#[tokio::test]
async fn test_repository_github() {
    let repository = Repository::GitHub {
        location: Box::new(Location::Simple("https://github.com/jhwohlgemuth/voxelcss".to_string())),
    };
    let release = repository.latest_release().await;
    assert!(release.is_some());
}
#[test]
fn test_archive() {
    let path = fixtures_dir().join("data");
    let output_file = archive(path.clone(), None, MimeType::Zip).unwrap();
    assert_eq!(output_file.file_name().unwrap().to_str().unwrap(), "data.zip");
    let file = File::open(output_file).unwrap();
    let mut zip = ZipArchive::new(file).unwrap();
    let names = (0..zip.len())
        .filter_map(|index| zip.by_index(index).ok().map(|entry| entry.name().to_string()))
        .collect::<Vec<_>>();
    assert!(names.iter().any(|name| name == "highlight/reference.pptx"));
}
#[test]
fn test_archive_round_trips_supported_formats() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    create_dir_all(source.join("nested")).unwrap();
    write(source.join("nested/data.txt"), "archive content").unwrap();
    for (index, format) in [MimeType::SevenZip, MimeType::Zip, MimeType::Tar, MimeType::Gzip].into_iter().enumerate() {
        let archive_path = archive(source.clone(), None, format.clone()).unwrap();
        let destination = temporary.path().join(format!("extract-{index}"));
        extract(archive_path, Some(destination.clone()), None).unwrap();
        assert_eq!(read_to_string(destination.join("nested/data.txt")).unwrap(), "archive content");
    }
}
#[test]
fn test_archive_rejects_destination_inside_source() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("source");
    create_dir_all(&source).unwrap();
    write(source.join("data.txt"), "archive content").unwrap();
    assert!(archive(source.clone(), Some(source.join("inside.zip")), MimeType::Zip).is_err());
}
#[test]
fn test_tar_extraction_accepts_current_directory_prefix() {
    let temporary = tempfile::tempdir().unwrap();
    let archive_path = temporary.path().join("prefixed.tar");
    let mut builder = tar::Builder::new(File::create(&archive_path).unwrap());
    let content = b"archive content";
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append_data(&mut header, "./nested/data.txt", content.as_slice()).unwrap();
    builder.finish().unwrap();
    let destination = temporary.path().join("output");
    extract(archive_path, Some(destination.clone()), Some(MimeType::Tar)).unwrap();
    assert_eq!(read_to_string(destination.join("nested/data.txt")).unwrap(), "archive content");
}
#[cfg(unix)]
#[test]
fn test_zip_extraction_rejects_preexisting_symlink_component() {
    use std::os::unix::fs::symlink;
    let temporary = tempfile::tempdir().unwrap();
    let archive_path = temporary.path().join("unsafe.zip");
    let mut writer = zip::ZipWriter::new(File::create(&archive_path).unwrap());
    writer.start_file("child/escape.txt", zip::write::SimpleFileOptions::default()).unwrap();
    std::io::Write::write_all(&mut writer, b"escape").unwrap();
    writer.finish().unwrap();
    let destination = temporary.path().join("output");
    let outside = temporary.path().join("outside");
    create_dir_all(&destination).unwrap();
    create_dir_all(&outside).unwrap();
    symlink(&outside, destination.join("child")).unwrap();
    assert!(extract(archive_path, Some(destination), Some(MimeType::Zip)).is_err());
    assert!(!outside.join("escape.txt").exists());
}
#[test]
fn test_zip_extraction_rejects_traversal() {
    let temporary = tempfile::tempdir().unwrap();
    let archive_path = temporary.path().join("unsafe.zip");
    let file = File::create(&archive_path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    writer.start_file("../escape.txt", zip::write::SimpleFileOptions::default()).unwrap();
    std::io::Write::write_all(&mut writer, b"escape").unwrap();
    writer.finish().unwrap();
    assert!(extract(archive_path, Some(temporary.path().join("output")), Some(MimeType::Zip)).is_err());
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
    assert_eq!(bag.payload.len(), 30);
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
    let calculated = file_checksum("../../tests/fixtures/glob/a.txt", None).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.checksum_value.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated.checksum_value, expected);
    }
    let calculated = file_checksum("../../tests/fixtures/glob/a.txt", Some(&ring::digest::SHA512)).unwrap();
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
    let files = files_all(fixtures_dir().join("glob"), Some(vec![MimeType::Json]));
    assert_eq!(files.len(), 3);
    let files = files_all(fixtures_dir().join("glob"), None::<Vec<String>>);
    assert_eq!(files.len(), 8);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "a.json");
}
#[test]
fn test_files_all_matches_extensions_case_insensitively() {
    let temporary = tempfile::tempdir().unwrap();
    let pdf = temporary.path().join("REPORT.PDF");
    write(&pdf, b"%PDF-1.7\n").unwrap();
    let files = files_all(temporary.path().to_path_buf(), Some(vec![MimeType::Pdf]));
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name(), pdf.file_name());
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
    let response = "crates/acorn-lib/assets/constants/keywords.csv\ncrates/acorn-lib/assets/constants/technology.csv".to_string();
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
        let files = files_all(fixtures_dir().join("glob/a.txt"), None::<Vec<String>>);
        let filtered = filter_ignored(files, None).unwrap();
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
fn test_directory_roots_reduces_files_to_unique_parents() {
    let paths = vec![PathBuf::from("data/one.json"), PathBuf::from("data/two.json")];
    assert_eq!(crate::io::directory_roots(&paths), vec![PathBuf::from("data")]);
}
#[test]
fn test_path_ext_compares_normalized_paths() {
    assert!(Path::new("./data/input.json").same_as(Path::new("data/input.json")));
    assert!(!Path::new("data/input.json").same_as(Path::new("data/other.json")));
}
#[test]
fn test_path_ext_rejects_unsafe_relative_paths() {
    assert!(Path::new("models/qwen.gguf").is_safe());
    assert!(!Path::new("").is_safe());
    assert!(!Path::new("../models/qwen.gguf").is_safe());
    assert!(!Path::new("/models/qwen.gguf").is_safe());
}
#[test]
fn test_path_ext_detects_windows_paths() {
    assert!(Path::new("C:/Users/acorn/project.json").is_windows());
    assert!(Path::new("C:\\Users\\acorn\\project.json").is_windows());
    assert!(!Path::new("./data/input.json").is_windows());
    assert!(!Path::new("/home/acorn/project.json").is_windows());
}
#[test]
fn test_remove_fields_recurses_without_mutating_input() {
    let input = serde_json::json!({"keep": 1, "nested": {"remove": 2, "keep": 3}});
    let output = remove_fields(input.clone(), &["remove"]);
    assert_eq!(input["nested"]["remove"], 2);
    assert_eq!(output, serde_json::json!({"keep": 1, "nested": {"keep": 3}}));
}
#[test]
fn test_fingerprint_entries_are_order_independent() {
    let first = Fingerprint::from_entries([("b", b"two".as_slice()), ("a", b"one".as_slice())]);
    let second = Fingerprint::from_entries([("a", b"one".as_slice()), ("b", b"two".as_slice())]);
    assert_eq!(first, second);
}
#[test]
fn test_fingerprint_entry_boundaries_affect_value() {
    assert_ne!(
        Fingerprint::from_entries([("a", b"bc".as_slice())]),
        Fingerprint::from_entries([("ab", b"c".as_slice())])
    );
    assert_ne!(Fingerprint::from_bytes(b"content"), Fingerprint::from_bytes(b"contenu"));
}
#[test]
fn test_fingerprint_files_include_paths_and_content() {
    let path = temp_dir().join("acorn-fingerprint-test.txt");
    write(&path, "first").expect("write first fingerprint fixture");
    let first = Fingerprint::from_file(&path).expect("fingerprint first fixture");
    write(&path, "second").expect("write second fingerprint fixture");
    let second = Fingerprint::from_file(&path).expect("fingerprint second fixture");
    let _ = remove_file(path);
    assert_ne!(first, second);
    assert_eq!(first.to_string().len(), 64);
}
#[test]
fn test_fingerprint_missing_file_is_an_error() {
    assert!(Fingerprint::from_file(temp_dir().join("acorn-fingerprint-missing")).is_err());
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
        location: Box::new(Location::Simple(
            "https://huggingface.co/hf-internal-testing/tiny-random-bert".to_string(),
        )),
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
fn test_retry_delay_honors_retry_after_header() {
    let mut headers = HeaderMap::new();
    headers.insert(RETRY_AFTER, HeaderValue::from_static("12"));
    assert_eq!(retry_delay(Some(&headers), 1, Timestamp::UNIX_EPOCH), SignedDuration::from_secs(12));
}
#[test]
fn test_retry_delay_honors_http_date_header() {
    let now = "1994-11-06T08:49:25Z".parse::<Timestamp>().unwrap();
    let value = "Sun, 06 Nov 1994 08:49:37 GMT";
    let mut headers = HeaderMap::new();
    headers.insert(RETRY_AFTER, HeaderValue::from_bytes(value.as_bytes()).unwrap());
    assert_eq!(retry_delay(Some(&headers), 1, now), SignedDuration::from_secs(12));
}
#[test]
fn test_retry_delay_uses_bounded_exponential_fallback() {
    assert_eq!(retry_delay(None, 1, Timestamp::UNIX_EPOCH), SignedDuration::from_millis(250));
    assert_eq!(retry_delay(None, 2, Timestamp::UNIX_EPOCH), SignedDuration::from_millis(500));
    assert_eq!(retry_delay(None, usize::MAX, Timestamp::UNIX_EPOCH), SignedDuration::from_secs(60));
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
fn test_slice_path() {
    let separator = MAIN_SEPARATOR;
    let short = PathBuf::from(format!("{separator}path{separator}to{separator}foo{separator}bar{separator}baz.txt"));
    // Paths shorter than or equal to the cutoff remain unchanged.
    assert_eq!(slice_path(&short, 100), short.display().to_string());
    assert_eq!(
        slice_path(&short, short.display().to_string().chars().count()),
        short.display().to_string()
    );
    let model = PathBuf::from(format!(
        "{separator}data{separator}projects{separator}.models{separator}unsloth{separator}NVIDIA-Nemotron-3-Super-120B-A12B-GGUF{separator}UD-Q4_K_M{separator}NVIDIA-Nemotron-3-Super-120B-A12B-UD-Q4_K_M-00002-of-00003.gguf"
    ));
    // The cutoff rounds right without splitting a directory name.
    assert_eq!(
        slice_path(model, 20),
        format!(
            "{separator}data{separator}projects{separator}.models{separator}...{separator}NVIDIA-Nemotron-3-Super-120B-A12B-UD-Q4_K_M-00002-of-00003.gguf"
        )
    );
    // A small cutoff still preserves the root folder and final child.
    assert_eq!(slice_path(short, 5), format!("{separator}path{separator}...{separator}baz.txt"));
    let direct_child = PathBuf::from(format!("{separator}path{separator}baz.txt"));
    // A path without removable middle components remains unchanged.
    assert_eq!(slice_path(&direct_child, 0), direct_child.display().to_string());
    let folder = PathBuf::from(format!(
        "{separator}résumé{separator}projects{separator}models{separator}latest{separator}"
    ));
    // Unicode folder names and a trailing separator are preserved.
    assert_eq!(
        slice_path(folder, 0),
        format!("{separator}résumé{separator}...{separator}latest{separator}")
    );
    #[cfg(windows)]
    {
        // Windows drive and UNC roots are normalized while retaining their complete platform prefixes.
        assert_eq!(slice_path(r"\\?\C:\Users\acorn\models\cache\model.gguf", 0), r"C:\Users\...\model.gguf");
        assert_eq!(slice_path(r"C:\Users\acorn\models\cache\model.gguf", 0), r"C:\Users\...\model.gguf");
        assert_eq!(
            slice_path(r"\\server\share\models\cache\model.gguf", 0),
            r"\\server\share\models\...\model.gguf"
        );
    }
}
#[test]
fn test_path_conversion_uses_platform_string_paths() {
    #[cfg(windows)]
    let (path, expected) = (
        Path::new(r"\\?\C:\Users\acorn\.config/opencode/opencode.jsonc"),
        r"C:\Users\acorn\.config\opencode\opencode.jsonc",
    );
    #[cfg(not(windows))]
    let (path, expected) = (
        Path::new("/home/acorn/.config/opencode/opencode.jsonc"),
        "/home/acorn/.config/opencode/opencode.jsonc",
    );
    assert_eq!(path.cross_platform_display(), expected);
    assert_eq!(<&Path as PathConversion>::cross_platform_display(&path), expected);
    assert_eq!(PathBuf::from(path).cross_platform_display(), expected);
    assert_eq!(path.display().to_string().to_cross_platform_path(), expected);
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
    drop(progress);
    assert!(terminal.contents().contains("INFO checkpoint"));
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
#[test]
fn test_walk_returns_portable_relative_file_paths() {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let root = temp_dir().join(format!("acorn-walk-{unique}"));
    let nested = root.join("models");
    create_dir_all(&nested).unwrap();
    write(root.join("config.json"), b"{}").unwrap();
    write(nested.join("weights.gguf"), b"model").unwrap();
    let paths = walk(&root).unwrap();
    assert_eq!(
        paths,
        ["config.json".to_string(), "models/weights.gguf".to_string()].into_iter().collect()
    );
    remove_dir_all(root).unwrap();
}
#[test]
fn test_oci_descriptor_media_types_map_to_model_layers() {
    for path in ["model.gguf", "model.onnx", "model.pt", "model.safetensors"] {
        assert!(MimeType::from(path).is_model_resource(), "{path} must be accepted");
    }
    for media_type in ["application/octet-stream", OCI_LAYER, OCI_LAYER_GZIP, ACORN_MODEL_LAYER] {
        let descriptor = Descriptor {
            media_type: media_type.to_string(),
            digest: OCI_TEST_DIGEST.to_string(),
            size: 1,
            annotations: Default::default(),
        };
        assert!(MimeType::from(descriptor).is_oci_layer(), "{media_type} must be accepted");
    }
    let descriptor = Descriptor {
        media_type: "application/vnd.example.unsupported".to_string(),
        digest: OCI_TEST_DIGEST.to_string(),
        size: 1,
        annotations: Default::default(),
    };
    assert!(!MimeType::from(descriptor).is_oci_layer());
}
#[test]
fn test_oci_parses_tag_digest_port_nested_repository_and_ipv6() {
    let tagged = OciReference::parse("oci://registry.example.org:5000/ai/models/qwen:Q4_K_M").unwrap();
    assert_eq!(tagged.registry(), "registry.example.org:5000");
    assert_eq!(tagged.repository(), "ai/models/qwen");
    assert_eq!(tagged.oras_reference(), "registry.example.org:5000/ai/models/qwen:Q4_K_M");
    let digest = OciReference::parse(&format!("oci://[2001:db8::1]:5000/models/qwen@{OCI_TEST_DIGEST}")).unwrap();
    assert!(matches!(digest.reference(), OciReferenceValue::Digest(value) if value == OCI_TEST_DIGEST));
    assert_eq!(
        digest.destination(Path::new("models"), OCI_TEST_DIGEST),
        PathBuf::from("models/_2001_db8__1__5000/models/qwen@sha256-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    );
}
#[test]
fn test_oci_rejects_ambiguous_or_unsafe_references() {
    for value in [
        "registry.example.org/models/qwen:v1",
        "oci://registry.example.org/models/qwen",
        "oci://user@registry.example.org/models/qwen:v1",
        "oci://registry.example.org/models/../qwen:v1",
        "oci://registry.example.org/models/qwen:v1#fragment",
        "oci://registry.example.org/models/qwen@sha256:abc",
    ] {
        assert!(OciReference::parse(value).is_err(), "{value} must be rejected");
    }
}
#[test]
fn test_oci_validates_artifact_paths_and_rejects_container_images() {
    for value in ["../model.gguf", "/model.gguf", "models/../../model.gguf"] {
        assert!(OciFile {
            media_type: "application/octet-stream".to_string(),
            digest: OCI_TEST_DIGEST.to_string(),
            size: 1,
            path: value.to_string(),
            installed_size: Some(1),
            layer_digest: OCI_TEST_DIGEST.to_string(),
        }
        .validate()
        .is_err());
    }
    let manifest = Manifest {
        schema_version: 2,
        media_type: OCI_IMAGE_MANIFEST.to_string(),
        artifact_type: None,
        config: Descriptor {
            media_type: OCI_IMAGE_CONFIG.to_string(),
            digest: OCI_TEST_DIGEST.to_string(),
            size: 1,
            annotations: Default::default(),
        },
        layers: vec![Descriptor {
            media_type: "application/octet-stream".to_string(),
            digest: OCI_TEST_DIGEST.to_string(),
            size: 1,
            annotations: [("org.opencontainers.image.title".to_string(), "model.gguf".to_string())].into(),
        }],
    };
    assert!(manifest.validate().is_err());
}
#[test]
fn test_oci_policy_rejects_non_gguf_model_resources_and_extensionless_store_paths() {
    for path in ["model.safetensors", "model.onnx", "model.pt"] {
        assert!(MimeType::from(path).is_model_resource(), "{path} must remain a generic model resource");
    }
    for path in [
        "model.safetensors",
        "model.onnx",
        "model.pt",
        "blobs/sha256-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "manifests/registry.ollama.ai/library/gpt-oss/20b",
    ] {
        let descriptor = Descriptor {
            media_type: "application/octet-stream".to_string(),
            digest: OCI_TEST_DIGEST.to_string(),
            size: 1,
            annotations: [("org.opencontainers.image.title".to_string(), path.to_string())].into(),
        };
        assert!(descriptor.of_model().is_err(), "{path} must be rejected");
    }
}
#[test]
fn test_modelpack_weight_layer_policy_is_gguf_only() {
    let descriptor = |media_type: &str, filepath: Option<&str>| Descriptor {
        media_type: media_type.to_string(),
        digest: OCI_TEST_DIGEST.to_string(),
        size: 1,
        annotations: filepath
            .map(|path| [("org.cncf.model.filepath".to_string(), path.to_string())].into())
            .unwrap_or_default(),
    };
    let raw = OciLayer::from(descriptor(MODELPACK_WEIGHT_RAW, Some("model.gguf")));
    let tar = OciLayer::from(descriptor(MODELPACK_WEIGHT_TAR, None));
    let tar_gzip = OciLayer::from(descriptor(MODELPACK_WEIGHT_TAR_GZIP, None));
    assert_eq!(raw.extraction, OciExtraction::None);
    assert_eq!(tar.extraction, OciExtraction::Tar);
    assert_eq!(tar_gzip.extraction, OciExtraction::TarGzip);
    assert!(raw.validate().is_ok());
    assert!(tar.validate().is_ok());
    assert!(tar_gzip.validate().is_ok());
    assert!(OciLayer::from(descriptor(MODELPACK_WEIGHT_RAW, None)).validate().is_err());
    assert!(OciLayer::from(descriptor(MODELPACK_WEIGHT_RAW, Some("model.safetensors")))
        .validate()
        .is_err());
    assert!(OciLayer::from(descriptor(MODELPACK_WEIGHT_TAR_ZSTD, None)).validate().is_err());
}
#[test]
fn test_split_gguf_paths_require_one_complete_primary_group() {
    let file = |path: &str| OciFile {
        media_type: "application/gguf".to_string(),
        digest: OCI_TEST_DIGEST.to_string(),
        size: 1,
        path: path.to_string(),
        installed_size: Some(1),
        layer_digest: OCI_TEST_DIGEST.to_string(),
    };
    assert!(OciFiles(vec![file("model-00001-of-00002.gguf"), file("model-00002-of-00002.gguf")])
        .validate_paths()
        .is_ok());
    assert!(OciFiles(vec![file("model-00001-of-00002.gguf")]).validate_paths().is_err());
    assert!(OciFiles(vec![file("model.gguf"), file("other.gguf")]).validate_paths().is_err());
    assert!(OciFiles(vec![file("mmproj-model.gguf")]).validate_paths().is_err());
    assert!(OciFiles(vec![file("model.gguf"), file("mmproj-model.gguf")]).validate_paths().is_ok());
}
#[test]
fn test_oci_validates_minimum_oras_version() {
    let minimum = SemanticVersion::from(MINIMUM_ORAS_VERSION);
    assert!(SemanticVersion::from("Version: 1.3.0\nGo version: go1.24").is_supported(minimum).is_ok());
    assert!(SemanticVersion::from("Version: 1.2.3").is_supported(minimum).is_err());
    assert!(SemanticVersion::from("unexpected output").is_supported(minimum).is_err());
}
#[test]
fn test_oci_transport_args_append_configured_registry_options() {
    let transport = OrasTransport {
        binary: PathBuf::from("oras"),
        options: OciTransportOptions {
            credential_env: None,
            username: Some("robot".to_string()),
            registry_config: Some(PathBuf::from("registry.json")),
            ca_file: Some(PathBuf::from("ca.pem")),
            client_cert: Some(PathBuf::from("client.pem")),
            client_key: Some(PathBuf::from("client-key.pem")),
            plain_http: true,
        },
    };
    assert_eq!(
        transport.transport_args(args!["pull", "registry.example/model:v1"], true),
        args![
            "pull",
            "registry.example/model:v1",
            "--password-stdin",
            "--username=robot",
            "--registry-config=registry.json",
            "--ca-file=ca.pem",
            "--cert-file=client.pem",
            "--key-file=client-key.pem",
            "--plain-http"
        ]
    );
}
#[cfg(unix)]
#[test]
fn test_oci_fake_oras_resolves_and_atomically_pulls() {
    let temp = tempfile::tempdir().unwrap();
    let binary = temp.path().join("oras");
    let model = temp.path().join("model.gguf");
    write(&model, minimal_gguf()).unwrap();
    let model_size = metadata(&model).unwrap().len();
    let model_digest = format!("sha256:{}", file_checksum(&model, None).unwrap());
    let script = format!(
        r#"#!/bin/sh
if [ "$1" = "manifest" ] && [ "$3" = "--descriptor" ]; then
  printf '%s' '{{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"{OCI_TEST_DIGEST}","size":42,"annotations":{{}}}}'
elif [ "$1" = "manifest" ]; then
  printf '%s' '{{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","artifactType":"application/vnd.acorn.model.v1","config":{{"mediaType":"application/vnd.oci.empty.v1+json","digest":"{OCI_TEST_DIGEST}","size":2,"annotations":{{}}}},"layers":[{{"mediaType":"application/octet-stream","digest":"{model_digest}","size":{model_size},"annotations":{{"org.opencontainers.image.title":"model-Q4_K_M.gguf"}}}}]}}'
elif [ "$1" = "blob" ]; then
  cp '{}' "$4"
else
  exit 1
fi
"#,
        model.display()
    );
    write(&binary, script).unwrap();
    set_permissions(&binary, Permissions::from_mode(0o700)).unwrap();
    let transport = OrasTransport {
        binary,
        options: OciTransportOptions::default(),
    };
    let reference = OciReference::parse("oci://registry.example.org/ai/qwen:v1").unwrap();
    let resolution = transport.resolve(&reference, &[], &[], &[Quantization::Q4kM]).unwrap();
    assert_eq!(resolution.resolved_digest, OCI_TEST_DIGEST);
    assert_eq!(
        resolution.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
        ["model-Q4_K_M.gguf"]
    );
    let destination = temp.path().join("published");
    transport.pull(&reference, &resolution, &destination).unwrap();
    assert_eq!(read(destination.join("model-Q4_K_M.gguf")).unwrap(), minimal_gguf());
    assert!(transport.pull(&reference, &resolution, &destination).is_err());
}
#[cfg(unix)]
#[test]
fn test_oci_fake_oras_resolves_and_pulls_raw_modelpack() {
    let temp = tempfile::tempdir().unwrap();
    let binary = temp.path().join("oras");
    let model = temp.path().join("model.gguf");
    write(&model, minimal_gguf()).unwrap();
    let model_size = metadata(&model).unwrap().len();
    let model_digest = format!("sha256:{}", file_checksum(&model, None).unwrap());
    let script = format!(
        r#"#!/bin/sh
if [ "$1" = "manifest" ] && [ "$3" = "--descriptor" ]; then
  printf '%s' '{{"mediaType":"{OCI_IMAGE_MANIFEST}","digest":"{OCI_TEST_DIGEST}","size":42,"annotations":{{}}}}'
elif [ "$1" = "manifest" ]; then
  printf '%s' '{{"schemaVersion":2,"mediaType":"{OCI_IMAGE_MANIFEST}","artifactType":"{MODELPACK_MANIFEST}","config":{{"mediaType":"{MODELPACK_CONFIG}","digest":"{OCI_TEST_DIGEST}","size":2,"annotations":{{}}}},"layers":[{{"mediaType":"{MODELPACK_WEIGHT_RAW}","digest":"{model_digest}","size":{model_size},"annotations":{{"org.cncf.model.filepath":"weights/model-Q4_K_M.gguf"}}}}]}}'
elif [ "$1" = "blob" ] && [ "$4" = "-" ]; then
  printf '%s' '{{}}'
elif [ "$1" = "blob" ]; then
  cp '{}' "$4"
else
  exit 1
fi
"#,
        model.display()
    );
    write(&binary, script).unwrap();
    set_permissions(&binary, Permissions::from_mode(0o700)).unwrap();
    let transport = OrasTransport {
        binary,
        options: OciTransportOptions::default(),
    };
    let reference = OciReference::parse("oci://registry.example.org/cncf/model:v1").unwrap();
    let resolution = transport.resolve(&reference, &[], &[], &[Quantization::Q4kM]).unwrap();
    assert_eq!(resolution.artifact_type.as_deref(), Some(MODELPACK_MANIFEST));
    assert_eq!(resolution.layers.len(), 1);
    assert_eq!(resolution.files[0].path, "weights/model-Q4_K_M.gguf");
    assert_eq!(resolution.transfer_size, model_size);
    assert_eq!(resolution.installed_size, Some(model_size));
    let destination = temp.path().join("published-modelpack");
    transport.pull(&reference, &resolution, &destination).unwrap();
    assert_eq!(read(destination.join("weights/model-Q4_K_M.gguf")).unwrap(), minimal_gguf());
}
#[cfg(unix)]
#[test]
fn test_oci_fake_oras_resolves_single_model_index_candidate() {
    let temp = tempfile::tempdir().unwrap();
    let binary = temp.path().join("oras");
    let model = temp.path().join("model.gguf");
    write(&model, minimal_gguf()).unwrap();
    let model_size = metadata(&model).unwrap().len();
    let model_digest = format!("sha256:{}", file_checksum(&model, None).unwrap());
    let child_digest = format!("sha256:{}", "b".repeat(64));
    let script = format!(
        r#"#!/bin/sh
if [ "$1" = "manifest" ] && [ "$3" = "--descriptor" ]; then
  printf '%s' '{{"mediaType":"{OCI_IMAGE_INDEX}","digest":"{OCI_TEST_DIGEST}","size":42,"annotations":{{}}}}'
elif [ "$1" = "manifest" ] && echo "$3" | grep -q '{OCI_TEST_DIGEST}'; then
  printf '%s' '{{"schemaVersion":2,"mediaType":"{OCI_IMAGE_INDEX}","manifests":[{{"mediaType":"{OCI_IMAGE_MANIFEST}","digest":"{child_digest}","size":42,"annotations":{{}}}}]}}'
elif [ "$1" = "manifest" ]; then
  printf '%s' '{{"schemaVersion":2,"mediaType":"{OCI_IMAGE_MANIFEST}","artifactType":"application/vnd.acorn.model.v1","config":{{"mediaType":"application/vnd.oci.empty.v1+json","digest":"{OCI_TEST_DIGEST}","size":2,"annotations":{{}}}},"layers":[{{"mediaType":"application/octet-stream","digest":"{model_digest}","size":{model_size},"annotations":{{"org.opencontainers.image.title":"model.gguf"}}}}]}}'
elif [ "$1" = "blob" ]; then
  cp '{}' "$4"
else
  exit 1
fi
"#,
        model.display()
    );
    write(&binary, script).unwrap();
    set_permissions(&binary, Permissions::from_mode(0o700)).unwrap();
    let transport = OrasTransport {
        binary,
        options: OciTransportOptions::default(),
    };
    let reference = OciReference::parse("oci://registry.example.org/cncf/index:v1").unwrap();
    let resolution = transport.resolve(&reference, &[], &[], &[]).unwrap();
    assert_eq!(resolution.resolved_digest, child_digest);
    let destination = temp.path().join("published-index");
    transport.pull(&reference, &resolution, &destination).unwrap();
    assert_eq!(read(destination.join("model.gguf")).unwrap(), minimal_gguf());
}
#[cfg(unix)]
#[test]
fn test_oci_fake_oras_defers_and_extracts_modelpack_archive_inventory() {
    let temp = tempfile::tempdir().unwrap();
    let binary = temp.path().join("oras");
    let source = temp.path().join("modelpack-layer");
    create_dir_all(source.join("weights")).unwrap();
    write(source.join("weights/model-Q4_K_M.gguf"), minimal_gguf()).unwrap();
    let layer = temp.path().join("weights.tar.gz");
    archive(source, Some(layer.clone()), MimeType::Gzip).unwrap();
    let layer_size = metadata(&layer).unwrap().len();
    let layer_digest = format!("sha256:{}", file_checksum(&layer, None).unwrap());
    let script = format!(
        r#"#!/bin/sh
if [ "$1" = "manifest" ] && [ "$3" = "--descriptor" ]; then
  printf '%s' '{{"mediaType":"{OCI_IMAGE_MANIFEST}","digest":"{OCI_TEST_DIGEST}","size":42,"annotations":{{}}}}'
elif [ "$1" = "manifest" ]; then
  printf '%s' '{{"schemaVersion":2,"mediaType":"{OCI_IMAGE_MANIFEST}","artifactType":"{MODELPACK_MANIFEST}","config":{{"mediaType":"{MODELPACK_CONFIG}","digest":"{OCI_TEST_DIGEST}","size":2,"annotations":{{}}}},"layers":[{{"mediaType":"{MODELPACK_WEIGHT_TAR_GZIP}","digest":"{layer_digest}","size":{layer_size},"annotations":{{}}}}]}}'
elif [ "$1" = "blob" ] && [ "$4" = "-" ]; then
  printf '%s' '{{}}'
elif [ "$1" = "blob" ]; then
  cp '{}' "$4"
else
  exit 1
fi
"#,
        layer.display()
    );
    write(&binary, script).unwrap();
    set_permissions(&binary, Permissions::from_mode(0o700)).unwrap();
    let transport = OrasTransport {
        binary,
        options: OciTransportOptions::default(),
    };
    let reference = OciReference::parse("oci://registry.example.org/cncf/archive:v1").unwrap();
    let resolution = transport.resolve(&reference, &[], &[], &[Quantization::Q4kM]).unwrap();
    assert!(resolution.files.is_empty());
    assert!(resolution.layers[0].inventory_deferred);
    assert_eq!(resolution.installed_size, None);
    let destination = temp.path().join("published-modelpack-archive");
    transport.pull(&reference, &resolution, &destination).unwrap();
    assert_eq!(read(destination.join("weights/model-Q4_K_M.gguf")).unwrap(), minimal_gguf());
}
#[cfg(unix)]
#[test]
fn test_oci_fake_oras_resolves_and_unpacks_modelkit_model_layer() {
    let temp = tempfile::tempdir().unwrap();
    let binary = temp.path().join("oras");
    let source = temp.path().join("model-layer");
    create_dir_all(&source).unwrap();
    let model = source.join("model.gguf");
    let layer = temp.path().join("model.tar.gz");
    write(&model, minimal_gguf()).unwrap();
    archive(source, Some(layer.clone()), MimeType::Gzip).unwrap();
    let layer_size = metadata(&layer).unwrap().len();
    let layer_digest = format!("sha256:{}", file_checksum(&layer, None).unwrap());
    let script = format!(
        r#"#!/bin/sh
if [ "$1" = "manifest" ] && [ "$3" = "--descriptor" ]; then
  printf '%s' '{{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"{OCI_TEST_DIGEST}","size":42,"annotations":{{}}}}'
elif [ "$1" = "manifest" ]; then
  printf '%s' '{{"schemaVersion":2,"config":{{"mediaType":"{KITOPS_MODELKIT_CONFIG}","digest":"{OCI_TEST_DIGEST}","size":100,"annotations":{{}}}},"layers":[{{"mediaType":"{KITOPS_MODELKIT_MODEL_GZIP}","digest":"{layer_digest}","size":{layer_size},"annotations":{{}}}}]}}'
elif [ "$1" = "blob" ] && [ "$4" = "-" ]; then
  printf '%s' '{{"manifestVersion":"1.0.0","model":{{"path":"model.gguf"}}}}'
elif [ "$1" = "blob" ]; then
  cp '{}' "$4"
else
  exit 1
fi
"#,
        layer.display()
    );
    write(&binary, script).unwrap();
    set_permissions(&binary, Permissions::from_mode(0o700)).unwrap();
    let transport = OrasTransport {
        binary,
        options: OciTransportOptions::default(),
    };
    let reference = OciReference::parse("oci://registry.example.org/jozu/model:v1").unwrap();
    let resolution = transport.resolve(&reference, &[], &[], &[]).unwrap();
    assert_eq!(resolution.files.len(), 1);
    assert_eq!(resolution.files[0].path, "model.gguf");
    assert_eq!(resolution.layers[0].media_type, KITOPS_MODELKIT_MODEL_GZIP);
    assert!(resolution.layers[0].inventory_deferred);
    let destination = temp.path().join("published-modelkit");
    transport.pull(&reference, &resolution, &destination).unwrap();
    assert_eq!(read(destination.join("model.gguf")).unwrap(), minimal_gguf());
}
