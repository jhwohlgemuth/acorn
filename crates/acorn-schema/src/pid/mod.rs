//! Persistent Identifiers (PID)
//!
//! Contains functions for working with persistent identifiers (PID) such as [`ORCID`], [`DOI`]s, and [RAiD](`raid`)s
//!
//! ### Features
//! - Best in class validation
//! - Convert persistent identifiers into standard formats
//! - Access the sub parts of a persistent identifier
//!
//! [RAiDs]: https://www.raid.org/
use crate::util::constants::{HTTP_URL, RE_ARXIV, RE_URL};
use acorn_core::prelude::alloc::{format, vec, String, ToString, Vec};
use acorn_core::util::{base32_crockford_decode, trim_unmatched_trailing_parentheses};
use bon::Builder;
use core::fmt;
#[cfg(feature = "std")]
use data_encoding::HEXLOWER;
#[cfg(feature = "std")]
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use strum::{EnumIs, EnumIter, IntoEnumIterator};

pub mod ark;
pub mod arxiv;
pub mod doi;
pub mod handle;
pub mod isbn;
pub mod isni;
pub mod orcid;
pub mod patent;
pub mod raid;
pub mod ror;
pub mod swhid;

pub use ark::ARK;
pub use arxiv::Arxiv;
pub use doi::DOI;
pub use handle::Handle;
pub use isbn::ISBN;
pub use isni::ISNI;
pub use orcid::ORCID;
pub use patent::Patent;
pub use raid::RAID;
pub use ror::ROR;
pub use swhid::SWHID;

const BETANUMERIC_DIGITS: &str = "0123456789bcdfghjkmnpqrstvwxz";

