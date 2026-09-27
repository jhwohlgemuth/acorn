//! Support for working with [BagIt](https://tools.ietf.org/html/rfc8493) format data.
//!
//! BagIt is a standardized file packaging format designed for digital preservation and data transfer.
//! This module provides utilities to create, verify, and work with BagIt packages containing research
//! activity data, metadata, and supporting documentation.
//!
use crate::{
    io::{archive, files_all, read_file, write_file, ApiResult, StringConversion},
    util::constants::app::{
        BAGIT_DECLARATION_FILE, BAGIT_ENCODING_LABEL, BAGIT_PAYLOAD_DIRECTORY, BAGIT_SUPPORTED_ENCODINGS, BAGIT_SUPPORTED_VERSIONS,
        BAGIT_VERSION_LABEL,
    },
};
use acorn_core::{
    prelude::HashSet,
    util::{ChecksumAlgorithm, MimeType},
};
use acorn_host::{
    fs::{file_checksum, SafePath},
    terminal::Label,
};
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::fmt;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
};
use tracing::{error, warn};

const LINE_ENDING: &str = if cfg!(windows) { "\r\n" } else { "\n" };
/// Trait for persisting BagIt data
pub trait Save {
    /// Persist BagIt data to a given path
    fn save<P>(&self, destination: P) -> ApiResult<PathBuf>
    where
        P: Into<PathBuf> + Clone;
    /// Persists BagIt data using a selected archive format and output path.
    fn save_as<P>(&self, destination: P, archive_format: MimeType, archive_destination: Option<PathBuf>) -> ApiResult<PathBuf>
    where
        P: Into<PathBuf> + Clone;
}
/// Container structure for details of BagIt formatted data.
///
/// A bag consists of a base directory containing payload files in a `data/` subdirectory,
/// along with manifest files and other tag files that provide metadata and checksums for
/// validation. This struct represents the core bag configuration defined in `bagit.txt`.
///
/// See <https://datatracker.ietf.org/doc/html/draft-kunze-bagit> for details
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init)]
pub struct Bag {
    /// Root directory path of the bag containing all bag files and directories
    pub base_directory: String,
    /// List of payload file paths relative to the bag's data/ directory
    #[builder(default = Vec::new())]
    pub payload: Vec<String>,
    /// Checksum algorithm used in payload and tag manifest files
    #[builder(default)]
    pub checksum_algorithm: ChecksumAlgorithm,
    /// BagIt specification version (e.g., "1.0")
    #[builder(default = String::from("1.0"))]
    pub version: String,
    /// Character encoding for tag files (recommended: "UTF-8")
    #[builder(default = String::from("UTF-8"))]
    pub encoding: String,
    /// Bag-Info metadata from `bag-info.txt`
    pub info: Option<BagInfo>,
}
/// Metadata describing the bag and its payload from `bag-info.txt`.
///
/// Bag-Info metadata elements are intended primarily for human use and are optional per the BagIt
/// specification. These fields correspond to the reserved metadata element names from RFC 8493.
///
/// See <https://datatracker.ietf.org/doc/html/draft-kunze-bagit> for details
#[derive(Builder, Clone, Debug, Display, Serialize, Deserialize, JsonSchema)]
#[display("Bag-Info")]
#[builder(start_fn = init)]
pub struct BagInfo {
    /// Organization transferring the content (Source-Organization)
    pub organization: Option<Vec<String>>,
    /// Mailing address of the source organization (Organization-Address)
    pub organization_address: Option<Vec<String>>,
    /// Person responsible for the content transfer (Contact-Name)
    pub contact_name: Option<Vec<String>>,
    /// International format telephone number of responsible person (Contact-Phone)
    pub contact_phone: Option<Vec<String>>,
    /// Fully qualified email address of responsible person (Contact-Email)
    pub contact_email: Option<Vec<String>>,
    /// Brief explanation of contents and provenance (External-Description)
    pub description: Option<Vec<String>>,
    /// Date (YYYY-MM-DD) that content was prepared for transfer (Bagging-Date)
    pub date: Option<String>,
    /// Sender-supplied identifier for the bag (External-Identifier)
    pub identifier: Option<Vec<String>>,
    /// Size or approximate size of the bag with abbreviation (Bag-Size)
    pub size: Option<String>,
    /// Bag count values as (N, T) tuples where N is ordinal position and T is total
    /// T is None if unknown ("?")
    pub count: Option<Vec<(u32, Option<u32>)>>,
}
/// A validated BagIt declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tag {
    /// BagIt specification version declared by the tag file.
    field: String,
    /// Tag file character encoding the tag file declared.
    encoding: String,
}

