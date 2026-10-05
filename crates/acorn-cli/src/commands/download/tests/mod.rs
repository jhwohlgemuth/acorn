//! Tests for the download command
//!
//! These tests validate local-bucket download execution with silent verbosity.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
extern crate alloc;
use crate::{
    cli::arguments::{ArchiveType, Download, DownloadModel, Import, ModelResolutionArgs, ModelSourceArgs, ModelSyncArgs, OutputFormat, SyncTarget},
    test::{
        server::{
            spawn_basic_model_config_server, spawn_file_server, spawn_huggingface_model_info_server, spawn_huggingface_model_search_server,
            spawn_huggingface_model_unavailable_server, spawn_ignored_range_server, spawn_no_range_server, spawn_range_server,
            spawn_retry_resume_server, spawn_sidecar_server,
        },
        util::{fixture_path, has_any_file, temp_test_dir, TestCleanup},
    },
};
use acorn::{
    io::{
        api::huggingface::{
            Downloaded, FileSelectionPolicy, HuggingFaceError, HuggingFaceRepository, HuggingFaceRepositoryFile, HuggingFaceRepositoryFiles,
        },
        bagit::{Bag, Save},
        config::{ApplicationConfiguration, AuthenticationRequirement, ModelEntry},
        database::{
            schema::{ActivityRow, Table},
            Database, Row,
        },
        download::{DownloadItem, DownloadItems, DownloadTask},
        InputOutput, SourceAction,
    },
    util::constants::env::{HUGGINGFACE_TOKEN_VARIABLE_NAMES, MODEL_WHITELIST},
};
use acorn_core::{
    prelude::HashSet,
    util::{merge, MarkdownSupport, MimeType},
};
use acorn_schema::{
    agent::{ModelDetails, ModelSelectors, Quantization, Weight, Weights},
    hardware::memory::Memory,
    research_activity::ResearchActivity,
    OneOrMany,
};
use alloc::sync::Arc;
use assert_cmd::Command;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use core::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::{
    fs::{create_dir_all, read, read_to_string, write},
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
};
use temp_env::{with_var, with_vars};
use tracing_subscriber::fmt::MakeWriter;

static HF_ENDPOINT_TEST_LOCK: Mutex<()> = Mutex::new(());
const OFFLINE: bool = false;
const TEST_THREADS: usize = 2;

#[derive(Clone)]
struct SharedWriter(Arc<Mutex<Vec<u8>>>);

struct SharedWriterGuard(Arc<Mutex<Vec<u8>>>);

impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedWriterGuard;
    fn make_writer(&'a self) -> Self::Writer {
        SharedWriterGuard(self.0.clone())
    }
}

