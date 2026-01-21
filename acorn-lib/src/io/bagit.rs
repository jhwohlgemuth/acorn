//! Support for working with [BagIt](https://tools.ietf.org/html/rfc8493) format data.
//!
//! BagIt is a standardized file packaging format designed for digital preservation and data transfer.
//! This module provides utilities to create, verify, and work with BagIt packages containing research
//! activity data, metadata, and supporting documentation.
//!
use crate::io::{files_all, ToAbsoluteString};
use crate::prelude::{create_dir_all, PathBuf};
use crate::util::Label;
use bon::Builder;
use core::fmt;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::error;

/// Trait for persisting BagIt data
pub trait Save {
    /// Persist BagIt data to a given path
    fn save<P>(&self, destination: P)
    where
        P: Into<PathBuf>;
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
#[display("Bag-Info: {date} {organization}")]
pub struct BagInfo {
    /// Organization transferring the content (Source-Organization)
    pub organization: String,
    /// Mailing address of the source organization (Organization-Address)
    pub organization_address: String,
    /// Person responsible for the content transfer (Contact-Name)
    pub contact_name: String,
    /// International format telephone number of responsible person (Contact-Phone)
    pub contact_phone: String,
    /// Fully qualified email address of responsible person (Contact-Email)
    pub contact_email: String,
    /// Brief explanation of contents and provenance (External-Description)
    pub description: String,
    /// Date (YYYY-MM-DD) that content was prepared for transfer (Bagging-Date)
    pub date: String,
    /// Sender-supplied identifier for the bag (External-Identifier)
    pub identifier: String,
    /// Size or approximate size of the bag with abbreviation (Bag-Size)
    pub size: String,
    /// Bag count as (N, T) tuple where N is ordinal position and T is total (Bag-Count)
    /// T is None if unknown ("?")
    pub count: (u32, Option<u32>),
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
    fn save<P>(&self, destination: P)
    where
        P: Into<PathBuf>,
    {
        let payload_directory = destination.into().join("data");
        match create_dir_all(payload_directory) {
            | Ok(_) => {
                // TODO: Copy payload files from self.base_directory to destination/data
                let bag = self.with_payload();
                dbg!(&bag);
                // TODO: Create manifest file and populate with payload file checksums
                // TODO: Create bagit.txt and populate with BagIt fields
                // TODO: Create bag-info.txt and populate with BagInfo fields
                // TODO: Archive self.base_directory
            }
            | Err(why) => error!(directory = self.base_directory, "=> {} Create - {why}", Label::fail()),
        }
    }
}
impl fmt::Display for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bag v{} ({}): {}", self.version, self.checksum_algorithm, self.base_directory)
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
