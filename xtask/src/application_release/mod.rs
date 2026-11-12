//! Application artifact policy, deterministic browser packaging and immutable publication.
use crate::{get_workspace_version, read_toml};
use core::fmt;
use reqwest::{blocking::Client, redirect::Policy, Method, Url};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};
use toml_edit::DocumentMut;
use zip::{write::SimpleFileOptions, CompressionMethod, DateTime, ZipArchive, ZipWriter};

const APPLICATIONS: [&str; 2] = ["acorn-desktop", "acorn-web"];
const BROWSER: &str = "acorn-web";
type TaskResult<T> = Result<T, String>;
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum State {
    Development,
    Planned,
    Preview,
    Ready,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Application {
    delivery: String,
    manifest: PathBuf,
    remaining_phases: Vec<String>,
    state: State,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    applications: BTreeMap<String, Application>,
    schema_version: u32,
    #[serde(default)]
    version: String,
}
#[derive(Debug)]
struct Paths {
    output: PathBuf,
    source: PathBuf,
}
#[derive(Debug, Deserialize, Serialize)]
struct Release {
    #[serde(flatten)]
    catalog: Catalog,
    component: String,
    files: BTreeMap<String, String>,
    revision: String,
}
impl Paths {
    fn parse(root: &Path, command: &str, args: &[String]) -> TaskResult<Self> {
        args.chunks(2).try_fold(
            Self {
                output: root.join("bin"),
                source: root.join("target/dx/acorn-web/release/web/public"),
            },
            |paths, pair| match pair {
                | [flag, path] if flag == "--output" => Ok(Self {
                    output: path.into(),
                    ..paths
                }),
                | [flag, path] if flag == "--source" && command == "package-web" => Ok(Self {
                    source: path.into(),
                    ..paths
                }),
                | _ => Err(format!("Invalid {command} arguments; expected --output DIR or package-web --source DIR")),
            },
        )
    }
}
impl fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Development => "development",
            | Self::Planned => "planned",
            | Self::Preview => "preview",
            | Self::Ready => "ready",
        })
    }
}
fn archive_path(output: &Path, version: &str) -> TaskResult<PathBuf> {
    semver::Version::parse(version)
        .map(|_| output.join(format!("acorn-web-v{version}.zip")))
        .map_err(|_| "Invalid workspace release version".into())
}
fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn checksum_line(archive: &Path, bytes: &[u8]) -> String {
    format!("{}  {}\n", checksum(bytes), archive.file_name().unwrap_or_default().to_string_lossy())
}
fn inventory(root: &Path) -> TaskResult<Catalog> {
    read_toml(root.join("Cargo.toml")).and_then(|workspace| {
        get_workspace_version(&workspace).and_then(|version| {
            fs::read(root.join("packaging/applications.json"))
                .map_err(|error| error.to_string())
                .and_then(|bytes| serde_json::from_slice::<Catalog>(&bytes).map_err(|error| error.to_string()))
                .and_then(|catalog| {
                    if catalog.schema_version != 1 || !catalog.applications.keys().map(String::as_str).eq(APPLICATIONS) {
                        Err("The publishing inventory must include acorn-web and acorn-desktop".into())
                    } else {
                        catalog
                            .applications
                            .iter()
                            .try_for_each(|(name, app)| validate_application(root, &workspace, name, app))
                            .map(|()| Catalog { version, ..catalog })
                    }
                })
        })
    })
}
fn package_web(source: &Path, output: &Path, catalog: Catalog, revision: String) -> TaskResult<PathBuf> {
    let publishable = catalog
        .applications
        .get(BROWSER)
        .is_some_and(|app| matches!(app.state, State::Preview | State::Ready));
    if !publishable {
        Err("acorn-web is not available for packaging".into())
    } else {
        archive_path(output, &catalog.version).and_then(|archive| {
            read_files(source, source).and_then(|files| {
                if !files.contains_key("index.html") || files.keys().filter(|name| name.ends_with(".wasm")).count() != 1 {
                    Err("Browser output requires index.html and exactly one WASM payload; run make web-build".into())
                } else if files.contains_key("release.json") {
                    Err("Browser output already contains reserved release.json".into())
                } else {
                    let release = Release {
                        catalog,
                        component: BROWSER.into(),
                        files: files.iter().map(|(name, bytes)| (name.clone(), checksum(bytes))).collect(),
                        revision,
                    };
                    serde_json::to_vec_pretty(&release)
                        .map_err(|error| error.to_string())
                        .and_then(|mut manifest| {
                            manifest.push(b'\n');
                            let entries = files.into_iter().chain([("release.json".into(), manifest)]).collect();
                            zip_bytes(entries).and_then(|bytes| {
                                fs::create_dir_all(output)
                                    .and_then(|()| fs::write(&archive, &bytes))
                                    .and_then(|()| fs::write(archive.with_extension("zip.sha256"), checksum_line(&archive, &bytes)))
                                    .map_err(|error| error.to_string())
                                    .map(|()| archive)
                            })
                        })
                }
            })
        })
    }
}
fn publish_web(root: &Path, output: &Path) -> TaskResult<()> {
    verify_web(root, output).and_then(|archive| {
        let registry = env::var("PKG_REGISTRY_URL").map_err(|_| "PKG_REGISTRY_URL is required".to_string());
        registry.and_then(|registry| {
            inventory(root).and_then(|catalog| {
                [archive.clone(), archive.with_extension("zip.sha256")].iter().try_for_each(|path| {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    upload_artifact(path, &format!("{}/acorn-web/v{}/{name}", registry.trim_end_matches('/'), catalog.version))
                })
            })
        })
    })
}
fn read_files(root: &Path, directory: &Path) -> TaskResult<BTreeMap<String, Vec<u8>>> {
    fs::symlink_metadata(directory).map_err(|error| error.to_string()).and_then(|metadata| {
        if metadata.file_type().is_symlink() {
            Err("Browser output must not contain symlinks".into())
        } else {
            fs::read_dir(directory).map_err(|error| error.to_string()).and_then(|mut entries| {
                entries.try_fold(BTreeMap::new(), |mut files, entry| {
                    entry.map_err(|error| error.to_string()).and_then(|entry| {
                        let path = entry.path();
                        entry.file_type().map_err(|error| error.to_string()).and_then(|kind| {
                            let found = if kind.is_symlink() {
                                Err("Browser output must not contain symlinks".into())
                            } else if kind.is_dir() {
                                read_files(root, &path)
                            } else if kind.is_file() {
                                path.strip_prefix(root).map_err(|error| error.to_string()).and_then(|relative| {
                                    let name = relative.to_str().ok_or_else(|| "Browser asset path is not UTF-8".to_string());
                                    name.and_then(|name| {
                                        fs::read(&path)
                                            .map_err(|error| error.to_string())
                                            .map(|bytes| BTreeMap::from([(name.replace('\\', "/"), bytes)]))
                                    })
                                })
                            } else {
                                Err("Browser output must contain only regular files and directories".into())
                            };
                            found.map(|found| {
                                files.extend(found);
                                files
                            })
                        })
                    })
                })
            })
        }
    })
}
/// Run one application release task without changing the process working directory.
pub(crate) fn run(command: &str, args: &[String]) -> TaskResult<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "Cannot locate workspace root".to_string());
    root.and_then(|root| match (command, args) {
        | ("upload-artifact", [source, destination]) => upload_artifact(Path::new(source), destination),
        | ("upload-artifact", _) => Err("Usage: cargo xtask upload-artifact FILE HTTPS_URL".into()),
        | ("check-release-apps", []) => inventory(root).map(|catalog| {
            catalog
                .applications
                .iter()
                .for_each(|(name, app)| println!("{name}: {} ({})", app.state, app.delivery));
        }),
        | ("check-release-apps", _) => Err("check-release-apps takes no arguments".into()),
        | _ => Paths::parse(root, command, args).and_then(|paths| match command {
            | "package-web" => inventory(root).and_then(|catalog| {
                Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .current_dir(root)
                    .output()
                    .map_err(|error| error.to_string())
                    .and_then(|output| {
                        if output.status.success() {
                            package_web(
                                &paths.source,
                                &paths.output,
                                catalog,
                                String::from_utf8_lossy(&output.stdout).trim().into(),
                            )
                            .map(|archive| println!("{}", archive.display()))
                        } else {
                            Err("Cannot read source revision".into())
                        }
                    })
            }),
            | "publish-web" => publish_web(root, &paths.output),
            | "verify-web" => verify_web(root, &paths.output).map(|path| println!("Verified {}", path.display())),
            | _ => Err(format!("Unknown application release command: {command}")),
        }),
    })
}
fn transfer(client: &Client, method: Method, destination: &Url, token: &str, body: Option<&[u8]>) -> TaskResult<(u16, Vec<u8>)> {
    (0..4)
        .map(|attempt| {
            if attempt > 0 {
                thread::sleep(Duration::from_secs(1 << (attempt - 1)));
            }
            let request = client.request(method.clone(), destination.clone()).header("JOB-TOKEN", token);
            let request = match body {
                | Some(body) => request.body(body.to_vec()),
                | None => request,
            };
            request
                .send()
                .and_then(|response| {
                    let status = response.status().as_u16();
                    response.bytes().map(|bytes| (status, bytes.to_vec()))
                })
                .map_err(|_| "Artifact HTTP request failed".to_string())
        })
        .find(|result| result.as_ref().is_ok_and(|(status, _)| *status != 429 && *status < 500))
        .unwrap_or_else(|| Err("Artifact HTTP request failed after four attempts".into()))
}
fn upload_artifact(source: &Path, destination: &str) -> TaskResult<()> {
    Url::parse(destination).map_err(|_| "Invalid release URL".to_string()).and_then(|url| {
        if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
            Err("Release upload requires HTTPS without URL credentials".into())
        } else {
            env::var("CI_JOB_TOKEN")
                .map_err(|_| "CI_JOB_TOKEN is required".to_string())
                .and_then(|token| {
                    fs::read(source).map_err(|error| error.to_string()).and_then(|bytes| {
                        Client::builder()
                            .redirect(Policy::none())
                            .timeout(Duration::from_secs(120))
                            .build()
                            .map_err(|_| "Cannot initialize release HTTP client".to_string())
                            .and_then(|client| upload_with(&bytes, |method, body| transfer(&client, method, &url, &token, body)))
                            .map(|()| println!("Release artifact verified or uploaded: {destination}"))
                    })
                })
        }
    })
}
fn upload_with(bytes: &[u8], mut request: impl FnMut(Method, Option<&[u8]>) -> TaskResult<(u16, Vec<u8>)>) -> TaskResult<()> {
    request(Method::GET, None).and_then(|(status, existing)| match status {
        | 200 if existing == bytes => Ok(()),
        | 200 => Err("Refusing to overwrite release artifact with different content".into()),
        | 404 => request(Method::PUT, Some(bytes)).and_then(|(status, _)| match status {
            | 200 | 201 => Ok(()),
            | status => Err(format!("Artifact upload failed: HTTP {status}")),
        }),
        | status => Err(format!("Unable to inspect release artifact: HTTP {status}")),
    })
}
fn validate_application(root: &Path, workspace: &DocumentMut, name: &str, app: &Application) -> TaskResult<()> {
    let member = format!("crates/{name}");
    let declared = workspace
        .get("workspace")
        .and_then(|item| item.get("members"))
        .and_then(|item| item.as_array())
        .is_some_and(|members| members.iter().any(|item| item.as_str() == Some(&member)));
    let expected = PathBuf::from(&member).join("Cargo.toml");
    if app.manifest != expected {
        Err(format!("{name}: unexpected application manifest path"))
    } else if app.state == State::Planned {
        if root.join(&app.manifest).exists() || declared {
            Err(format!(
                "{name}: implementation exists; integrate build/publication jobs and update its planned state"
            ))
        } else if app.remaining_phases.is_empty() {
            Err(format!("{name}: planned publication requires remaining phases"))
        } else {
            Ok(())
        }
    } else if !declared {
        Err(format!("{name}: active publication requires a workspace package"))
    } else {
        read_toml(root.join(&app.manifest)).and_then(|manifest| {
            let package = manifest.get("package");
            let inherits = package
                .and_then(|item| item.get("version"))
                .and_then(|item| item.get("workspace"))
                .and_then(|item| item.as_bool())
                == Some(true);
            let private = package.and_then(|item| item.get("publish")).and_then(|item| item.as_bool()) == Some(false);
            let named = package.and_then(|item| item.get("name")).and_then(|item| item.as_str()) == Some(name);
            (inherits && private && named)
                .then_some(())
                .ok_or_else(|| format!("{name}: applications must inherit the workspace version and disable crates.io publication"))
        })
    }
}
fn verify_web(root: &Path, output: &Path) -> TaskResult<PathBuf> {
    inventory(root).and_then(|catalog| {
        archive_path(output, &catalog.version).and_then(|archive| {
            fs::read(&archive).map_err(|error| error.to_string()).and_then(|bytes| {
                fs::read_to_string(archive.with_extension("zip.sha256"))
                    .map_err(|error| error.to_string())
                    .and_then(|expected| {
                        if expected != checksum_line(&archive, &bytes) {
                            Err("Browser archive checksum does not match".into())
                        } else {
                            ZipArchive::new(Cursor::new(bytes))
                                .map_err(|error| error.to_string())
                                .and_then(|mut bundle| {
                                    bundle
                                        .by_name("release.json")
                                        .map_err(|error| error.to_string())
                                        .and_then(|manifest| serde_json::from_reader::<_, Release>(manifest).map_err(|error| error.to_string()))
                                        .and_then(|release| {
                                            (release.catalog.version == catalog.version && release.component == BROWSER)
                                                .then_some(archive)
                                                .ok_or_else(|| "Browser archive does not match this workspace release".into())
                                        })
                                })
                        }
                    })
            })
        })
    })
}
fn zip_bytes(files: BTreeMap<String, Vec<u8>>) -> TaskResult<Vec<u8>> {
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644);
    files
        .into_iter()
        .try_fold(ZipWriter::new(Cursor::new(Vec::new())), |mut zip, (name, bytes)| {
            zip.start_file(name, options)
                .map_err(|error| error.to_string())
                .and_then(|()| zip.write_all(&bytes).map_err(|error| error.to_string()))
                .map(|()| zip)
        })
        .and_then(|zip| zip.finish().map(|cursor| cursor.into_inner()).map_err(|error| error.to_string()))
}

#[cfg(test)]
mod tests;