impl Bag {
    /// Verify the completeness and correctness of a bag
    /// ### Checks
    /// - `bagit.txt` exists and declares a supported version and tag file encoding
    /// - `manifest-<algorithm>.txt` exists for SHA-256 or SHA-512
    /// - `data/` exists
    /// - Checksum values in manifest file matches checksum values of files in `data/`
    pub fn verify<P>(path: P) -> ApiResult<()>
    where
        P: Into<PathBuf>,
    {
        Self::verify_payload(path).map(|_| ())
    }
    /// Verify a bag and return the absolute path of its verified `data/` payload directory.
    ///
    /// Verification completes before the payload is returned so callers can publish payload files only after the whole bag has been proven complete and correct.
    pub fn verify_payload<P>(path: P) -> ApiResult<PathBuf>
    where
        P: Into<PathBuf>,
    {
        path.into()
            .canonicalize()
            .map_err(|why| eyre!("Failed to resolve BagIt directory - {why}"))
            .and_then(|base_directory| {
                match (
                    base_directory.join(BAGIT_DECLARATION_FILE).is_file(),
                    base_directory.join(BAGIT_PAYLOAD_DIRECTORY).is_dir(),
                ) {
                    | (true, true) => Ok(base_directory),
                    | _ => Err(eyre!("Missing required BagIt declaration or payload directory")),
                }
            })
            .and_then(|base_directory| {
                read_file(base_directory.join(BAGIT_DECLARATION_FILE))
                    .and_then(|declaration| Bag::declaration(&declaration).map(|_| ()))
                    .map(|()| base_directory)
            })
            .and_then(|base_directory| {
                let manifests = [
                    ("manifest-sha256.txt", ChecksumAlgorithm::Sha256),
                    ("manifest-sha512.txt", ChecksumAlgorithm::Sha512),
                ]
                .into_iter()
                .filter(|(name, _)| base_directory.join(name).is_file())
                .collect::<Vec<_>>();
                match manifests.is_empty() {
                    | true => Err(eyre!("A complete SHA-256 or SHA-512 BagIt manifest is required")),
                    | false => manifests
                        .into_iter()
                        .try_for_each(|(name, algorithm)| verify_manifest(&base_directory, name, algorithm))
                        .map(|()| base_directory.join(BAGIT_PAYLOAD_DIRECTORY)),
                }
            })
    }
    /// Populate payload file listing from files present in self.base_directory "data" directory
    pub fn with_payload(&self) -> Self {
        let base_directory = PathBuf::from(self.base_directory.clone());
        let payload = files_all(base_directory.clone(), None::<Vec<String>>)
            .into_iter()
            .filter(|x| x.is_file())
            .flat_map(|x| x.strip_prefix(base_directory.to_absolute_path()).ok().map(|p| p.to_path_buf()))
            .map(|x| x.display().to_string())
            .collect::<Vec<_>>();
        let Bag {
            checksum_algorithm,
            version,
            encoding,
            info,
            ..
        } = self;
        Bag::init()
            .base_directory(self.base_directory.clone())
            .checksum_algorithm(checksum_algorithm.clone())
            .version(version.clone())
            .encoding(encoding.clone())
            .maybe_info(info.clone())
            .payload(payload)
            .build()
    }

