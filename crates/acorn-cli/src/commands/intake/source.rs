//! Resolve JSONL locations to an exact local review source without modifying a remote inbox.
use acorn::io::api::gitlab::{jsonl::read_snapshot, jsonl::Snapshot, service_url, Options};
use acorn::io::api::Configuration;
use acorn::io::config::{application::PowerAutomateIntake, ApplicationConfiguration};
use acorn::io::{read_file_snapshot, sync_parent_directory};
use acorn_core::{
    time::{Seconds, UnitOfTime},
    Location, Scheme,
};
use acorn_host::fs::{file_uri_to_path, SafePath};
use acorn_schema::hardware::memory::Memory;
use color_eyre::eyre::{eyre, Result};
use data_encoding::HEXLOWER;
use fluent_uri::Uri;
use ring::digest::{digest, SHA256};
use serde_json::json;
use std::fs::{DirBuilder, OpenOptions};
use std::io::{ErrorKind, Write};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

const MAX_SOURCE_BYTES: usize = 10 * 1024 * 1024;
#[derive(Debug)]
enum Source {
    File(PathBuf),
    GitLab {
        origin: String,
        project: String,
        branch: String,
        path: String,
    },
}
impl Source {
    async fn capture(&self, config: &Path, options: Options) -> Result<PathBuf> {
        match self {
            | Self::File(path) => Ok(path.clone()),
            | Self::GitLab {
                origin,
                project,
                branch,
                path,
            } => match service_url(&options) {
                | Ok(configured) if &configured == origin => {
                    let intake = PowerAutomateIntake {
                        branch: branch.clone(),
                        path: path.clone(),
                        poll_interval: Seconds::new(30),
                        max_file: Memory::mb(10),
                        max_record: Memory::kb(64),
                    };
                    read_snapshot(&intake, &options.with_identifier(project))
                        .await
                        .and_then(|snapshot| self.persist(config, snapshot))
                }
                | Ok(_) => Err(eyre!(
                    "GitLab source origin differs from CI_SERVER_HOST; set CI_SERVER_HOST to the intended GitLab host before using its credentials"
                )),
                | Err(why) => Err(why),
            },
        }
    }
    fn gitlab(value: &str, location: &Location) -> Result<Self> {
        Uri::parse(value).map_err(|_| eyre!("Invalid GitLab source URI")).and_then(|uri| {
            let safe = uri.authority().is_some_and(|authority| authority.userinfo().is_none())
                && uri.query().is_none()
                && uri.fragment().is_none()
                && !(location.scheme() == Scheme::HTTP && location.is_non_loopback_http_or_https());
            match safe {
                | false => Err(eyre!("Use a GitLab HTTPS file URL without credentials, query, or fragment")),
                | true => uri
                    .path()
                    .split('/')
                    .skip(1)
                    .map(|segment| {
                        segment
                            .decode()
                            .to_string()
                            .map(|value| value.into_owned())
                            .map_err(|_| eyre!("Source URI must decode to UTF-8"))
                    })
                    .collect::<Result<Vec<_>>>()
                    .and_then(|segments| {
                        let marker = segments.iter().position(|segment| segment == "-");
                        let origin = location.origin().ok_or_else(|| eyre!("GitLab source has no valid origin"));
                        match marker.and_then(|index| index.checked_add(1).and_then(|next| segments.get(..index).zip(segments.get(next..)))) {
                            | Some((project, [view, branch, tail @ ..])) if matches!(view.as_str(), "raw" | "blob") => {
                                let project = project.join("/");
                                let path = tail.join("/");
                                let valid = Location::from(&project).is_canonical_repository_path()
                                    && Location::from(&path).is_canonical_repository_path()
                                    && Location::from(branch).is_canonical_repository_path();
                                match valid {
                                    | true => origin.map(|origin| Self::GitLab {
                                        origin,
                                        project,
                                        branch: branch.clone(),
                                        path,
                                    }),
                                    | false => Err(eyre!("GitLab source requires a project, revision, and repository-relative file path")),
                                }
                            }
                            | _ => Err(eyre!(
                                "Use https://host/group/project/-/raw/ref/path (or /-/blob/ref/path); encode slashes within the revision as %2F"
                            )),
                        }
                    }),
            }
        })
    }
    fn parse(value: &str) -> Result<Self> {
        let location = Location::from(value);
        match location.scheme() {
            | Scheme::File => Uri::parse(value).map_err(|_| eyre!("Invalid file source URI")).and_then(|uri| {
                let local = uri.authority().is_none_or(|authority| matches!(authority.as_str(), "" | "localhost"));
                match local && uri.path().as_str().starts_with('/') && uri.query().is_none() && uri.fragment().is_none() {
                    | true => file_uri_to_path(format!("file://{}", uri.path())).map(Self::File).map_err(Into::into),
                    | false => Err(eyre!("Use an absolute local file URI without credentials, query, or fragment")),
                }
            }),
            | Scheme::Unsupported if location.is_path() && !value.contains("://") && !value.is_empty() => Ok(Self::File(PathBuf::from(value))),
            | Scheme::HTTP | Scheme::HTTPS => Self::gitlab(value, &location),
            | _ => Err(eyre!("Intake source must be a local path, file URI, or GitLab HTTPS file URL")),
        }
    }
    fn persist(&self, config: &Path, snapshot: Snapshot) -> Result<PathBuf> {
        let valid_commit = matches!(snapshot.commit.len(), 40 | 64) && snapshot.commit.bytes().all(|byte| byte.is_ascii_hexdigit());
        match (snapshot.last_commit_id.as_ref(), valid_commit, self) {
            | (None, _, _) => Err(eyre!("GitLab source file does not exist at the selected revision")),
            | (_, false, _) => Err(eyre!("GitLab did not return an immutable commit SHA")),
            | (
                _,
                true,
                Self::GitLab {
                    origin,
                    project,
                    branch,
                    path,
                },
            ) => {
                let receipt = json!({
                    "branch": branch, "commit": snapshot.commit, "gitlabService": origin,
                    "lastCommitId": snapshot.last_commit_id, "path": path, "project": project,
                    "sourceDigest": format!("sha256:{}", checksum(&snapshot.content))
                })
                .to_string();
                config
                    .parent()
                    .ok_or_else(|| eyre!("Intake configuration has no parent directory"))
                    .and_then(|root| {
                        SafePath::new(format!(".acorn/intake/sources/{}/logbook.jsonl", checksum(&receipt)))
                            .and_then(|path| path.materialize_under(root))
                            .map_err(Into::into)
                    })
                    .and_then(|source| {
                        source
                            .parent()
                            .ok_or_else(|| eyre!("Snapshot has no parent directory"))
                            .and_then(|parent| {
                                let mut builder = DirBuilder::new();
                                builder.recursive(true);
                                #[cfg(unix)]
                                builder.mode(0o700);
                                builder
                                    .create(parent)
                                    .map_err(color_eyre::Report::from)
                                    .and_then(|()| preserve(&source, &snapshot.content))
                                    .and_then(|()| preserve(&parent.join("source.json"), &receipt))
                            })
                            .map(|()| source)
                    })
            }
            | _ => Err(eyre!("Only GitLab sources require snapshot capture")),
        }
    }
}
fn checksum(value: &str) -> String {
    HEXLOWER.encode(digest(&SHA256, value.as_bytes()).as_ref())
}
pub(super) async fn prepare(config: &Path, value: &str, offline: bool) -> Result<PathBuf> {
    match Source::parse(value) {
        | Ok(Source::File(path)) => Ok(path),
        | Ok(_) if offline => Err(eyre!(
            "Remote intake sources are unavailable with --offline; use a previously captured local snapshot"
        )),
        | Ok(source) => match ApplicationConfiguration::load_local_intake(config) {
            | Ok((_, config)) => source.capture(&config, Options::from_env()).await,
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
fn preserve(path: &Path, content: &str) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    match options.open(path) {
        | Ok(mut file) => file
            .write_all(content.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(Into::into)
            .and_then(|()| sync_parent_directory(path)),
        | Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            read_file_snapshot(path, MAX_SOURCE_BYTES).and_then(|(_, current, _)| match current == content {
                | true => Ok(()),
                | false => Err(eyre!("Stored intake snapshot or provenance changed; refusing to overwrite it")),
            })
        }
        | Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests;
