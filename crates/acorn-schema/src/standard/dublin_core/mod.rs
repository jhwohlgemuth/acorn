//! Portable Dublin Core graph model and DCMI term registry.
use crate::codec::{CodecDiagnostic, DocumentCodec};
use crate::validation::{report_of, rules, Validate, ValidationReport};
use acorn_core::prelude::{format, vec, String, ToString, Vec};
use acorn_core::util::{MimeType, ToProse};
use acorn_core::validation::ValidationError;
use acorn_macros::With;
use alloc::collections::{BTreeMap, BTreeSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod relationships;
mod terms;

pub use crate::util::constants::app::{DCAM_NAMESPACE, DCTERMS_NAMESPACE, ELEMENTS_NAMESPACE, TYPE_NAMESPACE};
pub use relationships::relationships;
pub use terms::{term, terms};

/// One of the four namespaces published in the current DCMI vocabularies.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DublinCoreNamespace {
    /// DCMI Abstract Model vocabulary.
    AbstractModel,
    /// Legacy Dublin Core Metadata Element Set.
    MetadataElements,
    /// DCMI Metadata Terms vocabulary.
    MetadataTerms,
    /// DCMI Type Vocabulary.
    TypeVocabulary,
}
/// Kind of relationship explicitly declared by a published DCMI vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DublinCoreRelationshipKind {
    /// Formal RDF domain.
    Domain,
    /// Suggested DCAM domain.
    DomainIncludes,
    /// Equivalent RDF property.
    EquivalentProperty,
    /// Formal RDF range.
    Range,
    /// Suggested DCAM range.
    RangeIncludes,
    /// Informative related specification or registry.
    SeeAlso,
    /// RDFS superclass.
    SubClassOf,
}
/// An RDF resource identifier.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum DublinCoreResource {
    /// Document-scoped blank-node identifier without the `_:` prefix.
    BlankNode(String),
    /// Absolute IRI.
    Iri(String),
}
/// The kind of a term in a DCMI vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DublinCoreTermKind {
    /// RDF or RDFS class.
    Class,
    /// RDF property.
    Property,
    /// Datatype used to identify a literal syntax encoding scheme.
    SyntaxEncodingScheme,
    /// Class whose members form a controlled vocabulary.
    VocabularyEncodingScheme,
}
/// Publication status of a registered DCMI term.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DublinCoreTermStatus {
    /// A term in the legacy `/elements/1.1/` namespace.
    Legacy,
    /// A published current DCMI term.
    Published,
}
/// One RDF object value.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum DublinCoreValue {
    /// Literal object.
    Literal(DublinCoreLiteral),
    /// IRI or blank-node object.
    Resource(DublinCoreResource),
}
/// All statements about one RDF subject.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DublinCoreDescription {
    /// Predicate IRI to repeated object values.
    pub properties: BTreeMap<String, Vec<DublinCoreValue>>,
    /// IRI or blank-node subject.
    pub subject: DublinCoreResource,
    /// RDF class IRIs assigned to the subject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
}
/// A normalized Dublin Core RDF default graph.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DublinCoreDocument {
    /// Subject descriptions in deterministic order.
    pub descriptions: Vec<DublinCoreDescription>,
}
/// A literal RDF value.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DublinCoreLiteral {
    /// Optional absolute datatype IRI.
    pub datatype: Option<String>,
    /// Optional BCP 47 language tag.
    pub language: Option<String>,
    /// Lexical value.
    pub value: String,
}
/// One Dublin Core term IRI.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DublinCoreTerm(String);
/// Authoritative metadata for one registered DCMI term.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DublinCoreTermDefinition {
    /// English definition published as the term's RDFS comment.
    pub definition: &'static str,
    /// Date on which DCMI issued the term.
    pub issued: &'static str,
    /// Term kind.
    pub kind: DublinCoreTermKind,
    /// English preferred label.
    pub label: &'static str,
    /// Local name appended to the namespace IRI.
    pub local_name: &'static str,
    /// Namespace containing the term.
    pub namespace: DublinCoreNamespace,
    /// Publication status.
    pub status: DublinCoreTermStatus,
}
/// One relationship asserted for a registered DCMI term.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DublinCoreTermRelationship {
    /// Relationship kind.
    pub kind: DublinCoreRelationshipKind,
    /// Absolute target IRI.
    pub object: &'static str,
    /// Absolute registered term IRI.
    pub subject: &'static str,
}
impl DublinCoreDescription {
    fn normalized(self) -> Self {
        let Self {
            properties,
            subject,
            mut types,
        } = self;
        let properties = properties
            .into_iter()
            .map(|(predicate, mut values)| {
                values.sort();
                values.dedup();
                (predicate, values)
            })
            .collect();
        types.sort();
        types.dedup();
        Self { properties, subject, types }
    }
}
impl Validate for DublinCoreDescription {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = self.types.iter().enumerate().fold(
            ValidationReport::new().merge("subject", report_of(self.subject.validate())),
            |mut report, (index, value)| {
                if let Err(error) = rules::url(value) {
                    report.add(format!("types[{index}]"), error.with_message("Provide an absolute RDF class IRI"));
                }
                report
            },
        );
        self.properties
            .iter()
            .fold(report, |report, (predicate, values)| {
                let mut nested = ValidationReport::empty("", values.is_empty());
                if let Err(error) = rules::url(predicate) {
                    nested.add("", error.with_message("Provide an absolute RDF predicate IRI"));
                }
                values.iter().enumerate().for_each(|(index, value)| {
                    if let Err(value_report) = value.validate() {
                        nested.append_prefixed(&format!("[{index}]"), value_report);
                    }
                });
                report.merge(&format!("properties.{predicate}"), nested)
            })
            .finish()
    }
}
impl DocumentCodec for DublinCoreDocument {
    const NAME: &'static str = "Dublin Core document";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        let decoded = match format {
            | MimeType::Json => serde_json::from_str(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
            | MimeType::LdJson | MimeType::RdfXml | MimeType::Turtle => super::rdf::decode(content, format),
            | MimeType::Yaml => serde_norway::from_str(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
            | _ => Err(vec![CodecDiagnostic::unsupported(format, Self::NAME)]),
        };
        decoded.and_then(|document: Self| {
            document.validate().map_or_else(
                |report| {
                    Err(report
                        .iter()
                        .map(|issue| CodecDiagnostic::deserialize(issue.error.to_string()).with_path(Some(issue.path.clone())))
                        .collect())
                },
                |()| Ok(document.normalized()),
            )
        })
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        self.validate()
            .map_err(|report| CodecDiagnostic::serialize(report.to_string()))
            .and_then(|()| match format {
                | MimeType::Json => {
                    serde_json::to_string_pretty(&self.clone().normalized()).map_err(|error| CodecDiagnostic::serialize(error.to_string()))
                }
                | MimeType::LdJson | MimeType::RdfXml | MimeType::Turtle => super::rdf::encode(self, format),
                | MimeType::Yaml => {
                    serde_norway::to_string(&self.clone().normalized()).map_err(|error| CodecDiagnostic::serialize(error.to_string()))
                }
                | _ => Err(CodecDiagnostic::unsupported(format, Self::NAME)),
            })
    }
}
impl DublinCoreDocument {
    /// Construct and normalize a Dublin Core default graph.
    pub fn new(descriptions: Vec<DublinCoreDescription>) -> Self {
        Self { descriptions }.normalized()
    }
    /// Return a canonical graph with merged subjects and sorted, deduplicated values.
    pub fn normalized(self) -> Self {
        let descriptions = self
            .descriptions
            .into_iter()
            .map(DublinCoreDescription::normalized)
            .fold(
                BTreeMap::<DublinCoreResource, DublinCoreDescription>::new(),
                |mut descriptions, description| {
                    descriptions
                        .entry(description.subject.clone())
                        .and_modify(|current| {
                            current.types.extend(description.types.clone());
                            description.properties.iter().for_each(|(predicate, values)| {
                                current.properties.entry(predicate.clone()).or_default().extend(values.clone());
                            });
                        })
                        .or_insert(description);
                    descriptions
                },
            )
            .into_values()
            .map(DublinCoreDescription::normalized)
            .collect();
        Self { descriptions }
    }
    /// Look up every value of one predicate on one subject.
    pub fn values(&self, subject: &DublinCoreResource, predicate: &str) -> Option<&[DublinCoreValue]> {
        self.descriptions
            .iter()
            .find(|description| &description.subject == subject)
            .and_then(|description| description.properties.get(predicate))
            .map(Vec::as_slice)
    }
    /// Project graph values into local metadata names for quality analysis and crosswalks.
    pub fn metadata_values(&self) -> BTreeMap<String, Vec<String>> {
        self.descriptions.iter().fold(BTreeMap::new(), |mut fields, description| {
            if let DublinCoreResource::Iri(subject) = &description.subject {
                fields.entry("identifier".to_string()).or_insert_with(Vec::new).push(subject.clone());
            }
            if !description.types.is_empty() {
                fields
                    .entry("type".to_string())
                    .or_insert_with(Vec::new)
                    .extend(description.types.clone());
            }
            description.properties.iter().for_each(|(predicate, values)| {
                let name = predicate
                    .strip_prefix(DCTERMS_NAMESPACE)
                    .or_else(|| predicate.strip_prefix(ELEMENTS_NAMESPACE))
                    .unwrap_or(predicate)
                    .to_string();
                let values = values.iter().map(|value| match value {
                    | DublinCoreValue::Literal(value) => value.value.clone(),
                    | DublinCoreValue::Resource(DublinCoreResource::BlankNode(value)) => format!("_:{value}"),
                    | DublinCoreValue::Resource(DublinCoreResource::Iri(value)) => value.clone(),
                });
                fields.entry(name).or_insert_with(Vec::new).extend(values);
            });
            fields
        })
    }
}
impl ToProse for DublinCoreDocument {
    fn to_prose(&self) -> String {
        const PROSE_PROPERTIES: [&str; 6] = ["abstract", "alternative", "description", "subject", "tableOfContents", "title"];
        self.descriptions
            .iter()
            .flat_map(|description| description.properties.iter())
            .filter(|(predicate, _)| {
                predicate
                    .strip_prefix(DCTERMS_NAMESPACE)
                    .or_else(|| predicate.strip_prefix(ELEMENTS_NAMESPACE))
                    .is_some_and(|name| PROSE_PROPERTIES.contains(&name))
            })
            .flat_map(|(_, values)| values)
            .filter_map(|value| match value {
                | DublinCoreValue::Literal(value) if !value.value.trim().is_empty() => Some(value.value.clone()),
                | _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
impl Validate for DublinCoreDocument {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = self.descriptions.iter().enumerate().fold(
            ValidationReport::empty("descriptions", self.descriptions.is_empty()),
            |mut report, (index, description)| {
                if let Err(nested) = description.validate() {
                    report.append_prefixed(&format!("descriptions[{index}]"), nested);
                }
                report
            },
        );
        self.descriptions.iter().enumerate().for_each(|(index, description)| {
            if self.descriptions.iter().take(index).any(|value| value.subject == description.subject) {
                report.add(
                    format!("descriptions[{index}].subject"),
                    ValidationError::new("unique").with_message("Describe each RDF subject at most once"),
                );
            }
        });
        let recognized = self.descriptions.iter().any(|description| {
            description.types.iter().any(|value| term(value).is_some()) || description.properties.keys().any(|value| term(value).is_some())
        });
        if !recognized && !self.descriptions.is_empty() {
            report.add(
                "descriptions",
                ValidationError::new("dcmi-term").with_message("Provide at least one registered DCMI property or class"),
            );
        }
        report.finish()
    }
}
impl Validate for DublinCoreLiteral {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if self.language.is_some() && self.datatype.is_some() {
            report.add(
                "language",
                ValidationError::new("literal-kind").with_message("A literal cannot have both a language and a datatype"),
            );
        }
        if let Some(datatype) = &self.datatype {
            if let Err(error) = rules::url(datatype) {
                report.add("datatype", error.with_message("Provide an absolute RDF datatype IRI"));
            } else if !valid_syntax_encoding(datatype, &self.value) {
                report.add(
                    "value",
                    ValidationError::new("syntax-encoding").with_message(format!("Value does not conform to the {datatype} syntax encoding scheme")),
                );
            }
        }
        if self.language.as_deref().is_some_and(|language| !valid_language_tag(language)) {
            report.add(
                "language",
                ValidationError::new("language-tag").with_message("Provide a well-formed BCP 47 language tag"),
            );
        }
        report.finish()
    }
}
impl DublinCoreNamespace {
    /// Canonical namespace IRI.
    pub const fn iri(self) -> &'static str {
        match self {
            | Self::AbstractModel => DCAM_NAMESPACE,
            | Self::MetadataElements => ELEMENTS_NAMESPACE,
            | Self::MetadataTerms => DCTERMS_NAMESPACE,
            | Self::TypeVocabulary => TYPE_NAMESPACE,
        }
    }
    /// Conventional RDF prefix.
    pub const fn prefix(self) -> &'static str {
        match self {
            | Self::AbstractModel => "dcam",
            | Self::MetadataElements => "dc",
            | Self::MetadataTerms => "dcterms",
            | Self::TypeVocabulary => "dcmitype",
        }
    }
    /// Publication date of the current pinned vocabulary snapshot.
    pub const fn modified(self) -> &'static str {
        "2012-06-14"
    }
}
impl DublinCoreResource {
    /// Construct a document-scoped blank node.
    pub fn blank_node(value: impl Into<String>) -> Self {
        Self::BlankNode(value.into())
    }
    /// Construct an absolute IRI resource.
    pub fn iri(value: impl Into<String>) -> Self {
        Self::Iri(value.into())
    }
    /// Return an IRI, excluding blank nodes.
    pub fn as_iri(&self) -> Option<&str> {
        match self {
            | Self::BlankNode(_) => None,
            | Self::Iri(value) => Some(value),
        }
    }
}
impl Validate for DublinCoreResource {
    fn validate(&self) -> Result<(), ValidationReport> {
        let result = match self {
            | Self::BlankNode(value) => valid_blank_node(value)
                .then_some(())
                .ok_or_else(|| ValidationError::new("blank-node").with_message("Provide a non-empty RDF blank-node identifier")),
            | Self::Iri(value) => rules::url(value).map_err(|error| error.with_message("Provide an absolute RDF resource IRI")),
        };
        result.map_or_else(|error| Err(ValidationReport::from_error("", error)), |()| Ok(()))
    }
}
impl DublinCoreTerm {
    /// Construct a term from an absolute IRI.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        rules::url(&value).map(|()| Self(value))
    }
    /// Borrow the absolute term IRI.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Return the registered definition, if the IRI is a current DCMI term.
    pub fn definition(&self) -> Option<&'static DublinCoreTermDefinition> {
        term(self.as_str())
    }
}
impl DublinCoreTermDefinition {
    /// Return the absolute IRI of this term.
    pub fn iri(self) -> String {
        format!("{}{}", self.namespace.iri(), self.local_name)
    }
}
impl DublinCoreValue {
    /// Construct a literal, optionally tagged with a language or a DCMI datatype name.
    pub fn literal(value: &str, language: Option<&str>, datatype: Option<&str>) -> Self {
        Self::Literal(DublinCoreLiteral {
            datatype: datatype.map(|datatype| format!("{DCTERMS_NAMESPACE}{datatype}")),
            language: language.map(ToString::to_string),
            value: value.to_string(),
        })
    }
}
impl Validate for DublinCoreValue {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Literal(value) => value.validate(),
            | Self::Resource(value) => value.validate(),
        }
    }
}
fn finite_number(value: &str) -> bool {
    value.parse::<f64>().is_ok_and(f64::is_finite)
}
fn parse_named_values<'a>(value: &'a str, names: &[&str]) -> Option<BTreeMap<&'a str, &'a str>> {
    value
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .try_fold((BTreeMap::new(), BTreeSet::new()), |(mut values, mut seen), part| {
            part.split_once('=').and_then(|(name, value)| {
                let name = name.trim();
                let value = value.trim();
                (names.contains(&name) && !value.is_empty() && seen.insert(name)).then(|| {
                    values.insert(name, value);
                    (values, seen)
                })
            })
        })
        .map(|(values, _)| values)
}
fn valid_blank_node(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with("_:")
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '.' | '_'))
}
fn valid_box(value: &str) -> bool {
    parse_named_values(value, &["eastlimit", "name", "northlimit", "projection", "southlimit", "westlimit"]).is_some_and(|values| {
        values.get("northlimit").is_some_and(|value| rules::latitude(value).is_ok())
            && values.get("southlimit").is_some_and(|value| rules::latitude(value).is_ok())
            && values.get("eastlimit").is_some_and(|value| rules::longitude(value).is_ok())
            && values.get("westlimit").is_some_and(|value| rules::longitude(value).is_ok())
    })
}
fn valid_language_tag(value: &str) -> bool {
    let mut parts = value.split('-');
    parts.next().is_some_and(|part| {
        (part.eq_ignore_ascii_case("i") || part.eq_ignore_ascii_case("x") || (2..=8).contains(&part.len()))
            && part.chars().all(|character| character.is_ascii_alphabetic())
    }) && value.len() <= 63
        && parts.all(|part| !part.is_empty() && part.len() <= 8 && part.chars().all(|character| character.is_ascii_alphanumeric()))
}
fn valid_period(value: &str) -> bool {
    parse_named_values(value, &["end", "name", "scheme", "start"]).is_some_and(|values| {
        (values.contains_key("start") || values.contains_key("end") || values.contains_key("name"))
            && values.get("start").is_none_or(|value| valid_w3cdtf(value))
            && values.get("end").is_none_or(|value| valid_w3cdtf(value))
    })
}
fn valid_point(value: &str) -> bool {
    parse_named_values(value, &["east", "elevation", "name", "north", "projection", "units"]).is_some_and(|values| {
        values.get("east").is_some_and(|value| rules::longitude(value).is_ok())
            && values.get("north").is_some_and(|value| rules::latitude(value).is_ok())
            && values.get("elevation").is_none_or(|value| finite_number(value))
    })
}
fn valid_syntax_encoding(datatype: &str, value: &str) -> bool {
    match datatype.strip_prefix(DCTERMS_NAMESPACE) {
        | Some("Box") => valid_box(value),
        | Some("ISO3166") => value.len() == 2 && value.chars().all(|character| character.is_ascii_alphabetic()),
        | Some("ISO639-2" | "ISO639-3") => value.len() == 3 && value.chars().all(|character| character.is_ascii_alphabetic()),
        | Some("Period") => valid_period(value),
        | Some("Point") => valid_point(value),
        | Some("RFC1766" | "RFC3066" | "RFC4646" | "RFC5646") => valid_language_tag(value),
        | Some("URI") => rules::url(value).is_ok(),
        | Some("W3CDTF") => valid_w3cdtf(value),
        | _ => true,
    }
}
fn valid_w3cdtf(value: &str) -> bool {
    match value.len() {
        | 4 => value.chars().all(|character| character.is_ascii_digit()),
        | 7 => format!("{value}-01").parse::<jiff::civil::Date>().is_ok(),
        | 10 => value.parse::<jiff::civil::Date>().is_ok(),
        | _ => value.parse::<jiff::Timestamp>().is_ok(),
    }
}