impl Write for SharedWriterGuard {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("lock log buffer").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn candidate_file(path: &str) -> HuggingFaceRepositoryFile {
    HuggingFaceRepositoryFile {
        path: path.to_string(),
        size: None,
    }
}

/// Build bucket download arguments and run the download command.
#[allow(clippy::too_many_arguments)]
async fn download_bucket(
    config: &Option<PathBuf>,
    url: &[String],
    filter: &[String],
    ignore: &[String],
    output: &Option<PathBuf>,
    flatten: bool,
    clobber: bool,
    strict: bool,
    canonicalize: bool,
    archive_format: Option<ArchiveType>,
    database_path: &Option<PathBuf>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
    no_local_database: bool,
) -> crate::cli::Void {
    let arguments = Download {
        command: None,
        url: url.to_vec(),
        config: config.clone(),
        filter: filter.to_vec(),
        ignore: ignore.to_vec(),
        output: output.clone(),
        flatten,
        clobber,
        strict,
        canonicalize,
        archive_format,
        verbose: *verbose,
    };
    super::run(&arguments, database_path, threads, offline, no_local_database).await
}

#[allow(clippy::too_many_arguments)]
async fn execute_model(
    model: &[String],
    model_file: &Option<String>,
    sync: &Option<SyncTarget>,
    force: bool,
    filter: &[String],
    ignore: &[String],
    config: &Option<PathBuf>,
    output: &Option<PathBuf>,
    database_path: &Option<PathBuf>,
    verbose: &Verbosity,
    offline: bool,
    copy: bool,
    symlink: bool,
    skip_verify_checksum: bool,
    whitelist: &[String],
    no_fallback: bool,
    no_local_database: bool,
    quantization: &[Quantization],
    gpu_memory: &Option<Memory>,
    search_limit: usize,
    minimum_download_count: u64,
    interactive: bool,
    dry_run: bool,
    raw: bool,
    format: &OutputFormat,
) -> crate::cli::Void {
    let arguments = DownloadModel {
        source: ModelSourceArgs {
            model: model.to_vec(),
            model_file: model_file.clone(),
            config: config.clone(),
        },
        resolution: ModelResolutionArgs {
            no_fallback,
            interactive,
            search_limit,
            minimum_download_count,
        },
        synchronization: ModelSyncArgs { sync: *sync, force, dry_run },
        filter: filter.to_vec(),
        ignore: ignore.to_vec(),
        quantization: quantization.to_vec(),
        gpu_memory: gpu_memory.clone(),
        local_dir: output.clone(),
        copy,
        symlink,
        whitelist: whitelist.to_vec(),
        whitelist_file: None,
        skip_verify_checksum,
        raw,
        format: format.clone(),
        verbose: *verbose,
    };
    super::model::execute(super::model::Request {
        arguments: &arguments,
        whitelist,
        database_path,
        offline,
        no_local_database,
    })
    .await
}

#[allow(clippy::too_many_arguments)]
async fn import_local(
    config: &Option<PathBuf>,
    uri: &[String],
    filter: &[String],
    ignore: &[String],
    output: &Option<PathBuf>,
    flatten: bool,
    clobber: bool,
    database_path: &Option<PathBuf>,
    threads: usize,
    verbose: &Verbosity,
    offline: bool,
    no_local_database: bool,
) -> crate::cli::Void {
    let arguments = Import {
        command: None,
        uri: uri.to_vec(),
        config: config.clone(),
        filter: filter.to_vec(),
        ignore: ignore.to_vec(),
        output: output.clone(),
        flatten,
        clobber,
        archive_format: None,
        verbose: *verbose,
    };
    crate::commands::import::run(&arguments, database_path, threads, offline, no_local_database).await
}

fn model_plan_matches_base(base_model: &str) -> bool {
    let identifier = "unsloth/gpt-oss-2b-gguf";
    let (endpoint, handle) = spawn_huggingface_model_info_server(identifier, base_model);
    let whitelist = HashSet::from(["openai/gpt-oss-2b".to_string()]);
    let result = with_hf_endpoint(endpoint, || {
        tokio::runtime::Runtime::new().expect("create runtime").block_on(async {
            let entries = vec![ModelEntry::Selector(identifier.to_string())];
            let plans = super::model::plan::resolve_plans(&ModelSelectors::default(), &entries, &Default::default(), None, &None, false, false)
                .await
                .expect("resolve model plan");
            plans
                .into_iter()
                .next()
                .expect("model plan")
                .matches(&whitelist, false)
                .await
                .expect("match model whitelist")
        })
    });
    handle.join().expect("join metadata server");
    result
}

fn repository_files(files: Vec<HuggingFaceRepositoryFile>) -> HuggingFaceRepositoryFiles {
    HuggingFaceRepositoryFiles::new("acme/model", "main", files)
}

#[allow(clippy::too_many_arguments)]
async fn run_model(
    model: &[String],
    filter: &[String],
    ignore: &[String],
    config: &Option<PathBuf>,
    output: &Option<PathBuf>,
    database_path: &Option<PathBuf>,
    verbose: &Verbosity,
    offline: bool,
    copy: bool,
    symlink: bool,
    skip_verify_checksum: bool,
    whitelist: &[String],
    no_fallback: bool,
    search_limit: usize,
    interactive: bool,
    dry_run: bool,
    raw: bool,
    format: &OutputFormat,
) -> crate::cli::Void {
    execute_model(
        model,
        &None,
        &None,
        false,
        filter,
        ignore,
        config,
        output,
        database_path,
        verbose,
        offline,
        copy,
        symlink,
        skip_verify_checksum,
        whitelist,
        no_fallback,
        false,
        &[],
        &None,
        search_limit,
        100,
        interactive,
        dry_run,
        raw,
        format,
    )
    .await
}

fn with_hf_endpoint<T>(endpoint: String, test: impl FnOnce() -> T) -> T {
    let guard = HF_ENDPOINT_TEST_LOCK.lock().expect("lock Hugging Face endpoint tests");
    let result = with_vars(
        [
            ("HF_ENDPOINT", Some(endpoint.as_str())),
            (HUGGINGFACE_TOKEN_VARIABLE_NAMES[0], Some("")),
            (HUGGINGFACE_TOKEN_VARIABLE_NAMES[1], Some("")),
            (HUGGINGFACE_TOKEN_VARIABLE_NAMES[2], Some("")),
        ],
        test,
    );
    drop(guard);
    result
}

/// Build a BagIt archive containing a single research activity file, returning the archive path.
fn write_bagit_archive(root: &Path, format: MimeType, identifier: &str) -> Result<PathBuf> {
    let payload_dir = root.join("payload");
    write_rad(&payload_dir.join("index.json"), identifier)?;
    let archive = root.join(format!("bag.{}", format.file_type()));
    Bag::init()
        .base_directory(payload_dir.to_string_lossy().to_string())
        .payload(vec!["index.json".to_string()])
        .build()
        .save_as(root.join("bag"), format, Some(archive.clone()))?;
    Ok(archive)
}

/// Build a BagIt archive whose payload holds one research activity file per project folder.
///
/// The bag is written beneath a single enclosing directory when `enclosing` is set, which the
/// download path must normalize to the same payload layout as a bag at the archive root.
fn write_nested_bagit_archive(root: &Path, format: MimeType, enclosing: bool) -> Result<PathBuf> {
    let payload_dir = root.join("payload");
    let payload = ["project-a/index.json", "project-b/index.yaml"]
        .into_iter()
        .map(|relative| {
            write_rad(&payload_dir.join(relative), &format!("https://example.org/{relative}"))?;
            Ok(relative.to_string())
        })
        .collect::<Result<Vec<_>>>()?;
    let archive = root.join(format!("bag.{}", format.file_type()));
    let bag = Bag::init()
        .base_directory(payload_dir.to_string_lossy().to_string())
        .payload(payload)
        .build();
    match enclosing {
        | true => {
            let staged = root.join("staging");
            bag.write(&staged)?;
            let container = root.join("container");
            std::fs::create_dir_all(&container).expect("create enclosing container");
            std::fs::rename(staged, container.join("bag")).expect("move bag into container");
            acorn::io::archive(container, Some(archive.clone()), format).expect("archive enclosing container");
        }
        | false => {
            bag.save_as(root.join("bag"), format, Some(archive.clone()))?;
        }
    }
    Ok(archive)
}

fn write_rad(path: &std::path::Path, identifier: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut rad = ResearchActivity::default();
    rad.meta.identifier = identifier.to_string();
    rad.title = format!("Project {identifier}");
    let content = match path.extension().and_then(|value| value.to_str()) {
        | Some("yaml" | "yml") => serde_norway::to_string(&rad)?,
        | Some("md" | "markdown") => rad.to_markdown(),
        | _ => serde_json::to_string_pretty(&rad)?,
    };
    write(path, content)?;
    Ok(())
}
#[tokio::test]
async fn test_config_fixture_resolves_model_fallbacks() {
    let config = ApplicationConfiguration::read(fixture_path("config/with_models.json")).unwrap();
    let plans = super::model::plan::resolve_plans(
        &ModelSelectors::default(),
        &config.models.unwrap(),
        &Default::default(),
        None,
        &None,
        false,
        true,
    )
    .await
    .unwrap();
    let fallbacks = super::model::plan::ModelDownloadPlans(plans)
        .resolve(super::model::plan::DownloadOptions {
            offline: true,
            limit: 200,
            minimum_download_count: 100,
            interactive: false,
        })
        .await
        .into_iter()
        .map(ModelDetails::from)
        .map(|model| (model.id, model.fallback))
        .collect::<Vec<_>>();
    assert_eq!(
        fallbacks,
        vec![
            (Some("meta-llama/Llama-2-7b-hf".to_string()), None),
            (Some(PathBuf::from("./models/qwen.gguf").display().to_string()), None),
            (Some("https://huggingface.co/microsoft/phi-2".to_string()), None),
            (Some("hf-internal-testing/tiny-random-bert".to_string()), None),
            (Some("Qwen/Qwen2.5-0.5B-Instruct-GGUF".to_string()), None),
        ]
    );
}

#[tokio::test]
async fn test_config_fixture_resolves_models() {
    let config = ApplicationConfiguration::read(fixture_path("config/with_models.json")).unwrap();
    let plans = super::model::plan::resolve_plans(
        &ModelSelectors::default(),
        &config.models.unwrap(),
        &Default::default(),
        None,
        &None,
        false,
        true,
    )
    .await
    .unwrap();
    let resolved = plans.iter().map(|plan| (plan.name(), plan.selector.identifier())).collect::<Vec<_>>();
    assert_eq!(
        resolved,
        vec![
            ("meta-llama/Llama-2-7b-hf".to_string(), "meta-llama/Llama-2-7b-hf".to_string()),
            ("qwen".to_string(), PathBuf::from("./models/qwen.gguf").display().to_string()),
            (
                "https://huggingface.co/microsoft/phi-2".to_string(),
                "https://huggingface.co/microsoft/phi-2".to_string(),
            ),
            ("tiny-bert".to_string(), "hf-internal-testing/tiny-random-bert".to_string()),
            ("qwen-gguf".to_string(), "Qwen/Qwen2.5-0.5B-Instruct-GGUF".to_string()),
        ]
    );
}

#[tokio::test]
async fn test_config_fixture_resolves_whitelisted_models() -> Result<()> {
    let config = ApplicationConfiguration::read(fixture_path("config/with_models.json"))?;
    let (entries, whitelist) = config.model_entries_and_whitelist();
    let whitelist = whitelist.map(OneOrMany::into_vec).unwrap_or_default();
    assert_eq!(whitelist, vec!["meta-llama/Llama-2-7b-hf".to_string(), "openai/gpt-oss-2b".to_string()]);
    let whitelist = whitelist.into_iter().collect::<HashSet<_>>();
    let plans = super::model::plan::resolve_plans(&ModelSelectors::default(), &entries, &Default::default(), None, &None, false, true)
        .await
        .unwrap();
    let resolved = futures::future::join_all(plans.into_iter().map(|plan| {
        let whitelist = &whitelist;
        async move { plan.matches(whitelist, true).await.map(|matches| matches.then(|| plan.name())) }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    assert_eq!(resolved, vec!["meta-llama/Llama-2-7b-hf".to_string()]);
    Ok(())
}

#[test]
fn test_config_parses_hugging_face_location_model_entry() {
    let content = r#"{"models": [{"name": "tiny-bert", "source": {"provider": "huggingface", "location": {"scheme": "https", "uri": "https://huggingface.co/hf-internal-testing/tiny-random-bert", "revision": "main"}}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 1);
}

#[test]
fn test_config_parses_local_model_entry() {
    let content = r#"{"models": [{"name": "qwen-local", "source": {"provider": "git", "location": "file:./models/qwen.gguf"}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 1);
}

#[test]
fn test_config_parses_mixed_model_entries() {
    let content =
        r#"{"models": ["meta-llama/Llama-2-7b-hf", {"name": "qwen-local", "source": {"provider": "git", "location": "file:./models/qwen.gguf"}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 2);
}

#[test]
fn test_config_parses_model_download_options() {
    let content = r#"{
        "models": [{
            "name": "tiny-bert",
            "source": {"provider": "huggingface", "location": "https://huggingface.co/hf-internal-testing/tiny-random-bert"},
            "revision": "refs/pr/1",
            "auth": "required",
            "filter": ["Q4_K_M.*\\.gguf$"],
            "ignore": ["Q2_", "Q3_"]
        }]
    }"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    match models.first().unwrap() {
        | ModelEntry::Entry(entry) => {
            assert_eq!(entry.revision.as_deref(), Some("refs/pr/1"));
            assert_eq!(entry.auth, Some(AuthenticationRequirement::Required));
            assert_eq!(entry.filter.as_ref().unwrap(), &["Q4_K_M.*\\.gguf$".to_string()]);
            assert_eq!(entry.ignore.as_ref().unwrap(), &["Q2_".to_string(), "Q3_".to_string()]);
        }
        | ModelEntry::Selector(_) => panic!("expected detailed model entry"),
    }
}

#[test]
fn test_config_parses_model_metadata_constraints() {
    let config = ApplicationConfiguration::parse(
        r#"{"models":[{"name":"tiny","source":{"provider":"huggingface","location":"https://huggingface.co/acme/tiny"},"quantization":["Q5_K_M","Q4_K_M"],"gpuMemory":"1.5GB"}]}"#,
    )
    .unwrap();
    let options = config
        .models
        .and_then(|models| {
            models.into_iter().find_map(|entry| match entry {
                | ModelEntry::Entry(options) => Some(options),
                | ModelEntry::Selector(_) => None,
            })
        })
        .unwrap();
    assert_eq!(
        options
            .quantization
            .map(OneOrMany::into_vec)
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["Q5_K_M", "Q4_K_M"]
    );
    assert_eq!(options.gpu_memory.and_then(|memory| memory.checked_bytes()), Some(1_610_612_736));
}

#[test]
fn test_config_parses_models_field_empty() {
    let content = r#"{"buckets": []}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    assert!(config.models.is_none());
}

#[test]
fn test_config_parses_models_field_json() {
    let content = r#"{"models": ["meta-llama/Llama-2-7b-hf", "microsoft/phi-2"]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 2);
}

#[test]
fn test_config_parses_models_field_yaml() {
    let content = r#"
models:
  - meta-llama/Llama-2-7b-hf
  - microsoft/phi-2
"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let models = config.models.unwrap();
    assert_eq!(models.len(), 2);
}

#[test]
fn test_config_parses_single_model_whitelist_url() {
    let url = "https://example.com/models.json";
    let config = ApplicationConfiguration::parse(format!(r#"{{"whitelist":{{"models":"{url}"}}}}"#).as_str()).unwrap();
    assert_eq!(config.whitelist.and_then(|lookup| lookup.models), Some(OneOrMany::One(url.to_string())));
}

#[tokio::test]
async fn test_download_archive_format_overrides_url_inference() -> Result<()> {
    let temp_dir = temp_test_dir("download_bagit_archive_override");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    let archive = write_bagit_archive(&temp_dir, MimeType::Zip, "https://example.org/archive-override")?;
    let result = download_bucket(
        &None,
        &[archive.to_string_lossy().to_string()],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        false,
        false,
        Some(ArchiveType::Zip),
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(result.is_ok());
    assert!(read_to_string(output_dir.join("index.json"))?.contains("https://example.org/archive-override"));
    Ok(())
}

#[test]
fn test_download_archive_format_rejects_model_subcommand() {
    let temp_dir = temp_test_dir("download_archive_format_model_rejection");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output = Command::cargo_bin("acorn")
        .expect("locate acorn binary")
        .args([
            "--no-local-database",
            "download",
            "--archive-format",
            "zip",
            "model",
            "acme/model",
            "--dry-run",
        ])
        .output()
        .expect("run model download with bucket-only flag");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--archive-format is only supported for bucket downloads"));
}

#[tokio::test]
async fn test_download_archive_rejects_unverified_bag() {
    let temp_dir = temp_test_dir("download_bagit_archive_unverified");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    let tampered = temp_dir.join("tampered");
    let archive = write_bagit_archive(&temp_dir, MimeType::Zip, "https://example.org/archive-unverified").expect("create archive");
    // Corrupt the payload so the manifest checksum no longer matches.
    acorn::io::extract(archive.clone(), Some(tampered.clone()), None).expect("extract archive");
    write(tampered.join("data").join("index.json"), "tampered").expect("tamper with payload");
    acorn::io::archive(tampered, Some(archive.clone()), MimeType::Zip).expect("re-archive tampered payload");
    let result = download_bucket(
        &None,
        &[archive.to_string_lossy().to_string()],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(result.is_err(), "an archive with a checksum mismatch must be rejected");
    assert!(!output_dir.exists(), "no payload may be published from an unverified bag");
}

#[tokio::test]
async fn test_download_bagit_archive_applies_filter() -> Result<()> {
    let temp_dir = temp_test_dir("download_bagit_filter");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let archive = write_nested_bagit_archive(&temp_dir, MimeType::Zip, false)?;
    let output_dir = temp_dir.join("output");
    let result = download_bucket(
        &None,
        &[archive.to_string_lossy().to_string()],
        &["project-a".to_string()],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(result.is_ok());
    assert!(output_dir.join("project-a/index.json").is_file());
    assert!(!output_dir.join("project-b").exists(), "filtered payload must not be published");
    Ok(())
}

#[tokio::test]
async fn test_download_bagit_archive_rejects_remote_source_offline() -> Result<()> {
    let temp_dir = temp_test_dir("download_bagit_remote_offline");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let archive = write_nested_bagit_archive(&temp_dir, MimeType::Zip, false)?;
    let (address, _handle) = spawn_file_server("bag.zip", read(&archive)?);
    let output_dir = temp_dir.join("output");
    let result = download_bucket(
        &None,
        &[address],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        true,
        true,
    )
    .await;
    let why = result.unwrap_err().to_string();
    assert!(why.contains("Offline mode cannot transfer remote archive"), "unexpected error: {why}");
    assert!(!output_dir.exists(), "an offline remote archive must not publish payload");
    Ok(())
}

#[tokio::test]
async fn test_download_bagit_archive_resolves_single_enclosing_directory() -> Result<()> {
    for (format, archive_type) in [(MimeType::Zip, ArchiveType::Zip), (MimeType::Tar, ArchiveType::Tar)] {
        let temp_dir = temp_test_dir(&format!("download_bagit_enclosing_{}", format.file_type()));
        let _cleanup = TestCleanup::new(temp_dir.clone());
        let archive = write_nested_bagit_archive(&temp_dir, format.clone(), true)?;
        let output_dir = temp_dir.join("output");
        let result = download_bucket(
            &None,
            &[archive.to_string_lossy().to_string()],
            &[],
            &[],
            &Some(output_dir.clone()),
            false,
            false,
            false,
            false,
            Some(archive_type),
            &None,
            TEST_THREADS,
            &Verbosity::new(0, 1),
            false,
            true,
        )
        .await;
        assert!(result.is_ok(), "{} enclosing bag download failed", format.file_type());
        // The enclosing archive directory must not appear in the published payload.
        assert!(read_to_string(output_dir.join("project-a/index.json"))?.contains("https://example.org/project-a/index.json"));
        assert!(!output_dir.join("bag").exists());
        assert!(!output_dir.join("container").exists());
    }
    Ok(())
}

#[tokio::test]
async fn test_download_bagit_archive_round_trips_every_format() -> Result<()> {
    for (format, archive_type) in [
        (MimeType::SevenZip, ArchiveType::SevenZip),
        (MimeType::Zip, ArchiveType::Zip),
        (MimeType::Tar, ArchiveType::Tar),
        (MimeType::Gzip, ArchiveType::TarGzip),
    ] {
        let temp_dir = temp_test_dir(&format!("download_bagit_round_trip_{}", format.file_type()));
        let _cleanup = TestCleanup::new(temp_dir.clone());
        let archive = write_nested_bagit_archive(&temp_dir, format.clone(), false)?;
        let output_dir = temp_dir.join("output");
        let result = download_bucket(
            &None,
            &[archive.to_string_lossy().to_string()],
            &[],
            &[],
            &Some(output_dir.clone()),
            false,
            false,
            false,
            false,
            Some(archive_type),
            &None,
            TEST_THREADS,
            &Verbosity::new(0, 1),
            false,
            true,
        )
        .await;
        assert!(result.is_ok(), "{} archive download failed", format.file_type());
        // Project folders survive the data/ prefix removal, and tag files stay out of the output.
        assert!(read_to_string(output_dir.join("project-a/index.json"))?.contains("https://example.org/project-a/index.json"));
        assert!(read_to_string(output_dir.join("project-b/index.yaml"))?.contains("https://example.org/project-b/index.yaml"));
        assert!(!output_dir.join("bagit.txt").exists());
        assert!(!output_dir.join("manifest-sha256.txt").exists());
        assert!(!output_dir.join("data").exists());
    }
    Ok(())
}

#[tokio::test]
async fn test_download_bagit_archive_without_extension_uses_content_inference() -> Result<()> {
    let temp_dir = temp_test_dir("download_bagit_no_extension");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let archive = write_nested_bagit_archive(&temp_dir, MimeType::Zip, false)?;
    let unlabeled = temp_dir.join("unlabeled-bag");
    std::fs::copy(&archive, &unlabeled)?;
    // Content inference resolves the archive even though the name carries no extension.
    let inferred_dir = temp_dir.join("inferred");
    let inferred = download_bucket(
        &None,
        &[unlabeled.to_string_lossy().to_string()],
        &[],
        &[],
        &Some(inferred_dir.clone()),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(inferred.is_ok(), "inference should resolve an extensionless archive");
    assert!(inferred_dir.join("project-a/index.json").is_file());
    // The explicit override resolves the same archive for names that inference cannot classify.
    let explicit_dir = temp_dir.join("explicit");
    let explicit = download_bucket(
        &None,
        &[unlabeled.to_string_lossy().to_string()],
        &[],
        &[],
        &Some(explicit_dir.clone()),
        false,
        false,
        false,
        false,
        Some(ArchiveType::Zip),
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(explicit.is_ok(), "--archive-format should resolve an extensionless archive");
    assert!(explicit_dir.join("project-a/index.json").is_file());
    Ok(())
}

#[test]
fn test_download_canonicalize_rejects_model_subcommand() {
    let temp_dir = temp_test_dir("download_canonicalize_model_rejection");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let database_path = temp_dir.join("acorn.db");
    let output = Command::cargo_bin("acorn")
        .expect("locate acorn binary")
        .arg("--database-path")
        .arg(&database_path)
        .args(["download", "--canonicalize", "model", "acme/model", "--dry-run"])
        .output()
        .expect("run model download with bucket-only flag");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--canonicalize is only supported for bucket downloads"));
    assert!(!database_path.exists());
}

#[test]
fn test_download_clobber_rejects_model_subcommand() {
    let temp_dir = temp_test_dir("download_clobber_model_rejection");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let database_path = temp_dir.join("acorn.db");
    let output = Command::cargo_bin("acorn")
        .expect("locate acorn binary")
        .arg("--database-path")
        .arg(&database_path)
        .args(["download", "--clobber", "model", "acme/model", "--dry-run"])
        .output()
        .expect("run model download with bucket-only flag");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--clobber is only supported for bucket downloads"));
    assert!(!database_path.exists());
}

#[test]
fn test_download_clobber_replaces_selected_file_and_preserves_unrelated_file() {
    let temp_dir = temp_test_dir("download_clobber_bucket");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let bucket_dir = temp_dir.join("bucket");
    let output_dir = temp_dir.join("output");
    write_rad(&bucket_dir.join("index.json"), "replacement").expect("write replacement RAD");
    create_dir_all(&output_dir).expect("create output directory");
    write(output_dir.join("index.json"), "old").expect("write existing destination");
    write(output_dir.join("unrelated.txt"), "keep").expect("write unrelated destination");
    let output = Command::cargo_bin("acorn")
        .expect("locate acorn binary")
        .args(["--no-local-database", "import"])
        .arg(format!("file:{}/", bucket_dir.display()))
        .arg("--output")
        .arg(&output_dir)
        .arg("--clobber")
        .output()
        .expect("run clobbering bucket download");
    assert!(
        output.status.success(),
        "bucket download failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(read_to_string(output_dir.join("index.json"))
        .expect("read replacement")
        .contains("replacement"));
    assert_eq!(read_to_string(output_dir.join("unrelated.txt")).expect("read unrelated file"), "keep");
}

#[test]
fn test_download_flatten_rejects_model_subcommand() {
    let temp_dir = temp_test_dir("download_flatten_model_rejection");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let database_path = temp_dir.join("acorn.db");
    let output = Command::cargo_bin("acorn")
        .expect("locate acorn binary")
        .arg("--database-path")
        .arg(&database_path)
        .args(["download", "--flatten", "model", "acme/model", "--dry-run"])
        .output()
        .expect("run model download with bucket-only flag");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--flatten is only supported for bucket downloads"));
    assert!(!database_path.exists());
}

#[tokio::test]
async fn test_download_invalid_source_returns_error() {
    let result = download_bucket(
        &None,
        &["ssh://git@github.com/acme/science".to_string()],
        &[],
        &[],
        &Some(PathBuf::from("unused")),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(result.unwrap_err().to_string().contains("Invalid bucket source"));
}

#[test]
fn test_download_item_checksum_mismatch_returns_incomplete_download_error() {
    let temp_dir = temp_test_dir("download_model_checksum_mismatch");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let file_path = temp_dir.join("model.gguf.part");
    write(&file_path, b"abc").expect("write model part");
    let item = DownloadItem {
        url: "https://example.test/model.gguf".to_string(),
        path: "model.gguf".to_string(),
        size: Some(3),
        sha: Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
        headers: None,
    };
    let result = item.verify_checksum(&file_path, false);
    assert!(result.is_err());
    let text = format!("{}", result.expect_err("checksum error"));
    assert!(text.contains("Model download is incomplete"));
}

#[tokio::test]
async fn test_download_items_with_progress_are_concurrent() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind concurrent download server");
    let address = format!("http://{}", listener.local_addr().expect("concurrent download server address"));
    let active = Arc::new(AtomicUsize::new(0));
    let maximum_active = Arc::new(AtomicUsize::new(0));
    let server_active = active.clone();
    let server_maximum_active = maximum_active.clone();
    let handle = thread::spawn(move || {
        listener
            .incoming()
            .take(2)
            .map(|stream| {
                let active = server_active.clone();
                let maximum_active = server_maximum_active.clone();
                thread::spawn(move || {
                    let mut stream = stream.expect("accept model download request");
                    let mut request = [0_u8; 4096];
                    let _ = stream.read(&mut request).expect("read model download request");
                    let active_count = active.fetch_add(1, Ordering::SeqCst) + 1;
                    maximum_active.fetch_max(active_count, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(150));
                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx")
                        .expect("write model download response");
                    active.fetch_sub(1, Ordering::SeqCst);
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|handle| handle.join().expect("join model download handler"));
    });
    let destination = temp_test_dir("download_model_concurrent_progress");
    let _cleanup = TestCleanup::new(destination.clone());
    let items = ["first.gguf", "second.gguf"]
        .into_iter()
        .map(|path| DownloadItem {
            url: format!("{address}/{path}"),
            path: path.to_string(),
            size: Some(1),
            sha: None,
            headers: None,
        })
        .collect();
    let result = DownloadItems::new(&destination, items, false, false).download().await;
    assert!(result.is_ok(), "model shard downloads should succeed: {result:?}");
    handle.join().expect("join concurrent download server");
    assert_eq!(maximum_active.load(Ordering::SeqCst), 2, "visible model downloads should overlap");
}

#[tokio::test]
async fn test_download_local_bagit_archive_extracts_verified_payload() -> Result<()> {
    let temp_dir = temp_test_dir("download_bagit_archive_local");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    let archive = write_bagit_archive(&temp_dir, MimeType::Zip, "https://example.org/archive-local")?;
    let result = download_bucket(
        &None,
        &[archive.to_string_lossy().to_string()],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(result.is_ok());
    assert!(read_to_string(output_dir.join("index.json"))?.contains("https://example.org/archive-local"));
    assert!(!output_dir.join("bagit.txt").exists(), "tag files must not reach the output");
    assert!(
        !output_dir.join("data").exists(),
        "the payload directory must not be nested in the output"
    );
    Ok(())
}

#[tokio::test]
async fn test_download_local_bagit_archive_works_offline() -> Result<()> {
    let temp_dir = temp_test_dir("download_bagit_archive_offline");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    let archive = write_bagit_archive(&temp_dir, MimeType::Gzip, "https://example.org/archive-offline")?;
    let result = download_bucket(
        &None,
        &[archive.to_string_lossy().to_string()],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        true,
        true,
    )
    .await;
    assert!(result.is_ok(), "a local archive must download in offline mode");
    assert!(read_to_string(output_dir.join("index.json"))?.contains("https://example.org/archive-offline"));
    Ok(())
}

#[test]
fn test_download_model_basic_config_dry_run_output_snapshot() {
    let temp_dir = temp_test_dir("download_model_basic_config_snapshot");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join("basic.jsonc");
    let config = read_to_string(fixture_path("config/basic.jsonc"))
        .expect("read basic model config")
        .replace(
            r#""https://research.ornl.gov/api/models.json""#,
            r#"["openai/gpt-oss-20b", "openai/gpt-oss-120b"]"#,
        );
    write(&config_path, config).expect("write deterministic basic model config");
    let (endpoint, shutdown, handle) = spawn_basic_model_config_server();
    let mut command = Command::cargo_bin("acorn").expect("locate acorn binary");
    for variable in HUGGINGFACE_TOKEN_VARIABLE_NAMES {
        command.env(variable, "");
    }
    let output = command
        .env("HF_ENDPOINT", endpoint)
        .args(["--no-local-database", "-vv", "download", "model", "--config"])
        .arg(config_path)
        .arg("--dry-run")
        .output()
        .expect("run model download dry run");
    shutdown.send(()).expect("stop basic model config server");
    handle.join().expect("join basic model config server");
    assert!(
        output.status.success(),
        "dry-run basic model config failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stderr)
        .expect("UTF-8 logs")
        .replace("\u{1b}[33m WARN\u{1b}[0m", " WARN")
        .replace("\u{1b}[32m INFO\u{1b}[0m", " INFO")
        .replace('\u{1b}', r"\x1b");
    let text = text
        .lines()
        .filter(|line| {
            !line.contains("SPDX metadata")
                && !line.contains("GitLab programming language metadata")
                && !line.contains("Models.dev catalog")
                && !line.contains("Saved metadata for")
        })
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("download_model_basic_config_dry_run_output", text.trim_end());
}

#[tokio::test]
async fn test_download_model_config_resolves_models() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_config_resolve");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    write(
        &config_path,
        format!(r#"{{"models": ["{}"]}}"#, model_file.display().to_string().replace('\\', "/")),
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = run_model(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir),
        &None,
        &verbose,
        false,
        true,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "config-only model resolution should succeed: {result:?}");
    Ok(())
}

#[test]
fn test_download_model_config_skips_unresolved_models() {
    let temp_dir = temp_test_dir("download_model_config_skips_unresolved");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    let config_path = temp_dir.join(".acorn.json");
    write(&model_file, b"fake model data").expect("write local model");
    write(
        &config_path,
        format!(
            r#"{{"models": [{{"name": "tiny", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}}, "openai/gpt-oss-2b"]}}"#,
            model_file.display().to_string().replace('\\', "/")
        ),
    )
    .expect("write model config");
    let (endpoint, handle) = spawn_huggingface_model_search_server("unsloth/gpt-oss-20b-GGUF", "openai/gpt-oss-20b", None, 2);
    let result = with_hf_endpoint(endpoint, || {
        tokio::runtime::Runtime::new().expect("create runtime").block_on(run_model(
            &[],
            &[],
            &[],
            &Some(config_path),
            &Some(output_dir.clone()),
            &None,
            &Verbosity::new(0, 1),
            false,
            false,
            false,
            false,
            &[],
            false,
            20,
            false,
            false,
            false,
            &OutputFormat::default(),
        ))
    });
    handle.join().expect("join model search server");
    assert!(result.is_ok(), "resolved models should download without unresolved model failures");
    assert!(output_dir.join("tiny").join("tiny.gguf").exists());
}

#[tokio::test]
async fn test_download_model_config_whitelist_keeps_matching_name() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_config_whitelist_match");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let keep_file = temp_dir.join("keep.gguf");
    let drop_file = temp_dir.join("drop.gguf");
    write(&keep_file, b"fake keep model data")?;
    write(&drop_file, b"fake drop model data")?;
    write(
        &config_path,
        format!(
            r#"{{"models": [
            {{"name": "keep", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}},
            {{"name": "drop", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}}
        ], "whitelist": {{"models": ["keep"]}}}}"#,
            keep_file.display().to_string().replace('\\', "/"),
            drop_file.display().to_string().replace('\\', "/")
        ),
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = run_model(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "matching configuration whitelist entry should succeed");
    assert!(output_dir.join("keep").join("keep.gguf").exists());
    assert!(!output_dir.join("drop").join("drop.gguf").exists());
    Ok(())
}

#[test]
fn test_download_model_dry_run_fails_when_only_model_is_unavailable() {
    let temp_dir = temp_test_dir("download_model_only_unavailable");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let identifier = "acme/missing";
    write(&config_path, format!(r#"{{"models": ["{identifier}"]}}"#)).expect("write model config");
    let (endpoint, handle) = spawn_huggingface_model_unavailable_server(identifier);
    let result = with_hf_endpoint(endpoint, || {
        tokio::runtime::Runtime::new().expect("create runtime").block_on(run_model(
            &[],
            &[],
            &[],
            &Some(config_path),
            &Some(temp_dir.join("models")),
            &None,
            &Verbosity::new(0, 1),
            false,
            false,
            false,
            false,
            &[],
            false,
            20,
            false,
            true,
            false,
            &OutputFormat::default(),
        ))
    });
    handle.join().expect("join unavailable model server");
    let message = result.expect_err("an unavailable-only dry run should fail").to_string();
    assert_eq!(message, "No models found");
}

#[tokio::test]
async fn test_download_model_file_overrides_config() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_file_override");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let model = temp_dir.join("tiny.gguf");
    let model_file = temp_dir.join("models.txt");
    write(&model, b"fake model data")?;
    write(&model_file, model.display().to_string())?;
    write(&config_path, r#"{"models": ["configured/remote-model"]}"#)?;
    let result = execute_model(
        &[],
        &Some(model_file.display().to_string()),
        &None,
        false,
        &[],
        &[],
        &Some(config_path),
        &None,
        &None,
        &Verbosity::new(0, 1),
        true,
        false,
        false,
        false,
        &[],
        false,
        true,
        &[],
        &None,
        20,
        100,
        false,
        true,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "--model-file should prevent automatic config resolution");
    Ok(())
}

#[tokio::test]
async fn test_download_model_filter_keeps_matching_local_file() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_filter_local");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[model_file.display().to_string()],
        &["tiny\\.gguf$".to_string()],
        &[],
        &None,
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        true,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "matching filter should keep local model");
    assert!(output_dir.join("tiny").join("tiny.gguf").exists());
    Ok(())
}

#[tokio::test]
async fn test_download_model_ignore_removes_matching_local_file() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_ignore_local");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[model_file.display().to_string()],
        &[],
        &["tiny\\.gguf$".to_string()],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        false,
        true,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "ignore should remove the only selected model");
    Ok(())
}

#[tokio::test]
async fn test_download_model_invalid_filter_errors() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &["model.gguf".to_string()],
        &["[".to_string()],
        &[],
        &None,
        &None,
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "invalid filter regex should fail");
    Ok(())
}

#[tokio::test]
async fn test_download_model_local_config_copy() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_config_copy");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    let config_path = temp_dir.join(".acorn.json");
    write(&model_file, b"fake model data")?;
    write(
        &config_path,
        format!(
            r#"{{"models": [{{"name": "tiny", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}}]}}"#,
            model_file.display().to_string().replace('\\', "/")
        ),
    )?;
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "config copy mode should succeed");
    assert!(
        output_dir.join("tiny").join("tiny.gguf").exists(),
        "config copy mode should place files in output dir"
    );
    Ok(())
}

#[tokio::test]
async fn test_download_model_local_file_copy() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_copy");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[model_file.display().to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        true, // copy
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "local file copy should succeed");
    let model_dir = output_dir.join("tiny");
    assert!(model_dir.join("tiny.gguf").exists(), "copy mode should place files in output dir");
    Ok(())
}

#[tokio::test]
async fn test_download_model_local_file_reference() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_reference");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[model_file.display().to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "local file reference should succeed");
    // Reference mode: no copy into output dir
    let model_dir = output_dir.join("tiny");
    assert!(!model_dir.join("tiny.gguf").exists(), "reference mode should not copy files");
    Ok(())
}

#[tokio::test]
async fn test_download_model_local_missing_path_errors() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_missing");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &["./nonexistent/model.gguf".to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "missing local model path should return an error");
    Ok(())
}

