//! Direct CLI equivalence for native export operations.
#[cfg(feature = "chart")]
use acorn::io::chart::ChartOptions;
#[cfg(feature = "pdf")]
use acorn::io::export::pdf::{initialize_page, render_html_to_pdf, PdfMargins};
#[cfg(feature = "pdf")]
use acorn::io::export::updates::html::LogbookUpdatesHtml;
use acorn::io::mcp::{
    ExportBagInput, ExportBatchInput, ExportCrosswalkInput, ExportDocumentInput, ExportPreviewInput, ExportRenderInput, ExportRenderServices,
    ExportSchemaInput, ExportUpdatesPreviewInput,
};
use acorn::io::{
    export::{powerpoint, updates},
    ArchiveCandidate, ArchiveExtraction, InputOutput,
};
use acorn_core::util::MimeType;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::Standard;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn render_services() -> ExportRenderServices {
    ExportRenderServices::new(
        |_, _| Ok("<html><body><h1>Native fact sheet</h1><img src=\"00.png\"></body></html>".to_string()),
        render_chart,
        updates::html::ReportHtmlAssets {
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
#[cfg(feature = "chart")]
fn render_chart(activity: &ResearchActivity) -> color_eyre::eyre::Result<Option<Vec<u8>>> {
    ChartOptions::init().file_type("png").build().render(activity.aspect.as_ref())
}
#[cfg(not(feature = "chart"))]
fn render_chart(_activity: &ResearchActivity) -> color_eyre::eyre::Result<Option<Vec<u8>>> {
    Ok(None)
}
#[test]
fn test_bag_and_pdf_dry_runs_match_read_only_previews() {
    let root = tempdir().expect("temporary preview root");
    let bag_source = root.path().join("source");
    fs::create_dir(&bag_source).expect("bag source");
    fs::write(bag_source.join("item.txt"), "preview payload").expect("payload");
    let bag_destination = root.path().join("bag.zip");
    let bag_cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .arg("--no-local-database")
        .arg("export")
        .arg(&bag_source)
        .args(["--format", "bag", "--dry-run", "--output"])
        .arg(&bag_destination)
        .output()
        .expect("run BagIt dry-run");
    assert!(bag_cli.status.success(), "{}", String::from_utf8_lossy(&bag_cli.stderr));
    let bag_plan = ExportPreviewInput {
        source: PathBuf::from("source"),
        format: MimeType::Zip,
        destination: Some(PathBuf::from("bag.zip")),
        from: None,
        to: None,
        strict: false,
    }
    .execute(root.path())
    .expect("BagIt preview");
    assert_eq!(bag_plan.destination, root.path().canonicalize().expect("canonical root").join("bag.zip"));
    assert!(!bag_destination.exists());

    #[cfg(feature = "pdf")]
    {
        let pdf_source = root.path().join("index.json");
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/valid_project/index.json");
        fs::copy(fixture, &pdf_source).expect("copy PDF source");
        let output_dir = root.path().join("pdf");
        let pdf_cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .arg("--no-local-database")
            .arg("export")
            .arg(&pdf_source)
            .args(["--format", "pdf", "--dry-run", "--output"])
            .arg(&output_dir)
            .output()
            .expect("run PDF dry-run");
        assert!(pdf_cli.status.success(), "{}", String::from_utf8_lossy(&pdf_cli.stderr));
        let pdf_plan = ExportPreviewInput {
            source: PathBuf::from("index.json"),
            format: MimeType::Pdf,
            destination: Some(PathBuf::from("pdf")),
            from: None,
            to: None,
            strict: false,
        }
        .execute(root.path())
        .expect("PDF preview");
        assert!(pdf_plan
            .destination
            .starts_with(root.path().canonicalize().expect("canonical root").join("pdf")));
        assert!(!pdf_plan.destination.exists());
        assert!(!output_dir.exists());
    }
}
#[test]
fn test_bag_archive_matches_direct_command_bytes() {
    let root = tempdir().expect("temporary BagIt root");
    let source = root.path().join("source");
    fs::create_dir(&source).expect("source directory");
    fs::write(source.join("item.txt"), "BagIt payload").expect("payload");
    let direct_destination = root.path().join("direct.zip");
    let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .arg("--no-local-database")
        .arg("export")
        .arg(&source)
        .args(["--format", "bag", "--archive-format", "zip", "--output"])
        .arg(&direct_destination)
        .output()
        .expect("run direct BagIt export");
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    let routed = ExportBagInput {
        source: PathBuf::from("source"),
        destination: PathBuf::from("native.zip"),
        format: MimeType::Zip,
        overwrite: false,
    }
    .execute(root.path())
    .expect("routed BagIt export");
    assert_eq!(
        fs::read(direct_destination).expect("direct bag"),
        fs::read(routed.destination).expect("routed bag")
    );
}
#[cfg(feature = "pdf")]
#[tokio::test]
async fn test_direct_and_shared_pdf_renderers_create_valid_artifacts() {
    let chrome = std::env::var_os("CHROME_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"));
    if chrome.is_file() {
        let root = tempdir().expect("PDF render root");
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/valid_project/index.json");
        let before = fs::read(&source).expect("source bytes");
        let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args(["--format", "pdf", "--chrome-path"])
            .arg(&chrome)
            .arg("--output")
            .arg(root.path().join("direct"))
            .output()
            .expect("direct PDF command");
        assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
        let direct_pdf = fs::read(root.path().join("direct/valid-project.pdf")).expect("direct PDF");
        let shared_pdf_path = root.path().join("shared.pdf");
        let session = initialize_page(true, Some(&chrome)).await.expect("shared Chromium session");
        let shared = render_html_to_pdf(
            session.page(),
            "<html><body><h1>Shared PDF renderer</h1></body></html>",
            &shared_pdf_path,
            PdfMargins::FullBleed,
        )
        .await;
        session.close().await;
        shared.expect("shared PDF render");
        let shared_pdf = fs::read(&shared_pdf_path).expect("shared PDF");
        assert!(direct_pdf.starts_with(b"%PDF-"));
        assert!(shared_pdf.starts_with(b"%PDF-"));
        assert!(direct_pdf.len() > 1_000);
        assert!(shared_pdf.len() > 1_000);
        fs::copy(&source, root.path().join("index.json")).expect("copy native source");
        for image in ["00.png", "01.png"] {
            fs::copy(source.with_file_name(image), root.path().join(image)).expect("copy sibling image");
        }
        let native = ExportRenderInput {
            source: PathBuf::from("index.json"),
            destination: PathBuf::from("native/valid-project.pdf"),
            format: MimeType::Pdf,
            since: None,
            reference: None,
            chrome_path: Some(chrome.clone()),
            full_timestamp: false,
            overwrite: false,
        }
        .execute(root.path(), true, &render_services())
        .await
        .expect("native PDF render");
        assert!(fs::read(&native.destination).expect("native PDF").starts_with(b"%PDF-"));
        let images = Command::new("pdfimages")
            .arg("-list")
            .arg(&native.destination)
            .output()
            .expect("inspect PDF images");
        assert!(images.status.success(), "{}", String::from_utf8_lossy(&images.stderr));
        assert!(
            String::from_utf8_lossy(&images.stdout)
                .lines()
                .any(|line| line.split_whitespace().nth(2) == Some("image")),
            "sibling image missing from PDF"
        );
        assert_eq!(fs::read(&source).expect("unchanged source"), before);
        assert!(initialize_page(true, Some(&root.path().join("missing-chrome"))).await.is_err());
    }
}
#[test]
fn test_direct_and_shared_powerpoint_parts_match() {
    let root = tempdir().expect("PowerPoint comparison root");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/highlight");
    let source = fixtures.join("hecate/index.json");
    let reference = fixtures.join("reference.pptx");
    let before = fs::read(&source).expect("source bytes");
    let direct_dir = root.path().join("direct");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "export"])
        .arg(&source)
        .args(["--format", "powerpoint", "--reference"])
        .arg(&reference)
        .arg("--output")
        .arg(&direct_dir)
        .output()
        .expect("direct PowerPoint command");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    let data = ResearchActivity::read(&source).expect("research activity");
    let chart = render_chart(&data).expect("ASPECT chart");
    let shared_path = powerpoint::render(
        vec![source.clone()],
        &source,
        Some(&reference),
        &root.path().join("shared"),
        core::slice::from_ref(&chart),
        &[],
    )
    .expect("shared PowerPoint renderer");
    let direct_path = fs::read_dir(&direct_dir)
        .expect("direct output")
        .map(|entry| entry.expect("direct output entry").path())
        .find(|path| path.extension().is_some_and(|extension| extension == "pptx"))
        .expect("direct PPTX");
    let direct_parts = ArchiveCandidate::from(direct_path).extract_zip(None).expect("direct parts");
    let shared_parts = ArchiveCandidate::from(shared_path).extract_zip(None).expect("shared parts");
    for part in [
        "ppt/presentation.xml",
        "ppt/slides/slide1.xml",
        "ppt/slides/_rels/slide1.xml.rels",
        "ppt/notesSlides/notesSlide1.xml",
    ] {
        assert_eq!(
            fs::read(direct_parts.join(part)).expect("direct part"),
            fs::read(shared_parts.join(part)).expect("shared part"),
            "{part}"
        );
    }
    let second_dir = root.path().join("second");
    fs::create_dir(&second_dir).expect("second project directory");
    let second_source = second_dir.join("index.json");
    let mut second_data: serde_json::Value = serde_json::from_slice(&before).expect("second project data");
    *second_data
        .get_mut("meta")
        .and_then(|value| value.get_mut("identifier"))
        .expect("second project identifier") = serde_json::Value::String("hecate-two".to_string());
    fs::write(&second_source, serde_json::to_vec(&second_data).expect("second project JSON")).expect("second project source");
    fs::copy(fixtures.join("hecate/00.png"), second_dir.join("00.png")).expect("second project image");
    let combined_path = powerpoint::render(
        vec![source.clone(), second_source],
        &fixtures,
        Some(&reference),
        &root.path().join("combined"),
        &[chart.clone(), chart],
        &[],
    )
    .expect("shared combined PowerPoint renderer");
    let combined_parts = ArchiveCandidate::from(combined_path).extract_zip(None).expect("combined parts");
    assert!(combined_parts.join("ppt/slides/slide2.xml").is_file());
    assert!(combined_parts.join("ppt/notesSlides/notesSlide2.xml").is_file());
    assert_eq!(fs::read(&source).expect("unchanged source"), before);
}
#[test]
fn test_direct_and_shared_updates_documents_match() {
    let root = tempdir().expect("updates comparison root");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/report/materials-discovery/index.json");
    let before = fs::read(&source).expect("updates source bytes");
    let since = "2026-10-01T00:00:00Z";
    let projection = updates::render_updates(&source, since).expect("shared updates projection");
    for (format, extension) in [("markdown", "md"), ("docx", "docx")] {
        let direct_dir = root.path().join(format!("direct-{extension}"));
        let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args(["--target", "updates", "--since", since, "--format", format, "--output"])
            .arg(&direct_dir)
            .output()
            .expect("direct updates command");
        assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
        let shared_dir = root.path().join(format!("shared-{extension}"));
        fs::create_dir(&shared_dir).expect("shared updates directory");
        let shared_path = updates::updates_artifact_path(&projection, &shared_dir, extension).expect("shared updates destination");
        if extension == "md" {
            updates::write_markdown_updates(&projection, &shared_path).expect("shared Markdown");
        } else {
            updates::write_docx_updates(&projection, &shared_path).expect("shared DOCX");
        }
        let direct_path = updates::updates_artifact_path(&projection, &direct_dir, extension).expect("direct updates destination");
        assert_eq!(
            fs::read(direct_path).expect("direct updates artifact"),
            fs::read(shared_path).expect("shared updates artifact")
        );
    }
    let no_logbook = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/valid_project/index.json");
    let empty = updates::render_updates(&no_logbook, since).expect("missing logbook projection");
    assert!(empty.entries.is_empty());
    let empty_dir = root.path().join("empty-direct");
    let direct_empty = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "export"])
        .arg(&no_logbook)
        .args(["--target", "updates", "--since", since, "--format", "markdown", "--output"])
        .arg(&empty_dir)
        .output()
        .expect("direct empty updates command");
    assert!(direct_empty.status.success());
    let direct_empty_path = updates::updates_artifact_path(&empty, &empty_dir, "md").expect("empty artifact path");
    let shared_empty_path = root.path().join("shared-empty.md");
    updates::write_markdown_updates(&empty, &shared_empty_path).expect("shared empty updates");
    assert_eq!(
        fs::read(direct_empty_path).expect("direct empty updates"),
        fs::read(shared_empty_path).expect("shared empty updates")
    );
    assert!(updates::render_updates(&source, "invalid").is_err());
    let invalid_dir = root.path().join("invalid");
    let direct_invalid = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "export"])
        .arg(&source)
        .args(["--target", "updates", "--since", "invalid", "--format", "markdown", "--output"])
        .arg(&invalid_dir)
        .output()
        .expect("direct invalid updates command");
    assert!(!direct_invalid.status.success());
    assert!(!invalid_dir.join("materials-discovery-updates-invalid.md").exists());
    assert_eq!(fs::read(&source).expect("unchanged updates source"), before);
}
#[cfg(feature = "pdf")]
#[tokio::test]
async fn test_direct_and_shared_updates_pdf_render() {
    let chrome = std::env::var_os("CHROME_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"));
    if chrome.is_file() {
        let root = tempdir().expect("updates PDF root");
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/report/materials-discovery/index.json");
        let before = fs::read(&source).expect("updates source bytes");
        let since = "2026-10-01T00:00:00Z";
        let direct_dir = root.path().join("direct");
        let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args(["--target", "updates", "--since", since, "--format", "pdf", "--chrome-path"])
            .arg(&chrome)
            .arg("--output")
            .arg(&direct_dir)
            .output()
            .expect("direct updates PDF command");
        assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
        let projection = updates::render_updates(&source, since).expect("shared updates projection");
        let direct_path = updates::updates_artifact_path(&projection, &direct_dir, "pdf").expect("direct PDF path");
        let shared_path = root.path().join("shared.pdf");
        let assets = updates::html::ReportHtmlAssets {
            color_logo: String::new(),
            white_logo: String::new(),
            leaf_logo: String::new(),
            title_banner: String::new(),
            font_faces: String::new(),
            style: String::new(),
        };
        let html = projection.to_html(false, &assets);
        let session = initialize_page(true, Some(&chrome)).await.expect("shared updates Chromium session");
        let rendered = render_html_to_pdf(session.page(), &html, &shared_path, PdfMargins::Stylesheet).await;
        session.close().await;
        rendered.expect("shared updates PDF");
        assert!(fs::read(direct_path).expect("direct updates PDF").starts_with(b"%PDF-"));
        assert!(fs::read(shared_path).expect("shared updates PDF").starts_with(b"%PDF-"));
        fs::copy(&source, root.path().join("index.json")).expect("copy native updates source");
        let native_dir = root.path().join("native");
        let native_destination = updates::updates_artifact_path(&projection, &native_dir, "pdf").expect("native updates PDF path");
        let native = ExportRenderInput {
            source: PathBuf::from("index.json"),
            destination: native_destination
                .strip_prefix(root.path())
                .expect("relative updates destination")
                .to_path_buf(),
            format: MimeType::Pdf,
            since: Some(since.to_string()),
            reference: None,
            chrome_path: Some(chrome.clone()),
            full_timestamp: false,
            overwrite: false,
        }
        .execute(root.path(), true, &render_services())
        .await
        .expect("native updates PDF");
        assert!(fs::read(native.destination).expect("native updates artifact").starts_with(b"%PDF-"));
        assert_eq!(fs::read(source).expect("unchanged source"), before);
    }
}
#[test]
fn test_jsonl_batch_matches_direct_command_bytes() {
    let root = tempdir().expect("temporary export root");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/jsonl/rad-batch.jsonl");
    let jsonl = fs::read_to_string(fixture).expect("batch fixture");
    let activities = jsonl
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("RAD entry"))
        .collect::<Vec<_>>();
    let source = root.path().join("activities.json");
    let original = serde_json::to_vec(&activities).expect("serialize source");
    fs::write(&source, &original).expect("write source");
    let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .arg("--no-local-database")
        .arg("export")
        .arg(&source)
        .args(["--format", "jsonl"])
        .output()
        .expect("run direct JSONL export");
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    let routed = ExportBatchInput {
        source: PathBuf::from("activities.json"),
        destination: PathBuf::from("native.jsonl"),
        overwrite: false,
        from: None,
        to: None,
        strict: false,
    }
    .execute(root.path())
    .expect("routed JSONL export");
    assert_eq!(routed.artifacts.len(), 1);
    assert_eq!(
        fs::read(root.path().join("activities.jsonl")).expect("direct artifact"),
        fs::read(routed.destination).expect("routed artifact")
    );
    assert_eq!(fs::read(source).expect("unchanged source"), original);
}
#[test]
fn test_jsonl_crosswalk_dry_run_matches_read_only_preview() {
    let root = tempdir().expect("temporary crosswalk preview root");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/jsonl/datacite-batch.ndjson");
    let source = root.path().join("records.ndjson");
    fs::copy(fixture, &source).expect("copy crosswalk source");
    let output_dir = root.path().join("preview");
    fs::create_dir(&output_dir).expect("preview output directory");
    let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .arg("--no-local-database")
        .arg("export")
        .arg(&source)
        .args(["--format", "jsonl", "--from", "datacite", "--to", "dcat", "--dry-run", "--output"])
        .arg(&output_dir)
        .output()
        .expect("run direct crosswalk preview");
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    let plan = ExportPreviewInput {
        source: PathBuf::from("records.ndjson"),
        format: MimeType::JsonLines,
        destination: Some(PathBuf::from("preview/records.jsonl")),
        from: Some(Standard::Datacite),
        to: Some(Standard::Dcat),
        strict: false,
    }
    .execute(root.path())
    .expect("routed crosswalk preview");
    assert_eq!(
        plan.destination,
        root.path().canonicalize().expect("canonical root").join("preview/records.jsonl")
    );
    assert!(!plan.destination.exists());
}
#[test]
fn test_jsonl_crosswalk_matches_direct_command_bytes() {
    let root = tempdir().expect("temporary crosswalk root");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/jsonl/datacite-batch.ndjson");
    let source = root.path().join("records.ndjson");
    fs::copy(fixture, &source).expect("copy source batch");
    let direct_output = root.path().join("direct");
    fs::create_dir(&direct_output).expect("direct output directory");
    let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .arg("--no-local-database")
        .arg("export")
        .arg(&source)
        .args(["--format", "jsonl", "--from", "datacite", "--to", "dcat", "--output"])
        .arg(&direct_output)
        .output()
        .expect("run direct crosswalk export");
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    let routed = ExportBatchInput {
        source: PathBuf::from("records.ndjson"),
        destination: PathBuf::from("native.jsonl"),
        overwrite: false,
        from: Some(Standard::Datacite),
        to: Some(Standard::Dcat),
        strict: false,
    }
    .execute(root.path())
    .expect("routed crosswalk export");
    assert_eq!(
        fs::read(direct_output.join("records.jsonl")).expect("direct crosswalk"),
        fs::read(routed.destination).expect("routed crosswalk")
    );
}
#[tokio::test]
async fn test_native_powerpoint_matches_direct_cli_parts() {
    let root = tempdir().expect("native PowerPoint root");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/highlight");
    let project = root.path().join("hecate");
    fs::create_dir(&project).expect("project directory");
    let source = project.join("index.json");
    fs::copy(fixtures.join("hecate/index.json"), &source).expect("copy source");
    fs::copy(fixtures.join("hecate/00.png"), project.join("00.png")).expect("copy image");
    let reference = root.path().join("reference.pptx");
    fs::copy(fixtures.join("reference.pptx"), &reference).expect("copy reference");
    let direct_dir = root.path().join("direct");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "export"])
        .arg(&source)
        .args(["--format", "powerpoint", "--reference"])
        .arg(&reference)
        .arg("--output")
        .arg(&direct_dir)
        .output()
        .expect("direct PowerPoint export");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    let preview = ExportPreviewInput {
        source: PathBuf::from("hecate/index.json"),
        format: MimeType::Powerpoint,
        destination: Some(PathBuf::from("native")),
        from: None,
        to: None,
        strict: false,
    }
    .execute(root.path())
    .expect("native PowerPoint preview");
    let native = ExportRenderInput {
        source: PathBuf::from("hecate/index.json"),
        destination: preview
            .destination
            .strip_prefix(root.path().canonicalize().expect("canonical root"))
            .expect("relative destination")
            .to_path_buf(),
        format: MimeType::Powerpoint,
        since: None,
        reference: Some(PathBuf::from("reference.pptx")),
        chrome_path: None,
        full_timestamp: false,
        overwrite: false,
    }
    .execute(root.path(), true, &render_services())
    .await
    .expect("native PowerPoint render");
    let direct_path = fs::read_dir(&direct_dir)
        .expect("direct output directory")
        .map(|entry| entry.expect("direct entry").path())
        .find(|path| path.extension().is_some_and(|extension| extension == "pptx"))
        .expect("direct artifact");
    let direct_parts = ArchiveCandidate::from(direct_path).extract_zip(None).expect("direct parts");
    let native_parts = ArchiveCandidate::from(native.destination).extract_zip(None).expect("native parts");
    for part in ["ppt/presentation.xml", "ppt/slides/slide1.xml", "ppt/slides/_rels/slide1.xml.rels"] {
        assert_eq!(
            fs::read(direct_parts.join(part)).expect("direct part"),
            fs::read(native_parts.join(part)).expect("native part"),
            "{part}"
        );
    }
}
#[tokio::test]
async fn test_native_updates_documents_match_direct_cli_bytes() {
    let root = tempdir().expect("updates render root");
    let source = root.path().join("index.json");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/report/materials-discovery/index.json");
    let before = fs::read(&fixture).expect("fixture bytes");
    fs::write(&source, &before).expect("copy source");
    let since = "2026-10-01T00:00:00Z";
    let projection = updates::render_updates(&source, since).expect("updates projection");
    for (format, native_format, extension) in [("markdown", MimeType::Markdown, "md"), ("docx", MimeType::Docx, "docx")] {
        let direct_dir = root.path().join(format!("direct-{extension}"));
        let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args(["--target", "updates", "--since", since, "--format", format, "--output"])
            .arg(&direct_dir)
            .output()
            .expect("direct updates export");
        assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
        let native_dir = root.path().join(format!("native-{extension}"));
        let destination = updates::updates_artifact_path(&projection, &native_dir, extension).expect("native destination");
        let relative = destination.strip_prefix(root.path()).expect("relative destination").to_path_buf();
        let native = ExportRenderInput {
            source: PathBuf::from("index.json"),
            destination: relative,
            format: native_format,
            since: Some(since.to_string()),
            reference: None,
            chrome_path: None,
            full_timestamp: false,
            overwrite: false,
        }
        .execute(root.path(), true, &render_services())
        .await
        .expect("native updates export");
        let direct_path = updates::updates_artifact_path(&projection, &direct_dir, extension).expect("direct destination");
        assert_eq!(
            fs::read(direct_path).expect("direct artifact"),
            fs::read(native.destination).expect("native artifact")
        );
    }
    assert_eq!(fs::read(source).expect("unchanged source"), before);
}
#[test]
fn test_plain_jsonl_dry_run_keeps_source_and_destination_unchanged() {
    let root = tempdir().expect("temporary plain preview root");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/jsonl/rad-batch.jsonl");
    let source = root.path().join("records.json");
    let records = fs::read_to_string(fixture).expect("batch fixture");
    let values = records
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("RAD entry"))
        .collect::<Vec<_>>();
    let original = serde_json::to_vec(&values).expect("serialize source");
    fs::write(&source, &original).expect("write source");
    let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .arg("--no-local-database")
        .arg("export")
        .arg(&source)
        .args(["--format", "jsonl", "--dry-run"])
        .output()
        .expect("run direct dry-run");
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    let plan = ExportPreviewInput {
        source: PathBuf::from("records.json"),
        format: MimeType::JsonLines,
        destination: None,
        from: None,
        to: None,
        strict: false,
    }
    .execute(root.path())
    .expect("routed preview");
    assert_eq!(
        plan.destination,
        root.path().canonicalize().expect("canonical root").join("records.jsonl")
    );
    assert_eq!(fs::read(source).expect("unchanged source"), original);
    assert!(!plan.destination.exists());
}
#[test]
fn test_powerpoint_and_updates_dry_runs_match_read_only_previews() {
    let root = tempdir().expect("preview root");
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    let source = root.path().join("index.json");
    fs::copy(fixture_root.join("report/materials-discovery/index.json"), &source).expect("copy report source");
    let powerpoint_output = root.path().join("powerpoint");
    let powerpoint = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "export"])
        .arg(&source)
        .args(["--format", "powerpoint", "--dry-run", "--output"])
        .arg(&powerpoint_output)
        .output()
        .expect("PowerPoint dry-run");
    assert!(powerpoint.status.success(), "{}", String::from_utf8_lossy(&powerpoint.stderr));
    let powerpoint_plan = ExportPreviewInput {
        source: PathBuf::from("index.json"),
        format: MimeType::Powerpoint,
        destination: Some(PathBuf::from("powerpoint")),
        from: None,
        to: None,
        strict: false,
    }
    .execute(root.path())
    .expect("PowerPoint preview");
    assert_eq!(powerpoint_plan.destination.extension().and_then(|value| value.to_str()), Some("pptx"));
    assert!(!powerpoint_output.exists());
    assert!(!powerpoint_plan.destination.exists());

    let since = "2026-10-01T00:00:00Z";
    for (format, preview_format) in [("markdown", MimeType::Markdown), ("docx", MimeType::Docx), ("pdf", MimeType::Pdf)] {
        let output = root.path().join(format!("updates-{format}"));
        let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args(["--target", "updates", "--since", since, "--format", format, "--dry-run", "--output"])
            .arg(&output)
            .output()
            .expect("updates dry-run");
        assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
        let plan = ExportUpdatesPreviewInput {
            source: PathBuf::from("index.json"),
            since: since.to_string(),
            destination: PathBuf::from(format!("updates-{format}")),
            format: preview_format,
        }
        .execute(root.path())
        .expect("updates preview");
        assert!(plan
            .destination
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.starts_with("materials-discovery-updates-")));
        assert!(!output.exists());
        assert!(!plan.destination.exists());
    }
}
#[test]
fn test_schema_export_matches_direct_command_without_artifacts() {
    for (format, routed_format) in [("json", MimeType::Json), ("yaml", MimeType::Yaml)] {
        let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .arg("--no-local-database")
            .args(["export", "--schema", "--standard", "rads", "--format", format])
            .output()
            .expect("run direct schema export");
        assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
        let routed = ExportSchemaInput {
            standard: Standard::ResearchActivityData,
            format: routed_format,
        }
        .execute()
        .expect("routed schema");
        assert_eq!(routed.content.trim_end(), String::from_utf8_lossy(&cli.stdout).trim_end());
    }
}
#[test]
fn test_structured_graph_crosswalk_matches_direct_cli_bytes() {
    let root = tempdir().expect("crosswalk root");
    let source = root.path().join("datacite.json");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/schema/datacite.json");
    let original = fs::read(fixture).expect("DataCite fixture");
    fs::write(&source, &original).expect("copy source");
    for (cli_format, native_format, preview_format, extension) in [
        ("json-ld", MimeType::LdJson, MimeType::LdJson, "jsonld"),
        ("rdf-xml", MimeType::RdfXml, MimeType::RdfXml, "rdf"),
        ("turtle", MimeType::Turtle, MimeType::Turtle, "ttl"),
    ] {
        let preview_dir = root.path().join(format!("preview-{extension}"));
        let preview_cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args([
                "--from",
                "datacite",
                "--to",
                "dublin-core",
                "--format",
                cli_format,
                "--dry-run",
                "--output",
            ])
            .arg(&preview_dir)
            .output()
            .expect("direct graph preview");
        assert!(
            preview_cli.status.success(),
            "{cli_format}: {}",
            String::from_utf8_lossy(&preview_cli.stderr)
        );
        let preview = ExportPreviewInput {
            source: PathBuf::from("datacite.json"),
            format: preview_format,
            destination: Some(PathBuf::from(format!("preview-{extension}/datacite.{extension}"))),
            from: Some(Standard::Datacite),
            to: Some(Standard::DublinCore),
            strict: false,
        }
        .execute(root.path())
        .expect("native graph preview");
        assert!(!preview.destination.exists());
        assert!(!preview_dir.exists());
        let direct_dir = root.path().join(format!("direct-{extension}"));
        fs::create_dir(&direct_dir).expect("direct output directory");
        let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args(["--from", "datacite", "--to", "dublin-core", "--format", cli_format, "--output"])
            .arg(&direct_dir)
            .output()
            .expect("direct graph export");
        assert!(direct.status.success(), "{cli_format}: {}", String::from_utf8_lossy(&direct.stderr));
        let relative = PathBuf::from(format!("native-{extension}/datacite.{extension}"));
        fs::create_dir(root.path().join(format!("native-{extension}"))).expect("native output directory");
        let native = ExportCrosswalkInput {
            source: PathBuf::from("datacite.json"),
            destination: relative,
            format: native_format,
            from: Standard::Datacite,
            to: Standard::DublinCore,
            strict: false,
            overwrite: false,
        }
        .execute(root.path())
        .expect("native graph export");
        assert_eq!(
            fs::read(direct_dir.join(format!("datacite.{extension}"))).expect("direct graph"),
            fs::read(native.destination).expect("native graph"),
            "{cli_format}"
        );
    }
    assert_eq!(fs::read(source).expect("unchanged source"), original);
}
#[test]
fn test_text_document_dry_runs_match_read_only_previews() {
    let root = tempdir().expect("text preview root");
    let source = root.path().join("index.json");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/valid_project/index.json");
    let original = fs::read(fixture).expect("RAD fixture");
    fs::write(&source, &original).expect("copy source");
    for (cli_format, preview_format, filename) in [
        ("cff", MimeType::Cff, "CITATION.cff"),
        ("markdown", MimeType::Markdown, "index.md"),
        ("yaml", MimeType::Yaml, "index.yaml"),
        ("zon", MimeType::Zon, "index.zonf"),
    ] {
        let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .args(["--offline", "--no-local-database", "export"])
            .arg(&source)
            .args(["--format", cli_format, "--dry-run"])
            .output()
            .expect("text dry-run");
        assert!(direct.status.success(), "{cli_format}: {}", String::from_utf8_lossy(&direct.stderr));
        let preview = ExportPreviewInput {
            source: PathBuf::from("index.json"),
            format: preview_format,
            destination: None,
            from: None,
            to: None,
            strict: false,
        }
        .execute(root.path())
        .expect("native text preview");
        assert_eq!(preview.destination.file_name().and_then(|value| value.to_str()), Some(filename));
        assert!(!preview.destination.exists());
    }
    assert_eq!(fs::read(source).expect("unchanged source"), original);
}
#[test]
fn test_text_document_exports_match_direct_command_bytes() {
    for (format, routed_format, direct_name) in [
        ("cff", MimeType::Cff, "CITATION.cff"),
        ("markdown", MimeType::Markdown, "index.md"),
        ("yaml", MimeType::Yaml, "index.yaml"),
        ("zon", MimeType::Zon, "index.zonf"),
    ] {
        let root = tempdir().expect("temporary document root");
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/valid_project/index.json");
        let source = root.path().join("index.json");
        fs::copy(fixture, &source).expect("copy RAD source");
        let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
            .arg("--no-local-database")
            .arg("export")
            .arg(&source)
            .args(["--format", format])
            .output()
            .expect("run direct document export");
        assert!(cli.status.success(), "{format}: {}", String::from_utf8_lossy(&cli.stderr));
        let destination = format!("native.{}", routed_format.file_type());
        let routed = ExportDocumentInput {
            source: PathBuf::from("index.json"),
            destination: PathBuf::from(&destination),
            format: routed_format,
            overwrite: false,
        }
        .execute(root.path())
        .expect("routed document export");
        assert_eq!(
            fs::read(root.path().join(direct_name)).expect("direct artifact"),
            fs::read(routed.destination).expect("routed artifact")
        );
    }
}
