use super::*;
use acorn::io::mcp::ReportInput;
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(feature = "pdf")]
#[tokio::test]
async fn test_native_local_manifest_matches_direct_project_report() {
    let chrome = std::env::var_os("CHROME_PATH").map(PathBuf::from).or_else(|| {
        let path = PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome");
        path.is_file().then_some(path)
    });
    if let Some(chrome_path) = chrome {
        let mut direct = arguments(vec!["pilot".to_string()]);
        let project = direct.root.join("pilot");
        create_dir_all(&project).expect("project directory");
        let activity = ResearchActivity {
            logbook: Some(
                Logbook::init()
                    .entries(vec![LogbookEntry::init()
                        .category("software")
                        .identifier("release")
                        .project_id("pilot")
                        .state("completed")
                        .summary("Released the pilot")
                        .timestamp("2026-09-18T12:00:00Z")
                        .build()])
                    .build(),
            ),
            ..ResearchActivity::default()
        };
        write(project.join("index.json"), serde_json::to_vec_pretty(&activity).expect("project fixture")).expect("project source");
        direct.chrome_path = Some(chrome_path.clone());
        direct.output = direct.root.join("direct-reports");
        run(&direct, true).await.expect("direct report command");
        let destination = direct.root.join("native-reports");
        let native = execute_routed(
            ReportInput {
                config: direct.config.clone().expect("report config path"),
                project_id: "pilot".to_string(),
                from: direct.from.clone(),
                through: direct.through().to_string(),
                commit: direct.commit.clone().expect("source commit"),
                destination: PathBuf::from("native-reports"),
                pipeline_id: direct.pipeline_id.clone(),
                job_id: direct.job_id.clone(),
                full_timestamp: false,
                overwrite: false,
            },
            direct.root.clone(),
            destination.clone(),
            false,
            true,
            Some(chrome_path.clone()),
        )
        .await
        .expect("native project report");
        assert_eq!(native.status, ReportStatus::Published);
        let artifact = "pilot/20261001T000000Z-0123456789ab/manifest.json";
        let mut direct_manifest: ReportManifest =
            serde_json::from_slice(&read(direct.output.join(artifact)).expect("direct manifest")).expect("direct ledger");
        let mut native_manifest: ReportManifest =
            serde_json::from_slice(&read(destination.join(artifact)).expect("native manifest")).expect("native ledger");
        let project_pdf = "pilot/20261001T000000Z-0123456789ab/report.pdf";
        let direct_pdf = read(direct.output.join(project_pdf)).expect("direct project PDF");
        let native_pdf = read(destination.join(project_pdf)).expect("native project PDF");
        assert!(direct_pdf.starts_with(b"%PDF"));
        assert!(native_pdf.starts_with(b"%PDF"));
        assert_eq!(
            direct_manifest.checksum.take().expect("direct checksum"),
            data_encoding::HEXLOWER.encode(ring::digest::digest(&ring::digest::SHA256, &direct_pdf).as_ref())
        );
        assert_eq!(
            native_manifest.checksum.take().expect("native checksum"),
            data_encoding::HEXLOWER.encode(ring::digest::digest(&ring::digest::SHA256, &native_pdf).as_ref())
        );
        assert_eq!(direct_manifest, native_manifest);
        let direct_legend = read(direct.output.join("legend-page.pdf")).expect("direct legend");
        let native_legend = read(destination.join("legend-page.pdf")).expect("native legend");
        assert!(direct_legend.starts_with(b"%PDF"));
        assert!(native_legend.starts_with(b"%PDF"));
        assert_eq!(native.artifacts.len(), 3);
        let stale_pdf = destination.join(project_pdf);
        assert!(stale_pdf.is_file());
        let reused = execute_routed(
            ReportInput {
                config: direct.config.clone().expect("report config path"),
                project_id: "pilot".to_string(),
                from: Some("2026-09-25T00:00:00Z".to_string()),
                through: direct.through().to_string(),
                commit: direct.commit.clone().expect("source commit"),
                destination: PathBuf::from("native-reports"),
                pipeline_id: direct.pipeline_id.clone(),
                job_id: direct.job_id.clone(),
                full_timestamp: false,
                overwrite: true,
            },
            direct.root.clone(),
            destination.clone(),
            false,
            true,
            Some(chrome_path),
        )
        .await
        .expect("reused project report");
        assert_eq!(reused.status, ReportStatus::NoChanges);
        assert!(stale_pdf.is_file());
        assert!(!reused.artifacts.contains(&stale_pdf.display().to_string()));
        assert!(reused.artifacts.contains(&destination.join(artifact).display().to_string()));
    }
}
#[cfg(unix)]
#[tokio::test]
async fn test_routed_report_rejects_manifest_symlink_outside_root() {
    let direct = arguments(vec!["pilot".to_string()]);
    let project = direct.root.join("pilot");
    create_dir_all(&project).expect("project directory");
    write(
        project.join("index.json"),
        serde_json::to_vec_pretty(&ResearchActivity::default()).expect("project fixture"),
    )
    .expect("project source");
    let outside = tempfile::tempdir().expect("outside report root");
    let target = outside.path().join("manifest.json");
    write(&target, b"outside manifest").expect("outside manifest");
    let destination = direct.root.join("native-reports");
    let project_output = destination.join("pilot/20261001T000000Z-0123456789ab");
    create_dir_all(&project_output).expect("project output");
    symlink(&target, project_output.join("manifest.json")).expect("manifest symlink");
    let result = execute_routed(
        ReportInput {
            config: direct.config.clone().expect("report config path"),
            project_id: "pilot".to_string(),
            from: None,
            through: direct.through().to_string(),
            commit: direct.commit.clone().expect("source commit"),
            destination: PathBuf::from("native-reports"),
            pipeline_id: None,
            job_id: None,
            full_timestamp: false,
            overwrite: true,
        },
        direct.root,
        destination.clone(),
        false,
        true,
        None,
    )
    .await;
    assert!(result.is_err());
    assert_eq!(read(target).expect("outside manifest unchanged"), b"outside manifest");
    assert!(!destination.join("legend-page.pdf").exists());
}
#[cfg(unix)]
#[tokio::test]
async fn test_routed_report_rejects_source_symlink_outside_root() {
    let direct = arguments(vec!["pilot".to_string()]);
    let project = direct.root.join("pilot");
    create_dir_all(&project).expect("project directory");
    let outside = tempfile::tempdir().expect("outside report root");
    let source = outside.path().join("index.json");
    write(&source, serde_json::to_vec_pretty(&ResearchActivity::default()).expect("outside fixture")).expect("outside source");
    symlink(&source, project.join("index.json")).expect("source symlink");
    let destination = direct.root.join("native-reports");
    let result = execute_routed(
        ReportInput {
            config: direct.config.clone().expect("report config path"),
            project_id: "pilot".to_string(),
            from: None,
            through: direct.through().to_string(),
            commit: direct.commit.clone().expect("source commit"),
            destination: PathBuf::from("native-reports"),
            pipeline_id: None,
            job_id: None,
            full_timestamp: false,
            overwrite: false,
        },
        direct.root,
        destination.clone(),
        false,
        true,
        None,
    )
    .await;
    assert!(result.is_err());
    assert!(!destination.exists());
}