#[tokio::test]
async fn test_download_model_local_mixed_offline_errors() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_mixed_offline");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    // Local + remote in offline mode: should fail because remote is present
    let result = run_model(
        &[model_file.display().to_string(), "some-remote-model".to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        true, // offline
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "mixed local+remote in offline mode should fail");
    Ok(())
}

#[tokio::test]
async fn test_download_model_local_offline_succeeds() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_local_offline");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    write(&model_file, b"fake model data")?;
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[model_file.display().to_string()],
        &[],
        &[],
        &None,
        &Some(output_dir),
        &None,
        &verbose,
        true, // offline — local should still work
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "local model should work in offline mode");
    Ok(())
}

#[tokio::test]
async fn test_download_model_missing_config_reports_missing_file() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_missing_config");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let missing = temp_dir.join("missing.json");
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[],
        &[],
        &[],
        &Some(missing.clone()),
        &None,
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert_eq!(
        result.unwrap_err().to_string(),
        format!("Configuration file does not exist — {}", missing.display())
    );
    Ok(())
}

#[tokio::test]
async fn test_download_model_no_models_errors() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &[],
        &[],
        &[],
        &None,
        &None,
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "empty model list should return an error");
    Ok(())
}

#[tokio::test]
async fn test_download_model_offline_rejects() -> Result<()> {
    let verbose = Verbosity::new(0, 1);
    let result = run_model(
        &["some-model".to_string()],
        &[],
        &[],
        &None,
        &None,
        &None,
        &verbose,
        true,
        false,
        false,
        false,
        &[],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "offline mode should reject remote model download");
    Ok(())
}

