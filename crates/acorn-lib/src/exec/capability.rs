//! Per-invocation host capability broker for Rhai hooks.
use super::manifest::{FilesystemCapability, NetworkCapability, ScriptAuthorization};
use crate::io::{workflow::WorkflowAction, ApiResult};
use acorn_core::prelude::{String, Vec};
use acorn_host::fs::SafePath;
use alloc::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use bon::Builder;
use cap_std::{ambient_authority, fs::Dir};
use color_eyre::eyre::eyre;
use core::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use rhai::{Dynamic, Engine, EvalAltResult, Position};
use std::{
    io::{ErrorKind, Read},
    net::ToSocketAddrs,
    path::{Path, PathBuf},
    sync::Mutex,
};

struct CapabilityInner {
    limits: super::manifest::ResourceLimits,
    network: Option<NetworkCapability>,
    offline: bool,
    read: Option<DirectoryRoots>,
    state: Mutex<CapabilityState>,
    write: Option<DirectoryRoots>,
}
#[derive(Builder)]
#[builder(start_fn = init)]
struct CapabilityState {
    #[builder(default)]
    calls: usize,
    #[builder(default)]
    network_requests: usize,
    #[builder(default)]
    read_bytes: usize,
    #[builder(default)]
    read_files: BTreeSet<String>,
    #[builder(default)]
    used: BTreeSet<String>,
    #[builder(default)]
    write_bytes: usize,
    #[builder(default)]
    writes: Vec<FileProposal>,
}
struct DirectoryRoot {
    directory: Arc<Dir>,
    path: PathBuf,
}
struct DirectoryRoots {
    max_bytes: usize,
    max_files: usize,
    roots: BTreeMap<String, DirectoryRoot>,
}
/// One host-owned file replacement proposed by a script.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileProposal {
    /// Complete UTF-8 replacement content.
    pub content: String,
    /// Absolute destination beneath a granted write root.
    pub path: PathBuf,
}
/// Capability broker created fresh for one script invocation.
#[derive(Clone)]
pub struct InvocationCapability {
    inner: Arc<CapabilityInner>,
}
impl DirectoryRoot {
    fn is_valid_destination(&self, relative: &SafePath) -> Result<(), String> {
        relative
            .as_path()
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .filter(|parent| !parent.as_os_str().is_empty())
            .try_for_each(|parent| {
                self.directory
                    .symlink_metadata(parent)
                    .map_err(|why| format!("failed to inspect write destination `{}` — {why}", self.path.join(parent).display()))
                    .and_then(|metadata| match metadata.is_dir() && !metadata.file_type().is_symlink() {
                        | true => Ok(()),
                        | false => Err(format!("unsafe write destination `{}`", self.path.join(parent).display())),
                    })
            })
            .and_then(|()| match self.directory.symlink_metadata(relative.as_path()) {
                | Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                    Err(format!("unsafe write target `{}`", self.path.join(relative.as_path()).display()))
                }
                | Ok(_) => Ok(()),
                | Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
                | Err(why) => Err(format!(
                    "failed to inspect write target `{}` — {why}",
                    self.path.join(relative.as_path()).display()
                )),
            })
    }
}
impl DirectoryRoots {
    fn open(capability: &FilesystemCapability) -> ApiResult<Self> {
        capability
            .roots
            .iter()
            .try_fold(BTreeMap::new(), |mut roots, (name, path)| {
                Dir::open_ambient_dir(path, ambient_authority())
                    .map_err(|why| eyre!("failed to open capability root `{name}` at `{}` — {why}", path.display()))
                    .map(|directory| {
                        roots.insert(
                            name.clone(),
                            DirectoryRoot {
                                directory: Arc::new(directory),
                                path: path.clone(),
                            },
                        );
                        roots
                    })
            })
            .map(|roots| Self {
                max_bytes: capability.max_bytes,
                max_files: capability.max_files,
                roots,
            })
    }
    fn resolve(&self, root: &str, path: &str) -> Result<(&DirectoryRoot, SafePath), String> {
        self.roots
            .get(root)
            .ok_or_else(|| format!("unknown capability root `{root}`"))
            .and_then(|directory| SafePath::new(path).map_err(|why| why.to_string()).map(|relative| (directory, relative)))
    }
}
impl InvocationCapability {
    /// Create a broker containing only the capabilities granted by one manifest entry.
    pub fn new(authorization: &ScriptAuthorization, offline: bool, hook: WorkflowAction) -> ApiResult<Self> {
        authorization
            .capabilities
            .read
            .as_ref()
            .map(DirectoryRoots::open)
            .transpose()
            .and_then(|read| {
                authorization
                    .capabilities
                    .write
                    .as_ref()
                    .filter(|_| !matches!(hook, WorkflowAction::Check | WorkflowAction::Validate))
                    .map(DirectoryRoots::open)
                    .transpose()
                    .map(|write| (read, write))
            })
            .map(|(read, write)| Self {
                inner: Arc::new(CapabilityInner {
                    limits: authorization.limits.clone(),
                    network: authorization.capabilities.network.clone(),
                    offline,
                    read,
                    state: Mutex::new(CapabilityState::init().build()),
                    write,
                }),
            })
    }
    /// Register only the host functions granted to this invocation.
    pub fn register(&self, engine: &mut Engine) {
        if self.inner.read.is_some() {
            let exists = self.clone();
            let read = self.clone();
            engine.register_fn("exists", move |root: &str, path: &str| exists.exists(root, path).map_err(rhai_error));
            engine.register_fn("read_text", move |root: &str, path: &str| read.read_text(root, path).map_err(rhai_error));
        }
        if self.inner.write.is_some() {
            let write = self.clone();
            engine.register_fn("write_text", move |root: &str, path: &str, content: &str| {
                write.write_text(root, path, content).map_err(rhai_error)
            });
        }
        if self.inner.network.is_some() {
            let network = self.clone();
            engine.register_fn("http_get", move |url: &str| network.http_get(url).map_err(rhai_error));
        }
    }
    /// Consume the current ordered file proposals.
    pub fn proposals(&self) -> ApiResult<Vec<FileProposal>> {
        self.inner
            .state
            .lock()
            .map(|state| state.writes.clone())
            .map_err(|why| eyre!("script capability state is unavailable — {why}"))
    }
    /// Return the ordered effect names actually used by this invocation.
    pub fn used_effects(&self) -> ApiResult<Vec<String>> {
        self.inner
            .state
            .lock()
            .map(|state| state.used.iter().cloned().collect())
            .map_err(|why| eyre!("script capability state is unavailable — {why}"))
    }
    fn charge(&self, effect: &str) -> Result<(), String> {
        self.inner
            .state
            .lock()
            .map_err(|why| format!("script capability state is unavailable — {why}"))
            .and_then(|mut state| {
                state.calls = state.calls.saturating_add(1);
                state.used.insert(effect.to_string());
                match state.calls <= self.inner.limits.max_calls {
                    | true => Ok(()),
                    | false => Err("script capability-call limit exceeded".to_string()),
                }
            })
    }
    fn exists(&self, root: &str, path: &str) -> Result<bool, String> {
        self.charge("filesystem-read").and_then(|()| {
            self.inner
                .read
                .as_ref()
                .ok_or_else(|| "filesystem-read is not granted".to_string())
                .and_then(|roots| roots.resolve(root, path).map(|value| (roots, value)))
                .and_then(|(roots, (directory, relative))| {
                    self.charge_read_file(roots, root, path)
                        .map(|()| directory.directory.metadata(relative.as_path()).is_ok())
                })
        })
    }
    fn http_get(&self, input: &str) -> Result<String, String> {
        self.charge("network-read").and_then(|()| match self.inner.offline {
            | true => Err("network-read is unavailable while offline".to_string()),
            | false => self
                .inner
                .network
                .as_ref()
                .ok_or_else(|| "network-read is not granted".to_string())
                .and_then(|capability| restricted_get(input, capability, &self.inner.state)),
        })
    }
    fn read_text(&self, root: &str, path: &str) -> Result<String, String> {
        self.charge("filesystem-read").and_then(|()| {
            self.inner
                .read
                .as_ref()
                .ok_or_else(|| "filesystem-read is not granted".to_string())
                .and_then(|roots| roots.resolve(root, path).map(|value| (roots, value)))
                .and_then(|(roots, (directory, relative))| {
                    self.charge_read_file(roots, root, path).and_then(|()| {
                        directory
                            .directory
                            .open(relative.as_path())
                            .map_err(|why| format!("failed to open `{root}:{path}` — {why}"))
                            .and_then(|file| {
                                self.inner
                                    .state
                                    .lock()
                                    .map_err(|why| format!("script capability state is unavailable — {why}"))
                                    .map(|state| roots.max_bytes.saturating_sub(state.read_bytes))
                                    .and_then(|remaining| read_bounded(file, remaining, &format!("`{root}:{path}`")))
                                    .and_then(|bytes| String::from_utf8(bytes).map_err(|why| format!("`{root}:{path}` is not UTF-8 — {why}")))
                                    .and_then(|content| {
                                        self.inner
                                            .state
                                            .lock()
                                            .map_err(|why| format!("script capability state is unavailable — {why}"))
                                            .map(|mut state| {
                                                state.read_bytes = state.read_bytes.saturating_add(content.len());
                                                content
                                            })
                                    })
                            })
                    })
                })
        })
    }
    fn charge_read_file(&self, roots: &DirectoryRoots, root: &str, path: &str) -> Result<(), String> {
        self.inner
            .state
            .lock()
            .map_err(|why| format!("script capability state is unavailable — {why}"))
            .and_then(|mut state| {
                let key = format!("{root}:{path}");
                state.read_files.insert(key.clone());
                match state.read_files.len() <= roots.max_files {
                    | true => Ok(()),
                    | false => {
                        state.read_files.remove(&key);
                        Err("filesystem-read file limit exceeded".to_string())
                    }
                }
            })
    }
    fn write_text(&self, root: &str, path: &str, content: &str) -> Result<(), String> {
        self.charge("filesystem-write").and_then(|()| {
            self.inner
                .write
                .as_ref()
                .ok_or_else(|| "filesystem-write is not granted".to_string())
                .and_then(|roots| roots.resolve(root, path).map(|value| (roots, value)))
                .and_then(|(roots, (directory, relative))| {
                    directory.is_valid_destination(&relative).and_then(|()| {
                        let destination = directory.path.join(relative.as_path());
                        self.inner
                            .state
                            .lock()
                            .map_err(|why| format!("script capability state is unavailable — {why}"))
                            .and_then(|mut state| {
                                let write_bytes = state.write_bytes.saturating_add(content.len());
                                let existing = state.writes.iter().find(|proposal| proposal.path == destination);
                                match existing {
                                    | Some(proposal) if proposal.content == content => Ok(()),
                                    | Some(_) => Err(format!("conflicting file proposal for `{}`", destination.display())),
                                    | None if state.writes.len() >= roots.max_files => Err("filesystem-write file limit exceeded".to_string()),
                                    | None if write_bytes > roots.max_bytes => Err("filesystem-write byte limit exceeded".to_string()),
                                    | None => {
                                        state.write_bytes = write_bytes;
                                        state.writes.push(FileProposal {
                                            content: content.to_string(),
                                            path: destination,
                                        });
                                        Ok(())
                                    }
                                }
                            })
                    })
                })
        })
    }
}
fn authority(url: &reqwest::Url) -> Option<String> {
    url.host_str().map(|host| match url.port_or_known_default() {
        | Some(443) | None => host.to_string(),
        | Some(port) => format!("{host}:{port}"),
    })
}
fn ip_is_allowed(address: IpAddr) -> bool {
    match address {
        | IpAddr::V4(address) => ipv4_is_allowed(address),
        | IpAddr::V6(address) => ipv6_is_allowed(address),
    }
}
fn ipv4_is_allowed(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    !(address.is_broadcast()
        || address.is_documentation()
        || address.is_link_local()
        || address.is_loopback()
        || address.is_multicast()
        || address.is_private()
        || address.is_unspecified()
        || octets[0] == 0
        || octets[0] >= 240
        || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
        || (octets[0] == 192 && octets[1] == 88 && octets[2] == 99)
        || (octets[0] == 198 && matches!(octets[1], 18 | 19)))
}
fn ipv6_is_allowed(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    !(address.is_loopback()
        || address.is_multicast()
        || address.is_unspecified()
        || address.to_ipv4_mapped().is_some_and(|mapped| !ipv4_is_allowed(mapped))
        || segments[0] & 0xfe00 == 0xfc00
        || segments[0] & 0xffc0 == 0xfe80
        || (segments[0] == 0x2001 && segments[1] == 0x0db8))
}
fn read_bounded(reader: impl Read, maximum: usize, label: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(u64::try_from(maximum).unwrap_or(u64::MAX).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|why| format!("failed to read {label} — {why}"))
        .and_then(|_| match bytes.len() <= maximum {
            | true => Ok(bytes),
            | false => Err(format!("{label} byte limit exceeded")),
        })
}
fn restricted_get(input: &str, capability: &NetworkCapability, state: &Mutex<CapabilityState>) -> Result<String, String> {
    reqwest::Url::parse(input)
        .map_err(|why| format!("invalid network-read URL — {why}"))
        .and_then(|url| {
            match (
                url.scheme(),
                url.username().is_empty(),
                url.password().is_none(),
                url.fragment().is_none(),
            ) {
                | ("https", true, true, true) => Ok(url),
                | _ => Err("network-read requires an HTTPS URL without credentials or fragments".to_string()),
            }
        })
        .and_then(|url| {
            authority(&url)
                .ok_or_else(|| "network-read URL has no authority".to_string())
                .and_then(
                    |requested| match capability.hosts.iter().any(|host| host.eq_ignore_ascii_case(&requested)) {
                        | true => Ok((url, requested)),
                        | false => Err(format!("network-read authority `{requested}` is not allowlisted")),
                    },
                )
        })
        .and_then(|(url, requested)| {
            url.host_str()
                .ok_or_else(|| "network-read URL has no host".to_string())
                .map(str::to_string)
                .and_then(|host| {
                    let port = url.port_or_known_default().unwrap_or(443);
                    (host.as_str(), port)
                        .to_socket_addrs()
                        .map_err(|why| format!("failed to resolve `{requested}` — {why}"))
                        .map(|addresses| (host, addresses.collect::<Vec<SocketAddr>>()))
                })
                .and_then(
                    |(host, addresses)| match !addresses.is_empty() && addresses.iter().all(|address| ip_is_allowed(address.ip())) {
                        | true => Ok((url, host, addresses)),
                        | false => Err(format!("network-read authority `{requested}` resolves to a prohibited address")),
                    },
                )
        })
        .and_then(|(url, host, addresses)| {
            state
                .lock()
                .map_err(|why| format!("script capability state is unavailable — {why}"))
                .and_then(|mut state| {
                    state.network_requests = state.network_requests.saturating_add(1);
                    match state.network_requests <= capability.max_requests {
                        | true => Ok(()),
                        | false => Err("network-read request limit exceeded".to_string()),
                    }
                })
                .and_then(|()| {
                    reqwest::blocking::Client::builder()
                        .connect_timeout(Duration::from_millis(capability.timeout))
                        .no_proxy()
                        .redirect(reqwest::redirect::Policy::none())
                        .resolve_to_addrs(&host, &addresses)
                        .timeout(Duration::from_millis(capability.timeout))
                        .build()
                        .map_err(|why| format!("failed to construct network-read client — {why}"))
                        .and_then(|client| client.get(url).send().map_err(|why| format!("network-read request failed — {why}")))
                })
        })
        .and_then(|response| {
            let status = response.status();
            match status.is_success() {
                | false => Err(format!("network-read returned HTTP {status}")),
                | true => read_bounded(response, capability.max_response_bytes, "network response")
                    .and_then(|bytes| String::from_utf8(bytes).map_err(|why| format!("network-read response is not UTF-8 — {why}"))),
            }
        })
}
fn rhai_error(message: String) -> Box<EvalAltResult> {
    EvalAltResult::ErrorRuntime(Dynamic::from(message), Position::NONE).into()
}
