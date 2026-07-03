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
use crate::prelude::*;
use crate::schema::namespaces::{DEFAULT_ORCID_SCHEMA_URI, DEFAULT_ROR_SCHEMA_URI};
use crate::util::constants::{
    RE_ARK, RE_ARK_TEXT, RE_DOI, RE_DOI_TEXT, RE_ISBN, RE_ISBN_TEXT, RE_ORCID, RE_ORCID_TEXT, RE_RAID_TEXT, RE_ROR, RE_ROR_TEXT,
};
use crate::util::{base32_crockford_decode, regex_capture_lookup, ToStringChunks};
use bon::Builder;
use core::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod patent;
pub mod raid;

pub use patent::Patent;

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
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert_eq!("https://doi.org/10.1234/5678".format_as(PID::DOI), "10.1234/5678");
    /// assert_eq!("0000-0002-2057-9115".format_as(PID::ORCID), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    fn format_as(&self, pid_type: PID) -> String;
    /// Coerce `self` into given PID type.
    /// ```ignore
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// let doi = "https://doi.org/10.1234/5678".to_pid(PID::DOI).to_doi();
    /// assert_eq!(doi.suffix(), "5678");
    /// ```
    fn to_pid(&self, pid_type: PID) -> PersistentIdentifierInternal;
    /// Determines if `self` is of the given PID type.
    /// ```ignore
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://doi.org/10.1234/5678".is_pid(PID::DOI));
    /// ```
    fn is_pid(&self, _pid_type: PID) -> bool;
    /// Determines if `self` is an archival resource key (ARK)
    /// ```ignore
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://n2t.net/ark:12148/btv1b8449691v/f29".is_ark());
    /// ```
    fn is_ark(&self) -> bool;
    /// Determines if `self` is a DOI
    /// ```ingore
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://doi.org/10.1234/5678".is_doi());
    /// ```
    fn is_doi(&self) -> bool;
    /// Determines if `self` is an ISBN
    fn is_isbn(&self) -> bool {
        false
    }
    /// Determines if `self` is a ORCID
    /// ```ignore
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://orcid.org/0000-0000-0000-0000".is_orcid());
    /// ```
    fn is_orcid(&self) -> bool;
    /// Determines if `self` is a RAID
    /// ```ignore
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://raid.org/10.83962/fb5be317".is_raid());
    fn is_raid(&self) -> bool;
    /// Determines if `self` is a ROR
    /// ```ignore
    /// use acorn::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://ror.org/01qz5mb56".is_ror());
    fn is_ror(&self) -> bool;
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
/// Internal representation of a persistent identifier
#[derive(Default)]
pub struct PersistentIdentifierInternal {
    /// Raw string content of the (possible) PID
    value: String,
    /// Type of PID
    pid_type: PID,
}
/// Persistent Identifier (PID) types
///
/// PIDs are globally unique identifiers, resolvable on the Web, and associated with a set of additional descriptive metadata (ex. [`raid::Metadata`])
#[derive(Clone, Debug, Default)]
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
    /// Digital Object Identifier (DOI)
    ///
    /// See [`DOI`]
    DOI,
    /// International Standard Book Number (ISBN)
    ///
    /// See [`ISBN`]
    ISBN,
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
}
/// Archival Resource Key (ARK)
/// ### Notes
/// - ARKs are the only mainstream, non-siloed, non-paywalled identifiers that you can register to use in about 48 hours
/// - ARKs are decentralized
/// - There are no fees for ARKs, PURLs, and URNs
/// - ARKs give access to almost any kind of thing, whether digital, physical, abstract, person, group, etc.
/// - ARKs can be deleted
/// - ARKs support early object development
/// - ARKs that differ only by hyphens are considered identical
///
/// See the [ARK specification](https://datatracker.ietf.org/doc/draft-kunze-ark/) and <https://wiki.lyrasis.org/display/ARKs/ARK+Identifiers+FAQ> for more information
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ARK {
    /// The part of the ARK string that your organization is responsible for making unique.
    ///
    /// The first 2 or more characters constitue the shoulder of the ARK which must meet the following criteria:
    /// - Must start with one or more lowercase letters
    /// - Must end with a digit (non-zero preferred)
    /// - Must not contain vowels or the letter "l" (ell)
    /// - Must not contain any `/` characters (being opaque is part of the shoulder design)
    pub assigned_name: Option<String>,
    /// Prefix for NAAN (e.g., "ark:" or the older, "ark:/")
    ///
    /// <div class="warning">Label is mandatory</div>
    #[builder(default = "ark:".to_string())]
    pub label: String,
    /// Number (here represented as a string) identifying an organization that creates or assigns identifiers
    /// ### Notes
    /// - Since 2001, every assigned name assigning authority number (NAAN) has consisted of exactly five digits, specifically five beta-numeric digits
    /// - Any given identifier will have exactly one NAAN but may have more than one NMA (at a time or over time)
    /// - Similar to registration authority or prefix for [`DOI`]s, naming authority for [Handles], and namespace identifier for [URNs]
    ///
    /// [Handles]: https://handle.net/
    /// [URNs]: https://en.wikipedia.org/wiki/Uniform_Resource_Name
    pub name_assigning_authority_number: Option<String>,
    /// String identifying a service that accepts names and returns information about them
    /// ### Notes
    /// - Any given identifier will have exactly one NAAN but may have more than one NMA (at a time or over time)
    /// - Strictly speaking, NMA does not include the protocol (e.g., https), but since this implementation only supports HTTPS, we conflate what would be called the "resolver service" with NMA.
    pub name_mapping_authority: Option<String>,
    /// First section of optional "qualifier" part of ARK
    ///
    /// Generally serve as sub-namespaces to enabling grouping ARKs
    #[builder(default = Vec::new())]
    pub parts: Vec<String>,
    /// Last section of optional "qualifier" part of ARK
    ///
    /// Typically is used to identify a specific version of a resource (i.e., "pdf", "fr", "v3", etc.)
    #[builder(default = Vec::new())]
    pub variants: Vec<String>,
}
/// Digital Object Identifier (DOI)
///
/// DOIs consist of a DOI name which is resolved at <https://doi.org>, with the full URI formulated according to the pattern `https://doi.org/{DOI_name}`. DOI names in turn consist of a prefix and a suffix, separated by a forward slash. The prefix is a code indicating the registrant who issues the DOI, e.g., Harvard University Dataverse - 10.7910; Dryad Digital Repository - 10.5061. The suffix is the identifier, in any form, assigned by the registrant.[^doi]
///
/// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
///
/// [^doi]: `N. Juty, S. M. Wimalaratne, S. Soiland-Reyes, J. Kunze, C. A. Goble, and T. Clark, "Unique, Persistent, Resolvable: Identifiers as the Foundation of FAIR," Data Intellegence, vol. 2, no. 1-2, pp. 30-39, Jan. 2020, doi: 10.1162/dint_a_00025.`
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct DOI {
    /// Schema URI (i.e., <https://doi.org/>)
    pub schema_uri: Option<String>,
    /// Directory indicator
    /// ### Rules
    /// - Can contain only numeric values
    /// - Usually 10 but other indicators may be designated as compliant by the DOI Foundation
    pub directory_indicator: Option<String>,
    /// Registrant code
    /// ### Rules
    /// - Can contain only numeric values and one or several full stops which are used to subdivide the code
    /// - If the directory indicator is 10 then a registrant code is mandatory
    pub registrant_code: Option<String>,
    /// Suffix
    /// ### Rules
    /// - Shall be unique to the prefix element that precedes it
    /// - Can be a sequential number
    /// - Can be an identifier generated from or based on another system used by the registrant
    /// - No length limit is set to the suffix by the DOI System
    pub suffix: Option<String>,
}
/// International Standard Book Number (ISBN)
///
/// A 13-digit identification number and system, widely used in the international book trade for over 35 years and assigned through a network of [international ISBN Registration Agencies](https://www.isbn-international.org/).
/// ISBNs are used to identify each unique publication whether in the form of a physical book or related materials such as eBooks, software, mixed media etc.
/// ### Notes
/// - ISBNs are governed by the ISO 2108 standard.
/// - ISNBs can be expressed as [`DOI`]s (see [DOI system and the ISBN system](https://www.doi.org/the-identifier/resources/factsheets/doi-system-and-the-isbn-system)).
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
pub struct ISBN {
    /// Prefix element
    ///
    /// ISBN (GS1) Bookland prefix = `978.` or `979.`
    pub prefix_element: Option<String>,
    /// Registration group element
    ///
    /// 1-to-5-digit number that is valid within a single prefix element
    pub registration_group: Option<String>,
    /// Publication prefix element
    pub publisher: Option<String>,
    /// ISBN Title enumerator
    pub title: Option<String>,
    /// Check digit
    ///
    /// See [`isbn_check_digit`]
    pub check_digit: Option<String>,
}
/// Open Researcher and Contributor ID (ORCiD)[^orcid]
///
/// Disambiguates researchers, and connects people with their research activities. This includes employment affiliations, research outputs, funding, peer review activity, research resources, society membership, distinctions and other scholarly infrastructure.
///
/// See <https://orcid.org/> for more information
///
/// [^orcid]: `L. L. Haak, M. Fenner, L. Paglione, E. Pentz, and H. Ratner, "ORCID: a system to uniquely identify researchers," Learned Publishing, vol. 25, no. 4, pp. 259-264, 2012, doi: 10.1087/20120404.`
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ORCID {
    /// Schema URI (i.e., <https://orcid.org/>)
    pub schema_uri: Option<String>,
    /// 16 digit string with hyphens every 4 digits (for readability)
    /// <div class="warning">This value can be stored with or without hyphens. To ensure compliancy, use <code>ORCID::identifier</code> method to access ORCiD identifier.</div>
    pub identifier: Option<String>,
    /// The check digit is the last (16th) digit of the identifier
    /// ### Note
    /// Check digit should be verified IAW [ISO 7064, MOD 11-2](https://www.iso.org/standard/31531.html) (see [`iso7064_check_digit`])
    pub check_digit: Option<String>,
}
/// Research Activity Identifier (RAiD)[^raid]
///
/// RAiDs are expressed in the form of `https://raid.org/prefix/suffix`, resolvable through the RAiD portal operated by the ARDC[^ardc] at <https://raid.org/> —though they may still be resolved through any DOI or handle resolver.
///
/// RAiDs are governed by [ISO 23527](https://www.iso.org/standard/75931.html)
///
/// [^ardc]: [Australian Research Data Commons](https://ardc.edu.au/)
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct RAID {
    /// Schema URI (e.g., <https://www.raid.org/>)
    pub schema_uri: Option<String>,
    /// RAiD prefix value
    pub prefix: Option<String>,
    /// RAiD suffix value
    pub suffix: Option<String>,
    /// RAiD metadata
    ///
    /// Metadata associated with identifier. See <https://metadata.raid.org> for more information.
    pub metadata: Option<raid::Metadata>,
}
/// Research Organization Registry (ROR)[^ror]
///
/// A global, community-led registry of open persistent identifiers for research and funding organizations
///
/// [^ror]: https://ror.org/
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ROR {
    /// Schema URI (e.g., <https://ror.org/>)
    pub schema_uri: Option<String>,
    /// ROR identifier value
    pub identifier: Option<String>,
    /// The last two integers are a zero-padded checksum, 01 -98
    /// ### Note
    /// Check digits should be verified IAW [ISO 7064](https://www.iso.org/standard/31531.html)
    pub check_digit: Option<String>,
}
impl Betanumeric for char {
    fn is_betanumeric(&self) -> bool {
        BETANUMERIC_DIGITS.contains(*self)
    }
    fn to_betanumeric_ordinal(&self) -> Option<usize> {
        BETANUMERIC_DIGITS.chars().position(|x| x.eq(self))
    }
}
impl Default for ARK {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for DOI {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for ORCID {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for RAID {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for ROR {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for ARK {
    /// Format a ARK into a standard format of `"{NMA}{label}{NAAN}/{Assigned Name}/{Parts}{Variants}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nma = self.name_mapping_authority.clone().unwrap_or_default().trim_end_matches('/').to_string();
        let identifier = self.identifier();
        let result = [nma, identifier].into_iter().filter(|x| !x.is_empty()).collect::<Vec<String>>().join("/");
        write!(f, "{result}")
    }
}
impl fmt::Display for DOI {
    /// Format a DOI into a standard format of `"{prefix}/{suffix}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.identifier();
        write!(f, "{result}")
    }
}
impl fmt::Display for ISBN {
    /// Format a ISBN into a standard format
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.identifier();
        write!(f, "{result}")
    }
}
impl fmt::Display for ORCID {
    /// Format a ORCiD into a standard format of `"{schema_uri}{identifier}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let schema_uri = self.schema_uri();
        let identifier = self.identifier();
        let uri = if schema_uri.is_empty() { DEFAULT_ORCID_SCHEMA_URI } else { &schema_uri };
        let values = match &self.identifier {
            | Some(_) => [uri, &identifier].to_vec(),
            | None => vec![],
        };
        let result = values
            .into_iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/");
        write!(f, "{result}")
    }
}
impl fmt::Display for RAID {
    /// Format a RAID into a standard format of `"{prefix}/{suffix}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.identifier();
        write!(f, "{result}")
    }
}
impl fmt::Display for ROR {
    /// Format a ROR into a standard format of `"{schema_uri}{identifier}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let schema_uri = self.schema_uri();
        let result = self.identifier();
        if result.is_empty() {
            write!(f, "")
        } else {
            write!(f, "{schema_uri}{result}")
        }
    }
}
impl PersistentIdentifier for ARK {
    fn new() -> Self {
        ARK::init().build()
    }
    fn schema_uri(&self) -> String {
        let uri = match &self.name_mapping_authority {
            | Some(value) => value,
            | None => "",
        };
        uri.trim_end_matches("/").to_string()
    }
    fn identifier(&self) -> String {
        let values = [self.prefix(), self.suffix()];
        values
            .iter()
            .flatten()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/")
    }
    fn prefix(&self) -> Option<String> {
        match (self.name_assigning_authority_number.as_ref(), self.assigned_name.as_ref()) {
            | (Some(naan), Some(name)) => Some(format!("{}{}/{}", self.label.trim_end_matches('/'), naan, name)),
            | _ => None,
        }
    }
    fn suffix(&self) -> Option<String> {
        let parts = self.parts.join("/");
        let variants = self.variants.join(".");
        let qualifiers = [parts, variants];
        let result = qualifiers
            .iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join(".");
        Some(result)
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        let Self {
            name_assigning_authority_number: naan,
            assigned_name: name,
            ..
        } = self;
        let values = [naan.clone(), name.clone()];
        if values.iter().all(|x| x.is_some()) {
            let value = values.iter().flatten().map(String::from).collect::<Vec<String>>().join("/");
            if value.is_empty() {
                None
            } else {
                let trimmed = value.get(..value.len().saturating_sub(1)).unwrap_or_default();
                noid_check_digit(trimmed)
            }
        } else {
            None
        }
    }
}
impl PersistentIdentifier for DOI {
    fn new() -> Self {
        DOI::init().build()
    }
    fn schema_uri(&self) -> String {
        self.schema_uri.as_ref().cloned().unwrap_or_default().trim_end_matches("/").to_string()
    }
    fn identifier(&self) -> String {
        let values = [self.prefix(), self.suffix()];
        values
            .iter()
            .flatten()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/")
    }
    /// Get DOI prefix (i.e., "{directory_indicator}.{registrant_code}")
    fn prefix(&self) -> Option<String> {
        let values = [
            self.directory_indicator.as_ref().cloned().unwrap_or_default(),
            self.registrant_code.as_ref().cloned().unwrap_or_default(),
        ];
        let result = values
            .iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join(".");
        Some(result)
    }
    /// Get DOI suffix
    fn suffix(&self) -> Option<String> {
        fn postprocess(mut value: String) -> String {
            if value.ends_with(".") {
                value.pop();
            }
            value
        }
        let result = self.suffix.as_ref().cloned().unwrap_or_default();
        if !result.is_empty() {
            Some(postprocess(result))
        } else {
            None
        }
    }
    fn url(&self) -> String {
        let identifier = self.identifier();
        if identifier.is_empty() {
            String::new()
        } else {
            let uri = self.schema_uri();
            let schema = if uri.is_empty() { "https://doi.org" } else { &uri };
            format!("{}/{}", schema, identifier)
        }
    }
}
impl PersistentIdentifier for ISBN {
    fn new() -> Self {
        ISBN::init().build()
    }
    fn schema_uri(&self) -> String {
        "".to_string()
    }
    fn identifier(&self) -> String {
        let ISBN {
            prefix_element,
            registration_group,
            publisher,
            title,
            check_digit,
        } = self;
        [prefix_element, registration_group, publisher, title, check_digit]
            .into_iter()
            .map(|x| x.clone().unwrap_or_default())
            .collect::<Vec<String>>()
            .join("-")
    }
    /// Used to convert to ISBN-A DOI compatible value
    /// See <https://www.doi.org/the-identifier/resources/factsheets/doi-system-and-the-isbn-system>
    fn prefix(&self) -> Option<String> {
        let ISBN {
            prefix_element,
            registration_group,
            publisher,
            ..
        } = self;
        let result = format!(
            "{}.{}{}",
            prefix_element.clone().unwrap_or_default(),
            registration_group.clone().unwrap_or_default(),
            publisher.clone().unwrap_or_default()
        );
        Some(result)
    }
    /// Used to convert to ISBN-A DOI compatible value
    /// See <https://www.doi.org/the-identifier/resources/factsheets/doi-system-and-the-isbn-system>
    fn suffix(&self) -> Option<String> {
        let ISBN { title, check_digit, .. } = self;
        let result = [title, check_digit]
            .into_iter()
            .map(|x| x.clone().unwrap_or_default())
            .collect::<Vec<String>>()
            .join("");
        Some(result)
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        isbn_check_digit(self.identifier())
    }
}
impl From<ISBN> for DOI {
    fn from(isbn: ISBN) -> Self {
        DOI::init()
            .schema_uri("https://doi.org/")
            .directory_indicator("10")
            .maybe_registrant_code(isbn.prefix())
            .maybe_suffix(isbn.suffix())
            .build()
    }
}
impl From<DOI> for ISBN {
    fn from(doi: DOI) -> Self {
        let prefix = doi.prefix().unwrap_or_default().replace(".", "-");
        let suffix = match doi.suffix() {
            | Some(value) => {
                let check_digit = value.chars().last().unwrap_or_default().to_string();
                let title = value.get(..value.len().saturating_sub(1)).unwrap_or_default().to_string();
                format!("{title}-{check_digit}")
            }
            | None => "".to_string(),
        };
        let result = format!("{}-{suffix}", prefix.trim_start_matches("10-"));
        ISBN::from_string(result)
    }
}
impl PersistentIdentifier for ORCID {
    fn new() -> Self {
        ORCID::init().build()
    }
    /// Get ORCID schema URI
    /// ### Notes
    /// - Should always be "<https://orcid.org/>"
    fn schema_uri(&self) -> String {
        self.schema_uri.as_ref().cloned().unwrap_or_default().trim_end_matches("/").to_string()
    }
    /// Get ORCID identifier
    /// ### Notes
    /// - Will return an empty string if no identifier is present
    /// - Will always return a 19 character string with a hyphen every 4 characters (i.e., "0000-0000-0000-0000")
    fn identifier(&self) -> String {
        let stripped = self.identifier.as_ref().cloned().unwrap_or_default().replace("-", "");
        stripped.chunk(4).join("-")
    }
    fn suffix(&self) -> Option<String> {
        Some(self.identifier())
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        orcid_check_digit(self.identifier())
    }
}
impl PersistentIdentifier for RAID {
    fn new() -> Self {
        RAID::init().build()
    }
    fn schema_uri(&self) -> String {
        self.schema_uri.as_ref().cloned().unwrap_or_default().trim_end_matches("/").to_string()
    }
    fn prefix(&self) -> Option<String> {
        self.prefix.clone()
    }
    fn suffix(&self) -> Option<String> {
        self.suffix.clone()
    }
    fn identifier(&self) -> String {
        let values = [self.prefix(), self.suffix()];
        values
            .iter()
            .flatten()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/")
    }
}
impl PersistentIdentifier for ROR {
    fn new() -> Self {
        ROR::init().build()
    }
    fn schema_uri(&self) -> String {
        let processed = self
            .schema_uri
            .as_ref()
            .cloned()
            .unwrap_or_else(|| DEFAULT_ROR_SCHEMA_URI.to_string())
            .trim_end_matches("/")
            .replace(" ", "")
            .to_string();
        format!("{processed}/")
    }
    fn identifier(&self) -> String {
        self.identifier.clone().unwrap_or_default()
    }
    fn suffix(&self) -> Option<String> {
        self.identifier.clone()
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        ror_check_digit(&self.identifier()[1..])
    }
}
impl<T: AsRef<str>> PersistentIdentifierConvert<T> for T
where
    T: ToString,
{
    fn format_as(&self, pid_type: PID) -> String {
        match pid_type {
            | PID::ARK => ARK::format(self.as_ref()),
            | PID::DOI => DOI::format(self.as_ref()),
            | PID::ORCID => ORCID::format(self.as_ref()),
            | PID::RAID => RAID::format(self.as_ref()),
            | PID::ROR => <ROR as PersistentIdentifierParse>::format(self.as_ref()),
            | _ => self.as_ref().to_string(),
        }
    }
    fn to_pid(&self, pid_type: PID) -> PersistentIdentifierInternal {
        let value = self.as_ref().to_string();
        match pid_type {
            | PID::ARK => PersistentIdentifierInternal { value, pid_type: PID::ARK },
            | PID::DOI => PersistentIdentifierInternal { value, pid_type: PID::DOI },
            | PID::ORCID => PersistentIdentifierInternal { value, pid_type: PID::ORCID },
            | PID::RAID => PersistentIdentifierInternal { value, pid_type: PID::RAID },
            | PID::ROR => PersistentIdentifierInternal { value, pid_type: PID::ROR },
            | _ => PersistentIdentifierInternal::default(),
        }
    }
    fn is_pid(&self, pid_type: PID) -> bool {
        match pid_type {
            | PID::ARK => self.is_ark(),
            | PID::DOI => self.is_doi(),
            | PID::ORCID => self.is_orcid(),
            | PID::RAID => self.is_raid(),
            | PID::ROR => self.is_ror(),
            | _ => false,
        }
    }
    fn is_ark(&self) -> bool {
        ARK::is_valid(self.as_ref())
    }
    fn is_doi(&self) -> bool {
        DOI::is_valid(self.as_ref())
    }
    fn is_isbn(&self) -> bool {
        ISBN::is_valid(self.as_ref())
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
    /// Convert a `PersistentIdentifierInternal` to a `DOI`
    pub fn to_doi(&self) -> DOI {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::DOI => DOI::from_string(value),
            | _ => DOI::default(),
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
}
impl PersistentIdentifierParse for ARK {
    /// Find all [`ARK`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ARK;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| ARK::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting an [`ARK`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ARK, PersistentIdentifierParse};
    ///
    /// assert_eq!(ARK::format("ark:/1234/5678"), "ark:1234/5678");
    /// let expected = "https://n2t.net/ark:12148/btv1b8449691v/f29";
    /// assert_eq!(ARK::format(expected), expected);
    /// ```
    fn format(value: impl ToString) -> String {
        ARK::from_string(value.to_string()).to_string()
    }
    /// Create new [`ARK`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ARK, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let ark = ARK::from_string("https://n2t.net/ark:12148/btv1b8449691v/f42");
    /// assert_eq!(ark.suffix(), Some("f42".to_string()));
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["nma", "label", "naan", "assigned_name", "parts", "variants"];
        let pattern = format!("^{RE_ARK_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        let parts = match lookup.get("parts") {
            | Some(value) => value.split('/').map(String::from).collect(),
            | None => vec![],
        };
        let variants = match lookup.get("variants") {
            | Some(value) => value.split('.').map(String::from).collect(),
            | None => vec![],
        };
        ARK::init()
            .maybe_assigned_name(lookup.get("assigned_name").cloned())
            .maybe_label(lookup.get("label").cloned())
            .maybe_name_assigning_authority_number(lookup.get("naan").cloned())
            .maybe_name_mapping_authority(lookup.get("nma").cloned())
            .parts(parts)
            .variants(variants)
            .build()
    }
    /// Check if value is a valid [`ARK`]
    /// ### Conditions
    /// - ARKs are preferred to be "actionable" with the inclusion of a NMA URL, but are not required to be so (NMA is optional)
    /// - If ARK is to contain a URL, "https" is the only allowed scheme
    /// - Should have only one instance of "ark:" label
    /// - NAAN should be an integer
    /// - [Assigned name](`ARK::assigned_name`) should start with a valid [shoulder](https://arks.org/about/shoulders/)
    /// - Last character should be valid check digit (see [`noid_check_digit`])
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ARK, PersistentIdentifierParse};
    ///
    /// assert!(ARK::is_valid("ark:99166/w66d60p2"));
    /// assert!(ARK::is_valid("https://n2t.net/ark:12148/btv1b8449691v/f29"));
    /// ```
    fn is_valid(value: impl ToString) -> bool {
        let pid = ARK::from_string(value);
        let naan = pid.name_assigning_authority_number.unwrap_or_default();
        let naan_is_betanumeric = naan.chars().all(|x| x.is_betanumeric());
        let shoulder_starts_with_lowercase_letter = match pid.assigned_name {
            | Some(value) => match value.chars().next() {
                | Some(value) => value.is_ascii_lowercase() && !value.eq(&'l'),
                | None => false,
            },
            | None => false,
        };
        !naan.is_empty() && naan_is_betanumeric && shoulder_starts_with_lowercase_letter
    }
}
impl PersistentIdentifierParse for DOI {
    /// Find all [`DOI`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_DOI;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| DOI::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`DOI`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{DOI, PersistentIdentifierParse};
    ///
    /// assert_eq!(DOI::format("https://doi.org/10.1000/182"), "10.1000/182");
    /// assert_eq!(DOI::format("10.1000/182"), "10.1000/182");
    /// ```
    fn format(value: impl ToString) -> String {
        DOI::from_string(value).to_string()
    }
    /// Create new [`DOI`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{DOI, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let doi = DOI::from_string("https://doi.org/10.1000/182");
    /// assert_eq!(doi.prefix(), Some("10.1000".into()));
    /// assert_eq!(doi.suffix(), Some("182".into()));
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "directory_indicator", "prefix_element", "registrant_code", "suffix"];
        let pattern = format!("^{RE_DOI_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        DOI::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_directory_indicator(lookup.get("directory_indicator").cloned())
            .maybe_registrant_code(lookup.get("registrant_code").cloned())
            .maybe_suffix(lookup.get("suffix").cloned())
            .build()
    }
    /// Check if value is a valid [`DOI`]
    /// ### Conditions
    /// - Must match DOI regular expression (see [`RE_DOI_TEXT`])
    /// - Is valid with or without schema URI[^format]
    /// - `10.5555/` is not a valid DOI prefix
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{DOI, PersistentIdentifierParse};
    ///
    /// assert!(DOI::is_valid("https://doi.org/10.1000/182"));
    /// assert!(DOI::is_valid("10.1000/182"));
    /// assert!(!DOI::is_valid("10.5555/182"));
    /// ```
    ///
    /// [^format]: Use `DOI::format(value)` to ensure value is formatted correctly
    fn is_valid(value: impl ToString) -> bool {
        let pid = DOI::from_string(value.to_string());
        let prefix_is_valid = match pid.prefix() {
            | Some(x) => is_numeric(&x.replace(".", "")) && !x.eq("10.5555"),
            | _ => false,
        };
        let suffix_is_valid = pid.suffix().is_some();
        prefix_is_valid && suffix_is_valid
    }
}
impl PersistentIdentifierParse for ISBN {
    /// Find all [`ISBN`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ISBN;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| ISBN::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`ISBN`] from a string value
    fn format(value: impl ToString) -> String {
        ISBN::from_string(value).to_string()
    }
    /// Create new [`ISBN`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ISBN, PersistentIdentifierParse};
    ///
    /// let isbn = ISBN::from_string("978-0-306-40627-0");
    /// assert_eq!(isbn.prefix_element, Some("978".to_string()));
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["prefix_element", "registration_group", "publisher", "title", "check_digit"];
        let pattern = format!("^{RE_ISBN_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        ISBN::init()
            .maybe_prefix_element(lookup.get("prefix_element").cloned())
            .maybe_registration_group(lookup.get("registration_group").cloned())
            .maybe_publisher(lookup.get("publisher").cloned())
            .maybe_title(lookup.get("title").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid [`ISBN`]
    /// ### Conditions
    /// - Must be exactly 13 digits long (not including hyphens)
    /// - Must have a valid check digit (see [`isbn_check_digit`])
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ISBN, PersistentIdentifierParse};
    ///
    /// let isbn = ISBN::from_string("978-0-306-40627-0");
    /// assert!(ISBN::is_valid("978-0-306-40627-0"));
    /// assert!(ISBN::is_valid("9780306406270"));
    /// ```
    fn is_valid(value: impl ToString) -> bool {
        let pid = ISBN::from_string(value.to_string());
        let last = value.to_string().chars().last().unwrap_or_default();
        let has_valid_check_digit = match pid.check_digit() {
            | Some(chars) => chars.contains(&last),
            | _ => false,
        };
        let is_valid_length = value.to_string().replace("-", "").len() == 13;
        has_valid_check_digit && is_valid_length
    }
}
impl PersistentIdentifierParse for ORCID {
    /// Find all [`ORCID`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ORCID;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| ORCID::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`ORCID`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ORCID, PersistentIdentifierParse};
    ///
    /// assert_eq!(ORCID::format("https://orcid.org/0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(ORCID::format("0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    fn format(value: impl ToString) -> String {
        ORCID::from_string(value).to_string()
    }
    /// Create new [`ORCID`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ORCID, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let orcid = ORCID::from_string("https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(orcid.identifier(), "0000-0002-2057-9115");
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "identifier", "check_digit"];
        let pattern = format!("^{RE_ORCID_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        ORCID::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_identifier(lookup.get("identifier").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid [`ORCiD`]
    /// ### Conditions
    /// - ORCiD identifier must be 16 characters, 0 thru 9, or "X"
    /// - Last character of identifier must be a valid ISO 7064 check digit (see [`orcid_check_digit`])
    /// - Value can be valid with or without hyphens in the ORCiD identifier[^format]
    /// - Value can be valid with or without schema URI[^format]
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ORCID, PersistentIdentifierParse};
    ///
    /// assert!(ORCID::is_valid("https://orcid.org/0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000000220579115"));
    /// ```
    ///
    /// [^format]: Use `ORCID::format(value)` to ensure value is formatted correctly
    fn is_valid(value: impl ToString) -> bool {
        let pid = ORCID::from_string(value.to_string());
        let identifier = pid.identifier();
        let last = identifier.chars().last().unwrap_or_default();
        match orcid_check_digit(identifier.as_str()) {
            | Some(check_digit) => {
                if check_digit.contains(&last) {
                    identifier.len() == 19
                } else {
                    false
                }
            }
            | _ => false,
        }
    }
}
impl PersistentIdentifierParse for RAID {
    /// Find all [`RAID`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_DOI;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| RAID::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`RAID`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{RAID, PersistentIdentifierParse};
    ///
    /// assert_eq!(RAID::format("https://raid.org/10.83962/fb5be317"), "10.83962/fb5be317");
    /// ```
    fn format(value: impl ToString) -> String {
        RAID::from_string(value).to_string()
    }
    /// Create new [`RAID`] by parsing a raw string value
    /// ### Note
    /// > RAiD identifiers are [`DOI`] identifiers. See this [blog post by DataCite](https://datacite.org/blog/datacite-ardc-announce-partnership-to-deliver-the-raid-service/) for details.
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "directory_indicator", "registrant_code", "suffix"];
        let pattern = format!("^{RE_RAID_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        let directory_indicator = lookup.get("directory_indicator").cloned();
        let registrant_code = lookup.get("registrant_code").cloned();
        let prefix = [directory_indicator, registrant_code]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join(".");
        RAID::init()
            .prefix(prefix)
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_suffix(lookup.get("suffix").cloned())
            .build()
    }
    /// Check if value is a valid [`RAID`]
    /// > See [`DOI::is_valid`] for conditions, as RAiD identifiers are [`DOI`]s
    fn is_valid(value: impl ToString) -> bool {
        let pid = RAID::from_string(value.to_string());
        let prefix_is_valid = match pid.prefix() {
            | Some(x) => is_numeric(&x.replace(".", "")) && !x.eq("10.5555"),
            | _ => false,
        };
        let suffix_is_valid = pid.suffix().is_some();
        prefix_is_valid && suffix_is_valid
    }
}
impl PersistentIdentifierParse for ROR {
    /// Find all [`ROR`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ROR;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| ROR::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`ROR`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ROR, PersistentIdentifierParse};
    ///
    /// assert_eq!(ROR::format("https://ror.org/01qz5mb56"), "https://ror.org/01qz5mb56");
    /// assert_eq!(ROR::format("01qz5mb56"), "https://ror.org/01qz5mb56");
    /// ```
    fn format(value: impl ToString) -> String {
        ROR::from_string(value.to_string()).to_string()
    }
    /// Create new [`ROR`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ROR, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let ror = ROR::from_string("https://ror.org/01qz5mb56");
    /// assert_eq!(ror.identifier(), "01qz5mb56");
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "identifier", "check_digit"];
        let pattern = format!("^{RE_ROR_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        ROR::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_identifier(lookup.get("identifier").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid [`ROR`]
    /// ### Conditions
    /// - Exactly 9 characters long
    /// - Must have a valid check digits (last two characters are zero-padded checksum, 01-98) (see [`ror_check_digit`])
    /// - [Base32 Crockford](https://www.crockford.com/base32.html) encoded (i.e., digits 0-9 and letters A-Z except for I, L, O, and U)
    /// - Value can be valid with or without schema URI[^format]
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ROR, PersistentIdentifierParse};
    ///
    /// assert!(ROR::is_valid("https://ror.org/01qz5mb56"));
    /// assert!(ROR::is_valid("01qz5mb56"));
    /// ```
    ///
    /// [^format]: Use `ROR::format(value)` to ensure value is formatted correctly
    fn is_valid(value: impl ToString) -> bool {
        let pid = ROR::from_string(value.to_string());
        let identifier = pid.identifier();
        let last_two = identifier.chars().rev().take(2).collect::<String>().chars().rev().collect::<String>();
        if identifier.is_empty() {
            false
        } else {
            match ror_check_digit(&identifier[1..]) {
                | Some(check_digit) => {
                    if identifier.len() == 9 {
                        let calculated_last_two = check_digit.iter().collect::<String>();
                        calculated_last_two == last_two
                    } else {
                        false
                    }
                }
                | _ => false,
            }
        }
    }
}
/// ISBN check digit
/// ### Notes
/// - The check digit is the last (13th) digit of the identifier
/// - Each digit, from left to right, is alternately multiplied by 1 or 3, then those products are summed modulo 10
#[allow(clippy::arithmetic_side_effects)]
pub fn isbn_check_digit<S>(_value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
    const MODULUS: u32 = 10;
    let working = _value.as_ref().replace("-", "");
    let sum = working.chars().take(12).enumerate().fold(0, |acc, (index, x)| {
        let digit = x.to_digit(10).unwrap_or_default();
        let multiplier = if index % 2 == 0 { 1 } else { 3 };
        acc + (digit * multiplier)
    });
    let remainder = sum % MODULUS;
    let result = if remainder == 0 { 0 } else { MODULUS - remainder };
    char::from_digit(result, 10).map(|c| vec![c])
}
/// Calculate check xdigit ("extended digit") IAW [NOID check digit algorithm (NCDA)](https://metacpan.org/dist/Noid/view/noid#NOID-CHECK-DIGIT-ALGORITHM)
/// ### Notes
/// - Check digits are not expected to cover qualifiers
/// - If check digit is present in an ARK, by convention it is the right-most character of the so called "check zone"
/// - The "check zone" is composed of the NAAN and assigned name, separated by a forward slash
/// - Forward slashes do not contribute to the check digit sum, but do impact the character position index
/// - NCDA is guaranteed against single-character errors
/// - NCDA is guaranteed against transposition of two single characters
/// ### References
/// - <https://github.com/internetarchive/arklet>
/// - <https://github.com/no-reply/pynoid>
#[allow(clippy::arithmetic_side_effects)]
pub fn noid_check_digit<S>(value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
    const RADIX: usize = 29;
    let sum = value.as_ref().chars().enumerate().fold(0, |acc, (i, val)| {
        let position = i + 1;
        let ordinal = val.to_betanumeric_ordinal().unwrap_or(0);
        acc + (position * ordinal)
    });
    let remainder = sum % RADIX;
    to_betanumeric(remainder as u8).map(|c| vec![c])
}
/// Calculate check digit IAW [ISO 7064, MOD 11-2](https://www.iso.org/standard/31531.html)
///
/// "MOD 11-2" means modulus = 11 and radix = 2
///
/// ### Example
/// ```rust
/// use acorn::schema::pid::orcid_check_digit;
///
/// assert_eq!(orcid_check_digit("0000000220579115"), Some(vec!['5']));
/// assert_eq!(orcid_check_digit("0000-0002-2057-9115"), Some(vec!['5']));
/// ```
#[allow(clippy::arithmetic_side_effects)]
pub fn orcid_check_digit<S>(value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
    const MODULUS: u32 = 11;
    const RADIX: u32 = 2;
    let working = value.as_ref().replace("-", "").replace(" ", "");
    let sum = working.chars().take(15).fold(0, |acc, x| {
        let digit = x.to_digit(10).unwrap_or_default();
        (acc + digit) * RADIX
    });
    let remainder = sum % MODULUS;
    let result = (MODULUS + 1 - remainder) % MODULUS;
    if result == 10 {
        Some(vec!['X'])
    } else {
        char::from_digit(result, 10).map(|c| vec![c])
    }
}
/// Calculate check digit IAW [ISO 7064, MOD 97-10](https://www.iso.org/standard/31531.html)
///
/// "MOD 97-10" means modulus = 97 and radix = 10
///
/// ### Example
/// ```rust
/// use acorn::schema::pid::ror_check_digit;
///
/// assert_eq!(ror_check_digit("1qz5mb"), Some(vec!['5', '6']));
/// ```
/// ### References
/// - [ROR community Python implementation](https://github.com/ror-community/ror-api/blob/bd040a0d2558a478c06a89118a29eeb9b6142710/rorapi/management/commands/generaterorid.py)
/// - [DataCite Ruby implementation](https://github.com/datacite/base32-url/blob/master/lib/base32/url.rb)
#[allow(clippy::arithmetic_side_effects)]
pub fn ror_check_digit<S>(value: S) -> Option<Vec<char>>
where
    S: AsRef<str>,
{
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
    match base32_crockford_decode(working) {
        | Some(value) => {
            let remainder = (value * 100) % MODULUS;
            let checksum = (MODULUS + 1 - remainder) % MODULUS;
            let result = if checksum < 10 {
                format!("0{}", checksum).chars().collect()
            } else {
                checksum.to_string().chars().collect()
            };
            Some(result)
        }
        | None => None,
    }
}
fn to_betanumeric(value: u8) -> Option<char> {
    match BETANUMERIC_DIGITS.chars().enumerate().find(|(i, _)| *i == value as usize) {
        | Some((_, x)) => Some(x),
        | None => None,
    }
}
fn is_numeric(value: &str) -> bool {
    value.chars().all(|x| x.is_numeric())
}

#[cfg(test)]
mod tests;
