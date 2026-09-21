//! Deterministic MCP Bundle planning, creation, and inspection.
use super::{ToolCallContext, ToolRegistry};
use crate::{
    error::ApiResult,
    io::{CommandTemplate, DeterministicZipEntry, Platform},
    util::constants::app::{APPLICATION, MCPB_MANIFEST_VERSION, MCPB_SPEC_REVISION, ORGANIZATION, QUALIFIER},
};
use acorn_core::prelude::{String, Vec};
use acorn_host::fs::{file_checksum, SafePath};
use alloc::collections::{BTreeMap, BTreeSet};
use color_eyre::eyre::{eyre, Report};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{remove_file, rename, File},
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

#[derive(Debug)]
enum McpbError {
    DestinationExists,
    DestinationInvalid,
    InspectOutput(Box<str>),
    ParentMissing(PathBuf),
}
/// Supported MCPB server package kinds.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    /// A bundled native executable.
    Binary,
}
/// MCPB author metadata emitted by ACORN.
#[derive(bon::Builder, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Author {
    /// Author name.
    pub name: String,
}
/// MCPB creation inputs.
#[derive(Clone, Debug)]
pub struct BundleOptions {
    /// Final archive destination.
    pub output: PathBuf,
    /// Executable to package.
    pub binary: PathBuf,
    /// Target platform.
    pub platform: Platform,
    /// Whether an existing regular output file may be replaced.
    pub overwrite: bool,
}
/// A filesystem-free MCPB creation plan.
#[derive(Clone, Debug)]
pub struct BundlePlan {
    /// Source executable.
    pub binary: PathBuf,
    /// Final archive destination.
    pub output: PathBuf,
    /// Target platform.
    pub platform: Platform,
    /// Whether an existing regular output file may be replaced.
    pub overwrite: bool,
    /// Deterministic manifest.
    pub manifest: Manifest,
}
/// Result of materializing an MCPB archive.
#[derive(Clone, Debug)]
pub struct BundleReport {
    /// Final archive destination.
    pub output: PathBuf,
    /// Emitted manifest.
    pub manifest: Manifest,
    /// Lowercase SHA-256 of the archive.
    pub sha256: String,
    /// Archive size in bytes.
    pub size: u64,
}
/// MCPB compatibility declaration.
#[derive(bon::Builder, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Compatibility {
    /// Supported operating systems.
    pub platforms: Vec<Platform>,
}
/// MCPB manifest subset emitted by ACORN.
#[derive(bon::Builder, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Manifest {
    /// MCPB specification version.
    pub manifest_version: String,
    /// Machine-readable package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Package author.
    pub author: Author,
    /// Source repository in the MCPB manifest wire format.
    pub repository: ManifestRepository,
    /// Project homepage.
    pub homepage: String,
    /// Package documentation.
    pub documentation: String,
    /// SPDX license identifier.
    pub license: String,
    /// Server launch metadata.
    pub server: Server,
    /// Statically declared safe tool catalog.
    pub tools: Vec<ManifestTool>,
    /// Whether runtime discovery may add tools.
    pub tools_generated: bool,
    /// Platform compatibility.
    pub compatibility: Compatibility,
    /// Reviewed MCPB source revision used for this schema.
    #[serde(rename = "_meta")]
    pub metadata: BTreeMap<String, serde_json::Value>,
}
/// Source repository metadata in the MCPB manifest wire format.
///
/// This remains distinct from [`acorn_core::location::Repository`], whose tagged
/// location representation is intentionally incompatible with the MCPB schema.
#[derive(bon::Builder, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct ManifestRepository {
    /// Repository kind.
    #[serde(rename = "type")]
    pub kind: String,
    /// Repository URL.
    pub url: String,
}
/// Static MCPB tool declaration.
#[derive(bon::Builder, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct ManifestTool {
    /// Canonical tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
}
/// MCPB server declaration.
#[derive(bon::Builder, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Server {
    /// Server package kind.
    #[serde(rename = "type")]
    pub kind: ServerType,
    /// Bundle-relative executable path.
    pub entry_point: String,
    /// MCP launch configuration.
    pub mcp_config: CommandTemplate,
}
impl BundlePlan {
    /// Create a validated, immutable bundle plan.
    pub fn new(options: &BundleOptions, registry: &ToolRegistry) -> ApiResult<Self> {
        Self::is_valid_destination(&options.output, options.overwrite)
            .and_then(|()| {
                options
                    .binary
                    .symlink_metadata()
                    .map_err(|why| eyre!("Failed to inspect MCPB binary — {why}"))
            })
            .and_then(|metadata| match metadata.file_type().is_symlink() || !metadata.is_file() {
                | true => Err(eyre!("MCPB binary must be a regular file: {}", options.binary.display())),
                | false => options
                    .platform
                    .binary_name()
                    .and_then(|_| match options.platform.has_valid_binary(&options.binary) {
                        | true => Ok(()),
                        | false => Err(eyre!("Windows MCPB binary must use the .exe suffix")),
                    }),
            })
            .and_then(|()| {
                options
                    .output
                    .file_name()
                    .map(|_| ())
                    .ok_or_else(|| eyre!("MCPB output must name a file"))
            })
            .and_then(|()| Self::manifest(options.platform, registry))
            .and_then(|manifest| {
                serde_json::to_vec(&manifest)
                    .map(|_| manifest)
                    .map_err(|why| eyre!("Failed to validate MCPB manifest — {why}"))
            })
            .map(|manifest| Self {
                binary: options.binary.clone(),
                output: options.output.clone(),
                platform: options.platform,
                overwrite: options.overwrite,
                manifest,
            })
    }
    fn is_valid_destination(output: &Path, overwrite: bool) -> ApiResult<()> {
        match output.symlink_metadata() {
            | Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => Err(McpbError::DestinationInvalid.into()),
            | Ok(_) if !overwrite => Err(McpbError::DestinationExists.into()),
            | Err(why) if why.kind() != ErrorKind::NotFound => Err(McpbError::InspectOutput(why.to_string().into_boxed_str()).into()),
            | Ok(_) | Err(_) => {
                let parent = output
                    .parent()
                    .filter(|value| !value.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
                match parent.is_dir() {
                    | true => Ok(()),
                    | false => Err(McpbError::ParentMissing(parent.to_path_buf()).into()),
                }
            }
        }
    }
    fn manifest(platform: Platform, registry: &ToolRegistry) -> ApiResult<Manifest> {
        platform.binary_name().map(|binary_name| {
            let entry_point = format!("server/{binary_name}");
            let author = Author::init().name(env!("CARGO_PKG_AUTHORS")).build();
            let compatibility_platform = match platform {
                | Platform::Win64 => Platform::Win32,
                | platform => platform,
            };
            let compatibility = Compatibility::builder().platforms(vec![compatibility_platform]).build();
            let mcp_config = CommandTemplate::init()
                .command(format!("${{__dirname}}/{entry_point}"))
                .args(vec!["serve".to_string(), "mcp".to_string()])
                .build();
            let repository = ManifestRepository::init().kind("git").url(env!("CARGO_PKG_REPOSITORY")).build();
            let server = Server::init()
                .kind(ServerType::Binary)
                .entry_point(entry_point)
                .mcp_config(mcp_config)
                .build();
            let context = ToolCallContext::mcp_with_policy(false, false, usize::MAX);
            let tools = registry
                .available_definitions(context)
                .into_iter()
                .map(|tool| ManifestTool::init().name(tool.name).description(tool.description).build())
                .collect();
            let metadata_value = serde_json::json!({ "mcpb_spec_revision": MCPB_SPEC_REVISION });
            let metadata_key = format!("{QUALIFIER}.{ORGANIZATION}.{APPLICATION}");
            let metadata = [(metadata_key, metadata_value)].into_iter().collect();
            Manifest::init()
                .manifest_version(MCPB_MANIFEST_VERSION)
                .name(APPLICATION)
                .version(env!("CARGO_PKG_VERSION"))
                .description(env!("CARGO_PKG_DESCRIPTION"))
                .author(author)
                .repository(repository)
                .homepage("https://acorn.ornl.gov")
                .documentation("https://acorn.ornl.gov/packages/mcpb.html")
                .license(env!("CARGO_PKG_LICENSE"))
                .server(server)
                .tools(tools)
                .tools_generated(false)
                .compatibility(compatibility)
                .metadata(metadata)
                .build()
        })
    }
    /// Write this plan to an unoccupied archive path.
    pub fn write(&self, path: &Path) -> ApiResult<PathBuf> {
        match path.exists() {
            | true => Err(eyre!("MCPB temporary output already exists: {}", path.display())),
            | false => serde_json::to_vec_pretty(&self.manifest)
                .map_err(|why| eyre!("Failed to serialize MCPB manifest — {why}"))
                .and_then(|manifest| {
                    self.platform.binary_name().and_then(|binary_name| {
                        [
                            DeterministicZipEntry::from_bytes("manifest.json", manifest, 0o644),
                            DeterministicZipEntry::from_file(
                                format!("server/{binary_name}"),
                                self.binary.clone(),
                                match self.platform == Platform::Win32 {
                                    | true => 0o644,
                                    | false => 0o755,
                                },
                            ),
                        ]
                        .into_iter()
                        .collect::<ApiResult<Vec<_>>>()
                    })
                })
                .and_then(|entries| DeterministicZipEntry::create(path, &entries)),
        }
    }
}
impl TryFrom<BundlePlan> for BundleReport {
    type Error = Report;
    fn try_from(plan: BundlePlan) -> Result<Self, Self::Error> {
        BundlePlan::is_valid_destination(&plan.output, plan.overwrite)
            .and_then(|()| {
                plan.output
                    .file_name()
                    .map(|name| plan.output.with_file_name(format!(".{}.acorn-tmp", name.to_string_lossy())))
                    .ok_or_else(|| eyre!("MCPB output must name a file"))
            })
            .and_then(|temporary| {
                let result = plan
                    .write(&temporary)
                    .and_then(|_| Manifest::inspect(&temporary).map(|_| ()))
                    .and_then(|()| commit_temporary(&temporary, &plan.output, plan.overwrite));
                match result {
                    | Ok(()) => plan
                        .output
                        .metadata()
                        .map_err(|why| eyre!("Failed to inspect MCPB output — {why}"))
                        .and_then(|metadata| {
                            file_checksum(&plan.output, None)
                                .map(|value| value.to_string().to_ascii_lowercase())
                                .map_err(Into::<color_eyre::Report>::into)
                                .map(|sha256| Self {
                                    output: plan.output,
                                    manifest: plan.manifest,
                                    sha256,
                                    size: metadata.len(),
                                })
                        }),
                    | Err(why) => {
                        remove_file(&temporary).ok();
                        Err(why)
                    }
                }
            })
    }
}
impl Manifest {
    /// Inspect and structurally validate an ACORN MCPB archive.
    pub fn inspect(path: &Path) -> ApiResult<Self> {
        File::open(path)
            .map_err(|why| eyre!("Failed to open MCPB archive — {why}"))
            .and_then(|file| ZipArchive::new(file).map_err(|why| eyre!("Invalid MCPB ZIP archive — {why}")))
            .and_then(Self::inspect_archive)
    }
    fn inspect_archive(mut archive: ZipArchive<File>) -> ApiResult<Self> {
        match archive.len() == 2 {
            | false => Err(eyre!("ACORN MCPB archive must contain exactly two entries")),
            | true => (0..archive.len())
                .map(|index| {
                    archive
                        .by_index(index)
                        .map_err(|why| eyre!("Failed to inspect MCPB entry — {why}"))
                        .and_then(|entry| {
                            SafePath::new(entry.name())
                                .map(|path| path.into_path_buf())
                                .map_err(|why| eyre!("Unsafe MCPB entry name — {why}"))
                        })
                })
                .collect::<ApiResult<BTreeSet<_>>>()
                .and_then(|names| {
                    let mut content = String::new();
                    archive
                        .by_name("manifest.json")
                        .map_err(|why| eyre!("MCPB manifest is missing — {why}"))
                        .and_then(|mut entry| {
                            entry
                                .read_to_string(&mut content)
                                .map_err(|why| eyre!("Failed to read MCPB manifest — {why}"))
                        })
                        .map(|_| (names, content))
                })
                .and_then(|(names, content)| {
                    serde_json::from_str::<Manifest>(&content)
                        .map_err(|why| eyre!("Invalid MCPB manifest — {why}"))
                        .map(|manifest| (names, manifest))
                })
                .and_then(|(names, manifest)| {
                    let identity_valid = manifest.manifest_version == MCPB_MANIFEST_VERSION && manifest.name == APPLICATION;
                    let expected = [PathBuf::from("manifest.json"), PathBuf::from(&manifest.server.entry_point)]
                        .into_iter()
                        .collect::<BTreeSet<_>>();
                    match (identity_valid, names == expected) {
                        | (false, _) => Err(eyre!("Unsupported MCPB manifest identity")),
                        | (_, false) => Err(eyre!("ACORN MCPB archive contains unexpected entries")),
                        | (true, true) => archive
                            .by_name(&manifest.server.entry_point)
                            .map_err(|why| eyre!("MCPB executable is missing — {why}"))
                            .and_then(|binary| match binary.is_dir() {
                                | true => Err(eyre!("MCPB executable entry is not a file")),
                                | false => Ok(manifest),
                            }),
                    }
                }),
        }
    }
}
impl core::error::Error for McpbError {}
impl fmt::Display for McpbError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::DestinationExists => write!(formatter, "MCPB output already exists; pass --clobber to replace it"),
            | Self::DestinationInvalid => write!(formatter, "MCPB output exists and is not a regular file"),
            | Self::InspectOutput(reason) => write!(formatter, "Failed to inspect MCPB output — {reason}"),
            | Self::ParentMissing(path) => write!(formatter, "MCPB output parent directory does not exist: {}", path.display()),
        }
    }
}
#[cfg(not(windows))]
fn commit_temporary(temporary: &Path, output: &Path, _overwrite: bool) -> ApiResult<()> {
    rename(temporary, output).map_err(|why| eyre!("Failed to commit MCPB output — {why}"))
}
#[cfg(windows)]
fn commit_temporary(temporary: &Path, output: &Path, overwrite: bool) -> ApiResult<()> {
    match overwrite && output.exists() {
        | false => rename(temporary, output).map_err(|why| eyre!("Failed to commit MCPB output — {why}")),
        | true => {
            let backup = output.with_extension("mcpb.acorn-backup");
            match backup.exists() {
                | true => Err(eyre!("MCPB replacement backup already exists: {}", backup.display())),
                | false => rename(output, &backup)
                    .map_err(|why| eyre!("Failed to stage existing MCPB output — {why}"))
                    .and_then(|()| match rename(temporary, output) {
                        | Ok(()) => remove_file(&backup).map_err(|why| eyre!("Failed to remove MCPB replacement backup — {why}")),
                        | Err(why) => rename(&backup, output)
                            .map_err(|restore| eyre!("Failed to commit MCPB output ({why}) and restore prior output ({restore})"))
                            .and_then(|()| Err(eyre!("Failed to commit MCPB output — {why}"))),
                    }),
            }
        }
    }
}
