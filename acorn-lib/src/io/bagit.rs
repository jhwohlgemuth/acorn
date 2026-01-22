//! Support for working with [BagIt](https://tools.ietf.org/html/rfc8493) format data.
//!
//! BagIt is a standardized file packaging format designed for digital preservation and data transfer.
//! This module provides utilities to create, verify, and work with BagIt packages containing research
//! activity data, metadata, and supporting documentation.
//!
use crate::io::{archive, file_checksum, files_all, write_file, ToAbsoluteString};
use crate::prelude::{copy, create_dir_all, Error, PathBuf};
use crate::util::Label;
use bon::Builder;
use core::fmt;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

const LINE_ENDING: &str = if cfg!(windows) { "\r\n" } else { "\n" };
/// Trait for persisting BagIt data
pub trait Save {
    /// Persist BagIt data to a given path
    fn save<P>(&self, destination: P) -> Result<PathBuf, Error>
    where
        P: Into<PathBuf> + Clone;
}
/// Cryptographic hash algorithm used for BagIt checksum verification.
///
/// Specifies the algorithm to use when generating and verifying file checksums
/// for bag integrity validation.
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, JsonSchema)]
pub enum ChecksumAlgorithm {
    /// SHA-256 hash (recommended)
    #[default]
    #[display("sha256")]
    SHA256,
    /// MD5 hash (least secure, legacy support)
    #[display("md5")]
    MD5,
    /// SHA-1 hash (deprecated, legacy support)
    #[display("sha1")]
    SHA1,
    /// SHA-512 hash (highest security)
    #[display("sha512")]
    SHA512,
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
    pub organization: Option<String>,
    /// Mailing address of the source organization (Organization-Address)
    pub organization_address: Option<String>,
    /// Person responsible for the content transfer (Contact-Name)
    pub contact_name: Option<String>,
    /// International format telephone number of responsible person (Contact-Phone)
    pub contact_phone: Option<String>,
    /// Fully qualified email address of responsible person (Contact-Email)
    pub contact_email: Option<String>,
    /// Brief explanation of contents and provenance (External-Description)
    pub description: Option<String>,
    /// Date (YYYY-MM-DD) that content was prepared for transfer (Bagging-Date)
    pub date: Option<String>,
    /// Sender-supplied identifier for the bag (External-Identifier)
    pub identifier: Option<String>,
    /// Size or approximate size of the bag with abbreviation (Bag-Size)
    pub size: Option<String>,
    /// Bag count as (N, T) tuple where N is ordinal position and T is total (Bag-Count)
    /// T is None if unknown ("?")
    pub count: Option<(u32, Option<u32>)>,
}
impl Bag {
    /// Populate payload file listing from files present in self.base_directory "data" directory
    pub fn with_payload(&self) -> Self {
        let base_directory = PathBuf::from(self.base_directory.clone());
        let payload_directory = base_directory.join("data");
        let payload = files_all(payload_directory, None)
            .into_iter()
            .filter(|x| x.is_file())
            .flat_map(|x| x.strip_prefix(base_directory.to_absolute_string()).ok().map(|p| p.to_path_buf()))
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
impl Save for Bag {
    /// Persist files in a given directory as a archived ("zipped") BagIt format package
    fn save<P>(&self, destination: P) -> Result<PathBuf, Error>
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
                    let to = dest.join(&payload);
                    if let Some(parent) = to.parent() {
                        if let Err(why) = create_dir_all(parent) {
                            error!(
                                directory = parent.to_path_buf().to_absolute_string(),
                                "=> {} Create - {why}",
                                Label::fail()
                            );
                            return;
                        }
                        if let Err(why) = copy(from.clone(), to.clone()) {
                            error!(
                                from = from.to_absolute_string(),
                                to = to.to_absolute_string(),
                                "=> {} Copy - {why}",
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
                        let to = dest.join(&payload);
                        match file_checksum(to.clone(), Some(checksum_algorithm)) {
                            | Some(checksum) => {
                                format!("{acc}{checksum} {x}{LINE_ENDING}")
                            }
                            | None => {
                                warn!(payload = payload.to_absolute_string(), "=> {} Calculate checksum", Label::fail());
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
                    .and_then(|_| archive(dest, None))
            }
            | Err(why) => Err(why),
        }
    }
}
impl fmt::Display for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bag v{} ({}): {}", self.version, self.checksum_algorithm, self.base_directory)
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
        let base = [
            ("Source-Organization", organization),
            ("Organization-Address", organization_address),
            ("Contact-Name", contact_name),
            ("Contact-Phone", contact_phone),
            ("Contact-Email", contact_email),
            ("External-Description", description),
            ("Bagging-Date", date),
            ("External-Identifier", identifier),
            ("Bag-Size", size),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.as_ref().map(|v| (key.to_string(), v.clone())));
        // Option as iterator: Some(item) -> one element, None -> empty.
        let count = count.iter().map(|(index, total)| {
            let total = total.map_or_else(|| "?".to_string(), |value| value.to_string());
            ("Bag-Count".to_string(), format!("{index} of {total}"))
        });
        base.chain(count).collect()
    }
}
impl Default for BagInfo {
    fn default() -> Self {
        BagInfo::init().build()
    }
}
impl Save for BagInfo {
    fn save<P>(&self, destination: P) -> Result<PathBuf, Error>
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
}
impl From<ChecksumAlgorithm> for &'static ring::digest::Algorithm {
    fn from(value: ChecksumAlgorithm) -> Self {
        match value {
            | ChecksumAlgorithm::SHA256 => &ring::digest::SHA256,
            | ChecksumAlgorithm::SHA512 => &ring::digest::SHA512,
            | _ => unimplemented!(),
        }
    }
}
impl From<&str> for ChecksumAlgorithm {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            | "sha256" => ChecksumAlgorithm::SHA256,
            | "md5" => ChecksumAlgorithm::MD5,
            | "sha1" => ChecksumAlgorithm::SHA1,
            | "sha512" => ChecksumAlgorithm::SHA512,
            | _ => ChecksumAlgorithm::default(),
        }
    }
}
impl From<String> for ChecksumAlgorithm {
    fn from(value: String) -> Self {
        ChecksumAlgorithm::from(value.as_str())
    }
}