    /// Write the bag declaration, tag files, payload manifests, and payload copies into `destination`.
    ///
    /// The written directory is a complete, self-verifying bag directory that [`Bag::verify`] accepts;
    /// no archive is created by this method
    pub fn write(&self, destination: &Path) -> ApiResult<()> {
        let Bag {
            info,
            base_directory,
            checksum_algorithm,
            version,
            encoding,
            ..
        } = self;
        let payload_directory = destination.join(BAGIT_PAYLOAD_DIRECTORY);
        let base = PathBuf::from(base_directory.clone());
        let payloads = self.with_payload().payload.iter().map(SafePath::new).collect::<Result<Vec<_>, _>>();
        payloads.map_err(Into::<color_eyre::Report>::into).and_then(|payloads| {
            create_dir_all(payload_directory.clone())
                .map_err(|why| eyre!("Failed to create bag - {why}"))
                .and_then(|()| {
                    payloads.iter().try_for_each(|payload| {
                        let from = base.join(payload);
                        payload
                            .materialize_under(&payload_directory)
                            .map_err(Into::<color_eyre::Report>::into)
                            .and_then(|to| {
                                to.parent()
                                    .map_or(Ok(()), |parent| create_dir_all(parent).map_err(Into::into))
                                    .and_then(|()| copy(&from, &to).map(|_| ()).map_err(Into::into))
                            })
                            .map_err(|why: color_eyre::Report| {
                                error!(
                                    from = from.to_absolute_path(),
                                    to = payload_directory.to_absolute_path(),
                                    "=> {} Copy - {why}",
                                    Label::fail()
                                );
                                why
                            })
                    })
                })
                .and_then(|()| {
                    let declaration = format!("{BAGIT_VERSION_LABEL}: {version}\n{BAGIT_ENCODING_LABEL}: {encoding}\n");
                    let digest: &'static ring::digest::Algorithm = checksum_algorithm.clone().into();
                    let manifest = payloads
                        .into_iter()
                        .fold(String::new(), |acc, payload| {
                            let target = payload_directory.join(&payload);
                            match file_checksum(target.clone(), Some(digest)) {
                                | Ok(checksum) => format!("{acc}{checksum} {BAGIT_PAYLOAD_DIRECTORY}/{}{LINE_ENDING}", payload.as_path().display()),
                                | Err(_) => {
                                    warn!(payload = target.to_absolute_path(), "=> {} Calculate checksum", Label::fail());
                                    acc
                                }
                            }
                        })
                        .replace("\\", "/");
                    write_file(destination.join(BAGIT_DECLARATION_FILE), declaration)
                        .and_then(|()| match info.clone() {
                            | Some(bag_info) => bag_info.save(destination).map(|_| ()),
                            | None => Ok(()),
                        })
                        .and_then(|()| write_file(destination.join(format!("manifest-{checksum_algorithm}.txt")), manifest))
                })
        })
    }

