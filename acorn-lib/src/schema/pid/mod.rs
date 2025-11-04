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
use crate::constants::{DEFAULT_ORCID_SCHEMA_URI, RE_ARK_TEXT, RE_DOI, RE_DOI_TEXT, RE_ORCID, RE_ORCID_TEXT};
use crate::util::{regex_capture_lookup, ToStringChunks};
use bon::Builder;
use core::fmt::Display;

pub mod raid;

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
pub trait PersistentIdentifier: Display {
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
    fn suffix(&self) -> Option<String> {
        None
    }
    /// Get PID check digit (when applicable)
    fn check_digit(&self) -> Option<char> {
        None
    }
    /// Convert `self` into a string with a standard format
    fn format(&self) -> String {
        self.to_string()
    }
    /// Check if PID is valid
    fn is_valid(&self) -> bool {
        false
    }
}
/// Add coercion to persistent identifier (PID) functionality to string values
pub trait PersistentIdentifierConvert<T: AsRef<str>> {
    /// Convert `self` into a string standard format PID of a certain type
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert_eq!("https://doi.org/10.1234/5678".format_as(PID::DOI), "10.1234/5678");
    /// assert_eq!("0000-0002-2057-9115".format_as(PID::ORCID), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    fn format_as(&self, pid_type: PID) -> String;
    /// Coerce `self` into given PID type.
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// let doi = "https://doi.org/10.1234/5678".to_pid(PID::DOI).to_doi();
    /// assert_eq!(doi.suffix(), "5678");
    /// ```
    fn to_pid(&self, pid_type: PID) -> PersistentIdentifierInternal;
    /// Determines if `self` is of the given PID type.
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    /// assert!("https://doi.org/10.1234/5678".is_pid(PID::DOI));
    /// ```
    fn is_pid(&self, _pid_type: PID) -> bool {
        false
    }
    /// Determines if `self` is an archival resource key (ARK)
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://n2t.net/ark:12148/btv1b8449691v/f29".is_ark());
    /// ```
    fn is_ark(&self) -> bool {
        false
    }
    /// Determines if `self` is a DOI
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://doi.org/10.1234/5678".is_doi());
    /// ```
    fn is_doi(&self) -> bool {
        false
    }
    /// Determines if `self` is a ORCID
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://orcid.org/0000-0000-0000-0000".is_orcid());
    /// ```
    fn is_orcid(&self) -> bool {
        false
    }
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
    /// Open Researcher and Contributor ID (ORCiD)
    ///
    /// See [`ORCID`]
    ORCID,
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
    /// Unknown PID
    #[default]
    Unknown,
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
#[builder(start_fn = init)]
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
    /// - Since 2001, every assigned name assigning authority number (NAAN) has consisted of exactly five digits, specifically five [beta-numeric digits](`BETA_NUMERIC_DIGITS`)
    /// - Any given identifier will have exactly one NAAN but may have more than one NMA (at a time or over time)
    /// - Similar to registration authority or prefix for [`DOI`]s, naming authority for [Handles], and namespace identifier for [URNs]
    ///
    /// [Handles]: https://handle.net/
    /// [URNs]: https://en.wikipedia.org/wiki/Uniform_Resource_Name
    pub name_assigning_authority_number: Option<String>,
    /// String identifying a service that accepts names and returns information about them
    /// ### Notes
    /// > Any given identifier will have exactly one NAAN but may have more than one NMA (at a time or over time)
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
#[builder(start_fn = init)]
pub struct DOI {
    /// Schema URI (i.e., <https://doi.org/>)
    pub schema_uri: Option<String>,
    /// Directory indicator
    /// ### Rules
    /// - Can contain only numeric values
    /// - usually 10 but other indicators may be designated as compliant by the DOI Foundation
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
/// Open Researcher and Contributor ID (ORCiD)[^orcid]
///
/// Disambiguates researchers, and connects people with their research activities. This includes employment affiliations, research outputs, funding, peer review activity, research resources, society membership, distinctions and other scholarly infrastructure.
///
/// See <https://orcid.org/> for more information
///
/// [^orcid]: `L. L. Haak, M. Fenner, L. Paglione, E. Pentz, and H. Ratner, "ORCID: a system to uniquely identify researchers," Learned Publishing, vol. 25, no. 4, pp. 259-264, 2012, doi: 10.1087/20120404.`
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct ORCID {
    /// Schema URI (i.e., <https://orcid.org/>)
    pub schema_uri: Option<String>,
    /// 16 digit string with hyphens every 4 digits (for readability)
    /// <div class="warning">This value can be stored with or without hyphens. To ensure compliancy, use <code>ORCID::identifier</code> method to access ORCiD identifier.</div>
    pub identifier: Option<String>,
    /// The check digit is the last (16th) digit of the identifier
    /// ### Note
    /// Check digit should be verified IAW by [ISO 7064, MOD 11-2](https://www.iso.org/standard/31531.html) (see [`iso7064_check_digit`])
    pub check_digit: Option<String>,
}
impl ARK {
    /// Convenience method for easily parsing and formatting an ARK from a string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ARK;
    ///
    /// assert_eq!(ARK::format("ark:/1234/5678"), "ark:/1234/5678");
    /// assert_eq!(ARK::format("https://n2t.net/ark:12148/btv1b8449691v/f29"), "ark:12148/btv1b8449691v/f29");
    /// ```
    pub fn format<S>(value: S) -> String
    where
        S: AsRef<str>,
    {
        ARK::from_string(value).to_string()
    }
    /// Create new ARK by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ARK;
    ///
    /// let doi = ARK::from_string("");
    ///
    /// ```
    pub fn from_string<S>(value: S) -> ARK
    where
        S: AsRef<str>,
    {
        let names = ["nma", "schema_uri", "label", "naan", "assigned_name", "parts", "variants"];
        let lookup = regex_capture_lookup(RE_ARK_TEXT, value.as_ref(), names.to_vec());
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
    /// Check if value is a valid ARK
    /// ### Conditions
    /// - ARKs are preferred to be "actionable" with the inclusion of a NMA URL, but are not required to be so (NMA is optional)
    /// - If ARK is to contain a URL, "https" is the only allowed scheme
    /// - Should have only one instance of "ark:" label
    /// - NAAN should be an integer
    /// - [Assigned name](`ARK::assigned_name`) should start with a valid [shoulder](https://arks.org/about/shoulders/)
    /// - Last character should be valid check digit (see [`noid_check_digit`])
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ARK;
    ///
    /// assert!(ARK::is_valid("https://n2t.net/ark:99166/w66d60p2"));
    /// assert!(ARK::is_valid("https://n2t.net/ark:12148/btv1b8449691v/f29"));
    /// ```
    pub fn is_valid<S>(value: S) -> bool
    where
        S: AsRef<str>,
    {
        let ark = ARK::from_string(value);
        // TODO: Test NAAN length? Test betanumeric?
        let naan_is_integer = match ark.name_assigning_authority_number {
            | Some(value) => match value.parse::<u32>() {
                | Ok(_) => true,
                | Err(_) => false,
            },
            | None => false,
        };
        let shoulder_starts_with_lowercase_letter = match ark.assigned_name {
            | Some(value) => match value.chars().next() {
                | Some(value) => value.is_ascii_lowercase() && !value.eq(&'l'),
                | None => false,
            },
            | None => false,
        };
        naan_is_integer && shoulder_starts_with_lowercase_letter
    }
}
impl DOI {
    /// Convenience method for easily parsing and formatting a DOI from a string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::DOI;
    ///
    /// assert_eq!(DOI::format("https://doi.org/10.1000/182"), "10.1000/182");
    /// assert_eq!(DOI::format("10.1000/182"), "10.1000/182");
    /// ```
    pub fn format<S>(value: S) -> String
    where
        S: AsRef<str>,
    {
        DOI::from_string(value).to_string()
    }
    /// Create new DOI by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::DOI;
    ///
    /// let doi = DOI::from_string("https://doi.org/10.1000/182");
    /// assert_eq!(doi.prefix(), "10.1000");
    /// assert_eq!(doi.suffix(), "182");
    /// ```
    pub fn from_string<S>(value: S) -> DOI
    where
        S: AsRef<str>,
    {
        let names = ["schema_uri", "directory_indicator", "registrant_code", "suffix"];
        let lookup = regex_capture_lookup(RE_DOI_TEXT, value.as_ref(), names.to_vec());
        DOI::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_directory_indicator(lookup.get("directory_indicator").cloned())
            .maybe_registrant_code(lookup.get("registrant_code").cloned())
            .maybe_suffix(lookup.get("suffix").cloned())
            .build()
    }
    /// Check if value is a valid DOI
    /// ### Conditions
    /// - Must match DOI regular expression (see [`RE_DOI_TEXT`])
    /// - Is valid with or without schema URI[^format]
    /// - `10.5555/` is not a valid DOI prefix
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::DOI;
    ///
    /// assert!(DOI::is_valid("https://doi.org/10.1000/182"));
    /// assert!(DOI::is_valid("10.1000/182"));
    /// ```
    ///
    /// [^format]: Use `DOI::format(value)` to ensure value is formatted correctly
    pub fn is_valid<S>(value: S) -> bool
    where
        S: AsRef<str>,
    {
        match RE_DOI.is_match(value.as_ref()) {
            | Ok(x) if x && !value.as_ref().contains("10.5555/") => true,
            | _ => false,
        }
    }
}
impl ORCID {
    /// Convenience method for easily parsing and formatting a ORCID from a string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ORCID;
    ///
    /// assert_eq!(ORCID::format("https://orcid.org/0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(ORCID::format("0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    pub fn format<S>(value: S) -> String
    where
        S: AsRef<str>,
    {
        ORCID::from_string(value).to_string()
    }
    /// Create new ORCID by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ORCID;
    ///
    /// let orcid = ORCID::from_string("https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(orcid.identifier(), "0000-0002-2057-9115");
    /// ```
    pub fn from_string<S>(value: S) -> ORCID
    where
        S: AsRef<str>,
    {
        let names = ["schema_uri", "identifier", "check_digit"];
        let lookup = regex_capture_lookup(RE_ORCID_TEXT, value.as_ref(), names.to_vec());
        ORCID::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_identifier(lookup.get("identifier").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid ORCiD
    /// ### Conditions
    /// - ORCiD identifier must be 16 characters, 0 thru 9, or "X"
    /// - Last character of identifier must be a valid ISO 7064 check digit (see [`iso7064_check_digit`])
    /// - Value can be valid with or without hyphens in the ORCiD identifier[^format]
    /// - Value can be valid with or without schema URI[^format]
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ORCID;
    ///
    /// assert!(ORCID::is_valid("https://orcid.org/0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000000220579115"));
    /// ```
    ///
    /// [^format]: Use `ORCID::format(value)` to ensure value is formatted correctly
    pub fn is_valid<S>(value: S) -> bool
    where
        S: AsRef<str>,
    {
        let orcid = ORCID::from_string(value.as_ref());
        let identifier = orcid.identifier();
        let last = identifier.chars().last().unwrap_or_default();
        match iso7064_check_digit(identifier.as_str()) {
            | Some(check_digit) => match RE_ORCID.is_match(value.as_ref()) {
                | Ok(true) if check_digit.eq(&last) && identifier.len() == 19 => true,
                | _ => false,
            },
            | _ => false,
        }
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
impl Display for ARK {
    /// Format a ARK into a standard format of `"{NMA}{label}{NAAN}/{Assigned Name}/{Parts}{Variants}"`
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let nma = self.name_mapping_authority.clone().unwrap_or_default().trim_end_matches('/').to_string();
        let identifier = self.identifier();
        let result = [nma, identifier].into_iter().filter(|x| !x.is_empty()).collect::<Vec<String>>().join("/");
        write!(f, "{result}")
    }
}
impl Display for DOI {
    /// Format a DOI into a standard format of `"{prefix}/{suffix}"`
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let result = self.identifier();
        write!(f, "{result}")
    }
}
impl Display for ORCID {
    /// Format a ORCiD into a standard format of `"{schema_uri}{identifier}"`
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
            .join("");
        write!(f, "{result}")
    }
}
impl PersistentIdentifier for ARK {
    fn new() -> Self {
        ARK::init().build()
    }
    fn format(&self) -> String {
        self.to_string()
    }
    fn schema_uri(&self) -> String {
        let uri = match &self.name_mapping_authority {
            | Some(value) => value,
            | None => "",
        };
        uri.to_string()
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
        let Self {
            label,
            name_assigning_authority_number,
            assigned_name,
            ..
        } = self;
        if [name_assigning_authority_number.clone(), assigned_name.clone()]
            .iter()
            .all(|x| x.is_some())
        {
            let result = format!(
                "{}{}/{}",
                label.trim_end_matches('/'),
                name_assigning_authority_number.as_ref().unwrap(),
                assigned_name.as_ref().unwrap()
            );
            Some(result)
        } else {
            None
        }
    }
    /// Returns consistent string representation of ARK qualifiers which often name subobjects
    /// of a persistent object that are less stable and less opaquely named than the parent object
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
    fn check_digit(&self) -> Option<char> {
        let Self {
            name_assigning_authority_number: naan,
            assigned_name: name,
            ..
        } = self;
        let values = [naan.clone(), name.clone()];
        if values.iter().all(|x| x.is_some()) {
            let value = values.iter().flatten().map(String::from).collect::<Vec<String>>().join("/");
            noid_check_digit(value)
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
        self.schema_uri.as_ref().cloned().unwrap_or_default()
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
        let result = self.suffix.as_ref().cloned().unwrap_or_default();
        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
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
        self.schema_uri.as_ref().cloned().unwrap_or_default()
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
    fn check_digit(&self) -> Option<char> {
        iso7064_check_digit(self.identifier())
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
            | _ => self.as_ref().to_string(),
        }
    }
    fn to_pid(&self, _pid_type: PID) -> PersistentIdentifierInternal {
        match _pid_type {
            | PID::ARK => PersistentIdentifierInternal {
                value: self.as_ref().to_string(),
                pid_type: PID::ARK,
            },
            | PID::DOI => PersistentIdentifierInternal {
                value: self.as_ref().to_string(),
                pid_type: PID::DOI,
            },
            | PID::ORCID => PersistentIdentifierInternal {
                value: self.as_ref().to_string(),
                pid_type: PID::ORCID,
            },
            | _ => PersistentIdentifierInternal::default(),
        }
    }
    fn is_pid(&self, pid_type: PID) -> bool {
        match pid_type {
            | PID::ARK => self.is_ark(),
            | PID::DOI => self.is_doi(),
            | PID::ORCID => self.is_orcid(),
            | _ => false,
        }
    }
    fn is_ark(&self) -> bool {
        ARK::is_valid(self.as_ref())
    }
    fn is_doi(&self) -> bool {
        DOI::is_valid(self.as_ref())
    }
    fn is_orcid(&self) -> bool {
        ORCID::is_valid(self.as_ref())
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
}
impl Betanumeric for char {
    fn is_betanumeric(&self) -> bool {
        BETANUMERIC_DIGITS.contains(*self)
    }
    fn to_betanumeric_ordinal(&self) -> Option<usize> {
        BETANUMERIC_DIGITS.chars().position(|x| x.eq(self))
    }
}
/// Calculate check digit IAW [ISO 7064, MOD 11-2](https://www.iso.org/standard/31531.html)
///
/// "MOD 11-2" means modulus = 11 and radix = 2
///
/// ### Example
/// ```rust
/// use acorn_lib::schema::pid::iso7064_check_digit;
///
/// assert_eq!(iso7064_check_digit("0000000220579115"), 5);
/// assert_eq!(iso7064_check_digit("0000-0002-2057-9115"), 5);
/// ```
pub fn iso7064_check_digit<S>(value: S) -> Option<char>
where
    S: AsRef<str>,
{
    const MODULUS: u32 = 11;
    const RADIX: u32 = 2;
    let working = value.as_ref().replace("-", "");
    let sum = working.chars().take(15).fold(0, |acc, x| {
        let digit = x.to_digit(10).unwrap_or_default();
        (acc + digit) * RADIX
    });
    let remainder = sum % MODULUS;
    let result = (MODULUS + 1 - remainder) % MODULUS;
    if result == 10 {
        Some('X')
    } else {
        char::from_digit(result, 10)
    }
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
pub fn noid_check_digit<S>(value: S) -> Option<char>
where
    S: AsRef<str>,
{
    const RADIX: usize = 29;
    let sum = value.as_ref().chars().enumerate().fold(0, |acc, (i, val)| {
        let position = i + 1;
        let ordinal = match val.is_betanumeric() {
            | true => val.to_betanumeric_ordinal().unwrap(),
            | false => 0,
        };
        acc + (position * ordinal)
    });
    let remainder = sum % RADIX;
    to_betanumeric(remainder as u8)
}
fn to_betanumeric(value: u8) -> Option<char> {
    match BETANUMERIC_DIGITS.chars().enumerate().find(|(i, _)| *i == value as usize) {
        | Some((_, x)) => Some(x),
        | None => None,
    }
}

#[cfg(test)]
mod tests;
