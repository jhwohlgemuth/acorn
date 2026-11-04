use super::super::export::*;
use crate::io::export::updates::html::ReportHtmlAssets;
use crate::io::export::{updates, StandardSchema};
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use crate::io::mcp::ToolRegistry;
use acorn_core::util::MimeType;
use acorn_schema::standard::Standard;
use serde_json::json;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn render_services() -> ExportRenderServices {
    ExportRenderServices::new(
        |_, _| Ok("<html><body>fixture</body></html>".to_string()),
        |_| Ok(None),
        ReportHtmlAssets {
            color_logo: String::new(),
            white_logo: String::new(),
            leaf_logo: String::new(),
            title_banner: String::new(),
            font_faces: String::new(),
            style: String::new(),
        },
        Vec::new(),
    )
}
#[tokio::test]
async fn test_bag_export_requires_exact_authority_and_rejects_source_overlap() {
    let root = tempdir().expect("temporary BagIt root");
    let source = root.path().join("source");
    fs::create_dir(&source).expect("source directory");
    fs::write(source.join("item.txt"), "BagIt payload").expect("payload");
    let registry = ToolRegistry::default()
        .register_export_bag_capability(root.path())
        .expect("register BagIt export");
    let selected = registry
        .select_capability_binding("acorn.export::bag", None, None, CapabilitySelectionContext::default())
        .expect("select BagIt export");
    let arguments = json!({"source":"source","destination":"nested/bag.zip","format":"zip","overwrite":false});
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("nested/bag.zip").exists());
    let applied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.export::bag".to_string(),
                    binding_id: "native:acorn.export::bag".to_string(),
                    arguments,
                    destination: "nested/bag.zip".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(applied.outcome, ExecutionOutcome::Completed);
    assert_eq!(applied.artifacts.len(), 1);
    assert!(root.path().join("nested/bag.zip").is_file());
    assert_eq!(fs::read_to_string(source.join("item.txt")).expect("source unchanged"), "BagIt payload");
    for (format, destination) in [(MimeType::Tar, "bag.tar"), (MimeType::Gzip, "bag.tar.gz"), (MimeType::SevenZip, "bag.7z")] {
        let archive = ExportBagInput {
            source: PathBuf::from("source"),
            destination: PathBuf::from(destination),
            format,
            overwrite: false,
        }
        .execute(root.path())
        .expect("archive format");
        assert!(archive.destination.is_file());
    }
    assert!(ExportBagInput {
        source: PathBuf::from("."),
        destination: PathBuf::from("nested.zip"),
        format: MimeType::Zip,
        overwrite: false,
    }
    .execute(root.path())
    .is_err());
}
#[tokio::test]
async fn test_batch_export_requires_exact_authority_and_preserves_source() {
    let root = tempdir().expect("temporary export root");
    let source = root.path().join("activities.json");
    let jsonl = fs::read_to_string(crate::test::utils::fixture_path("jsonl/rad-batch.jsonl")).expect("batch fixture");
    let activities = jsonl
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("RAD entry"))
        .collect::<Vec<_>>();
    let original = serde_json::to_vec(&activities).expect("serialize batch");
    fs::write(&source, &original).expect("write source");
    let registry = ToolRegistry::default()
        .register_export_batch_capability(root.path())
        .expect("register batch export");
    let selected = registry
        .select_capability_binding("acorn.export::batch", None, None, CapabilitySelectionContext::default())
        .expect("select batch export");
    let arguments = json!({"source":"activities.json","destination":"nested/activities.jsonl","overwrite":false});
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("nested/activities.jsonl").exists());
    let written = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.export::batch".to_string(),
                    binding_id: "native:acorn.export::batch".to_string(),
                    arguments,
                    destination: "nested/activities.jsonl".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(written.outcome, ExecutionOutcome::Completed);
    assert_eq!(written.artifacts.len(), 1);
    assert_eq!(fs::read(&source).expect("source unchanged"), original);
    assert!(fs::read_to_string(root.path().join("nested/activities.jsonl"))
        .expect("written JSONL")
        .ends_with('\n'));
    let written_bytes = fs::read(root.path().join("nested/activities.jsonl")).expect("written batch bytes");
    assert!(ExportBatchInput {
        source: PathBuf::from("nested/activities.jsonl"),
        destination: PathBuf::from("nested/activities.jsonl"),
        overwrite: true,
        from: None,
        to: None,
        strict: false,
    }
    .execute(root.path())
    .is_err());
    assert_eq!(
        fs::read(root.path().join("nested/activities.jsonl")).expect("unchanged batch bytes"),
        written_bytes
    );
    assert!(ExportBatchInput {
        source: PathBuf::from("../escape.json"),
        destination: PathBuf::from("escape.jsonl"),
        overwrite: false,
        from: None,
        to: None,
        strict: false,
    }
    .execute(root.path())
    .is_err());
}
#[cfg(unix)]
#[test]
fn test_document_export_rejects_source_aliases() {
    let root = tempdir().expect("document root");
    let source = root.path().join("index.json");
    fs::copy(crate::test::utils::fixture_path("data/valid_project/index.json"), &source).expect("RAD fixture");
    let original = fs::read(&source).expect("original RAD");
    symlink(&source, root.path().join("alias.yaml")).expect("source symlink");
    fs::hard_link(&source, root.path().join("hardlink.yaml")).expect("source hard link");
    for alias in ["alias.yaml", "hardlink.yaml"] {
        assert!(
            ExportDocumentInput {
                source: PathBuf::from("index.json"),
                destination: PathBuf::from(alias),
                format: MimeType::Yaml,
                overwrite: true,
            }
            .execute(root.path())
            .is_err(),
            "accepted source alias {alias}"
        );
        assert_eq!(fs::read(&source).expect("unchanged RAD"), original);
    }
}
#[tokio::test]
async fn test_document_export_requires_authority_and_preserves_source() {
    let root = tempdir().expect("temporary document root");
    let source = root.path().join("index.json");
    fs::copy(crate::test::utils::fixture_path("data/valid_project/index.json"), &source).expect("copy RAD source");
    let original = fs::read(&source).expect("original RAD bytes");
    let registry = ToolRegistry::default()
        .register_export_document_capability(root.path())
        .expect("register document export");
    let selected = registry
        .select_capability_binding("acorn.export::document", None, None, CapabilitySelectionContext::default())
        .expect("select document export");
    let arguments = json!({"source":"index.json","destination":"nested/artifact.yaml","format":"yaml","overwrite":false});
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("nested/artifact.yaml").exists());
    let written = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.export::document".to_string(),
                    binding_id: "native:acorn.export::document".to_string(),
                    arguments,
                    destination: "nested/artifact.yaml".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(written.outcome, ExecutionOutcome::Completed);
    assert_eq!(written.artifacts.len(), 1);
    assert_eq!(fs::read(&source).expect("unchanged RAD bytes"), original);
    assert!(ExportDocumentInput {
        source: PathBuf::from("index.json"),
        destination: PathBuf::from("index.json"),
        format: MimeType::Json,
        overwrite: true,
    }
    .execute(root.path())
    .is_err());
    assert_eq!(fs::read(source).expect("unchanged source after collision"), original);
}
#[tokio::test]
async fn test_export_preview_plans_bag_pdf_and_jsonl_without_writes() {
    let root = tempdir().expect("temporary preview root");
    let bag_source = root.path().join("source");
    fs::create_dir(&bag_source).expect("source directory");
    fs::write(bag_source.join("item.txt"), "preview payload").expect("payload");
    let pdf_source = root.path().join("index.json");
    fs::copy(crate::test::utils::fixture_path("data/valid_project/index.json"), &pdf_source).expect("copy PDF source");
    let registry = ToolRegistry::default()
        .register_export_preview_capability(root.path())
        .expect("register preview");
    let selected = registry
        .select_capability_binding("acorn.export::preview", None, None, CapabilitySelectionContext::default())
        .expect("select preview");
    for (source, format) in [("source", "zip"), ("index.json", "pdf"), ("index.json", "json-lines")] {
        let result = registry
            .execute_selected(
                &selected,
                json!({"source":source,"format":format,"destination":null,"from":null,"to":null,"strict":false}),
                ExecutionContext {
                    grounded: true,
                    ..ExecutionContext::default()
                },
            )
            .await;
        assert_eq!(result.outcome, ExecutionOutcome::Completed, "{format}");
        assert!(result.artifacts.is_empty());
        let destination = result
            .content
            .expect("preview content")
            .get("destination")
            .and_then(serde_json::Value::as_str)
            .expect("destination")
            .to_string();
        assert!(!Path::new(&destination).exists(), "{format} preview wrote an artifact");
    }
}
#[cfg(all(unix, feature = "powerpoint"))]
#[tokio::test]
async fn test_powerpoint_rejects_unbounded_media_before_rendering() {
    let root = tempdir().expect("render root");
    let outside = tempdir().expect("outside media root");
    let image = outside.path().join("secret.png");
    fs::write(&image, b"outside image").expect("outside media");
    symlink(&image, root.path().join("linked.png")).expect("media symlink");
    fs::write(root.path().join("reference.pptx"), b"unused reference").expect("reference deck");
    let fixture = fs::read(crate::test::utils::fixture_path("data/valid_project/index.json")).expect("RAD fixture");
    let traversal = format!(
        "../{}/secret.png",
        outside.path().file_name().expect("outside directory name").to_string_lossy()
    );
    for url in [image.display().to_string(), traversal, "linked.png".to_string()] {
        let mut activity: serde_json::Value = serde_json::from_slice(&fixture).expect("RAD object");
        activity
            .get_mut("meta")
            .and_then(serde_json::Value::as_object_mut)
            .expect("RAD metadata")
            .insert("media".to_string(), json!({"contentUrl":url}));
        fs::write(root.path().join("index.json"), serde_json::to_vec(&activity).expect("serialize RAD")).expect("RAD source");
        let result = ExportRenderInput {
            source: PathBuf::from("index.json"),
            destination: PathBuf::from("render/unused.pptx"),
            format: MimeType::Powerpoint,
            since: None,
            reference: Some(PathBuf::from("reference.pptx")),
            chrome_path: None,
            full_timestamp: false,
            overwrite: false,
        };
        let preview = ExportPreviewInput {
            source: result.source.clone(),
            destination: Some(PathBuf::from("render")),
            format: MimeType::Powerpoint,
            from: None,
            to: None,
            strict: false,
        }
        .execute(root.path())
        .expect("render destination");
        let result = ExportRenderInput {
            destination: preview
                .destination
                .strip_prefix(root.path().canonicalize().expect("canonical root"))
                .expect("relative output")
                .to_path_buf(),
            ..result
        }
        .execute(root.path(), true, &render_services())
        .await
        .expect_err("unbounded media");
        assert!(
            result.to_string().contains("media") || result.to_string().contains("permitted root"),
            "{result}"
        );
    }
}
#[tokio::test]
async fn test_render_export_requires_exact_artifact_authority() {
    let root = tempdir().expect("render root");
    let source = root.path().join("index.json");
    fs::copy(crate::test::utils::fixture_path("report/materials-discovery/index.json"), &source).expect("copy source");
    let since = "2026-10-01T00:00:00Z";
    let projection = updates::render_updates(&source, since).expect("updates projection");
    let destination = updates::updates_artifact_path(&projection, &root.path().join("render"), "md")
        .expect("updates destination")
        .strip_prefix(root.path())
        .expect("relative destination")
        .to_path_buf();
    let registry = ToolRegistry::default()
        .register_export_render_capability(root.path(), true, render_services())
        .expect("register render");
    let selected = registry
        .select_capability_binding("acorn.export::render", None, None, CapabilitySelectionContext::default())
        .expect("select render");
    let arguments = json!({
        "source":"index.json",
        "destination":destination,
        "format":"markdown",
        "since":since,
        "reference":null,
        "chrome_path":null,
        "full_timestamp":false,
        "overwrite":false
    });
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("render").exists());
    let authorized = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.export::render".to_string(),
                    binding_id: "native:acorn.export::render".to_string(),
                    arguments,
                    destination: destination.display().to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(authorized.outcome, ExecutionOutcome::Completed, "{authorized:?}");
    assert_eq!(authorized.artifacts.len(), 1);
    assert!(root.path().join(&destination).is_file());
    assert!(ExportRenderInput {
        source: PathBuf::from("index.json"),
        destination: PathBuf::from("wrong.md"),
        format: MimeType::Markdown,
        since: Some(since.to_string()),
        reference: None,
        chrome_path: None,
        full_timestamp: false,
        overwrite: false,
    }
    .execute(root.path(), true, &render_services())
    .await
    .is_err());
}
#[tokio::test]
async fn test_schema_export_is_read_only_and_rejects_write_arguments() {
    let registry = ToolRegistry::default()
        .register_export_schema_capability()
        .expect("register schema export");
    let selected = registry
        .select_capability_binding("acorn.export::schema", None, None, CapabilitySelectionContext::default())
        .expect("select schema export");
    let result = registry
        .execute_selected(
            &selected,
            json!({"standard":"research-activity-data","format":"json"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert!(result.artifacts.is_empty());
    let content = result
        .content
        .expect("schema output")
        .get("content")
        .and_then(serde_json::Value::as_str)
        .expect("schema text")
        .to_string();
    assert!(serde_json::from_str::<serde_json::Value>(&content).is_ok());
    let direct = Standard::ResearchActivityData.schema(&MimeType::Json).expect("direct schema");
    assert_eq!(content, direct);

    let rejected = registry
        .execute_selected(
            &selected,
            json!({"standard":"research-activity-data","format":"json","destination":"schema.json"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(rejected.outcome, ExecutionOutcome::InvalidInput);
    assert!(ExportSchemaInput {
        standard: Standard::Oci,
        format: MimeType::Json,
    }
    .execute()
    .is_err());
}
#[tokio::test]
async fn test_structured_crosswalk_requires_exact_artifact_authority() {
    let root = tempdir().expect("crosswalk root");
    let source = root.path().join("datacite.json");
    let original = fs::read(crate::test::utils::fixture_path("schema/datacite.json")).expect("DataCite fixture");
    fs::write(&source, &original).expect("copy source");
    let registry = ToolRegistry::default()
        .register_export_crosswalk_capability(root.path())
        .expect("register crosswalk");
    let selected = registry
        .select_capability_binding("acorn.export::crosswalk", None, None, CapabilitySelectionContext::default())
        .expect("select crosswalk");
    let arguments = json!({
        "source":"datacite.json",
        "destination":"output/datacite.jsonld",
        "format":"ld-json",
        "from":"datacite",
        "to":"dublin-core",
        "strict":false,
        "overwrite":false
    });
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("output").exists());
    let authorized = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.export::crosswalk".to_string(),
                    binding_id: "native:acorn.export::crosswalk".to_string(),
                    arguments,
                    destination: "output/datacite.jsonld".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(authorized.outcome, ExecutionOutcome::Completed, "{authorized:?}");
    assert!(root.path().join("output/datacite.jsonld").is_file());
    assert_eq!(fs::read(source).expect("unchanged source"), original);
}
