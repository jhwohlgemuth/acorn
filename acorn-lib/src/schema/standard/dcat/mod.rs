//! DCAT (Data Catalog Vocabulary) v3 schema models
//!
//! These types model the DCAT v3 structure per the W3C Recommendation (22 August 2024).
//! Covers catalogs, datasets, dataset series, distributions, data services, and
//! supporting types for spatial/temporal coverage, checksums, and relationships.
//!
//! DCAT namespace: `http://www.w3.org/ns/dcat#`
//!
//! References:
//! - [DCAT v3](https://www.w3.org/TR/vocab-dcat-3/)
//! - [DCAT v2](https://www.w3.org/TR/vocab-dcat-2/)
#[cfg(feature = "std")]
use crate::io::{read_file, write_file, ApiResult, InputOutput};
use crate::prelude::*;
use crate::schema::standard::crosswalk::{self, mapping::datacite_to_dcat, CrosswalkError, FieldValue, Fields, SchemaBuilder, SchemaExtractor};
use crate::schema::standard::datacite::{self, RelationType};
use crate::schema::validate::is_url;
use crate::schema::{Date as PeriodOfTime, OneOrMany};
#[cfg(feature = "std")]
use crate::util::MimeType;
use crate::util::{Checksum, ToProse};
#[cfg(feature = "std")]
use crate::PathBuf;
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::Validate;

pub(crate) mod validate;
use validate::{is_document_refs_urls, is_one_or_many_urls};

