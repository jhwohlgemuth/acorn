//! Support for working with [BagIt](https://tools.ietf.org/html/rfc8493) format data.
//!
//! BagIt is a standardized file packaging format designed for digital preservation and data transfer.
//! This module provides utilities to create, verify, and work with BagIt packages containing research
//! activity data, metadata, and supporting documentation.
//!
use crate::io::{archive, file_checksum, files_all, read_file, write_file, ApiResult, StringConversion};
use crate::prelude::{copy, create_dir_all, HashSet, Path, PathBuf};
use crate::util::{ChecksumAlgorithm, Label, MimeType};
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
impl Bag {
    /// Verify the completeness and correctness of a bag
    /// ### Checks
    /// - `bagit.txt` exists
    /// - `manifest-<algorithm>.txt` exists
    /// - `data/` exists
    /// - Checksum values in manifest file matches checksum values of files in `data/`
    pub fn verify<P>(path: P) -> ApiResult<()>
    where
        P: Into<PathBuf>,
    {
        path.into()
            .canonicalize()
            .map_err(|why| eyre!("Failed to resolve BagIt directory - {why}"))
            .and_then(
                |base_directory| match (base_directory.join("bagit.txt").is_file(), base_directory.join("data").is_dir()) {
                    | (true, true) => Ok(base_directory),
                    | _ => Err(eyre!("Missing required BagIt declaration or payload directory")),
                },
            )
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
                        .try_for_each(|(name, algorithm)| verify_manifest(&base_directory, name, algorithm)),
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
        let Bag { info, version, encoding, .. } = self;
        let dest = destination.clone().into();
        let payload_directory = dest.join("data");
        match create_dir_all(payload_directory.clone()) {
            | Ok(_) => {
                let base = PathBuf::from(self.base_directory.clone());
                let bag = self.with_payload();
                let checksum_algorithm: &'static ring::digest::Algorithm = self.checksum_algorithm.clone().into();
                bag.payload.clone().into_iter().for_each(|x| {
                    let payload = PathBuf::from(x);
                    let from = base.join(&payload);
                    let to = dest.join("data").join(&payload);
                    let created = to.parent().map_or(Ok(()), create_dir_all);
                    match created {
                        | Ok(()) => {
                            if let Err(why) = copy(from.clone(), to.clone()) {
                                error!(
                                    from = from.to_absolute_path(),
                                    to = to.to_absolute_path(),
                                    "=> {} Copy - {why}",
                                    Label::fail()
                                );
                            }
                        }
                        | Err(why) => {
                            error!(
                                directory = to.to_path_buf().to_absolute_path(),
                                "=> {} Create parent - {why}",
                                Label::fail()
                            );
                        }
                    }
                });
                let bag_declaration_content = format!("BagIt-Version: {version}\nTag-File-Character-Encoding: {encoding}\n");
                let payload_manifest_content = bag
                    .payload
                    .into_iter()
                    .fold(String::new(), |acc, x| {
                        let payload = PathBuf::from(x.clone());
                        let to = dest.join("data").join(&payload);
                        match file_checksum(to.clone(), Some(checksum_algorithm)) {
                            | Some(checksum) => {
                                format!("{acc}{checksum} data/{x}{LINE_ENDING}")
                            }
                            | None => {
                                warn!(payload = payload.to_absolute_path(), "=> {} Calculate checksum", Label::fail());
                                acc
                            }
                        }
                    })
                    .replace("\\", "/");
                write_file(dest.clone().join("bagit.txt"), bag_declaration_content)
                    .map(|_| dest.clone())
                    .and_then(|_| match info.clone() {
                        | Some(bag_info) => bag_info.save(destination.clone()),
                        | None => Ok(dest.clone()),
                    })
                    .and_then(|_| {
                        let file_name = format!("manifest-{}.txt", self.checksum_algorithm);
                        let file_path = dest.clone().join(&file_name);
                        write_file(file_path, payload_manifest_content).map(|_| dest.clone())
                    })
                    .and_then(|_| archive(dest.clone(), archive_destination, archive_format))
            }
            | Err(why) => Err(eyre!("Failed to create bag - {why}")),
        }
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
impl From<ChecksumAlgorithm> for &'static ring::digest::Algorithm {
    fn from(value: ChecksumAlgorithm) -> Self {
        match value {
            | ChecksumAlgorithm::Sha512 => &ring::digest::SHA512,
            | _ => &ring::digest::SHA256,
        }
    }
}
fn verify_manifest(base: &Path, name: &str, algorithm: ChecksumAlgorithm) -> ApiResult<()> {
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
                            .map(|(checksum, path)| (checksum.to_string(), path.to_string()))
                            .ok_or_else(|| eyre!("Malformed BagIt manifest entry: {line}"))
                    })
                    .collect::<ApiResult<Vec<_>>>()
            })
        })
        .and_then(|entries| {
            let listed = entries.iter().map(|(_, path)| PathBuf::from(path)).collect::<HashSet<_>>();
            let actual = files_all(base.join("data"), None::<Vec<String>>)
                .into_iter()
                .filter(|path| path.is_file())
                .filter_map(|path| path.strip_prefix(base).ok().map(Path::to_path_buf))
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
                file_checksum(base.join(&relative), Some(digest))
                    .map(|checksum| checksum.checksum_value.to_ascii_lowercase())
                    .ok_or_else(|| eyre!("Failed to checksum BagIt payload: {relative}"))
                    .and_then(|calculated| match calculated == expected.to_ascii_lowercase() {
                        | true => Ok(()),
                        | false => Err(eyre!("Checksum mismatch in payload file = {relative}")),
                    })
            })
        })
}
