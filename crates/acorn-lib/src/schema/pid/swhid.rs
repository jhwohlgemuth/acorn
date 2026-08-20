//! Software Hash Identifier (SWHID) parsing and formatting.
use crate::prelude::*;
use crate::schema::pid::{PersistentIdentifier, PersistentIdentifierParse};
use crate::util::constants::app::{IDENTIFIERS_ORG_URI, N2T_URI, SOFTWARE_HERITAGE_ARCHIVE_URI, SOFTWARE_HERITAGE_RESOLVER_URI, SWHID_SCHEMA_URI};
use crate::util::constants::RE_SWHID;
use core::fmt;
use core::str::FromStr;
use fluent_uri::UriRef;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// SWHID object type
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectType {
    /// File content.
    #[serde(rename = "cnt")]
    Content,
    /// Directory tree.
    #[serde(rename = "dir")]
    Directory,
    /// Version-control release.
    #[serde(rename = "rel")]
    Release,
    /// Version-control revision.
    #[serde(rename = "rev")]
    Revision,
    /// Repository snapshot.
    #[serde(rename = "snp")]
    Snapshot,
}
/// Inclusive SWHID fragment range
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FragmentRange {
    /// First line or byte.
    pub start: u64,
    /// Optional inclusive last line or byte.
    pub end: Option<u64>,
}
/// Qualified SWHID context and fragment selectors
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Qualifiers {
    /// Software origin URI.
    pub origin: Option<String>,
    /// Snapshot where the object was observed.
    pub visit: Option<Box<SWHID>>,
    /// Directory, revision, release, or snapshot anchoring `path`.
    pub anchor: Option<Box<SWHID>>,
    /// Absolute path from the anchor root.
    pub path: Option<String>,
    /// Inclusive content line range.
    pub lines: Option<FragmentRange>,
    /// Inclusive content byte range.
    pub bytes: Option<FragmentRange>,
}
/// A SoftWare Hash IDentifier (SWHID) v1.2 core or qualified identifier
///
/// See [ISO/IEC 18670:2025](https://www.iso.org/standard/89985.html) and the
/// [canonical SWHID v1.2 specification](https://www.swhid.org/specification/v1.2/).
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SWHID {
    /// Identified software object type
    pub object_type: ObjectType,
    /// Lowercase hexadecimal 20-byte object identifier
    pub object_id: String,
    /// Optional context and fragment qualifiers
    pub qualifiers: Qualifiers,
}
/// Error returned when a SWHID cannot be parsed or validated
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwhidParseError {
    message: String,
}
impl fmt::Display for FragmentRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.end {
            | Some(end) => write!(formatter, "{}-{end}", self.start),
            | None => write!(formatter, "{}", self.start),
        }
    }
}
impl FromStr for ObjectType {
    type Err = SwhidParseError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            | "cnt" => Ok(Self::Content),
            | "dir" => Ok(Self::Directory),
            | "rel" => Ok(Self::Release),
            | "rev" => Ok(Self::Revision),
            | "snp" => Ok(Self::Snapshot),
            | _ => Err(SwhidParseError::new("unsupported SWHID object type")),
        }
    }
}
impl ObjectType {
    /// Return the SWHID object tag
    pub const fn as_str(self) -> &'static str {
        match self {
            | Self::Content => "cnt",
            | Self::Directory => "dir",
            | Self::Release => "rel",
            | Self::Revision => "rev",
            | Self::Snapshot => "snp",
        }
    }
}
impl Default for SWHID {
    fn default() -> Self {
        Self {
            object_type: ObjectType::Content,
            object_id: String::new(),
            qualifiers: Qualifiers::default(),
        }
    }
}
impl fmt::Display for SWHID {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.core_identifier())?;
        let Qualifiers {
            origin,
            visit,
            anchor,
            path,
            lines,
            bytes,
        } = &self.qualifiers;
        origin
            .as_ref()
            .map(|value| write!(formatter, ";origin={}", encode_qualifier(value)))
            .transpose()?;
        visit
            .as_ref()
            .map(|value| write!(formatter, ";visit={}", value.core_identifier()))
            .transpose()?;
        anchor
            .as_ref()
            .map(|value| write!(formatter, ";anchor={}", value.core_identifier()))
            .transpose()?;
        path.as_ref()
            .map(|value| write!(formatter, ";path={}", encode_qualifier(value)))
            .transpose()?;
        lines.as_ref().map(|value| write!(formatter, ";lines={value}")).transpose()?;
        bytes.as_ref().map(|value| write!(formatter, ";bytes={value}")).transpose()?;
        Ok(())
    }
}
impl FromStr for SWHID {
    type Err = SwhidParseError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = strip_resolver(value.trim());
        let mut parts = value.split(';');
        let core = parts.next().unwrap_or_default();
        parse_core(core).and_then(|(object_type, object_id)| {
            parts
                .try_fold(Qualifiers::default(), parse_qualifier)
                .map(|qualifiers| Self {
                    object_type,
                    object_id,
                    qualifiers,
                })
                .and_then(|parsed| parsed.validate().map(|()| parsed))
        })
    }
}
impl PersistentIdentifier for SWHID {
    fn new() -> Self {
        Self::default()
    }
    fn schema_uri(&self) -> String {
        SWHID_SCHEMA_URI.to_string()
    }
    fn identifier(&self) -> String {
        self.to_string()
    }
    fn prefix(&self) -> Option<String> {
        Some(format!("swh:1:{}", self.object_type.as_str()))
    }
    fn suffix(&self) -> Option<String> {
        (!self.object_id.is_empty()).then(|| self.object_id.clone())
    }
    fn url(&self) -> String {
        format!("{SOFTWARE_HERITAGE_ARCHIVE_URI}/{self}")
    }
}
impl PersistentIdentifierParse for SWHID {
    fn find_all(value: impl ToString) -> Vec<Self> {
        RE_SWHID
            .find_iter(&value.to_string())
            .filter_map(Result::ok)
            .filter_map(|matched| trim_candidate(matched.as_str()).parse().ok())
            .collect()
    }
    fn format(value: impl ToString) -> String {
        value
            .to_string()
            .parse::<Self>()
            .map_or_else(|_| String::new(), |identifier| identifier.to_string())
    }
    fn from_string(value: impl ToString) -> Self {
        value.to_string().parse().unwrap_or_default()
    }
    fn is_valid(value: impl ToString) -> bool {
        value.to_string().parse::<Self>().is_ok()
    }
}
impl SWHID {
    /// Return the canonical core identifier without qualifiers
    pub fn core_identifier(&self) -> String {
        format!("swh:1:{}:{}", self.object_type.as_str(), self.object_id)
    }
    fn validate(&self) -> Result<(), SwhidParseError> {
        let object_is_content = self.object_type == ObjectType::Content;
        let fragment_count = usize::from(self.qualifiers.lines.is_some()).saturating_add(usize::from(self.qualifiers.bytes.is_some()));
        let fragment_is_valid = fragment_count <= 1 && (fragment_count == 0 || object_is_content);
        let visit_is_valid = self.qualifiers.visit.as_ref().is_none_or(|visit| {
            self.qualifiers.origin.is_some() && visit.object_type == ObjectType::Snapshot && visit.qualifiers == Qualifiers::default()
        });
        let anchor_is_valid = self.qualifiers.anchor.as_ref().is_none_or(|anchor| {
            self.qualifiers.path.is_some() && anchor.qualifiers == Qualifiers::default() && anchor.object_type != ObjectType::Content
        });
        let path_is_valid = self.qualifiers.path.is_none() == self.qualifiers.anchor.is_none();
        match (fragment_is_valid, visit_is_valid, anchor_is_valid, path_is_valid) {
            | (false, _, _, _) => Err(SwhidParseError::new(
                "SWHID fragment qualifiers require content and are mutually exclusive",
            )),
            | (_, false, _, _) => Err(SwhidParseError::new("SWHID visit requires origin and a core snapshot identifier")),
            | (_, _, false, _) => Err(SwhidParseError::new("SWHID anchor requires path and cannot identify content")),
            | (_, _, _, false) => Err(SwhidParseError::new("SWHID path and anchor must be supplied together")),
            | _ => Ok(()),
        }
    }
}
impl core::error::Error for SwhidParseError {}
impl fmt::Display for SwhidParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl SwhidParseError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}
fn encode_qualifier(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            | '%' => "%25".chars().collect::<Vec<_>>(),
            | ';' => "%3B".chars().collect(),
            | ' ' => "%20".chars().collect(),
            | character => vec![character],
        })
        .collect()
}
fn parse_core(value: &str) -> Result<(ObjectType, String), SwhidParseError> {
    let mut parts = value.split(':');
    let scheme = parts.next().unwrap_or_default();
    let version = parts.next().unwrap_or_default();
    let object_type = parts.next().unwrap_or_default();
    let object_id = parts.next().unwrap_or_default();
    let complete = parts.next().is_none();
    let digest_is_valid = object_id.len() == 40 && object_id.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    match (scheme, version, complete, digest_is_valid) {
        | ("swh", "1", true, true) => object_type.parse().map(|object_type| (object_type, object_id.to_string())),
        | (_, "1", _, _) if scheme != "swh" => Err(SwhidParseError::new("invalid SWHID scheme")),
        | ("swh", _, _, _) if version != "1" => Err(SwhidParseError::new("unsupported SWHID scheme version")),
        | _ => Err(SwhidParseError::new("invalid SWHID core identifier")),
    }
}
fn parse_origin(value: &str) -> Result<String, SwhidParseError> {
    let valid_uri = valid_percent_encoding(value) && UriRef::parse(value).ok().is_some_and(|uri| uri.scheme().is_some());
    match (valid_uri, urlencoding::decode(value)) {
        | (true, Ok(value)) => Ok(value.into_owned()),
        | _ => Err(SwhidParseError::new("invalid SWHID origin URI")),
    }
}
fn parse_path(value: &str) -> Result<String, SwhidParseError> {
    match valid_percent_encoding(value).then(|| urlencoding::decode(value)) {
        | Some(Ok(value)) if value.starts_with('/') => Ok(value.into_owned()),
        | _ => Err(SwhidParseError::new("SWHID path must be absolute and correctly escaped")),
    }
}
fn parse_qualifier(qualifiers: Qualifiers, value: &str) -> Result<Qualifiers, SwhidParseError> {
    let (key, raw) = value.split_once('=').ok_or_else(|| SwhidParseError::new("invalid SWHID qualifier"))?;
    match (raw.is_empty(), key) {
        | (true, _) => Err(SwhidParseError::new("empty SWHID qualifier")),
        | (false, "origin") if qualifiers.origin.is_none() => parse_origin(raw).map(|origin| Qualifiers {
            origin: Some(origin),
            ..qualifiers
        }),
        | (false, "visit") if qualifiers.visit.is_none() => raw.parse().map(|visit| Qualifiers {
            visit: Some(Box::new(visit)),
            ..qualifiers
        }),
        | (false, "anchor") if qualifiers.anchor.is_none() => raw.parse().map(|anchor| Qualifiers {
            anchor: Some(Box::new(anchor)),
            ..qualifiers
        }),
        | (false, "path") if qualifiers.path.is_none() => parse_path(raw).map(|path| Qualifiers {
            path: Some(path),
            ..qualifiers
        }),
        | (false, "lines") if qualifiers.lines.is_none() => parse_range(raw, 1).map(|lines| Qualifiers {
            lines: Some(lines),
            ..qualifiers
        }),
        | (false, "bytes") if qualifiers.bytes.is_none() => parse_range(raw, 0).map(|bytes| Qualifiers {
            bytes: Some(bytes),
            ..qualifiers
        }),
        | (false, "origin" | "visit" | "anchor" | "path" | "lines" | "bytes") => Err(SwhidParseError::new("duplicate SWHID qualifier")),
        | _ => Err(SwhidParseError::new("unknown SWHID qualifier")),
    }
}
fn parse_range(value: &str, minimum: u64) -> Result<FragmentRange, SwhidParseError> {
    let parsed = value.split_once('-').map_or_else(
        || value.parse::<u64>().map(|start| (start, None)),
        |(start, end)| start.parse::<u64>().and_then(|start| end.parse::<u64>().map(|end| (start, Some(end)))),
    );
    match parsed {
        | Ok((start, end)) if start >= minimum && end.is_none_or(|end| end >= start) => Ok(FragmentRange { start, end }),
        | _ => Err(SwhidParseError::new("invalid SWHID fragment range")),
    }
}
fn strip_resolver(value: &str) -> &str {
    let stripped = [
        SOFTWARE_HERITAGE_RESOLVER_URI,
        SOFTWARE_HERITAGE_ARCHIVE_URI,
        IDENTIFIERS_ORG_URI,
        N2T_URI,
    ]
    .into_iter()
    .find_map(|prefix| value.strip_prefix(prefix).and_then(|value| value.strip_prefix('/')))
    .unwrap_or(value);
    if stripped.contains(';') {
        stripped
    } else {
        stripped.strip_suffix('/').unwrap_or(stripped)
    }
}
fn trim_candidate(value: &str) -> &str {
    value.trim_matches(|character: char| matches!(character, '<' | '>' | '[' | ']' | '{' | '}' | '(' | ')' | ',' | '.'))
}
fn valid_percent_encoding(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.iter().enumerate().all(|(index, byte)| {
        *byte != b'%'
            || bytes
                .get(index.saturating_add(1)..index.saturating_add(3))
                .is_some_and(|pair| pair.iter().all(u8::is_ascii_hexdigit))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = "swh:1:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b";
    const REVISION: &str = "swh:1:rev:2db189928c94d62a3b4757b3eec68f0a4d4113f0";
    const SNAPSHOT: &str = "swh:1:snp:d7f1b9eb7ccb596c2622c4780febaa02549830f9";

    #[test]
    fn test_find_all_preserves_qualified_identifier() {
        let value = format!("See {CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;lines=9-15.");
        let found = SWHID::find_all(value);
        assert_eq!(found.len(), 1);
        assert_eq!(
            found.first().and_then(|identifier| identifier.qualifiers.lines),
            Some(FragmentRange { start: 9, end: Some(15) })
        );
    }
    #[test]
    fn test_parse_core_object_types() {
        ["cnt", "dir", "rel", "rev", "snp"].into_iter().for_each(|kind| {
            let value = format!("swh:1:{kind}:0123456789abcdef0123456789abcdef01234567");
            assert!(SWHID::is_valid(value));
        });
    }
    #[test]
    fn test_parse_qualified_identifier_canonicalizes_order() {
        let value = format!("{CONTENT};lines=9-15;path=/src/main.rs;anchor={REVISION};origin=https://example.org/repo;visit={SNAPSHOT}");
        let parsed: SWHID = value.parse().expect("qualified SWHID should parse");
        assert_eq!(
            parsed.to_string(),
            format!("{CONTENT};origin=https://example.org/repo;visit={SNAPSHOT};anchor={REVISION};path=/src/main.rs;lines=9-15")
        );
        assert_eq!(parsed.core_identifier(), CONTENT);
    }
    #[test]
    fn test_rejects_invalid_core_and_qualifiers() {
        let invalid = [
            "swh:2:cnt:0123456789abcdef0123456789abcdef01234567",
            "swh:1:CNT:0123456789abcdef0123456789abcdef01234567",
            "swh:1:cnt:0123456789ABCDEF0123456789abcdef01234567",
            "swh:1:cnt:0123456789abcdef",
            "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;lines=0",
            "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;lines=1;bytes=0",
            "swh:1:rev:0123456789abcdef0123456789abcdef01234567;lines=1",
            "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;unknown=value",
            "swh:1:cnt:0123456789abcdef0123456789abcdef01234567;origin=https://example.org/%zz",
        ];
        invalid.into_iter().for_each(|value| assert!(!SWHID::is_valid(value), "{value}"));
    }
    #[test]
    fn test_resolver_url_normalizes_to_bare_identifier() {
        let value = format!("https://archive.softwareheritage.org/{CONTENT}");
        assert_eq!(SWHID::format(value), CONTENT);
    }
}