#[tokio::test]
async fn test_download_model_positional_overrides_config() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_positional_override");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let model_file = temp_dir.join("tiny.gguf");
    let model_path = model_file.display().to_string();
    write(&model_file, b"fake model data")?;
    write(
        &config_path,
        r#"{"models":["configured/remote-model"],"whitelist":{"models":"https://invalid.example/models.json"}}"#,
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = run_model(
        &[model_path],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir),
        &None,
        &verbose,
        true,
        true,
        false,
        false,
        &["tiny".to_string()],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "CLI whitelist and positional model should override configuration inputs");
    Ok(())
}

#[tokio::test]
async fn test_download_model_reads_model_file() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_file");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let model = temp_dir.join("tiny.gguf");
    let model_file = temp_dir.join("models.txt");
    write(&model, b"fake model data")?;
    write(&model_file, model.display().to_string())?;
    let result = execute_model(
        &[],
        &Some(model_file.display().to_string()),
        &None,
        false,
        &[],
        &[],
        &None,
        &None,
        &None,
        &Verbosity::new(0, 1),
        true,
        false,
        false,
        false,
        &[],
        false,
        true,
        &[],
        &None,
        20,
        100,
        false,
        true,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "local model from --model-file should resolve in offline mode");
    Ok(())
}

#[tokio::test]
async fn test_download_model_syncs_downloaded_model() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_sync");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("models");
    let model_file = temp_dir.join("tiny.gguf");
    let config_path = temp_dir.join("acorn.json");
    let opencode_path = temp_dir.join("opencode.json");
    write(&model_file, b"fake model data")?;
    write(
        &config_path,
        serde_json::json!({"config":{"opencode":{"path":opencode_path.display().to_string()}}}).to_string(),
    )?;
    let result = execute_model(
        &[model_file.display().to_string()],
        &None,
        &Some(SyncTarget::Opencode),
        true,
        &[],
        &[],
        &Some(config_path.clone()),
        &Some(output_dir),
        &None,
        &Verbosity::new(0, 1),
        true,
        true,
        false,
        false,
        &[],
        false,
        true,
        &[],
        &None,
        20,
        100,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok());
    assert!(read_to_string(opencode_path)?.contains("tiny"));
    let identifiers = ApplicationConfiguration::read(config_path)?
        .models
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| match entry {
            | ModelEntry::Selector(identifier) => Some(identifier),
            | ModelEntry::Entry(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(identifiers, vec!["tiny"]);
    Ok(())
}

#[tokio::test]
async fn test_download_model_whitelist_errors_when_no_name_matches() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_whitelist_no_match");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    write(
        &config_path,
        r#"{"models": [{"name": "keep", "source": {"provider": "huggingface", "location": "https://huggingface.co/hf-internal-testing/tiny-random-bert"}}]}"#,
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = run_model(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir),
        &None,
        &verbose,
        true,
        false,
        false,
        false,
        &["missing".to_string()],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_err(), "non-matching whitelist should fail");
    Ok(())
}