/// Add utility functions for working with beta numeric values
///
/// Mostly intended for working with [NCDA](`noid_check_digit`)
pub trait Betanumeric {
    /// Check if `self` is a betanumeric value
    fn is_betanumeric(&self) -> bool {
        false
    }
    /// Convert `self` into a betanumeric ordinal value
    /// ### Example
    /// > `w` -> `26`
    fn to_betanumeric_ordinal(&self) -> Option<usize>;
}
/// Provides common functions for working with persistent identifiers (PID)
pub trait PersistentIdentifier: fmt::Display {
    /// Create a new PID
    fn new() -> Self;
    /// Get standardized form of schema URI for a PID
    /// ### Examples
    /// - `https://doi.org`
    /// - `https://orcid.org`
    fn schema_uri(&self) -> String;
    /// Get PID identifier section
    /// ### Examples
    /// - `ark:1234/x5678` for [`ARK`]
    /// - `10.1234/5678` for [`DOI`]
    /// - `0000-0002-2057-9115` for [`ORCID`]
    fn identifier(&self) -> String;
    /// Get PID prefix (different interpretation depending on PID type)
    ///
    /// Not every PID type has a prefix, but generally every PID has a "first" part that can losely be considered a "prefix"
    fn prefix(&self) -> Option<String> {
        None
    }
    /// Get PID suffix (different interpretation depending on PID type)
    ///
    /// Not every PID type has a suffix, but generally every PID has a "second" part that can losely be considered a "suffix"
    fn suffix(&self) -> Option<String>;
    /// Get PID check digit (when applicable)
    fn check_digit(&self) -> Option<Vec<char>> {
        None
    }
    /// Get fully resolved URL of the PID with its schema URI
    fn url(&self) -> String {
        String::new()
    }
}
/// Add coercion to persistent identifier (PID) functionality to string values
pub trait PersistentIdentifierConvert<T: AsRef<str>> {
    /// Convert `self` into a string standard format PID of a certain type
    /// ```ignore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert_eq!("https://doi.org/10.1234/5678".format_as(PID::DOI), "10.1234/5678");
    /// assert_eq!("0000-0002-2057-9115".format_as(PID::ORCID), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    fn format_as(&self, pid_type: PID) -> String;
    /// Coerce `self` into given PID type.
    /// ```ignore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// let doi = "https://doi.org/10.1234/5678".to_pid(PID::DOI).to_doi();
    /// assert_eq!(doi.suffix(), "5678");
    /// ```
    fn to_pid(&self, pid_type: PID) -> PersistentIdentifierInternal;
    /// Determines if `self` is of the given PID type.
    /// ```ignore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://doi.org/10.1234/5678".is_pid(PID::DOI));
    /// ```
    fn is_pid(&self, _pid_type: PID) -> bool;
    /// Determines if `self` is an archival resource key (ARK)
    /// ```ignore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://n2t.net/ark:12148/btv1b8449691v/f29".is_ark());
    /// ```
    fn is_ark(&self) -> bool;
    /// Determines if `self` is an arXiv identifier
    fn is_arxiv(&self) -> bool;
    /// Determines if `self` is a DOI
    /// ```ingore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://doi.org/10.1234/5678".is_doi());
    /// ```
    fn is_doi(&self) -> bool;
    /// Determines if `self` is a Handle identifier.
    fn is_handle(&self) -> bool;
    /// Determines if `self` is an ISBN
    fn is_isbn(&self) -> bool {
        false
    }
    /// Determines if `self` is an ISNI
    fn is_isni(&self) -> bool;
    /// Determines if `self` is a ORCID
    /// ```ignore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://orcid.org/0000-0000-0000-0000".is_orcid());
    /// ```
    fn is_orcid(&self) -> bool;
    /// Determines if `self` is a RAID
    /// ```ignore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://raid.org/10.83962/fb5be317".is_raid());
    /// ```````
    fn is_raid(&self) -> bool;
    /// Determines if `self` is a ROR
    /// ```ignore
    /// use acorn_schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://ror.org/01qz5mb56".is_ror());
    /// ```
    fn is_ror(&self) -> bool;
    /// Determines if `self` is a Software Hash Identifier.
    fn is_swhid(&self) -> bool;
}
/// Trait for working with persistent identifiers (PID) as and within string values
pub trait PersistentIdentifierParse {
    /// Find all PID values present in a string
    fn find_all(value: impl ToString) -> Vec<Self>
    where
        Self: Sized;
    /// Parse and format a PID according to its associated canonical format
    fn format(value: impl ToString) -> String;
    /// Instantiate a PID from a string
    fn from_string(value: impl ToString) -> Self
    where
        Self: Sized;
    /// Determine if a string is a valid PID
    fn is_valid(value: impl ToString) -> bool;
}
/// Persistent Identifier (PID) types
///
/// PIDs are globally unique identifiers, resolvable on the Web, and associated with a set of additional descriptive metadata (ex. [`raid::Metadata`])
#[derive(Clone, Debug, Default, EnumIs, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
pub enum PID {
    /// Unknown PID
    #[default]
    Unknown,
    /// Archival Resource Key (ARK)
    ///
    /// Widely used persistent identifier, supported by the California Digital Library \[21\], in collaboration with DuraSpace. ARKs work similarly to DOIs, but are more permissive in design.[^ark]
    ///
    /// [^ark]: `M. Stocker et al., "Persistent Identification of Instruments," Data Science Journal, vol. 19, p. 18, May 2020, doi: 10.5334/dsj-2020-018.`
    ARK,
    /// arXiv identifier
    ///
    /// See [`Arxiv`]
    Arxiv,
    /// Digital Object Identifier (DOI)
    ///
    /// See [`DOI`]
    DOI,
    /// Handle System identifier
    ///
    /// See [`Handle`]
    Handle,
    /// International Standard Book Number (ISBN)
    ///
    /// See [`ISBN`]
    ISBN,
    /// International Standard Name Identifier (ISNI)
    ///
    /// See [`ISNI`]
    ISNI,
    /// Open Researcher and Contributor ID (ORCiD)
    ///
    /// See [`ORCID`]
    ORCID,
    /// Patent Number
    Patent,
    /// Persistent Identification of Instruments (PIDINST)
    /// ### Citation
    /// ```text
    /// M. Stocker et al., "Persistent Identification of Instruments," Data Science Journal, vol. 19, p. 18, May 2020, doi: 10.5334/dsj-2020-018.
    /// ```
    PIDINST,
    /// Research Activity Identifier (RAiD)
    ///
    /// Developed by tthe Australian Research Data Commons (ARDC), used to identify research projects and activities for access by research communities worldwide
    ///
    /// The ARDC and [DataCite](https://datacite.org/) have entered an agreement to use DataCite [`DOI`]s as RAiD identifiers
    ///
    /// See [`raid`] module
    RAID,
    /// Research Organization Registry (ROR)
    ///
    /// Global, community-led registry of open persistent identifiers for research organizations
    ///
    /// See <https://www.ror.org/> for more information
    ROR,
    /// Software Hash Identifier (SWHID)
    ///
    /// Intrinsic identifier for software content, directories, revisions, releases, and snapshots.
    SWHID,
    /// Public HTTP or HTTPS artifact URL
    URL,
}
/// DOI or arXiv publication identifier type.
#[derive(Clone, Debug)]
pub enum PublicationIdentifierType {
    /// Digital Object Identifier
    Doi(DOI),
    /// arXiv identifier
    Arxiv(Arxiv),
    /// Unsupported publication identifier
    Unknown,
}
/// A persistent identifier type paired with its value.
#[derive(Builder, Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct Identifier {
    /// Persistent identifier type.
    pub kind: PID,
    /// Identifier value, either raw or normalized.
    pub value: String,
}
/// Internal representation of a persistent identifier
#[derive(Default)]
pub struct PersistentIdentifierInternal {
    /// Raw string content of the (possible) PID
    value: String,
    /// Type of PID
    pid_type: PID,
}
impl Identifier {
    /// Discover every supported persistent identifier in text.
    pub fn find_all(content: &str) -> Vec<Self> {
        let raid = content
            .split_whitespace()
            .filter(|value| value.to_ascii_lowercase().contains("raid"))
            .filter_map(|value| {
                Self {
                    kind: PID::RAID,
                    value: value.to_string(),
                }
                .normalized()
            })
            .collect::<Vec<_>>();
        let parsed = PID::iter()
            .filter(|kind| kind.is_discoverable() && !kind.is_raid())
            .flat_map(|kind| kind.find_all(content))
            .filter(|identifier| identifier.kind != PID::DOI || !raid.iter().any(|value| value.value == identifier.value));
        raid.iter()
            .cloned()
            .chain(parsed)
            .fold(Vec::new(), |identifiers, identifier| match identifiers.contains(&identifier) {
                | true => identifiers,
                | false => identifiers.into_iter().chain([identifier]).collect(),
            })
    }
    /// Create an identifier with an unknown type.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            kind: PID::Unknown,
            value: value.into(),
        }
    }
    /// Normalize this identifier as one supported persistent or URL artifact identifier.
    pub fn normalized(&self) -> Option<Self> {
        let trimmed = self
            .value
            .trim()
            .trim_matches(|character: char| matches!(character, '<' | '>' | '(' | ')' | '[' | ']' | ',' | ';'))
            .trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
        let trimmed = trim_unmatched_trailing_parentheses(trimmed);
        match self.kind {
            | PID::ARK => Self::parsed::<ARK>(PID::ARK, trimmed),
            | PID::Arxiv => Self::parsed::<Arxiv>(PID::Arxiv, trimmed),
            | PID::DOI => Self::parsed::<DOI>(PID::DOI, trimmed),
            | PID::Handle => Self::parsed::<Handle>(PID::Handle, trimmed),
            | PID::ISBN => Self::parsed::<ISBN>(PID::ISBN, trimmed),
            | PID::ISNI => Self::parsed::<ISNI>(PID::ISNI, trimmed),
            | PID::ORCID => Self::parsed::<ORCID>(PID::ORCID, trimmed),
            | PID::Patent => Self::parsed::<Patent>(PID::Patent, trimmed),
            | PID::RAID => Self::parsed::<RAID>(PID::RAID, trimmed),
            | PID::ROR => Self::parsed::<ROR>(PID::ROR, trimmed),
            | PID::SWHID => Self::parsed::<SWHID>(PID::SWHID, trimmed),
            | PID::URL if HTTP_URL.is_match(trimmed).unwrap_or(false) => Some(Self {
                kind: PID::URL,
                value: trimmed.trim_end_matches('/').to_string(),
            }),
            | PID::Unknown => {
                let lowercase = trimmed.to_ascii_lowercase();
                let primary = if lowercase.starts_with("raid:") || lowercase.starts_with("https://raid.org/") {
                    PID::RAID
                } else {
                    PID::DOI
                };
                [
                    PID::Arxiv,
                    primary,
                    PID::ARK,
                    PID::Handle,
                    PID::ISBN,
                    PID::ORCID,
                    PID::ISNI,
                    PID::Patent,
                    PID::ROR,
                    PID::SWHID,
                    PID::URL,
                ]
                .into_iter()
                .find_map(|kind| {
                    Self {
                        kind,
                        value: self.value.clone(),
                    }
                    .normalized()
                })
            }
            | _ => None,
        }
    }
    /// Normalize an exact project identity key.
    pub fn normalize(value: &str) -> String {
        let value = value.trim();
        match value.split_once(':') {
            | Some((prefix, identifier)) => {
                let prefix = prefix.to_ascii_lowercase();
                let kind = PID::from(prefix.as_str());
                let lowercase = kind.is_arxiv() || kind.is_doi() || kind.is_raid() || kind.is_isbn() || kind.is_patent() || kind.is_swhid();
                if lowercase {
                    format!("{prefix}:{}", identifier.trim().to_ascii_lowercase())
                } else {
                    format!("{prefix}:{}", identifier.trim())
                }
            }
            | None => value.to_string(),
        }
    }
    /// Return this identifier as a normalized exact project identity key.
    pub fn identity_key(&self) -> String {
        match self.kind {
            | PID::Arxiv => {
                let identifier = Arxiv::from_string(&self.value).work_identifier();
                Self::normalize(&format!("arxiv:{}", identifier.trim_start_matches("arXiv:")))
            }
            | PID::SWHID => Self::normalize(&format!("swhid:{}", SWHID::from_string(&self.value).core_identifier())),
            | _ => Self::normalize(&format!("{}:{}", self.kind.as_str(), self.value)),
        }
    }
    fn parsed<T: PersistentIdentifierParse + fmt::Display>(kind: PID, value: &str) -> Option<Self> {
        let formatted = T::format(value);
        match T::is_valid(&formatted) {
            | true => Some(Self { kind, value: formatted }),
            | false => T::find_all(value)
                .first()
                .map(T::format)
                .filter(|value| T::is_valid(value))
                .map(|value| Self { kind, value }),
        }
    }
    /// Return the deterministic identifier hash used in artifact paths
    #[cfg(feature = "std")]
    pub fn identifier_hash(&self) -> String {
        HEXLOWER.encode(digest(&SHA256, self.value.as_bytes()).as_ref())[..12].to_string()
    }
}
impl PID {
    /// Discover identifiers of this type in text.
    pub fn find_all(&self, content: &str) -> Vec<Identifier> {
        match self {
            | Self::ARK => ARK::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::Arxiv => Arxiv::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::DOI => DOI::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::Handle => Handle::find_explicit(content).into_iter().map(Identifier::from).collect(),
            | Self::ISBN => ISBN::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::ISNI => ISNI::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::ORCID => ORCID::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::Patent => Patent::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::RAID => RAID::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::ROR => ROR::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::SWHID => SWHID::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::URL => RE_URL
                .find_iter(content)
                .filter_map(Result::ok)
                .filter_map(|matched| {
                    Identifier::new(matched.as_str())
                        .normalized()
                        .filter(|identifier| identifier.kind == PID::URL)
                })
                .collect(),
            | _ => Vec::new(),
        }
    }
}
impl<'a> From<&'a Identifier> for &'a str {
    fn from(identifier: &'a Identifier) -> Self {
        identifier.kind.as_str()
    }
}
impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<ARK> for Identifier {
    fn from(value: ARK) -> Self {
        Self {
            kind: PID::ARK,
            value: value.to_string(),
        }
    }
}
impl From<Arxiv> for Identifier {
    fn from(value: Arxiv) -> Self {
        Self {
            kind: PID::Arxiv,
            value: value.to_string(),
        }
    }
}
impl From<DOI> for Identifier {
    fn from(value: DOI) -> Self {
        Self {
            kind: PID::DOI,
            value: value.to_string(),
        }
    }
}
impl From<Handle> for Identifier {
    fn from(value: Handle) -> Self {
        Self {
            kind: PID::Handle,
            value: value.to_string(),
        }
    }
}
impl From<ISBN> for Identifier {
    fn from(value: ISBN) -> Self {
        Self {
            kind: PID::ISBN,
            value: value.to_string(),
        }
    }
}
impl From<ISNI> for Identifier {
    fn from(value: ISNI) -> Self {
        Self {
            kind: PID::ISNI,
            value: value.to_string(),
        }
    }
}
impl From<ORCID> for Identifier {
    fn from(value: ORCID) -> Self {
        Self {
            kind: PID::ORCID,
            value: value.to_string(),
        }
    }
}
impl From<Patent> for Identifier {
    fn from(value: Patent) -> Self {
        Self {
            kind: PID::Patent,
            value: value.to_string(),
        }
    }
}
impl From<RAID> for Identifier {
    fn from(value: RAID) -> Self {
        Self {
            kind: PID::RAID,
            value: value.to_string(),
        }
    }
}
impl From<ROR> for Identifier {
    fn from(value: ROR) -> Self {
        Self {
            kind: PID::ROR,
            value: value.to_string(),
        }
    }
}
impl From<SWHID> for Identifier {
    fn from(value: SWHID) -> Self {
        Self {
            kind: PID::SWHID,
            value: value.to_string(),
        }
    }
}
impl PID {
    /// Whether this PID type participates in artifact discovery
    pub fn is_discoverable(&self) -> bool {
        self.is_ark()
            || self.is_arxiv()
            || self.is_doi()
            || self.is_handle()
            || self.is_isbn()
            || self.is_isni()
            || self.is_orcid()
            || self.is_patent()
            || self.is_raid()
            || self.is_ror()
            || self.is_swhid()
            || self.is_url()
    }
    /// Return the stable lowercase path component
    pub fn as_str(&self) -> &'static str {
        match self {
            | Self::ARK => "ark",
            | Self::Arxiv => "arxiv",
            | Self::DOI => "doi",
            | Self::Handle => "handle",
            | Self::ISBN => "isbn",
            | Self::ISNI => "isni",
            | Self::ORCID => "orcid",
            | Self::Patent => "patent",
            | Self::PIDINST => "pidinst",
            | Self::RAID => "raid",
            | Self::ROR => "ror",
            | Self::SWHID => "swhid",
            | Self::URL => "url",
            | _ => "unknown",
        }
    }
    /// Whether this PID type can independently identify a research project.
    pub fn is_project_identifier(&self) -> bool {
        self.is_doi() || self.is_arxiv() || self.is_raid() || self.is_isbn() || self.is_patent() || self.is_ark() || self.is_swhid()
    }
}
impl fmt::Display for PID {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl Serialize for PID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for PID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|value| {
            Self::iter()
                .find(|pid| pid.as_str().eq_ignore_ascii_case(&value))
                .ok_or_else(|| serde::de::Error::custom(format!("unknown PID type `{value}`")))
        })
    }
}
impl From<&str> for PID {
    fn from(value: &str) -> Self {
        Self::iter()
            .find(|pid| pid.as_str().eq_ignore_ascii_case(value.trim()))
            .unwrap_or_default()
    }
}
impl Betanumeric for char {
    fn is_betanumeric(&self) -> bool {
        BETANUMERIC_DIGITS.contains(*self)
    }
    fn to_betanumeric_ordinal(&self) -> Option<usize> {
        BETANUMERIC_DIGITS.chars().position(|x| x.eq(self))
    }
}
impl<T: AsRef<str>> PersistentIdentifierConvert<T> for T
where
    T: ToString,
{
    fn format_as(&self, pid_type: PID) -> String {
        match pid_type {
            | PID::ARK => ARK::format(self.as_ref()),
            | PID::Arxiv => Arxiv::format(self.as_ref()),
            | PID::DOI => DOI::format(self.as_ref()),
            | PID::Handle => Handle::format(self.as_ref()),
            | PID::ISBN => ISBN::format(self.as_ref()),
            | PID::ISNI => ISNI::format(self.as_ref()),
            | PID::ORCID => ORCID::format(self.as_ref()),
            | PID::Patent => Patent::format(self.as_ref()),
            | PID::RAID => RAID::format(self.as_ref()),
            | PID::ROR => <ROR as PersistentIdentifierParse>::format(self.as_ref()),
            | PID::SWHID => SWHID::format(self.as_ref()),
            | _ => self.as_ref().to_string(),
        }
    }
    fn to_pid(&self, pid_type: PID) -> PersistentIdentifierInternal {
        let value = self.as_ref().to_string();
        match pid_type {
            | PID::ARK => PersistentIdentifierInternal { value, pid_type: PID::ARK },
            | PID::Arxiv => PersistentIdentifierInternal { value, pid_type: PID::Arxiv },
            | PID::DOI => PersistentIdentifierInternal { value, pid_type: PID::DOI },
            | PID::Handle => PersistentIdentifierInternal {
                value,
                pid_type: PID::Handle,
            },
            | PID::ISBN => PersistentIdentifierInternal { value, pid_type: PID::ISBN },
            | PID::ISNI => PersistentIdentifierInternal { value, pid_type: PID::ISNI },
            | PID::ORCID => PersistentIdentifierInternal { value, pid_type: PID::ORCID },
            | PID::Patent => PersistentIdentifierInternal {
                value,
                pid_type: PID::Patent,
            },
            | PID::RAID => PersistentIdentifierInternal { value, pid_type: PID::RAID },
            | PID::ROR => PersistentIdentifierInternal { value, pid_type: PID::ROR },
            | PID::SWHID => PersistentIdentifierInternal { value, pid_type: PID::SWHID },
            | _ => PersistentIdentifierInternal::default(),
        }
    }
    fn is_pid(&self, pid_type: PID) -> bool {
        match pid_type {
            | PID::ARK => self.is_ark(),
            | PID::Arxiv => self.is_arxiv(),
            | PID::DOI => self.is_doi(),
            | PID::Handle => self.is_handle(),
            | PID::ISBN => self.is_isbn(),
            | PID::ISNI => self.is_isni(),
            | PID::ORCID => self.is_orcid(),
            | PID::Patent => Patent::is_valid(self.as_ref()),
            | PID::RAID => self.is_raid(),
            | PID::ROR => self.is_ror(),
            | PID::SWHID => self.is_swhid(),
            | _ => false,
        }
    }
    fn is_ark(&self) -> bool {
        ARK::is_valid(self.as_ref())
    }
    fn is_arxiv(&self) -> bool {
        Arxiv::is_valid(self.as_ref())
    }
    fn is_doi(&self) -> bool {
        DOI::is_valid(self.as_ref())
    }
    fn is_handle(&self) -> bool {
        Handle::is_valid(self.as_ref())
    }
    fn is_isbn(&self) -> bool {
        ISBN::is_valid(self.as_ref())
    }
    fn is_isni(&self) -> bool {
        ISNI::is_valid(self.as_ref())
    }
    fn is_orcid(&self) -> bool {
        ORCID::is_valid(self.as_ref())
    }
    fn is_raid(&self) -> bool {
        RAID::is_valid(self.as_ref())
    }
    fn is_ror(&self) -> bool {
        ROR::is_valid(self.as_ref())
    }
    fn is_swhid(&self) -> bool {
        SWHID::is_valid(self.as_ref())
    }
}
impl PersistentIdentifierInternal {
    /// Convert a `PersistentIdentifierInternal` to an `ARK`
    pub fn to_ark(&self) -> ARK {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::ARK => ARK::from_string(value),
            | _ => ARK::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to an [`Arxiv`].
    pub fn to_arxiv(&self) -> Arxiv {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::Arxiv => Arxiv::from_string(value),
            | _ => Arxiv::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to a `DOI`
    pub fn to_doi(&self) -> DOI {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::DOI => DOI::from_string(value),
            | _ => DOI::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to a [`Handle`].
    pub fn to_handle(&self) -> Handle {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::Handle => Handle::from_string(value),
            | _ => Handle::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to an `ISBN`
    pub fn to_isbn(&self) -> ISBN {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::ISBN => ISBN::from_string(value),
            | _ => ISBN::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to a `ORCID`
    pub fn to_orcid(&self) -> ORCID {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::ORCID => ORCID::from_string(value),
            | _ => ORCID::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to an `ISNI`
    pub fn to_isni(&self) -> ISNI {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::ISNI => ISNI::from_string(value),
            | _ => ISNI::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to a [`Patent`].
    pub fn to_patent(&self) -> Patent {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::Patent => Patent::from_string(value),
            | _ => Patent::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to a `RAID`
    pub fn to_raid(&self) -> RAID {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::RAID => RAID::from_string(value),
            | _ => RAID::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to a `ROR`
    pub fn to_ror(&self) -> ROR {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::ROR => ROR::from_string(value),
            | _ => ROR::default(),
        }
    }
    /// Convert a `PersistentIdentifierInternal` to a `SWHID`.
    pub fn to_swhid(&self) -> SWHID {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::SWHID => SWHID::from_string(value),
            | _ => SWHID::default(),
        }
    }
}
impl From<&str> for PublicationIdentifierType {
    fn from(value: &str) -> Self {
        let match_covers_value = RE_ARXIV
            .find(value)
            .ok()
            .flatten()
            .is_some_and(|matched| matched.start() == 0 && matched.end() == value.len());
        let explicitly_labeled_arxiv = match_covers_value && Arxiv::is_valid(value);
        match (DOI::is_valid(value), explicitly_labeled_arxiv) {
            | (true, _) => Self::Doi(DOI::from_string(value)),
            | (_, true) => Self::Arxiv(Arxiv::from_string(value)),
            | _ => Self::Unknown,
        }
    }
}
/// Calculates the check character for an ISBN-10 or ISBN-13 value.
///
/// Hyphens and spaces are removed before selecting the algorithm from the compact
/// length. ISBN-10 weights the first nine digits from 10 through 2 and applies
/// modulo 11, representing a result of 10 as `X`. ISBN-13 weights the first
/// twelve digits alternately by 1 and 3 and applies modulo 10.
///
/// Returns `None` when the compact value is not 10 or 13 characters long, when a
/// payload character is not decimal, or when checked arithmetic fails.
///
/// ## Examples
/// ```rust
/// use acorn_schema::pid::{ISBN, PersistentIdentifier, PersistentIdentifierParse};
///
/// let isbn_10 = ISBN::from_string("0-8044-2957-X");
/// let isbn_13 = ISBN::from_string("978-0-306-40627-0");
/// assert_eq!(isbn_10.check_digit(), Some(vec!['X']));
/// assert_eq!(isbn_13.check_digit(), Some(vec!['0']));
/// ```
fn mod_10_or_11_check_digit<S>(value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
    let working = value
        .as_ref()
        .chars()
        .filter(|character| !matches!(character, '-' | ' '))
        .collect::<String>();
    match working.len() {
        | 10 => working
            .chars()
            .take(9)
            .enumerate()
            .try_fold(0_u32, |sum, (index, character)| {
                character.to_digit(10).and_then(|digit| {
                    u32::try_from(index)
                        .ok()
                        .and_then(|index| 10_u32.checked_sub(index))
                        .and_then(|weight| digit.checked_mul(weight))
                        .and_then(|weighted| sum.checked_add(weighted))
                })
            })
            .and_then(|sum| sum.checked_rem(11))
            .and_then(|remainder| 11_u32.checked_sub(remainder))
            .and_then(|complement| complement.checked_rem(11))
            .and_then(|check_digit| match check_digit {
                | 10 => Some(vec!['X']),
                | value => char::from_digit(value, 10).map(|character| vec![character]),
            }),
        | 13 => working
            .chars()
            .take(12)
            .enumerate()
            .try_fold(0_u32, |sum, (index, character)| {
                character.to_digit(10).and_then(|digit| {
                    index
                        .checked_rem(2)
                        .map(|remainder| if remainder == 0 { 1 } else { 3 })
                        .and_then(|weight| digit.checked_mul(weight))
                        .and_then(|weighted| sum.checked_add(weighted))
                })
            })
            .and_then(|sum| sum.checked_rem(10))
            .and_then(|remainder| 10_u32.checked_sub(remainder))
            .and_then(|complement| complement.checked_rem(10))
            .and_then(|check_digit| char::from_digit(check_digit, 10).map(|character| vec![character])),
        | _ => None,
    }
}
/// Calculates an ISO 7064 MOD 11-2 check character for an ORCID or ISNI value.
///
/// Hyphens and spaces are removed, after which the first 15 characters are folded
/// through `(remainder + digit) * 2 mod 11`. The final complement is returned as
/// one decimal character, or as `X` when the result is 10. A non-decimal payload
/// character contributes zero; PID parsing is responsible for enforcing the
/// identifier's character set before validation.
///
/// Returns `None` only when checked arithmetic or character conversion fails.
///
/// ## Examples
/// ```rust
/// use acorn_schema::pid::{ISNI, ORCID, PersistentIdentifier, PersistentIdentifierParse};
///
/// let isni = ISNI::from_string("0000000492299539");
/// let orcid = ORCID::from_string("0000-0002-2816-415X");
/// assert_eq!(isni.check_digit(), Some(vec!['9']));
/// assert_eq!(orcid.check_digit(), Some(vec!['X']));
/// ```
fn mod_11_2_check_digit<S>(value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
    const COMPLEMENT: u32 = 12;
    const MODULUS: u32 = 11;
    const RADIX: u32 = 2;
    let working = value.as_ref().replace("-", "").replace(" ", "");
    let remainder = working.chars().take(15).try_fold(0_u32, |remainder, value| {
        let digit = value.to_digit(10).unwrap_or_default();
        remainder
            .checked_add(digit)
            .and_then(|sum| sum.checked_mul(RADIX))
            .and_then(|product| product.checked_rem(MODULUS))
    });
    remainder
        .and_then(|remainder| COMPLEMENT.checked_sub(remainder))
        .and_then(|value| value.checked_rem(MODULUS))
        .and_then(|result| match result {
            | 10 => Some(vec!['X']),
            | value => char::from_digit(value, 10).map(|value| vec![value]),
        })
}
/// Calculates the two-character MOD 97-10 checksum used by a ROR identifier.
///
/// Hyphens and spaces are removed before the first six Crockford Base32
/// characters are decoded. The decoded value is multiplied by 100, reduced
/// modulo 97, and complemented; the result is returned as two zero-padded decimal
/// characters.
///
/// Returns `None` when the Base32 payload cannot be decoded or checked arithmetic
/// fails.
///
/// ## Example
/// ```rust
/// use acorn_schema::pid::{PersistentIdentifier, PersistentIdentifierParse, ROR};
///
/// let ror = ROR::from_string("01qz5mb56");
/// assert_eq!(ror.check_digit(), Some(vec!['5', '6']));
/// ```
fn mod_97_10_check_digit<S>(value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
    const COMPLEMENT: u128 = 98;
    const MODULUS: u128 = 97;
    let working = value
        .as_ref()
        .replace("-", "")
        .replace(" ", "")
        .chars()
        .take(6)
        .map(String::from)
        .collect::<Vec<_>>()
        .join("");
    base32_crockford_decode(working)
        .and_then(|value| value.checked_mul(100))
        .and_then(|value| value.checked_rem(MODULUS))
        .and_then(|remainder| COMPLEMENT.checked_sub(remainder))
        .and_then(|complement| complement.checked_rem(MODULUS))
        .map(|checksum| format!("{checksum:02}").chars().collect())
}
/// Calculates a NOID check character using the NCDA beta-numeric alphabet.
///
/// Each character's beta-numeric ordinal is multiplied by its one-based position.
/// The running total is reduced modulo 29, and the resulting ordinal is converted
/// back into a single beta-numeric character. Characters outside the alphabet
/// contribute an ordinal of zero.
///
/// Returns `None` when checked arithmetic or ordinal conversion fails.
///
/// ## Example
/// ```rust
/// use acorn_schema::pid::noid_check_digit;
///
/// assert_eq!(noid_check_digit("13030/xf93gt2"), Some(vec!['q']));
/// ```
pub fn noid_check_digit<S>(value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
    const RADIX: usize = 29;
    let remainder = value.as_ref().chars().enumerate().try_fold(0_usize, |acc, (index, value)| {
        let ordinal = value.to_betanumeric_ordinal().unwrap_or(0);
        index
            .checked_rem(RADIX)
            .and_then(|position| position.checked_add(1))
            .and_then(|position| position.checked_mul(ordinal))
            .and_then(|weighted| acc.checked_add(weighted))
            .and_then(|sum| sum.checked_rem(RADIX))
    });
    remainder
        .and_then(|value| u8::try_from(value).ok())
        .and_then(to_betanumeric)
        .map(|value| vec![value])
}
fn to_betanumeric(value: u8) -> Option<char> {
    match BETANUMERIC_DIGITS.chars().enumerate().find(|(i, _)| *i == value as usize) {
        | Some((_, x)) => Some(x),
        | None => None,
    }
}

#[cfg(all(test, feature = "std"))]
mod tests;