/// A collection of datasets published separately but sharing characteristics (`dcat:DatasetSeries`)
///
/// Added in DCAT 3. Sub-class of `dcat:Dataset`.
/// See <https://www.w3.org/TR/vocab-dcat-3/#Class:Dataset_Series>
///
/// Inherits all `Dataset` properties. Reuse `Dataset` with an appropriate
/// `type_` value, or use this type to make the series nature explicit in code.
pub type DatasetSeries = Dataset;
/// A `conformsTo` value represented as either a URI or DCAT-US standard object.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ConformsTo {
    /// Standard URI.
    Uri(String),
    /// DCAT-US standard object.
    Standard(ConformsToStandard),
}
/// A DCAT contact point represented as either a vCard URI or DCAT-US `Kind`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ContactPoint {
    /// Contact point URI.
    Uri(String),
    /// DCAT-US vCard contact object.
    Kind(Kind),
}
/// A document reference represented as either a URI or DCAT-US document object.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DocumentRef {
    /// Document URI.
    Uri(String),
    /// DCAT-US document object.
    Document(Document),
}
/// A publisher represented as either a W3C DCAT agent or DCAT-US organization.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Publisher {
    /// DCAT-US organization.
    Organization(UsOrganization),
    /// W3C DCAT agent.
    Agent(Agent),
}
/// An agent (person or organization) as a `foaf:Agent`
///
/// Used for `dcterms:creator`, `dcterms:publisher`, etc.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Agent {
    /// Agent name (`foaf:name`)
    pub name: Option<String>,
    /// Agent homepage URI (`foaf:homepage`)
    #[serde(default)]
    #[validate(custom(function = "is_one_or_many_urls"))]
    pub homepage: Option<OneOrMany<String>>,
    /// Agent email (`foaf:mbox`)
    pub email: Option<String>,
    /// Agent identifier(s) e.g. ORCID, ROR (`dcterms:identifier`)
    #[serde(default)]
    pub identifier: Option<OneOrMany<String>>,
}
/// A curated collection of metadata about resources (`dcat:Catalog`)
///
/// Sub-class of `dcat:Dataset`. See <https://www.w3.org/TR/vocab-dcat-3/#Class:Catalog>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Catalog {
    /// JSON-LD node identifier.
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Catalog title(s) (`dcterms:title`)
    #[serde(default)]
    pub title: Option<OneOrMany<String>>,
    /// Description(s) (`dcterms:description`)
    #[serde(default)]
    pub description: Option<OneOrMany<String>>,
    /// Unique identifier(s) (`dcterms:identifier`)
    #[serde(default)]
    pub identifier: Option<OneOrMany<String>>,
    /// Publication date as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Last modification date as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// Language code(s) per ISO 639-1 (`dcterms:language`)
    #[serde(default)]
    pub language: Option<OneOrMany<String>>,
    /// Publisher (`dcterms:publisher`)
    #[validate(nested)]
    pub publisher: Option<Publisher>,
    /// Creator(s) (`dcterms:creator`, DCAT 2)
    #[validate(nested)]
    pub creator: Option<Vec<Agent>>,
    /// Contact point IRI(s) for vCard (`dcat:contactPoint`)
    #[serde(rename = "contactPoint", default)]
    #[validate(nested)]
    pub contact_point: Option<OneOrMany<ContactPoint>>,
    /// Keywords (`dcat:keyword`)
    #[serde(default)]
    pub keywords: Option<OneOrMany<String>>,
    /// Theme/category IRI(s) (`dcat:theme`)
    #[serde(default)]
    pub themes: Option<OneOrMany<String>>,
    /// License document URI (`dcterms:license`)
    pub license: Option<String>,
    /// Rights statement URI (`dcterms:rights`)
    pub rights: Option<String>,
    /// Access rights statement URI (`dcterms:accessRights`)
    #[serde(rename = "accessRights")]
    pub access_rights: Option<String>,
    /// ODRL policy IRI (`odrl:hasPolicy`)
    #[serde(rename = "hasPolicy")]
    pub has_policy: Option<String>,
    /// Standards conformed to, as IRIs (`dcterms:conformsTo`)
    #[serde(rename = "conformsTo", default)]
    #[validate(nested)]
    pub conforms_to: Option<OneOrMany<ConformsTo>>,
    /// Landing page URI(s) (`dcat:landingPage`)
    #[serde(rename = "landingPage", default)]
    #[validate(custom(function = "is_document_refs_urls"), nested)]
    pub landing_page: Option<OneOrMany<DocumentRef>>,
    /// Relations to other resources, as IRIs (`dcterms:relation`)
    #[serde(default)]
    pub relation: Option<OneOrMany<String>>,
    /// Resource type IRI(s) (`dcterms:type`)
    #[serde(rename = "type", default)]
    pub type_: Option<OneOrMany<String>>,
    /// Version indicator (`dcat:version`, DCAT 3)
    pub version: Option<String>,
    /// Version notes (`adms:versionNotes`, DCAT 3)
    #[serde(rename = "versionNotes")]
    pub version_notes: Option<String>,
    /// IRI of the previous version (`dcat:previousVersion`, DCAT 3)
    #[serde(rename = "previousVersion")]
    pub previous_version: Option<String>,
    /// IRI(s) of versioned snapshots (`dcat:hasVersion`, DCAT 3)
    #[serde(rename = "hasVersion", default)]
    pub has_version: Option<OneOrMany<String>>,
    /// IRI of the current version (`dcat:hasCurrentVersion`, DCAT 3)
    #[serde(rename = "hasCurrentVersion")]
    pub has_current_version: Option<String>,
    /// IRI of the resource this one replaces (`dcterms:replaces`, DCAT 3)
    pub replaces: Option<String>,
    /// Life-cycle status IRI (`adms:status`, DCAT 3)
    pub status: Option<String>,
    /// Related resources that reference this catalog (`dcterms:isReferencedBy`, DCAT 2)
    #[serde(rename = "isReferencedBy", default)]
    pub is_referenced_by: Option<OneOrMany<String>>,
    /// Qualified relationships to other resources (`dcat:qualifiedRelation`, DCAT 2)
    #[validate(nested)]
    #[serde(rename = "qualifiedRelation")]
    pub qualified_relation: Option<Vec<Relationship>>,
    /// Inherited dataset distributions (`dcat:distribution`)
    #[validate(nested)]
    pub distribution: Option<Vec<Distribution>>,
    /// Update frequency IRI (`dcterms:accrualPeriodicity`)
    pub frequency: Option<String>,
    /// Spatial coverage (`dcterms:spatial`)
    #[validate(nested)]
    pub spatial: Option<Vec<Location>>,
    /// Minimum spatial separation in meters (`dcat:spatialResolutionInMeters`, DCAT 2)
    #[serde(rename = "spatialResolutionInMeters")]
    pub spatial_resolution_in_meters: Option<f64>,
    /// Temporal coverage (`dcterms:temporal`)
    #[validate(nested)]
    pub temporal: Option<Vec<PeriodOfTime>>,
    /// Minimum time period resolvable as ISO 8601 duration (`dcat:temporalResolution`, DCAT 2)
    #[serde(rename = "temporalResolution")]
    pub temporal_resolution: Option<String>,
    /// Activity IRI(s) that generated this catalog (`prov:wasGeneratedBy`, DCAT 2)
    #[serde(rename = "wasGeneratedBy", default)]
    pub was_generated_by: Option<OneOrMany<String>>,
    /// Catalog homepage URI (`foaf:homepage`)
    #[serde(default)]
    #[validate(custom(function = "is_document_refs_urls"), nested)]
    pub homepage: Option<OneOrMany<DocumentRef>>,
    /// Knowledge organization system IRI(s) for classifying resources (`dcat:themeTaxonomy`)
    #[serde(rename = "themeTaxonomy", default)]
    pub theme_taxonomy: Option<OneOrMany<String>>,
    /// Dataset IRI(s) listed in this catalog (`dcat:dataset`)
    #[serde(default)]
    pub dataset: Option<OneOrMany<String>>,
    /// Data service IRI(s) listed in this catalog (`dcat:service`)
    #[validate(nested)]
    pub service: Option<Vec<DataService>>,
    /// Sub-catalog IRI(s) listed in this catalog (`dcat:catalog`)
    #[serde(default)]
    pub catalog: Option<OneOrMany<String>>,
    /// Catalog records for resources in this catalog (`dcat:record`)
    #[validate(nested)]
    pub record: Option<Vec<CatalogRecord>>,
}
/// Metadata record for a cataloged resource (`dcat:CatalogRecord`)
///
/// Optional. Used when catalog-entry provenance (e.g., listing date) differs
/// from resource provenance. See <https://www.w3.org/TR/vocab-dcat-3/#Class:Catalog_Record>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct CatalogRecord {
    /// Record title (`dcterms:title`)
    pub title: Option<String>,
    /// Record description (`dcterms:description`)
    pub description: Option<String>,
    /// Date the resource was listed in the catalog as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Date of the most recent change to the catalog entry as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// IRI of the cataloged resource this record describes (`foaf:primaryTopic`)
    #[serde(rename = "primaryTopic")]
    pub primary_topic: String,
    /// Standards the record conforms to, as IRIs (`dcterms:conformsTo`)
    #[serde(rename = "conformsTo", default)]
    pub conforms_to: Option<OneOrMany<String>>,
}
/// A standard or profile referenced by `dcterms:conformsTo`.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ConformsToStandard {
    /// JSON-LD node identifier.
    #[serde(rename = "@id")]
    #[validate(custom(function = "is_url"))]
    pub id: Option<String>,
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Standard title.
    pub title: Option<String>,
    /// Standard identifier.
    pub identifier: Option<String>,
}
/// A collection of operations providing access to data (`dcat:DataService`)
///
/// Added in DCAT 2. Sub-class of `dcat:Resource`.
/// See <https://www.w3.org/TR/vocab-dcat-3/#Class:Data_Service>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct DataService {
    /// JSON-LD node identifier.
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Service title(s) (`dcterms:title`)
    #[serde(default)]
    pub title: Option<OneOrMany<String>>,
    /// Description(s) (`dcterms:description`)
    #[serde(default)]
    pub description: Option<OneOrMany<String>>,
    /// Unique identifier(s) (`dcterms:identifier`)
    #[serde(default)]
    pub identifier: Option<OneOrMany<String>>,
    /// Publication date as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Last modification date as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// Language code(s) per ISO 639-1 (`dcterms:language`)
    #[serde(default)]
    pub language: Option<OneOrMany<String>>,
    /// Publisher (`dcterms:publisher`)
    #[validate(nested)]
    pub publisher: Option<Publisher>,
    /// Creator(s) (`dcterms:creator`, DCAT 2)
    #[validate(nested)]
    pub creator: Option<Vec<Agent>>,
    /// Contact point IRI(s) for vCard (`dcat:contactPoint`)
    #[serde(rename = "contactPoint", default)]
    #[validate(nested)]
    pub contact_point: Option<OneOrMany<ContactPoint>>,
    /// Keywords (`dcat:keyword`)
    #[serde(default)]
    pub keywords: Option<OneOrMany<String>>,
    /// Theme/category IRI(s) (`dcat:theme`)
    #[serde(default)]
    pub themes: Option<OneOrMany<String>>,
    /// License document URI (`dcterms:license`)
    pub license: Option<String>,
    /// Rights statement URI (`dcterms:rights`)
    pub rights: Option<String>,
    /// Access rights statement URI (`dcterms:accessRights`)
    #[serde(rename = "accessRights")]
    pub access_rights: Option<String>,
    /// ODRL policy IRI (`odrl:hasPolicy`)
    #[serde(rename = "hasPolicy")]
    pub has_policy: Option<String>,
    /// Standards conformed to, as IRIs (`dcterms:conformsTo`)
    #[serde(rename = "conformsTo", default)]
    #[validate(nested)]
    pub conforms_to: Option<OneOrMany<ConformsTo>>,
    /// Landing page URI(s) (`dcat:landingPage`)
    #[serde(rename = "landingPage", default)]
    #[validate(custom(function = "is_document_refs_urls"), nested)]
    pub landing_page: Option<OneOrMany<DocumentRef>>,
    /// Resource type IRI(s) (`dcterms:type`)
    #[serde(rename = "type", default)]
    pub type_: Option<OneOrMany<String>>,
    /// Version indicator (`dcat:version`, DCAT 3)
    pub version: Option<String>,
    /// Version notes (`adms:versionNotes`, DCAT 3)
    #[serde(rename = "versionNotes")]
    pub version_notes: Option<String>,
    /// IRI of the previous version (`dcat:previousVersion`, DCAT 3)
    #[serde(rename = "previousVersion")]
    pub previous_version: Option<String>,
    /// IRI(s) of versioned snapshots (`dcat:hasVersion`, DCAT 3)
    #[serde(rename = "hasVersion", default)]
    pub has_version: Option<OneOrMany<String>>,
    /// IRI of the current version (`dcat:hasCurrentVersion`, DCAT 3)
    #[serde(rename = "hasCurrentVersion")]
    pub has_current_version: Option<String>,
    /// IRI of the resource this one replaces (`dcterms:replaces`, DCAT 3)
    pub replaces: Option<String>,
    /// Life-cycle status IRI (`adms:status`, DCAT 3)
    pub status: Option<String>,
    /// Qualified relationships to other resources (`dcat:qualifiedRelation`, DCAT 2)
    #[validate(nested)]
    #[serde(rename = "qualifiedRelation")]
    pub qualified_relation: Option<Vec<Relationship>>,
    /// Root location or primary endpoint IRI(s) (`dcat:endpointURL`)
    #[serde(rename = "endpointURL")]
    #[validate(custom(function = "is_one_or_many_urls"))]
    pub endpoint_url: OneOrMany<String>,
    /// Endpoint description IRI(s) or documents (`dcat:endpointDescription`)
    #[serde(rename = "endpointDescription", default)]
    pub endpoint_description: Option<OneOrMany<String>>,
    /// Dataset IRI(s) served by this service (`dcat:servesDataset`)
    #[serde(rename = "servesDataset", default)]
    pub serves_dataset: Option<OneOrMany<String>>,
}
/// A collection of data published or curated by a single agent (`dcat:Dataset`)
///
/// Sub-class of `dcat:Resource`. The conceptual dataset, not any particular
/// serialization. See <https://www.w3.org/TR/vocab-dcat-3/#Class:Dataset>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Dataset {
    /// JSON-LD node identifier.
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Dataset title(s) (`dcterms:title`)
    #[serde(default)]
    pub title: Option<OneOrMany<String>>,
    /// Description(s) (`dcterms:description`)
    #[serde(default)]
    pub description: Option<OneOrMany<String>>,
    /// Unique identifier(s) (`dcterms:identifier`)
    #[serde(default)]
    pub identifier: Option<OneOrMany<String>>,
    /// Publication date as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Last modification date as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// Language code(s) per ISO 639-1 (`dcterms:language`)
    #[serde(default)]
    pub language: Option<OneOrMany<String>>,
    /// Publisher (`dcterms:publisher`)
    #[validate(nested)]
    pub publisher: Option<Publisher>,
    /// Creator(s) (`dcterms:creator`, DCAT 2)
    #[validate(nested)]
    pub creator: Option<Vec<Agent>>,
    /// Contact point IRI(s) for vCard (`dcat:contactPoint`)
    #[serde(rename = "contactPoint", default)]
    #[validate(nested)]
    pub contact_point: Option<OneOrMany<ContactPoint>>,
    /// Keywords (`dcat:keyword`)
    #[serde(default)]
    pub keywords: Option<OneOrMany<String>>,
    /// Theme/category IRI(s) (`dcat:theme`)
    #[serde(default)]
    pub themes: Option<OneOrMany<String>>,
    /// License document URI (`dcterms:license`)
    pub license: Option<String>,
    /// Rights statement URI (`dcterms:rights`)
    pub rights: Option<String>,
    /// Access rights statement URI (`dcterms:accessRights`)
    #[serde(rename = "accessRights")]
    pub access_rights: Option<String>,
    /// ODRL policy IRI (`odrl:hasPolicy`)
    #[serde(rename = "hasPolicy")]
    pub has_policy: Option<String>,
    /// Standards conformed to, as IRIs (`dcterms:conformsTo`)
    #[serde(rename = "conformsTo", default)]
    #[validate(nested)]
    pub conforms_to: Option<OneOrMany<ConformsTo>>,
    /// Landing page URI(s) (`dcat:landingPage`)
    #[serde(rename = "landingPage", default)]
    #[validate(custom(function = "is_document_refs_urls"), nested)]
    pub landing_page: Option<OneOrMany<DocumentRef>>,
    /// Relations to other resources, as IRIs (`dcterms:relation`)
    #[serde(default)]
    pub relation: Option<OneOrMany<String>>,
    /// Resource type IRI(s) (`dcterms:type`)
    #[serde(rename = "type", default)]
    pub type_: Option<OneOrMany<String>>,
    /// Version indicator (`dcat:version`, DCAT 3)
    pub version: Option<String>,
    /// Version notes (`adms:versionNotes`, DCAT 3)
    #[serde(rename = "versionNotes")]
    pub version_notes: Option<String>,
    /// IRI of the previous version (`dcat:previousVersion`, DCAT 3)
    #[serde(rename = "previousVersion")]
    pub previous_version: Option<String>,
    /// IRI(s) of versioned snapshots (`dcat:hasVersion`, DCAT 3)
    #[serde(rename = "hasVersion", default)]
    pub has_version: Option<OneOrMany<String>>,
    /// IRI of the current version (`dcat:hasCurrentVersion`, DCAT 3)
    #[serde(rename = "hasCurrentVersion")]
    pub has_current_version: Option<String>,
    /// IRI of the resource this one replaces (`dcterms:replaces`, DCAT 3)
    pub replaces: Option<String>,
    /// Life-cycle status IRI (`adms:status`, DCAT 3)
    pub status: Option<String>,
    /// Related resources that reference this dataset (`dcterms:isReferencedBy`, DCAT 2)
    #[serde(rename = "isReferencedBy", default)]
    pub is_referenced_by: Option<OneOrMany<String>>,
    /// Parts of this resource, as IRIs (`dcterms:hasPart`, DCAT 3)
    #[serde(rename = "hasPart", default)]
    pub has_part: Option<OneOrMany<String>>,
    /// Qualified relationships to other resources (`dcat:qualifiedRelation`, DCAT 2)
    #[validate(nested)]
    #[serde(rename = "qualifiedRelation")]
    pub qualified_relation: Option<Vec<Relationship>>,
    /// IRI of the first resource in a series (`dcat:first`, DCAT 3)
    pub first: Option<String>,
    /// IRI of the last resource in a series (`dcat:last`, DCAT 3)
    pub last: Option<String>,
    /// IRI of the previous resource in a series (`dcat:prev`, DCAT 3)
    pub previous: Option<String>,
    /// Available distributions (`dcat:distribution`)
    #[validate(nested)]
    pub distribution: Option<Vec<Distribution>>,
    /// Update frequency IRI (`dcterms:accrualPeriodicity`)
    pub frequency: Option<String>,
    /// Dataset series IRI(s) this dataset belongs to (`dcat:inSeries`, DCAT 3)
    #[serde(rename = "inSeries", default)]
    pub in_series: Option<OneOrMany<String>>,
    /// Spatial coverage (`dcterms:spatial`)
    #[validate(nested)]
    pub spatial: Option<Vec<Location>>,
    /// Minimum spatial separation in meters (`dcat:spatialResolutionInMeters`, DCAT 2)
    #[serde(rename = "spatialResolutionInMeters")]
    pub spatial_resolution_in_meters: Option<f64>,
    /// Temporal coverage (`dcterms:temporal`)
    #[validate(nested)]
    pub temporal: Option<Vec<PeriodOfTime>>,
    /// Minimum time period resolvable as ISO 8601 duration (`dcat:temporalResolution`, DCAT 2)
    #[serde(rename = "temporalResolution")]
    pub temporal_resolution: Option<String>,
    /// Activity IRI(s) that generated this dataset (`prov:wasGeneratedBy`, DCAT 2)
    #[serde(rename = "wasGeneratedBy", default)]
    pub was_generated_by: Option<OneOrMany<String>>,
}
/// A specific representation of a dataset (`dcat:Distribution`)
///
/// See <https://www.w3.org/TR/vocab-dcat-3/#Class:Distribution>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Distribution {
    /// JSON-LD node identifier.
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Distribution title (`dcterms:title`)
    #[serde(default)]
    pub title: Option<OneOrMany<String>>,
    /// Free-text description (`dcterms:description`)
    #[serde(default)]
    pub description: Option<OneOrMany<String>>,
    /// Publication date as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Last modification date as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// License document URI (`dcterms:license`)
    pub license: Option<String>,
    /// Access rights statement URI (`dcterms:accessRights`)
    #[serde(rename = "accessRights")]
    pub access_rights: Option<String>,
    /// Rights statement URI (`dcterms:rights`)
    pub rights: Option<String>,
    /// ODRL policy IRI (`odrl:hasPolicy`)
    #[serde(rename = "hasPolicy")]
    pub has_policy: Option<String>,
    /// DCAT-US access restriction statement.
    #[serde(rename = "accessRestriction")]
    pub access_restriction: Option<String>,
    /// DCAT-US use restriction statement.
    #[serde(rename = "useRestriction")]
    pub use_restriction: Option<String>,
    /// URL(s) providing access to this distribution (`dcat:accessURL`)
    #[serde(rename = "accessURL")]
    #[validate(custom(function = "is_one_or_many_urls"))]
    pub access_url: OneOrMany<String>,
    /// Data service IRI(s) giving access to this distribution (`dcat:accessService`)
    #[serde(rename = "accessService", default)]
    pub access_service: Option<OneOrMany<String>>,
    /// Direct download URL(s) (`dcat:downloadURL`)
    #[serde(rename = "downloadURL", default)]
    #[validate(custom(function = "is_one_or_many_urls"))]
    pub download_url: Option<OneOrMany<String>>,
    /// Size in bytes (`dcat:byteSize`)
    #[serde(rename = "byteSize")]
    pub byte_size: Option<u64>,
    /// Minimum spatial separation in meters (`dcat:spatialResolutionInMeters`)
    #[serde(rename = "spatialResolutionInMeters")]
    pub spatial_resolution_in_meters: Option<f64>,
    /// Minimum time period resolvable as ISO 8601 duration (`dcat:temporalResolution`)
    #[serde(rename = "temporalResolution")]
    pub temporal_resolution: Option<String>,
    /// Standards the distribution conforms to, as IRIs (`dcterms:conformsTo`)
    #[serde(rename = "conformsTo", default)]
    #[validate(nested)]
    pub conforms_to: Option<OneOrMany<ConformsTo>>,
    /// IANA media type IRI (`dcat:mediaType`)
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    /// File format IRI or string (`dcterms:format`)
    pub format: Option<String>,
    /// Compression format IANA media type IRI (`dcat:compressFormat`, DCAT 2)
    #[serde(rename = "compressFormat")]
    pub compress_format: Option<String>,
    /// Packaging format IANA media type IRI (`dcat:packageFormat`, DCAT 2)
    #[serde(rename = "packageFormat")]
    pub package_format: Option<String>,
    /// Checksum for integrity verification (`spdx:checksum`, DCAT 3)
    #[validate(nested)]
    pub checksum: Option<Checksum>,
}
/// A document resource used for landing pages and homepages.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Document {
    /// JSON-LD node identifier.
    #[serde(rename = "@id")]
    #[validate(custom(function = "is_url"))]
    pub id: Option<String>,
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Document title.
    pub title: Option<String>,
    /// URL used to access the document.
    #[serde(rename = "accessURL")]
    #[validate(custom(function = "is_url"))]
    pub access_url: Option<String>,
}
/// A vCard contact point (`vcard:Kind`) used by DCAT-US.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Kind {
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Formatted contact name (`vcard:fn`).
    #[serde(rename = "fn")]
    pub fn_: String,
    /// Contact email as a `mailto:` URI (`vcard:hasEmail`).
    #[serde(rename = "hasEmail")]
    pub has_email: String,
    /// Contact telephone URI or string (`vcard:tel`).
    pub tel: Option<String>,
    /// Contact organization name (`vcard:organization-name`).
    #[serde(rename = "organization-name")]
    pub organization_name: Option<String>,
}
/// A spatial region or named place (`dcterms:Location`)
///
/// See <https://www.w3.org/TR/vocab-dcat-3/#Class:Location>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Location {
    /// Full geometry as WKT or other literal encoding (`locn:geometry`)
    pub geometry: Option<String>,
    /// Geographic bounding box as WKT or other literal (`dcat:bbox`)
    pub bbox: Option<String>,
    /// Geographic centroid as WKT or other literal (`dcat:centroid`)
    pub centroid: Option<String>,
}
/// Qualified relationship between resources (`dcat:Relationship`)
///
/// Added in DCAT 2. See <https://www.w3.org/TR/vocab-dcat-3/#Class:Relationship>
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Relationship {
    /// IRI of the related resource (`dcterms:relation`)
    pub relation: String,
    /// IRI of the role the related resource plays (`dcat:hadRole`)
    #[serde(rename = "hadRole", alias = "role")]
    pub had_role: RelationType,
}
/// A DCAT-US publisher organization.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct UsOrganization {
    /// JSON-LD node identifier.
    #[serde(rename = "@id")]
    #[validate(custom(function = "is_url"))]
    pub id: Option<String>,
    /// JSON-LD node type.
    #[serde(rename = "@type")]
    pub jsonld_type: Option<String>,
    /// Organization name.
    pub name: String,
    /// Preferred label.
    #[serde(rename = "prefLabel")]
    pub pref_label: Option<String>,
    /// Alternative label.
    #[serde(rename = "altLabel")]
    pub alt_label: Option<String>,
    /// Parent organization.
    #[serde(rename = "subOrganizationOf")]
    #[validate(nested)]
    pub sub_organization_of: Option<Box<UsOrganization>>,
}
impl DocumentRef {
    /// Returns the URI represented by this document reference, when present.
    pub fn url(&self) -> Option<&str> {
        match self {
            | Self::Uri(value) => Some(value),
            | Self::Document(document) => document.access_url.as_deref().or(document.id.as_deref()),
        }
    }
}
#[cfg(feature = "std")]
impl InputOutput for Dataset {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Dataset> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Json => Dataset::read_json(source),
            | MimeType::Yaml => Dataset::read_yaml(source),
            | _ => Err(eyre!("Unsupported DCAT data file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Dataset> {
        read_file(path).and_then(|content| {
            serde_json::from_str::<OneOrMany<Dataset>>(&content)
                .map_err(|why| eyre!("Failed to parse JSON DCAT dataset — {why}"))
                .and_then(|value| match value {
                    | OneOrMany::One(dataset) => Ok(dataset),
                    | OneOrMany::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one DCAT dataset but found none")),
                        | len => Err(eyre!("Expected one DCAT dataset but found {len}")),
                    },
                })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Dataset> {
        read_file(path).and_then(|content| {
            serde_norway::from_str::<OneOrMany<Dataset>>(&content)
                .map_err(|why| eyre!("Failed to parse YAML DCAT dataset — {why}"))
                .and_then(|value| match value {
                    | OneOrMany::One(dataset) => Ok(dataset),
                    | OneOrMany::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one DCAT dataset but found none")),
                        | len => Err(eyre!("Expected one DCAT dataset but found {len}")),
                    },
                })
        })
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported DCAT data file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON DCAT dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML DCAT dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl Publisher {
    /// Returns the publisher name.
    pub fn name(&self) -> Option<&str> {
        match self {
            | Self::Organization(organization) => Some(organization.name.as_str()),
            | Self::Agent(agent) => agent.name.as_deref(),
        }
    }
}
impl SchemaBuilder for Dataset {
    fn build_from_fields(fields: &Fields) -> Result<Self, CrosswalkError> {
        let identifier = fields.get_string_vec_opt("identifier").map(OneOrMany::Many);
        let mut title = None;
        if let Some(title_str) = fields.get_string_opt("title") {
            title = Some(OneOrMany::Many(vec![title_str]));
        }
        let mut description = None;
        if let Some(desc_str) = fields.get_string_opt("description") {
            description = Some(OneOrMany::Many(vec![desc_str]));
        }
        let issued = fields.get_date_opt("issued");
        let language = fields.get_string_vec_opt("language").map(OneOrMany::Many);
        let keywords = fields.get_string_vec_opt("keywords").map(OneOrMany::Many);
        let themes = fields.get_string_vec_opt("themes").map(OneOrMany::Many);
        let version = fields.get_string_opt("version");
        let mut publisher = None;
        if let Some(pub_name) = fields.get_string_opt("publisher") {
            publisher = Some(Publisher::Agent(Agent {
                name: Some(pub_name),
                homepage: None,
                email: None,
                identifier: None,
            }));
        }
        let mut creator = None;
        if let Some(creator_names) = fields.get_string_vec_opt("creators") {
            creator = Some(
                creator_names
                    .into_iter()
                    .map(|name| Agent {
                        name: Some(name),
                        homepage: None,
                        email: None,
                        identifier: None,
                    })
                    .collect(),
            );
        }
        let license = fields.get_iri_opt("license");
        let landing_page = fields
            .get_string_opt("landing_page")
            .or_else(|| fields.get_iri_opt("landing_page"))
            .map(|value| OneOrMany::Many(vec![DocumentRef::Uri(value)]));
        let mut spatial = None;
        if let Some(spatial_str) = fields.get_string_opt("spatial") {
            spatial = Some(vec![Location {
                geometry: Some(spatial_str),
                bbox: None,
                centroid: None,
            }]);
        }
        Ok(Dataset {
            id: None,
            jsonld_type: Some("dcat:Dataset".to_string()),
            title,
            description,
            identifier,
            issued,
            modified: None,
            language,
            publisher,
            creator,
            contact_point: None,
            keywords,
            themes,
            license,
            rights: None,
            access_rights: None,
            has_policy: None,
            conforms_to: None,
            landing_page,
            relation: None,
            type_: None,
            version,
            version_notes: None,
            previous_version: None,
            has_version: None,
            has_current_version: None,
            replaces: None,
            status: None,
            is_referenced_by: None,
            has_part: None,
            qualified_relation: None,
            first: None,
            last: None,
            previous: None,
            distribution: None,
            frequency: None,
            in_series: None,
            spatial,
            spatial_resolution_in_meters: None,
            temporal: None,
            temporal_resolution: None,
            was_generated_by: None,
        })
    }
}
impl SchemaExtractor for Dataset {
    fn extract_fields(&self) -> Fields {
        let mut fields = Fields::new();
        if let Some(identifiers) = &self.identifier {
            if !identifiers.is_empty() {
                fields.insert("identifier", FieldValue::StringVec(identifiers.as_slice().to_vec()));
            }
        }
        if let Some(titles) = &self.title {
            if let Some(first) = titles.first() {
                fields.insert("title", FieldValue::String(first.clone()));
                if titles.len() > 1 {
                    let alt_titles: Vec<String> = titles.iter().skip(1).cloned().collect();
                    fields.insert("alternative-titles", FieldValue::StringVec(alt_titles));
                }
            }
        }
        if let Some(descriptions) = &self.description {
            if let Some(first) = descriptions.first() {
                fields.insert("description", FieldValue::String(first.clone()));
            }
        }
        if let Some(issued) = &self.issued {
            fields.insert("issued", FieldValue::Date(issued.clone()));
        }
        if let Some(keywords) = &self.keywords {
            fields.insert("keywords", FieldValue::StringVec(keywords.as_slice().to_vec()));
        }
        if let Some(themes) = &self.themes {
            fields.insert("themes", FieldValue::StringVec(themes.as_slice().to_vec()));
        }
        if let Some(language) = &self.language {
            fields.insert("language", FieldValue::StringVec(language.as_slice().to_vec()));
        }
        if let Some(version) = &self.version {
            fields.insert("version", FieldValue::String(version.clone()));
        }
        if let Some(publisher) = &self.publisher {
            if let Some(name) = publisher.name() {
                fields.insert("publisher", FieldValue::String(name.to_string()));
            }
        }
        if let Some(creators) = &self.creator {
            let creator_names: Vec<String> = creators.iter().filter_map(|c| c.name.clone()).collect();
            if !creator_names.is_empty() {
                fields.insert("creators", FieldValue::StringVec(creator_names));
            }
        }
        if let Some(license) = &self.license {
            fields.insert("license", FieldValue::IRI(license.clone()));
        }
        if let Some(spatial) = &self.spatial {
            if let Some(first) = spatial.first() {
                if let Some(geometry) = &first.geometry {
                    fields.insert("spatial", FieldValue::String(geometry.clone()));
                }
            }
        }
        fields
    }
}
impl ToProse for Dataset {
    fn to_prose(&self) -> String {
        self.title
            .iter()
            .flatten()
            .cloned()
            .chain(self.description.iter().flatten().cloned())
            .chain(self.keywords.iter().flatten().cloned())
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}
impl TryFrom<&datacite::Record> for Dataset {
    type Error = CrosswalkError;

    fn try_from(record: &datacite::Record) -> Result<Self, Self::Error> {
        Dataset::try_from(record.clone())
    }
}
impl TryFrom<datacite::Record> for Dataset {
    type Error = CrosswalkError;

    fn try_from(record: datacite::Record) -> Result<Self, Self::Error> {
        let mapping = datacite_to_dcat();
        crosswalk::convert(&record, &mapping).map(|(dataset, _)| dataset)
    }
}