#[test]
fn test_download_model_whitelist_fails_when_only_model_is_unavailable() {
    let temp_dir = temp_test_dir("download_model_whitelist_only_unavailable");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let identifier = "acme/missing";
    write(&config_path, format!(r#"{{"models": ["{identifier}"]}}"#)).expect("write model config");
    let (endpoint, handle) = spawn_huggingface_model_unavailable_server(identifier);
    let result = with_hf_endpoint(endpoint, || {
        tokio::runtime::Runtime::new().expect("create runtime").block_on(run_model(
            &[],
            &[],
            &[],
            &Some(config_path),
            &Some(temp_dir.join("models")),
            &None,
            &Verbosity::new(0, 1),
            false,
            false,
            false,
            false,
            &["keep".to_string()],
            false,
            20,
            false,
            true,
            false,
            &OutputFormat::default(),
        ))
    });
    handle.join().expect("join unavailable model server");
    let message = result.expect_err("an unavailable-only batch should fail").to_string();
    assert!(message.contains("No models were found"), "unexpected error: {message}");
}

#[tokio::test]
async fn test_download_model_whitelist_file_keeps_matching_name() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_whitelist_file_match");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let whitelist_path = temp_dir.join("whitelist.yaml");
    let keep_file = temp_dir.join("keep.gguf");
    let drop_file = temp_dir.join("drop.gguf");
    write(&keep_file, b"fake keep model data")?;
    write(&drop_file, b"fake drop model data")?;
    write(&whitelist_path, "- name: keep\n")?;
    write(
        &config_path,
        format!(
            r#"{{"models": [
            {{"name": "keep", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}},
            {{"name": "drop", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}}
        ]}}"#,
            keep_file.display().to_string().replace('\\', "/"),
            drop_file.display().to_string().replace('\\', "/")
        ),
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let whitelist = super::model::Whitelist::from(Vec::new())
        .resolve(&Some(whitelist_path.display().to_string()), false)
        .await?;
    let result = run_model(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &whitelist.0,
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "matching whitelist file entry should succeed");
    assert!(output_dir.join("keep").join("keep.gguf").exists());
    assert!(!output_dir.join("drop").join("drop.gguf").exists());
    Ok(())
}

#[tokio::test]
async fn test_download_model_whitelist_keeps_matching_name() -> Result<()> {
    let temp_dir = temp_test_dir("download_model_whitelist_match");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let keep_file = temp_dir.join("keep.gguf");
    let drop_file = temp_dir.join("drop.gguf");
    write(&keep_file, b"fake keep model data")?;
    write(&drop_file, b"fake drop model data")?;
    write(
        &config_path,
        format!(
            r#"{{"models": [
            {{"name": "keep", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}},
            {{"name": "drop", "copy": true, "source": {{"provider": "git", "location": "file:{}"}}}}
        ]}}"#,
            keep_file.display().to_string().replace('\\', "/"),
            drop_file.display().to_string().replace('\\', "/")
        ),
    )?;
    let verbose = Verbosity::new(0, 1);
    let output_dir = temp_dir.join("models");
    let result = run_model(
        &[],
        &[],
        &[],
        &Some(config_path),
        &Some(output_dir.clone()),
        &None,
        &verbose,
        false,
        false,
        false,
        false,
        &["keep".to_string()],
        false,
        200,
        false,
        false,
        false,
        &OutputFormat::default(),
    )
    .await;
    assert!(result.is_ok(), "matching whitelist entry should succeed");
    assert!(output_dir.join("keep").join("keep.gguf").exists());
    assert!(!output_dir.join("drop").join("drop.gguf").exists());
    Ok(())
}

#[test]
fn test_download_model_whitelist_skips_unavailable_model_when_another_matches() {
    let temp_dir = temp_test_dir("download_model_whitelist_skips_unavailable");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let config_path = temp_dir.join(".acorn.json");
    let keep_file = temp_dir.join("keep.gguf");
    let identifier = "acme/missing";
    write(&keep_file, b"fake keep model data").expect("write local model");
    write(
        &config_path,
        format!(
            r#"{{"models": [
            {{"name": "keep", "source": {{"provider": "git", "location": "file:{}"}}}},
            "{identifier}"
        ]}}"#,
            keep_file.display().to_string().replace('\\', "/")
        ),
    )
    .expect("write model config");
    let (endpoint, handle) = spawn_huggingface_model_unavailable_server(identifier);
    let result = with_hf_endpoint(endpoint, || {
        tokio::runtime::Runtime::new().expect("create runtime").block_on(run_model(
            &[],
            &[],
            &[],
            &Some(config_path),
            &Some(temp_dir.join("models")),
            &None,
            &Verbosity::new(0, 1),
            false,
            false,
            false,
            false,
            &["keep".to_string()],
            false,
            20,
            false,
            true,
            false,
            &OutputFormat::default(),
        ))
    });
    handle.join().expect("join unavailable model server");
    assert!(result.is_ok(), "an unavailable model should not fail a batch containing a matching model");
}

#[tokio::test]
async fn test_download_rejects_local_source() {
    let result = download_bucket(
        &None,
        &["file:./tests/fixtures/data/bucket/".to_string()],
        &[],
        &[],
        &Some(PathBuf::from("unused")),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_download_remote_bagit_archive_uses_url_inference() -> Result<()> {
    let temp_dir = temp_test_dir("download_bagit_archive_remote");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    let archive = write_bagit_archive(&temp_dir, MimeType::Zip, "https://example.org/archive-remote")?;
    let (address, handle) = spawn_file_server("bag.zip", read(&archive)?);
    let result = download_bucket(
        &None,
        &[address],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        false,
        false,
        None,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    match result {
        | Ok(()) => handle.join().expect("join archive server"),
        | Err(why) => panic!("remote archive download failed: {why}"),
    }
    assert!(read_to_string(output_dir.join("index.json"))?.contains("https://example.org/archive-remote"));
    Ok(())
}

#[test]
fn test_download_strict_rejects_model_subcommand() {
    let temp_dir = temp_test_dir("download_strict_model_rejection");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let database_path = temp_dir.join("acorn.db");
    let output = Command::cargo_bin("acorn")
        .expect("locate acorn binary")
        .arg("--database-path")
        .arg(&database_path)
        .args(["download", "--strict", "model", "acme/model", "--dry-run"])
        .output()
        .expect("run model download with bucket-only flag");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--strict is only supported for bucket downloads"));
    assert!(!database_path.exists());
}

#[test]
fn test_download_task_rejects_unsafe_paths() {
    let destination = temp_test_dir("download_model_unsafe_path");
    let _cleanup = TestCleanup::new(destination.clone());
    ["", "../model.gguf", "/model.gguf", r"C:\model.gguf", r"..\model.gguf"]
        .into_iter()
        .for_each(|path| {
            let item = DownloadItem {
                url: "https://example.org/model.gguf".to_string(),
                path: path.to_string(),
                size: None,
                sha: None,
                headers: None,
            };
            assert!(
                DownloadTask::new(item, &destination, true, false).is_err(),
                "expected unsafe path: {path}"
            );
        });
}

#[tokio::test]
async fn test_download_task_restarts_partial_file_without_http_range_support() {
    let payload = b"abcdefghij".to_vec();
    let (url, handle) = spawn_no_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_no_range_restart");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
        headers: None,
    };
    let part = item.partial_path(&destination).expect("resolve partial download path");
    write(&part, b"stale").expect("seed stale partial file");
    let task = DownloadTask::new(item, &destination, true, false).expect("create download task");
    let result = task.download().await;
    assert!(result.is_ok(), "restart download should succeed: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read restarted file");
    assert_eq!(
        final_bytes, payload,
        "download should restart from byte zero without appending stale content"
    );
    handle.join().expect("join no-range server thread");
}

#[tokio::test]
async fn test_download_task_resumes_partial_file_with_http_range() {
    let payload = b"abcdefghij".to_vec();
    let (url, handle) = spawn_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_range_resume");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    match create_dir_all(&destination) {
        | Ok(()) => {}
        | Err(why) => panic!("create destination directory failed: {why}"),
    }
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
        headers: None,
    };
    let part = item.partial_path(&destination).expect("resolve partial download path");
    if let Some(parent) = part.parent() {
        match create_dir_all(parent) {
            | Ok(()) => {}
            | Err(why) => panic!("create parent directory failed: {why}"),
        }
    }
    match write(&part, &payload[..4]) {
        | Ok(()) => {}
        | Err(why) => panic!("seed partial file failed: {why}"),
    }
    let task = DownloadTask::new(item, &destination, true, false).expect("create download task");
    let result = task.download().await;
    assert!(result.is_ok(), "resume download should succeed: {result:?}");
    let final_file = destination.join("model.gguf");
    assert!(final_file.exists(), "final file should be moved into place");
    let final_bytes = match std::fs::read(final_file) {
        | Ok(bytes) => bytes,
        | Err(why) => panic!("read resumed file failed: {why}"),
    };
    assert_eq!(final_bytes, payload, "resumed download should append remaining bytes");
    handle.join().expect("join server thread");
}

#[tokio::test]
async fn test_download_task_retry_resumes_from_updated_partial_file_length() {
    let payload = b"abcdefghij".to_vec();
    let temp_dir = temp_test_dir("download_model_retry_resume_updated_offset");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let part = destination.join("model.gguf.part");
    write(&part, &payload[..4]).expect("seed partial file");
    let (url, handle) = spawn_retry_resume_server(payload.clone(), part.clone(), 4, 7);
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
        headers: None,
    };
    let task = DownloadTask::new(item, &destination, true, false).expect("create download task");
    let result = task.download().await;
    assert!(result.is_ok(), "retry should resume from updated partial length: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read retried file");
    assert_eq!(final_bytes, payload, "retry should not duplicate bytes from the original resume offset");
    handle.join().expect("join retry-resume server thread");
}

#[tokio::test]
async fn test_download_task_skip_verify_checksum_allows_checksum_mismatch() {
    let payload = b"checksum-skip-payload".to_vec();
    let (url, handle) = spawn_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_skip_checksum_mismatch");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
        headers: None,
    };
    let part = item.partial_path(&destination).expect("resolve partial download path");
    write(&part, &payload[..4]).expect("seed partial file");
    let task = DownloadTask::new(item, &destination, true, true).expect("create download task");
    let result = task.download().await;
    assert!(result.is_ok(), "skip checksum should allow mismatch: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read downloaded file");
    assert_eq!(final_bytes, payload);
    handle.join().expect("join range server thread");
}