    /// Parse and validate a `bagit.txt` declaration.
    ///
    /// Both the version and encoding labels are required. The version must be one ACORN supports and the encoding must be UTF-8, in any of its accepted spellings.
    pub fn declaration(content: &str) -> ApiResult<Tag> {
        let version = Tag::value(content, BAGIT_VERSION_LABEL).and_then(|version| match BAGIT_SUPPORTED_VERSIONS.contains(&version.as_str()) {
            | true => Ok(version),
            | false => Err(eyre!(
                "Unsupported BagIt version '{version}' — supported versions are {}",
                BAGIT_SUPPORTED_VERSIONS.join(", ")
            )),
        });
        version.and_then(|version| {
            Tag::value(content, BAGIT_ENCODING_LABEL)
                .and_then(|encoding| Tag::encoding_supported(&encoding))
                .map(|encoding| Tag { field: version, encoding })
        })
    }
}
impl fmt::Display for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bag v{} ({}): {}", self.version, self.checksum_algorithm, self.base_directory)
    }
}
impl Save for Bag {
    /// Persist files in a given directory as a archived ("zipped") BagIt format package
    fn save<P>(&self, destination: P) -> ApiResult<PathBuf>
    where
        P: Into<PathBuf> + Clone,
    {
        self.save_as(destination, MimeType::Zip, None)
    }
    fn save_as<P>(&self, destination: P, archive_format: MimeType, archive_destination: Option<PathBuf>) -> ApiResult<PathBuf>
    where
        P: Into<PathBuf> + Clone,
    {
        let dest = destination.clone().into();
        self.write(&dest).and_then(|()| archive(dest, archive_destination, archive_format))
    }
}
impl BagInfo {
    /// Enumerate BagIt reserved metadata elements as ordered key/value pairs
    pub fn entries(&self) -> Vec<(String, String)> {
        let BagInfo {
            organization,
            organization_address,
            contact_name,
            contact_phone,
            contact_email,
            description,
            date,
            identifier,
            size,
            count,
        } = self;
        let repeatable = [
            ("Source-Organization", organization),
            ("Organization-Address", organization_address),
            ("Contact-Name", contact_name),
            ("Contact-Phone", contact_phone),
            ("Contact-Email", contact_email),
            ("External-Description", description),
            ("External-Identifier", identifier),
        ]
        .into_iter()
        .flat_map(|(key, values)| {
            values
                .iter()
                .flat_map(|items| items.iter())
                .map(move |value| (key.to_string(), value.clone()))
        });
        let count = count.iter().flat_map(|items| items.iter()).map(|(index, total)| {
            let total = total.map_or_else(|| "?".to_string(), |value| value.to_string());
            ("Bag-Count".to_string(), format!("{index} of {total}"))
        });
        let single = [("Bagging-Date", date.clone()), ("Bag-Size", size.clone())]
            .into_iter()
            .filter_map(|(key, value)| value.map(|v| (key.to_string(), v)));
        repeatable.chain(count).chain(single).collect()
    }
}
impl Default for BagInfo {
    fn default() -> Self {
        BagInfo::init().build()
    }
}
impl Save for BagInfo {
    fn save<P>(&self, destination: P) -> ApiResult<PathBuf>
    where
        P: Into<PathBuf> + Clone,
    {
        let dest = destination.clone().into();
        let content = self
            .entries()
            .into_iter()
            .fold(String::new(), |acc, (key, value)| format!("{acc}{key}: {value}{LINE_ENDING}"));
        write_file(dest.join("bag-info.txt"), content).map(|_| dest)
    }
    fn save_as<P>(&self, destination: P, _archive_format: MimeType, _archive_destination: Option<PathBuf>) -> ApiResult<PathBuf>
    where
        P: Into<PathBuf> + Clone,
    {
        self.save(destination)
    }
}
impl Tag {
    /// BagIt specification version declared by the tag file.
    pub fn field(&self) -> &str {
        &self.field
    }
    /// Tag file character encoding the tag file declared.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
    /// Read the single value that a tag file declares for one label.
    ///
    /// Blank lines and `#` comments are skipped.
    /// Every remaining entry must be `Label: Value`, and a missing, empty, or repeated label is rejected.
    pub fn value(content: &str, label: &str) -> ApiResult<String> {
        let expected = label.to_ascii_lowercase();
        let entries = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                line.split_once(':')
                    .map(|(candidate, value)| (candidate.trim().to_ascii_lowercase(), value.trim().to_string()))
                    .ok_or_else(|| eyre!("Malformed BagIt declaration entry: {line}"))
            })
            .collect::<Result<Vec<_>, Report>>();
        entries.and_then(|entries| {
            let values = entries
                .into_iter()
                .filter(|(candidate, _)| candidate == &expected)
                .map(|(_, value)| value)
                .collect::<Vec<_>>();
            match values.as_slice() {
                | [value] if !value.is_empty() => Ok(value.clone()),
                | [] => Err(eyre!("BagIt declaration is missing {label}")),
                | _ => Err(eyre!("BagIt declaration repeats {label}")),
            }
        })
    }
    /// Accept a tag file character encoding, compared without `-` or `_` separators.
    ///
    /// ACORN only reads UTF-8, so any spelling of it is accepted and returned unchanged.
    pub fn encoding_supported(encoding: &str) -> ApiResult<String> {
        let normalized = encoding.replace(['-', '_'], "").to_ascii_uppercase();
        match BAGIT_SUPPORTED_ENCODINGS.contains(&normalized.as_str()) {
            | true => Ok(encoding.to_string()),
            | false => Err(eyre!("Unsupported BagIt tag file encoding '{encoding}' - only UTF-8 is supported")),
        }
    }
}
/// Verify one payload manifest against the files it must describe.
fn verify_manifest(base: &Path, name: &str, algorithm: ChecksumAlgorithm) -> ApiResult<()> {
    // `files_all` reports canonical paths without the Windows verbatim prefix, so payload
    // entries are compared against the same spelling rather than the caller's spelling.
    let root = PathBuf::from(base.to_path_buf().to_absolute_path());
    let expected_length = match algorithm {
        | ChecksumAlgorithm::Sha256 => Ok(64),
        | ChecksumAlgorithm::Sha512 => Ok(128),
        | _ => Err(eyre!("Unsupported BagIt manifest algorithm")),
    };
    read_file(base.join(name))
        .and_then(|content| {
            expected_length.and_then(|expected_length| {
                content
                    .lines()
                    .map(|line| {
                        line.find(char::is_whitespace)
                            .map(|index| (&line[..index], line[index..].trim_start()))
                            .filter(|(checksum, path)| {
                                let valid_length = checksum.len() == expected_length;
                                let valid_checksum = checksum.bytes().all(|byte| byte.is_ascii_hexdigit());
                                let valid_path = path.starts_with("data/");
                                valid_length && valid_checksum && valid_path
                            })
                            .map(|(checksum, path)| (checksum.to_string(), path))
                            .ok_or_else(|| eyre!("Malformed BagIt manifest entry: {line}"))
                            .and_then(|(checksum, path)| {
                                SafePath::new(path)
                                    .map(|path| (checksum, path))
                                    .map_err(|why| eyre!("Unsafe BagIt manifest entry: {line} — {why}"))
                            })
                    })
                    .collect::<ApiResult<Vec<_>>>()
            })
        })
        .and_then(|entries| {
            let listed = entries.iter().map(|(_, path)| path.clone()).collect::<HashSet<_>>();
            let actual = files_all(root.join(BAGIT_PAYLOAD_DIRECTORY), None::<Vec<String>>)
                .into_iter()
                .filter(|path| path.is_file())
                .filter_map(|path| path.strip_prefix(&root).ok().map(Path::to_path_buf))
                .filter_map(|path| SafePath::new(path).ok())
                .collect::<HashSet<_>>();
            match (listed.len() == entries.len(), listed == actual) {
                | (false, _) => Err(eyre!("Duplicate payload path in BagIt manifest")),
                | (_, false) => Err(eyre!("BagIt manifest does not list the complete payload")),
                | _ => Ok(entries),
            }
        })
        .and_then(|entries| {
            let digest: &'static ring::digest::Algorithm = algorithm.into();
            entries.into_iter().try_for_each(|(expected, relative)| {
                relative
                    .materialize_under(&root)
                    .map_err(Into::<color_eyre::Report>::into)
                    .and_then(|path| file_checksum(path, Some(digest)).map_err(Into::<color_eyre::Report>::into))
                    .map(|checksum| checksum.checksum_value.to_ascii_lowercase())
                    .and_then(|calculated| match calculated == expected.to_ascii_lowercase() {
                        | true => Ok(()),
                        | false => Err(eyre!("Checksum mismatch in payload file = {}", relative.as_path().display())),
                    })
            })
        })
}
