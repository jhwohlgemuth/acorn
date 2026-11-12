use super::*;
use std::{io::Read, net::TcpListener};
use tempfile::TempDir;

fn fixture() -> TempDir {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("packaging")).unwrap();
    fs::write(
        root.join("packaging/applications.json"),
        include_str!("../../../packaging/applications.json"),
    )
    .unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/acorn-web\"]\n[workspace.package]\nversion = \"1.2.3\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("crates/acorn-web")).unwrap();
    fs::write(
        root.join("crates/acorn-web/Cargo.toml"),
        "[package]\nname = \"acorn-web\"\nversion.workspace = true\npublish = false\n",
    )
    .unwrap();
    fs::create_dir(root.join("public")).unwrap();
    fs::write(root.join("public/index.html"), "<!doctype html><title>ACORN</title>").unwrap();
    fs::write(root.join("public/app.wasm"), b"\0asm\x01\0\0\0").unwrap();
    temp
}
fn write_catalog(root: &Path, catalog: &Catalog) {
    fs::write(root.join("packaging/applications.json"), serde_json::to_vec(catalog).unwrap()).unwrap();
}

#[test]
fn test_catalog_catches_unintegrated_desktop_and_missing_coverage() {
    let temp = fixture();
    let root = temp.path();
    let mut catalog = inventory(root).unwrap();
    fs::create_dir_all(root.join("crates/acorn-desktop")).unwrap();
    fs::write(root.join("crates/acorn-desktop/Cargo.toml"), "[package]\nname = \"acorn-desktop\"\n").unwrap();
    assert!(inventory(root).unwrap_err().contains("integrate build/publication"));
    catalog.applications.remove("acorn-desktop");
    write_catalog(root, &catalog);
    assert!(inventory(root).unwrap_err().contains("include acorn-web and acorn-desktop"));
}
#[test]
fn test_catalog_rejects_independent_versions_and_crates_io() {
    let temp = fixture();
    for policy in [
        "version = \"1.2.3\"\npublish = false",
        "version.workspace = true\npublish = true",
        "version.workspace = true",
    ] {
        fs::write(
            temp.path().join("crates/acorn-web/Cargo.toml"),
            format!("[package]\nname = \"acorn-web\"\n{policy}\n"),
        )
        .unwrap();
        assert!(inventory(temp.path()).unwrap_err().contains("inherit the workspace version"));
    }
}
#[test]
fn test_catalog_tracks_desktop_development_before_publication() {
    let temp = fixture();
    let root = temp.path();
    let mut catalog = inventory(root).unwrap();
    catalog.applications.get_mut("acorn-desktop").unwrap().state = State::Development;
    write_catalog(root, &catalog);
    assert!(inventory(root).unwrap_err().contains("workspace package"));
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/acorn-desktop\", \"crates/acorn-web\"]\n[workspace.package]\nversion = \"1.2.3\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("crates/acorn-desktop")).unwrap();
    fs::write(
        root.join("crates/acorn-desktop/Cargo.toml"),
        "[package]\nname = \"acorn-desktop\"\nversion.workspace = true\npublish = false\n",
    )
    .unwrap();
    assert_eq!(inventory(root).unwrap().applications["acorn-desktop"].state, State::Development);
}
#[test]
fn test_package_is_reproducible_and_records_hashes_and_pending_desktop() {
    let temp = fixture();
    let root = temp.path();
    let catalog = inventory(root).unwrap();
    let archive = package_web(&root.join("public"), &root.join("one"), catalog.clone(), "abc123".into()).unwrap();
    let second = package_web(&root.join("public"), &root.join("two"), catalog, "abc123".into()).unwrap();
    assert_eq!(fs::read(&archive).unwrap(), fs::read(second).unwrap());
    let mut zip = ZipArchive::new(fs::File::open(&archive).unwrap()).unwrap();
    let release: Release = serde_json::from_reader(zip.by_name("release.json").unwrap()).unwrap();
    assert_eq!(release.catalog.version, "1.2.3");
    assert_eq!(release.catalog.applications["acorn-desktop"].state, State::Planned);
    assert_eq!(release.files["app.wasm"], checksum(&fs::read(root.join("public/app.wasm")).unwrap()));
    assert_eq!(release.revision, "abc123");
    assert_eq!(verify_web(root, &root.join("one")).unwrap(), archive);
}
#[test]
fn test_package_rejects_missing_or_duplicate_wasm_and_reserved_manifest() {
    let temp = fixture();
    let root = temp.path();
    let package = || package_web(&root.join("public"), &root.join("out"), inventory(root).unwrap(), "abc123".into());
    fs::remove_file(root.join("public/app.wasm")).unwrap();
    assert!(package().unwrap_err().contains("exactly one WASM"));
    fs::write(root.join("public/app.wasm"), "wasm").unwrap();
    fs::write(root.join("public/extra.wasm"), "wasm").unwrap();
    assert!(package().unwrap_err().contains("exactly one WASM"));
    fs::remove_file(root.join("public/extra.wasm")).unwrap();
    fs::write(root.join("public/release.json"), "{}").unwrap();
    assert!(package().unwrap_err().contains("reserved release.json"));
}
#[cfg(unix)]
#[test]
fn test_package_rejects_symlinks() {
    let temp = fixture();
    let root = temp.path();
    std::os::unix::fs::symlink(root.join("Cargo.toml"), root.join("public/link")).unwrap();
    assert!(
        package_web(&root.join("public"), &root.join("out"), inventory(root).unwrap(), "abc123".into())
            .unwrap_err()
            .contains("symlinks")
    );
}
#[test]
fn test_paths_reject_incomplete_and_unknown_arguments() {
    for args in [vec!["--source"], vec!["--unknown", "dir"], vec!["--output"]] {
        assert!(Paths::parse(Path::new("."), "package-web", &args.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
    }
    assert!(Paths::parse(Path::new("."), "publish-web", &["--source".into(), "dir".into()]).is_err());
    assert!(archive_path(Path::new("."), "../../escape").is_err());
}
#[test]
fn test_transfer_sends_token_and_bytes_without_following_redirects() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = Url::parse(&format!("http://{}/artifact", listener.local_addr().unwrap())).unwrap();
    let server = thread::spawn(move || {
        [404, 201, 302].map(|status| {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                stream.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            let headers = String::from_utf8(request).unwrap();
            let length = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length: ")
                        .map(|length| length.parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            let mut body = vec![0; length];
            stream.read_exact(&mut body).unwrap();
            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Length: 0\r\nConnection: close\r\nLocation: /redirected\r\n\r\n"
            )
            .unwrap();
            (headers, body)
        })
    });
    let client = Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    upload_with(b"release", |method, body| transfer(&client, method, &destination, "test-token", body)).unwrap();
    assert_eq!(transfer(&client, Method::GET, &destination, "test-token", None).unwrap().0, 302);
    let requests = server.join().unwrap();
    assert!(requests[0].0.starts_with("GET /artifact "));
    assert!(requests[1].0.starts_with("PUT /artifact "));
    assert_eq!(requests[1].1, b"release");
    assert!(requests
        .iter()
        .all(|(headers, _)| headers.to_ascii_lowercase().contains("job-token: test-token\r\n")));
}
#[test]
fn test_upload_accepts_identical_bytes_and_only_creates_missing_artifacts() {
    for (status, existing, expected, upload) in [
        (200, b"release".as_slice(), true, false),
        (200, b"other", false, false),
        (404, b"", true, true),
        (403, b"", false, false),
        (302, b"", false, false),
    ] {
        let mut methods = Vec::new();
        let result = upload_with(b"release", |method, body| {
            methods.push(method.clone());
            if method == Method::GET {
                assert_eq!(body, None);
                Ok((status, existing.to_vec()))
            } else {
                assert_eq!(method, Method::PUT);
                assert_eq!(body, Some(b"release".as_slice()));
                Ok((201, Vec::new()))
            }
        });
        assert_eq!(result.is_ok(), expected);
        assert_eq!(methods, if upload { vec![Method::GET, Method::PUT] } else { vec![Method::GET] });
    }
}
#[test]
fn test_upload_rejects_failed_put_and_transport_errors() {
    for status in [302, 403, 409, 500] {
        assert!(upload_with(b"release", |method, _| Ok((if method == Method::GET { 404 } else { status }, Vec::new()))).is_err());
    }
    assert!(upload_with(b"release", |_, _| Err("HTTP request failed".into())).is_err());
    for destination in ["http://example.test/package", "https://user:secret@example.test/package", "invalid"] {
        assert!(upload_artifact(Path::new("missing"), destination).is_err());
    }
}
#[test]
fn test_verify_rejects_tampering_and_wrong_release_version() {
    let temp = fixture();
    let root = temp.path();
    let output = root.join("out");
    let archive = package_web(&root.join("public"), &output, inventory(root).unwrap(), "abc123".into()).unwrap();
    fs::write(archive.with_extension("zip.sha256"), "bad checksum").unwrap();
    assert!(verify_web(root, &output).unwrap_err().contains("checksum"));
    let bytes = fs::read(&archive).unwrap();
    fs::write(archive.with_extension("zip.sha256"), checksum_line(&archive, &bytes)).unwrap();
    let mut bundle = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut release: Release = serde_json::from_reader(bundle.by_name("release.json").unwrap()).unwrap();
    release.catalog.version = "9.9.9".into();
    let bytes = zip_bytes(BTreeMap::from([("release.json".into(), serde_json::to_vec(&release).unwrap())])).unwrap();
    fs::write(&archive, &bytes).unwrap();
    fs::write(archive.with_extension("zip.sha256"), checksum_line(&archive, &bytes)).unwrap();
    assert!(verify_web(root, &output).unwrap_err().contains("does not match"));
}