#[tokio::test]
async fn test_download_task_succeeds_with_real_sha256_sidecar_validation() {
    let model_bytes = b"checksum-success-payload".to_vec();
    let sidecar_dir = temp_test_dir("download_model_checksum_sidecar_seed");
    let _sidecar_cleanup = TestCleanup::new(sidecar_dir.clone());
    let sidecar_file = sidecar_dir.join("model.gguf");
    match write(&sidecar_file, &model_bytes) {
        | Ok(()) => {}
        | Err(why) => panic!("write sidecar seed file failed: {why}"),
    }
    let digest = acorn_host::fs::file_checksum(&sidecar_file, None)
        .expect("checksum should be computed")
        .checksum_value;
    let sidecar_content = format!("{digest}  model.gguf\n");
    let (base_url, handle) = spawn_sidecar_server(model_bytes.clone(), sidecar_content);

    let served_sidecar = match acorn::io::http::get(format!("{base_url}/repo/resolve/main/model.gguf.sha256"))
        .send()
        .await
    {
        | Ok(response) => response.text().await.expect("read sidecar response body"),
        | Err(why) => panic!("fetch served sidecar failed: {why}"),
    };
    let served_digest = served_sidecar
        .split_whitespace()
        .find(|token| token.len() == 64 && token.chars().all(|character| character.is_ascii_hexdigit()))
        .map(|value| value.to_ascii_lowercase())
        .expect("extract digest from sidecar");

    let destination = temp_test_dir("download_model_checksum_sidecar_dest");
    let _dest_cleanup = TestCleanup::new(destination.clone());
    match create_dir_all(&destination) {
        | Ok(()) => {}
        | Err(why) => panic!("create destination directory failed: {why}"),
    }
    let item = DownloadItem {
        url: format!("{base_url}/repo/resolve/main/model.gguf"),
        path: "model.gguf".to_string(),
        size: Some(model_bytes.len() as u64),
        sha: Some(served_digest),
        headers: None,
    };
    let task = DownloadTask::new(item, &destination, true, false).expect("create download task");
    let result = task.download().await;
    assert!(result.is_ok(), "download with valid sidecar checksum should succeed: {result:?}");

    let final_file = destination.join("model.gguf");
    assert!(final_file.exists(), "final file should exist after successful checksum validation");
    let final_bytes = match std::fs::read(final_file) {
        | Ok(bytes) => bytes,
        | Err(why) => panic!("read downloaded model failed: {why}"),
    };
    assert_eq!(final_bytes, model_bytes);
    handle.join().expect("join sidecar server thread");
}

#[tokio::test]
async fn test_download_task_truncates_when_range_request_returns_200() {
    let payload = b"abcdefghij".to_vec();
    let (url, handle) = spawn_ignored_range_server(payload.clone());
    let temp_dir = temp_test_dir("download_model_ignored_range_restart");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let destination = temp_dir.join("models");
    create_dir_all(&destination).expect("create destination directory");
    let item = DownloadItem {
        url,
        path: "model.gguf".to_string(),
        size: Some(payload.len() as u64),
        sha: None,
        headers: None,
    };
    let part = item.partial_path(&destination).expect("resolve partial download path");
    write(&part, &payload[..4]).expect("seed partial file");
    let task = DownloadTask::new(item, &destination, true, false).expect("create download task");
    let result = task.download().await;
    assert!(result.is_ok(), "ignored range download should succeed: {result:?}");
    let final_bytes = std::fs::read(destination.join("model.gguf")).expect("read restarted file");
    assert_eq!(final_bytes, payload, "200 OK to ranged GET should truncate partial file before writing");
    handle.join().expect("join ignored-range server thread");
}

#[test]
fn test_download_zero_items_prints_warning() {
    let temp_dir = temp_test_dir("download_zero_items_warning");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let bucket = format!("file:{}/", fixture_path("data/bucket").display());
    let output = Command::cargo_bin("acorn")
        .expect("locate acorn binary")
        .args(["--no-local-database", "import"])
        .arg(bucket)
        .args(["--filter", "^does-not-match$"])
        .arg("--output")
        .arg(temp_dir.join("output"))
        .output()
        .expect("run zero-item bucket download");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains("CAUTION"));
    assert!(!combined.contains("SUCCESS"));
    assert!(combined.contains("Download complete"));
    assert!(combined.contains("0") && combined.contains("processed"));
    assert!(combined.contains("hint:"));
}

#[test]
fn test_downloaded_gguf_quantization_is_merged_into_weights() {
    let existing = Weights(vec![Weight {
        label: "Hugging Face".to_string(),
        url: "https://huggingface.co/openai/gpt-oss-20b".to_string(),
        is_open: Some(true),
        quantization: None,
        size: None,
    }]);
    let repository = Downloaded {
        identifier: "unsloth/gpt-oss-20b-GGUF".to_string(),
        revision: "main".to_string(),
        files: vec!["gpt-oss-20b-Q4_K_M.gguf".to_string(), "README.md".to_string()],
    };
    let weights = repository.merge_weights(existing);
    assert_eq!(weights.0.len(), 2);
    assert_eq!(
        weights.0.last().and_then(|weight| weight.quantization.as_ref()),
        Some(&Quantization::Q4kM)
    );
    assert_eq!(
        weights.0.last().map(|weight| weight.url.as_str()),
        Some("https://huggingface.co/unsloth/gpt-oss-20b-GGUF/resolve/main/gpt-oss-20b-Q4_K_M.gguf")
    );
}

#[test]
fn test_gguf_fallback_trigger_conditions() {
    let _error = color_eyre::eyre::eyre!(HuggingFaceError::NoGgufModelFiles);
    let files = repository_files(vec![candidate_file("config.json")]);
    let options = acorn::io::api::huggingface::Options::init().identifier("openai/gpt-oss-2b").build();
    assert!(files.should_use_fallback(&options));
    let offline = acorn::io::api::huggingface::Options::init()
        .identifier("openai/gpt-oss-2b")
        .offline(true)
        .build();
    assert!(!files.should_use_fallback(&offline));
    let filtered = acorn::io::api::huggingface::Options::init()
        .identifier("openai/gpt-oss-2b")
        .filter("safetensors")
        .build();
    assert!(!files.should_use_fallback(&filtered));
    let disabled = acorn::io::api::huggingface::Options::init()
        .identifier("openai/gpt-oss-2b")
        .no_fallback(true)
        .build();
    assert!(!files.should_use_fallback(&disabled));
    let invalid_identifier = acorn::io::api::huggingface::Options::init().identifier("gpt-oss-2b").build();
    assert!(!files.should_use_fallback(&invalid_identifier));
    assert!(!repository_files(vec![candidate_file("model.gguf")]).should_use_fallback(&options));
}

#[test]
fn test_hf_token_not_exposed_in_model_download_error_display() {
    let token = "test-secret-token-12345";
    with_var("HF_TOKEN", Some(token), || {
        let config = ApplicationConfiguration::parse(
            r#"{"models":[{"name":"tiny","source":{"provider":"huggingface","location":"https://huggingface.co/hf-internal-testing/tiny-random-bert"}}]}"#,
        )
        .expect("parse config");
        let plans = futures::executor::block_on(super::model::plan::resolve_plans(
            &ModelSelectors::from(vec!["tiny".to_string()]),
            &config.models.unwrap_or_default(),
            &Default::default(),
            None,
            &None,
            false,
            true,
        ))
        .expect("resolve plans");
        assert!(!plans.is_empty());
        let displays = plans
            .iter()
            .map(|plan| format!("{}", super::model::plan::ModelDownloadError::MissingAuth(plan.name())))
            .collect::<Vec<_>>();
        assert!(displays.iter().all(|line| !line.contains(token)));
        let urls = plans
            .iter()
            .map(|plan| format!("https://huggingface.co/{}/resolve/main/model.gguf", plan.selector.identifier()))
            .collect::<Vec<_>>();
        assert!(urls.iter().all(|url| !url.contains(token)));
    });
}

#[test]
fn test_hf_token_not_exposed_in_model_resolution_logs() {
    let token = "test-secret-token-12345";
    let logs = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedWriter(logs.clone());
    let subscriber = tracing_subscriber::fmt().with_ansi(false).with_writer(writer).finish();
    with_var("HF_TOKEN", Some(token), || {
        tracing::subscriber::with_default(subscriber, || {
            let _ = futures::executor::block_on(super::model::plan::resolve_plans(
                &ModelSelectors::from(vec!["hf-internal-testing/tiny-random-bert".to_string()]),
                &[],
                &Default::default(),
                None,
                &None,
                false,
                true,
            ));
        });
    });
    let text = String::from_utf8(logs.lock().expect("lock logs").clone()).unwrap_or_default();
    assert!(!text.contains(token));
}

#[test]
fn test_hf_token_not_persisted_in_activity_rows() {
    let token = "test-secret-token-12345";
    with_var("HF_TOKEN", Some(token), || {
        let temp_dir = temp_test_dir("download_model_activity_redaction");
        let _cleanup = TestCleanup::new(temp_dir.clone());
        let database_path = temp_dir.join("activity.duckdb");
        let database = Database::<Table>::from_path(Some(database_path.clone()));
        database.migrate().expect("migrate db");
        super::log_activity(
            "download model",
            temp_dir.join("models").display().to_string(),
            &Some(database_path.clone()),
            true,
        );
        let row = ActivityRow::default()
            .select(Some(database_path), |_| true)
            .expect("select activity row")
            .expect("activity row exists");
        let columns = [
            row.command.unwrap_or_default(),
            row.user_path.unwrap_or_default(),
            row.executed_at.map(|value| value.to_string()).unwrap_or_default(),
            row.success.map(|value| value.to_string()).unwrap_or_default(),
        ];
        assert!(columns.iter().all(|value| !value.contains(token)));
    });
}

#[test]
fn test_huggingface_filter_combined() {
    let files = repository_files(vec![
        candidate_file("model.Q4_K_M.gguf"),
        candidate_file("model.Q4_K_S.gguf"),
        candidate_file("model.Q2_K.gguf"),
        candidate_file("config.json"),
    ]);
    let result = files.filter(Some("\\.gguf$"), Some("Q2_")).unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|f| f.path.ends_with(".gguf") && !f.path.contains("Q2_")));
}

#[test]
fn test_huggingface_filter_extension() {
    let files = repository_files(vec![
        candidate_file("model.Q4_K_M.gguf"),
        candidate_file("model.Q4_K_S.gguf"),
        candidate_file("config.json"),
        candidate_file("tokenizer.model"),
    ]);
    let result = files.filter(Some("\\.gguf$"), None).unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|f| f.path.ends_with(".gguf")));
}

#[test]
fn test_huggingface_filter_ignore() {
    let files = repository_files(vec![
        candidate_file("model.Q4_K_M.gguf"),
        candidate_file("model.Q2_K.gguf"),
        candidate_file("model.Q3_K.gguf"),
    ]);
    let result = files.filter(None, Some("Q2_")).unwrap();
    assert_eq!(result.len(), 1);
    assert!(result.iter().all(|f| f.path != "model.Q2_K.gguf"));
    assert_eq!(result.first().map(|f| f.path.as_str()), Some("model.Q4_K_M.gguf"));
}

#[test]
fn test_huggingface_filter_works_with_complex_patterns() {
    let files = repository_files(vec![candidate_file("model.gguf")]);
    let result = files.filter(Some("[a-z]+"), None);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_import_bagit_archive_round_trips_every_format() -> Result<()> {
    for format in [MimeType::SevenZip, MimeType::Zip, MimeType::Tar, MimeType::Gzip] {
        let temp_dir = temp_test_dir(&format!("import_bagit_round_trip_{}", format.file_type()));
        let _cleanup = TestCleanup::new(temp_dir.clone());
        let archive = write_nested_bagit_archive(&temp_dir, format.clone(), false)?;
        let output_dir = temp_dir.join("output");
        let result = import_local(
            &None,
            &[archive.to_string_lossy().to_string()],
            &[],
            &[],
            &Some(output_dir.clone()),
            false,
            false,
            &None,
            TEST_THREADS,
            &Verbosity::new(0, 1),
            false,
            true,
        )
        .await;
        assert!(result.is_ok(), "{} archive import failed", format.file_type());
        assert!(read_to_string(output_dir.join("project-a/index.json"))?.contains("https://example.org/project-a/index.json"));
        assert!(!output_dir.join("bagit.txt").exists());
    }
    Ok(())
}

#[tokio::test]
async fn test_import_ingests_and_replays_valid_rad() -> Result<()> {
    let temp_dir = temp_test_dir("download_ingests_valid_rad");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let bucket_dir = temp_dir.join("bucket");
    let output_dir = temp_dir.join("output");
    let database_path = Some(temp_dir.join("acorn.db"));
    create_dir_all(&bucket_dir)?;
    write_rad(&bucket_dir.join("index.json"), "downloaded-project")?;
    let url = vec![format!("file:{}/", bucket_dir.display())];
    let verbose = Verbosity::new(0, 1);
    for index in 0..2 {
        let run_output = output_dir.join(index.to_string());
        import_local(
            &None,
            &url,
            &[],
            &[],
            &Some(run_output),
            false,
            false,
            &database_path,
            TEST_THREADS,
            &verbose,
            OFFLINE,
            false,
        )
        .await?;
    }
    let database = Database::<Table>::from_path(database_path);
    assert_eq!(database.row_count(Table::ResearchActivities)?, 1);
    assert!(output_dir.join("0/index.json").exists());
    assert!(output_dir.join("1/index.json").exists());
    Ok(())
}

#[tokio::test]
async fn test_import_ingests_multiple_rad_formats_and_honors_filter() -> Result<()> {
    let temp_dir = temp_test_dir("download_multiple_rad");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let bucket_dir = temp_dir.join("bucket");
    let output_dir = temp_dir.join("output");
    let database_path = Some(temp_dir.join("acorn.db"));
    write_rad(&bucket_dir.join("keep/index.json"), "kept-json")?;
    write_rad(&bucket_dir.join("keep/project.yaml"), "kept-yaml")?;
    write_rad(&bucket_dir.join("keep/project.md"), "kept-markdown")?;
    write(bucket_dir.join("keep/README.md"), "# Documentation\n")?;
    write_rad(&bucket_dir.join("skip/index.json"), "filtered-out")?;
    import_local(
        &None,
        &[format!("file:{}/", bucket_dir.display())],
        &["keep".to_string()],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        &database_path,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        OFFLINE,
        false,
    )
    .await?;
    let database = Database::<Table>::from_path(database_path);
    assert_eq!(database.row_count(Table::ResearchActivities)?, 3);
    assert!(output_dir.join("keep/index.json").exists());
    assert!(output_dir.join("keep/project.yaml").exists());
    assert!(output_dir.join("keep/project.md").exists());
    assert!(!output_dir.join("keep/README.md").exists());
    assert!(!output_dir.join("skip/index.json").exists());
    Ok(())
}

#[tokio::test]
async fn test_import_invalid_remote_source_returns_error() {
    let result = import_local(
        &None,
        &["http://github.com/acme/science".to_string()],
        &[],
        &[],
        &Some(PathBuf::from("unused")),
        false,
        false,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        false,
        true,
    )
    .await;
    assert!(result.unwrap_err().to_string().contains("Invalid bucket source"));
}

#[tokio::test]
async fn test_import_local_config_accepts_bare_bucket_source() -> Result<()> {
    let temp_dir = temp_test_dir("import_local_config_bare_source");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    let config_path = temp_dir.join("config.json");
    let bucket_uri = format!("file:{}/", fixture_path("data/bucket").display()).replace('\\', "/");
    write(&config_path, format!(r#"{{"buckets":["{bucket_uri}"]}}"#))?;
    let result = import_local(
        &Some(config_path),
        &[],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        &None,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        OFFLINE,
        true,
    )
    .await;
    assert!(result.is_ok(), "bare bucket source should import through configuration");
    assert!(has_any_file(&output_dir), "output should contain copied files");
    Ok(())
}

#[tokio::test]
async fn test_import_local_config_silent() -> Result<()> {
    let temp_dir = temp_test_dir("download_local_config_silent");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let config_path = temp_dir.join("config.json");
    let bucket_path = fixture_path("data/bucket");
    let bucket_uri = format!("file:{}", bucket_path.display()).replace('\\', "/");
    let config = read_to_string(fixture_path("config/with_local_buckets.json"))?.replace("file:./tests/fixtures/data/bucket/", &bucket_uri);
    write(&config_path, config)?;
    let filter: Vec<String> = vec![];
    let ignore: Vec<String> = vec![];
    let verbose = Verbosity::new(0, 1);
    let result = import_local(
        &Some(config_path),
        &[],
        &filter,
        &ignore,
        &Some(output_dir.clone()),
        false,
        false,
        &None,
        TEST_THREADS,
        &verbose,
        OFFLINE,
        true,
    )
    .await;
    assert!(result.is_ok(), "download from local config should succeed");
    assert!(has_any_file(&output_dir), "output should contain copied files");
    Ok(())
}

#[tokio::test]
async fn test_import_local_url_flattens_and_ingests_rad() -> Result<()> {
    let temp_dir = temp_test_dir("download_local_url_flatten");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let bucket_dir = temp_dir.join("bucket");
    let output_dir = temp_dir.join("output");
    let database_path = Some(temp_dir.join("acorn.db"));
    write_rad(&bucket_dir.join("nested/index.json"), "flattened-project")?;
    import_local(
        &None,
        &[format!("file:{}/", bucket_dir.display())],
        &[],
        &[],
        &Some(output_dir.clone()),
        true,
        false,
        &database_path,
        TEST_THREADS,
        &Verbosity::new(0, 1),
        OFFLINE,
        false,
    )
    .await?;
    let database = Database::<Table>::from_path(database_path);
    assert_eq!(database.row_count(Table::ResearchActivities)?, 1);
    assert!(output_dir.join("index.json").is_file());
    assert!(!output_dir.join("nested").exists());
    Ok(())
}

#[tokio::test]
async fn test_import_local_url_silent() -> Result<()> {
    let temp_dir = temp_test_dir("download_local_url_silent");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let output_dir = temp_dir.join("output");
    create_dir_all(&output_dir)?;
    let url = vec![format!("file:{}/", fixture_path("data/bucket").display())];
    let filter: Vec<String> = vec![];
    let ignore: Vec<String> = vec![];
    let verbose = Verbosity::new(0, 1);
    let result = import_local(
        &None,
        &url,
        &filter,
        &ignore,
        &Some(output_dir.clone()),
        false,
        false,
        &None,
        TEST_THREADS,
        &verbose,
        OFFLINE,
        true,
    )
    .await;
    assert!(result.is_ok(), "import from local URL should succeed");
    assert!(output_dir.exists(), "output directory should exist");
    Ok(())
}

#[tokio::test]
async fn test_import_no_local_database_disables_candidate_persistence() -> Result<()> {
    let temp_dir = temp_test_dir("download_no_candidate_persistence");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let bucket_dir = temp_dir.join("bucket");
    let database_path = temp_dir.join("acorn.db");
    write_rad(&bucket_dir.join("index.json"), "not-persisted")?;
    import_local(
        &None,
        &[format!("file:{}/", bucket_dir.display())],
        &[],
        &[],
        &Some(temp_dir.join("output")),
        false,
        false,
        &Some(database_path.clone()),
        TEST_THREADS,
        &Verbosity::new(0, 1),
        OFFLINE,
        true,
    )
    .await?;
    assert!(!database_path.exists());
    Ok(())
}

#[tokio::test]
async fn test_import_preserves_malformed_rad_and_fails_ingestion() -> Result<()> {
    let temp_dir = temp_test_dir("download_malformed_rad");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let bucket_dir = temp_dir.join("bucket");
    let output_dir = temp_dir.join("output");
    create_dir_all(&bucket_dir)?;
    write(bucket_dir.join("index.json"), "{ malformed")?;
    let result = import_local(
        &None,
        &[format!("file:{}/", bucket_dir.display())],
        &[],
        &[],
        &Some(output_dir.clone()),
        false,
        false,
        &Some(temp_dir.join("acorn.db")),
        TEST_THREADS,
        &Verbosity::new(0, 1),
        OFFLINE,
        false,
    )
    .await;
    assert!(result.is_err());
    assert!(output_dir.join("index.json").exists());
    Ok(())
}

#[test]
fn test_model_download_plan_matches_base() {
    assert!(model_plan_matches_base("openai/gpt-oss-2b"));
    assert!(!model_plan_matches_base("other/model"));
}

#[test]
fn test_model_merge_patterns_keeps_config_and_cli_patterns() {
    let merged = merge(&["Q4".to_string()], Some(&["\\.gguf$".to_string(), "Q4".to_string()]));
    assert_eq!(merged, vec!["Q4".to_string(), "\\.gguf$".to_string()]);
}

#[test]
fn test_model_plan_logs_whitelist_rejection() {
    let identifier = "mozilla/test-llama";
    let (endpoint, handle) = spawn_huggingface_model_info_server(identifier, "meta-llama/other");
    let whitelist = HashSet::from(["openai/gpt-oss-20b".to_string()]);
    let logs = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(SharedWriter(logs.clone()))
        .finish();
    let matches = with_hf_endpoint(endpoint, || {
        tracing::subscriber::with_default(subscriber, || {
            tokio::runtime::Runtime::new().expect("create runtime").block_on(async {
                let entries = vec![ModelEntry::Selector(identifier.to_string())];
                let plans = super::model::plan::resolve_plans(&ModelSelectors::default(), &entries, &Default::default(), None, &None, false, false)
                    .await
                    .expect("resolve model plan");
                plans
                    .into_iter()
                    .next()
                    .expect("model plan")
                    .matches(&whitelist, false)
                    .await
                    .expect("match model whitelist")
            })
        })
    });
    handle.join().expect("join metadata server");
    let text = String::from_utf8(logs.lock().expect("lock logs").clone()).unwrap_or_default();
    assert!(!matches);
    assert!(text.contains("REJECTED"));
    assert!(text.contains(identifier));
    assert!(text.contains("did not match whitelist"));
}

#[tokio::test]
async fn test_model_resolve_plan_matches_config_name() {
    let content = r#"{"models": [{"name": "tiny", "revision": "dev", "filter": ["\\.gguf$"], "source": {"provider": "huggingface", "location": "https://huggingface.co/acme/tiny"}}]}"#;
    let config = ApplicationConfiguration::parse(content).unwrap();
    let plans = super::model::plan::resolve_plans(
        &ModelSelectors::from(vec!["tiny".to_string()]),
        &config.models.unwrap(),
        &Default::default(),
        None,
        &None,
        false,
        false,
    )
    .await
    .unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans.first().unwrap().name(), "tiny");
    assert_eq!(plans.first().unwrap().revision, "dev");
    assert_eq!(plans.first().unwrap().filter, vec!["\\.gguf$".to_string()]);
}

#[test]
fn test_model_select_preferred_file_errors_when_ambiguous() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = repository_files(vec![candidate_file("model.Q5_K_M.gguf"), candidate_file("model.Q6_K.gguf")]);
    let selected = files.select(&policy);
    assert!(selected.is_err());
}

#[test]
fn test_model_select_preferred_file_keeps_complete_shard_set() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = repository_files(vec![
        candidate_file("Q4_K_M/model-Q4_K_M-00001-of-00002.gguf"),
        candidate_file("Q4_K_M/model-Q4_K_M-00002-of-00002.gguf"),
        candidate_file("Q5_K_M/model-Q5_K_M.gguf"),
    ]);
    let selected = files.select(&policy).unwrap();
    assert_eq!(selected.len(), 2);
    assert!(selected.iter().all(|file| file.path.contains("Q4_K_M")));
}

#[test]
fn test_model_select_preferred_file_keeps_complete_shard_set_without_preferred_marker() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = repository_files(vec![
        candidate_file("model-MXFP4-00001-of-00003.gguf"),
        candidate_file("model-MXFP4-00002-of-00003.gguf"),
        candidate_file("model-MXFP4-00003-of-00003.gguf"),
    ]);
    let selected = files.select(&policy).unwrap();
    assert_eq!(selected.len(), 3);
    assert!(selected.iter().all(|file| file.path.contains("MXFP4")));
}

#[test]
fn test_model_select_preferred_file_prefers_marker() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = repository_files(vec![candidate_file("model.Q5_K_M.gguf"), candidate_file("model.Q4_K_M.gguf")]);
    let selected = files.select(&policy).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected.first().map(|file| file.path.as_str()), Some("model.Q4_K_M.gguf"));
}

#[test]
fn test_model_select_preferred_file_prefers_repository_model_over_auxiliary_models() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = HuggingFaceRepositoryFiles::new(
        "ggml-org/gpt-oss-120b-GGUF",
        "main",
        vec![
            candidate_file("eagle3-gpt-oss-120b-BF16.gguf"),
            candidate_file("eagle3-gpt-oss-120b-Q8_0.gguf"),
            candidate_file("gpt-oss-120b-MXFP4.gguf"),
        ],
    );
    let selected = files.select(&policy).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected.first().map(|file| file.path.as_str()), Some("gpt-oss-120b-MXFP4.gguf"));
}

#[test]
fn test_model_select_preferred_file_rejects_incomplete_shard_set() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = repository_files(vec![
        candidate_file("Q4_K_M/model-Q4_K_M-00001-of-00003.gguf"),
        candidate_file("Q4_K_M/model-Q4_K_M-00002-of-00003.gguf"),
    ]);
    assert!(files.select(&policy).is_err());
}

#[test]
fn test_model_select_preferred_file_summarizes_large_ambiguity() {
    let policy = FileSelectionPolicy {
        preferred_marker: "Q4_K_M",
        no_match_message: "No GGUF model files found",
    };
    let files = repository_files(vec![
        candidate_file("model.Q2_K.gguf"),
        candidate_file("model.Q3_K_M.gguf"),
        candidate_file("model.Q5_K_M.gguf"),
    ]);
    let error = files.select(&policy).unwrap_err().to_string();
    assert!(error.contains("3 GGUF files"));
    assert!(!error.contains("model.Q2_K.gguf"));
}

#[test]
fn test_model_whitelist_precedence_is_cli_config_environment() {
    with_var(MODEL_WHITELIST, Some("environment-one, environment-two"), || {
        let configured = Some(OneOrMany::One("configuration".to_string()));
        let cli = futures::executor::block_on(super::model::Whitelist::prioritized(
            &["command-line".to_string()],
            configured.clone(),
            true,
        ))
        .unwrap();
        let config = futures::executor::block_on(super::model::Whitelist::prioritized(&[], configured, true)).unwrap();
        let environment = futures::executor::block_on(super::model::Whitelist::prioritized(&[], None, true)).unwrap();
        assert_eq!(cli.0, vec!["command-line"]);
        assert_eq!(config.0, vec!["configuration"]);
        assert_eq!(environment.0, vec!["environment-one", "environment-two"]);
    });
}

#[test]
fn test_parse_model_whitelist_file_from_json_model_detail() -> Result<()> {
    let names = super::model::Whitelist::parse(r#"{"name":"Keep","id":"acme/keep"}"#.to_string())?;
    assert_eq!(names.0, vec!["Keep".to_string(), "acme/keep".to_string()]);
    Ok(())
}

#[test]
fn test_parse_model_whitelist_file_from_json_model_details() -> Result<()> {
    let names = super::model::Whitelist::parse(r#"[{"name":"Keep","id":"acme/keep"},{"id":"drop"}]"#.to_string())?;
    assert_eq!(names.0, vec!["Keep".to_string(), "acme/keep".to_string(), "drop".to_string()]);
    Ok(())
}

#[test]
fn test_parse_model_whitelist_file_from_json_names() -> Result<()> {
    let names = super::model::Whitelist::parse(r#"["keep", "drop"]"#.to_string())?;
    assert_eq!(names.0, vec!["keep".to_string(), "drop".to_string()]);
    Ok(())
}

#[test]
fn test_parse_model_whitelist_file_from_plain_names() -> Result<()> {
    let names = super::model::Whitelist::parse(" keep \n\ndrop\n".to_string())?;
    assert_eq!(names.0, vec!["keep".to_string(), "drop".to_string()]);
    Ok(())
}

#[test]
fn test_parse_model_whitelist_file_from_yaml_model_details() -> Result<()> {
    let names = super::model::Whitelist::parse("- name: keep\n- id: drop\n".to_string())?;
    assert_eq!(names.0, vec!["keep".to_string(), "drop".to_string()]);
    Ok(())
}

#[test]
fn test_parse_model_whitelist_file_rejects_model_without_name_or_id() {
    let result = super::model::Whitelist::parse("- family: llama\n".to_string());
    assert!(result.is_err());
}

#[test]
fn test_resolve_cli_action() {
    assert_eq!(SourceAction::from_options(false, false), None);
    assert_eq!(SourceAction::from_options(true, false), Some(SourceAction::Copy));
    assert_eq!(SourceAction::from_options(false, true), Some(SourceAction::Symlink));
}

#[tokio::test]
async fn test_resolve_configured_model_whitelist_url() -> Result<()> {
    let (endpoint, handle) = spawn_huggingface_model_info_server("keep", "acme/base");
    let source = format!("{endpoint}/api/models/keep");
    let names = super::model::Whitelist::from_configuration(Some(OneOrMany::One(source)), false).await?;
    handle.join().expect("join whitelist server");
    assert_eq!(names.0, vec!["keep".to_string()]);
    Ok(())
}

#[test]
fn test_resolve_default_config_path_none_when_missing() -> Result<()> {
    let temp_dir = temp_test_dir("download_default_config_missing");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let resolved = ApplicationConfiguration::resolve_in(&None, &temp_dir);
    assert!(resolved.is_none(), "resolver should return none when no default config files exist");
    Ok(())
}

#[test]
fn test_resolve_default_config_path_prefers_standard_names() -> Result<()> {
    let temp_dir = temp_test_dir("download_default_config_resolution");
    let expected = temp_dir.join(".acorn.json");
    write(&expected, "{}")?;
    let resolved = ApplicationConfiguration::resolve_in(&None, &temp_dir);
    assert_eq!(resolved, Some(expected));
    let _cleanup = TestCleanup::new(temp_dir.clone());
    Ok(())
}

#[tokio::test]
async fn test_resolve_model_whitelist_from_file_uri() -> Result<()> {
    let temp_dir = temp_test_dir("resolve_model_whitelist_file_uri");
    let _cleanup = TestCleanup::new(temp_dir.clone());
    let whitelist_path = temp_dir.join("whitelist.txt");
    write(&whitelist_path, "keep\n")?;
    let source = Some(if cfg!(windows) {
        format!("file:///{}", whitelist_path.to_string_lossy().replace('\\', "/"))
    } else {
        format!("file://{}", whitelist_path.display())
    });
    let names = super::model::Whitelist::from(vec!["inline".to_string()]).resolve(&source, false).await?;
    assert_eq!(names.0, vec!["inline".to_string(), "keep".to_string()]);
    Ok(())
}

#[tokio::test]
async fn test_resolve_model_whitelist_rejects_remote_uri_offline() {
    let source = Some("https://example.com/models.yaml".to_string());
    let result = super::model::Whitelist::from(Vec::new()).resolve(&source, true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resolve_model_whitelist_txt_fixture() -> Result<()> {
    let source = Some(fixture_path("whitelist.txt").display().to_string());
    let whitelist = super::model::Whitelist::from(Vec::new()).resolve(&source, false).await?;
    assert_eq!(whitelist.0, vec!["keep".to_string(), "acme/drop".to_string()]);
    Ok(())
}

#[tokio::test]
async fn test_resolve_model_whitelist_yaml_fixture() -> Result<()> {
    let source = Some(fixture_path("whitelist.yaml").display().to_string());
    let whitelist = super::model::Whitelist::from(Vec::new()).resolve(&source, false).await?;
    assert_eq!(whitelist.0, vec!["keep".to_string(), "acme/keep".to_string(), "acme/drop".to_string()]);
    Ok(())
}

#[test]
fn test_resolve_models_excludes_fallback_below_minimum_download_count() {
    let base_model = "openai/gpt-oss-20b";
    let fallback = "unsloth/gpt-oss-20b-GGUF";
    let (endpoint, handle) = spawn_huggingface_model_search_server(fallback, base_model, None, 2);
    let logs = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(SharedWriter(logs.clone()))
        .finish();
    let resolved = with_hf_endpoint(endpoint, || {
        tracing::subscriber::with_default(subscriber, || {
            tokio::runtime::Runtime::new().expect("create runtime").block_on(async {
                let entries = vec![ModelEntry::Selector(base_model.to_string())];
                let plans = super::model::plan::resolve_plans(&ModelSelectors::default(), &entries, &Default::default(), None, &None, false, false)
                    .await
                    .expect("resolve model plan");
                super::model::plan::ModelDownloadPlans(plans)
                    .resolve(super::model::plan::DownloadOptions {
                        offline: false,
                        limit: 20,
                        minimum_download_count: 143,
                        interactive: false,
                    })
                    .await
            })
        })
    });
    handle.join().expect("join model search server");
    let text = String::from_utf8(logs.lock().expect("lock logs").clone()).unwrap_or_default();
    assert!(resolved.is_empty());
    assert!(text.contains("REJECTED"));
    assert!(text.contains(fallback));
    assert!(text.contains(base_model));
    assert!(text.contains("142 below minimum 143 popularity"));
    assert!(!text.contains(format!("{fallback}-secondary").as_str()));
}

#[test]
fn test_resolve_models_finds_gemma_fallback_with_object_base_models_metadata() {
    let base_model = "google/gemma-4-31B-it";
    let fallback = "unsloth/gemma-4-31B-it-GGUF";
    let (endpoint, handle) = spawn_huggingface_model_search_server(fallback, base_model, None, 3);
    let resolved = with_hf_endpoint(endpoint, || {
        tokio::runtime::Runtime::new().expect("create runtime").block_on(async {
            let entries = vec![ModelEntry::Selector(base_model.to_string())];
            let plans = super::model::plan::resolve_plans(&ModelSelectors::default(), &entries, &Default::default(), None, &None, false, false)
                .await
                .expect("resolve model plan");
            super::model::plan::ModelDownloadPlans(plans)
                .resolve(super::model::plan::DownloadOptions {
                    offline: false,
                    limit: 20,
                    minimum_download_count: 0,
                    interactive: false,
                })
                .await
        })
    });
    handle.join().expect("join model search server");
    assert_eq!(resolved.len(), 1, "Gemma should resolve through its GGUF fallback");
    let details = resolved.into_iter().next().map(ModelDetails::from).expect("resolved model details");
    assert_eq!(details.limit.map(|limit| limit.context), Some(131_072));
}
