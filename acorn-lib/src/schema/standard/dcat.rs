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
use super::datacite;
use crate::schema::standard::crosswalk::{SchemaBuilder, SchemaExtractor};
use crate::schema::validate::{is_url, is_urls};
use crate::schema::Date as PeriodOfTime;
use crate::util::Checksum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::Validate;

/// A collection of datasets published separately but sharing characteristics (`dcat:DatasetSeries`)
///
/// Added in DCAT 3. Sub-class of `dcat:Dataset`.
/// See <https://www.w3.org/TR/vocab-dcat-3/#Class:Dataset_Series>
///
/// Inherits all `Dataset` properties. Reuse `Dataset` with an appropriate
/// `type_` value, or use this type to make the series nature explicit in code.
pub type DatasetSeries = Dataset;
/// An agent (person or organization) as a `foaf:Agent`
///
/// Used for `dcterms:creator`, `dcterms:publisher`, etc.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Agent {
    /// Agent name (`foaf:name`)
    pub name: Option<String>,
    /// Agent homepage URI (`foaf:homepage`)
    #[validate(custom(function = "is_urls"))]
    pub homepage: Option<Vec<String>>,
    /// Agent email (`foaf:mbox`)
    pub email: Option<String>,
    /// Agent identifier(s) e.g. ORCID, ROR (`dcterms:identifier`)
    pub identifier: Option<Vec<String>>,
}
/// A curated collection of metadata about resources (`dcat:Catalog`)
///
/// Sub-class of `dcat:Dataset`. See <https://www.w3.org/TR/vocab-dcat-3/#Class:Catalog>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Catalog {
    /// Catalog title(s) (`dcterms:title`)
    pub title: Option<Vec<String>>,
    /// Description(s) (`dcterms:description`)
    pub description: Option<Vec<String>>,
    /// Unique identifier(s) (`dcterms:identifier`)
    pub identifier: Option<Vec<String>>,
    /// Publication date as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Last modification date as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// Language code(s) per ISO 639-1 (`dcterms:language`)
    pub language: Option<Vec<String>>,
    /// Publisher (`dcterms:publisher`)
    #[validate(nested)]
    pub publisher: Option<Agent>,
    /// Creator(s) (`dcterms:creator`, DCAT 2)
    #[validate(nested)]
    pub creator: Option<Vec<Agent>>,
    /// Contact point IRI(s) for vCard (`dcat:contactPoint`)
    #[serde(rename = "contactPoint")]
    pub contact_point: Option<Vec<String>>,
    /// Keywords (`dcat:keyword`)
    pub keywords: Option<Vec<String>>,
    /// Theme/category IRI(s) (`dcat:theme`)
    pub themes: Option<Vec<String>>,
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
    #[serde(rename = "conformsTo")]
    pub conforms_to: Option<Vec<String>>,
    /// Landing page URI(s) (`dcat:landingPage`)
    #[serde(rename = "landingPage")]
    #[validate(custom(function = "is_urls"))]
    pub landing_page: Option<Vec<String>>,
    /// Relations to other resources, as IRIs (`dcterms:relation`)
    pub relation: Option<Vec<String>>,
    /// Resource type IRI(s) (`dcterms:type`)
    #[serde(rename = "type")]
    pub type_: Option<Vec<String>>,
    /// Version indicator (`dcat:version`, DCAT 3)
    pub version: Option<String>,
    /// Version notes (`adms:versionNotes`, DCAT 3)
    #[serde(rename = "versionNotes")]
    pub version_notes: Option<String>,
    /// IRI of the previous version (`dcat:previousVersion`, DCAT 3)
    #[serde(rename = "previousVersion")]
    pub previous_version: Option<String>,
    /// IRI(s) of versioned snapshots (`dcat:hasVersion`, DCAT 3)
    #[serde(rename = "hasVersion")]
    pub has_version: Option<Vec<String>>,
    /// IRI of the current version (`dcat:hasCurrentVersion`, DCAT 3)
    #[serde(rename = "hasCurrentVersion")]
    pub has_current_version: Option<String>,
    /// IRI of the resource this one replaces (`dcterms:replaces`, DCAT 3)
    pub replaces: Option<String>,
    /// Life-cycle status IRI (`adms:status`, DCAT 3)
    pub status: Option<String>,
    /// Related resources that reference this catalog (`dcterms:isReferencedBy`, DCAT 2)
    #[serde(rename = "isReferencedBy")]
    pub is_referenced_by: Option<Vec<String>>,
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
    #[serde(rename = "wasGeneratedBy")]
    pub was_generated_by: Option<Vec<String>>,
    /// Catalog homepage URI (`foaf:homepage`)
    #[validate(custom(function = "is_url"))]
    pub homepage: Option<String>,
    /// Knowledge organization system IRI(s) for classifying resources (`dcat:themeTaxonomy`)
    #[serde(rename = "themeTaxonomy")]
    pub theme_taxonomy: Option<Vec<String>>,
    /// Dataset IRI(s) listed in this catalog (`dcat:dataset`)
    pub dataset: Option<Vec<String>>,
    /// Data service IRI(s) listed in this catalog (`dcat:service`)
    pub service: Option<Vec<String>>,
    /// Sub-catalog IRI(s) listed in this catalog (`dcat:catalog`)
    pub catalog: Option<Vec<String>>,
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
    #[serde(rename = "conformsTo")]
    pub conforms_to: Option<Vec<String>>,
}
/// A collection of operations providing access to data (`dcat:DataService`)
///
/// Added in DCAT 2. Sub-class of `dcat:Resource`.
/// See <https://www.w3.org/TR/vocab-dcat-3/#Class:Data_Service>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct DataService {
    /// Service title(s) (`dcterms:title`)
    pub title: Option<Vec<String>>,
    /// Description(s) (`dcterms:description`)
    pub description: Option<Vec<String>>,
    /// Unique identifier(s) (`dcterms:identifier`)
    pub identifier: Option<Vec<String>>,
    /// Publication date as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Last modification date as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// Language code(s) per ISO 639-1 (`dcterms:language`)
    pub language: Option<Vec<String>>,
    /// Publisher (`dcterms:publisher`)
    #[validate(nested)]
    pub publisher: Option<Agent>,
    /// Creator(s) (`dcterms:creator`, DCAT 2)
    #[validate(nested)]
    pub creator: Option<Vec<Agent>>,
    /// Contact point IRI(s) for vCard (`dcat:contactPoint`)
    #[serde(rename = "contactPoint")]
    pub contact_point: Option<Vec<String>>,
    /// Keywords (`dcat:keyword`)
    pub keywords: Option<Vec<String>>,
    /// Theme/category IRI(s) (`dcat:theme`)
    pub themes: Option<Vec<String>>,
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
    #[serde(rename = "conformsTo")]
    pub conforms_to: Option<Vec<String>>,
    /// Landing page URI(s) (`dcat:landingPage`)
    #[serde(rename = "landingPage")]
    #[validate(custom(function = "is_urls"))]
    pub landing_page: Option<Vec<String>>,
    /// Resource type IRI(s) (`dcterms:type`)
    #[serde(rename = "type")]
    pub type_: Option<Vec<String>>,
    /// Version indicator (`dcat:version`, DCAT 3)
    pub version: Option<String>,
    /// Version notes (`adms:versionNotes`, DCAT 3)
    #[serde(rename = "versionNotes")]
    pub version_notes: Option<String>,
    /// IRI of the previous version (`dcat:previousVersion`, DCAT 3)
    #[serde(rename = "previousVersion")]
    pub previous_version: Option<String>,
    /// IRI(s) of versioned snapshots (`dcat:hasVersion`, DCAT 3)
    #[serde(rename = "hasVersion")]
    pub has_version: Option<Vec<String>>,
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
    #[validate(custom(function = "is_urls"))]
    pub endpoint_url: Vec<String>,
    /// Endpoint description IRI(s) or documents (`dcat:endpointDescription`)
    #[serde(rename = "endpointDescription")]
    pub endpoint_description: Option<Vec<String>>,
    /// Dataset IRI(s) served by this service (`dcat:servesDataset`)
    #[serde(rename = "servesDataset")]
    pub serves_dataset: Option<Vec<String>>,
}
/// A collection of data published or curated by a single agent (`dcat:Dataset`)
///
/// Sub-class of `dcat:Resource`. The conceptual dataset, not any particular
/// serialization. See <https://www.w3.org/TR/vocab-dcat-3/#Class:Dataset>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Dataset {
    /// Dataset title(s) (`dcterms:title`)
    pub title: Option<Vec<String>>,
    /// Description(s) (`dcterms:description`)
    pub description: Option<Vec<String>>,
    /// Unique identifier(s) (`dcterms:identifier`)
    pub identifier: Option<Vec<String>>,
    /// Publication date as ISO 8601 string (`dcterms:issued`)
    pub issued: Option<String>,
    /// Last modification date as ISO 8601 string (`dcterms:modified`)
    pub modified: Option<String>,
    /// Language code(s) per ISO 639-1 (`dcterms:language`)
    pub language: Option<Vec<String>>,
    /// Publisher (`dcterms:publisher`)
    #[validate(nested)]
    pub publisher: Option<Agent>,
    /// Creator(s) (`dcterms:creator`, DCAT 2)
    #[validate(nested)]
    pub creator: Option<Vec<Agent>>,
    /// Contact point IRI(s) for vCard (`dcat:contactPoint`)
    #[serde(rename = "contactPoint")]
    pub contact_point: Option<Vec<String>>,
    /// Keywords (`dcat:keyword`)
    pub keywords: Option<Vec<String>>,
    /// Theme/category IRI(s) (`dcat:theme`)
    pub themes: Option<Vec<String>>,
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
    #[serde(rename = "conformsTo")]
    pub conforms_to: Option<Vec<String>>,
    /// Landing page URI(s) (`dcat:landingPage`)
    #[serde(rename = "landingPage")]
    #[validate(custom(function = "is_urls"))]
    pub landing_page: Option<Vec<String>>,
    /// Relations to other resources, as IRIs (`dcterms:relation`)
    pub relation: Option<Vec<String>>,
    /// Resource type IRI(s) (`dcterms:type`)
    #[serde(rename = "type")]
    pub type_: Option<Vec<String>>,
    /// Version indicator (`dcat:version`, DCAT 3)
    pub version: Option<String>,
    /// Version notes (`adms:versionNotes`, DCAT 3)
    #[serde(rename = "versionNotes")]
    pub version_notes: Option<String>,
    /// IRI of the previous version (`dcat:previousVersion`, DCAT 3)
    #[serde(rename = "previousVersion")]
    pub previous_version: Option<String>,
    /// IRI(s) of versioned snapshots (`dcat:hasVersion`, DCAT 3)
    #[serde(rename = "hasVersion")]
    pub has_version: Option<Vec<String>>,
    /// IRI of the current version (`dcat:hasCurrentVersion`, DCAT 3)
    #[serde(rename = "hasCurrentVersion")]
    pub has_current_version: Option<String>,
    /// IRI of the resource this one replaces (`dcterms:replaces`, DCAT 3)
    pub replaces: Option<String>,
    /// Life-cycle status IRI (`adms:status`, DCAT 3)
    pub status: Option<String>,
    /// Related resources that reference this dataset (`dcterms:isReferencedBy`, DCAT 2)
    #[serde(rename = "isReferencedBy")]
    pub is_referenced_by: Option<Vec<String>>,
    /// Parts of this resource, as IRIs (`dcterms:hasPart`, DCAT 3)
    #[serde(rename = "hasPart")]
    pub has_part: Option<Vec<String>>,
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
    #[serde(rename = "inSeries")]
    pub in_series: Option<Vec<String>>,
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
    #[serde(rename = "wasGeneratedBy")]
    pub was_generated_by: Option<Vec<String>>,
}
/// A specific representation of a dataset (`dcat:Distribution`)
///
/// See <https://www.w3.org/TR/vocab-dcat-3/#Class:Distribution>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Distribution {
    /// Distribution title (`dcterms:title`)
    pub title: Option<Vec<String>>,
    /// Free-text description (`dcterms:description`)
    pub description: Option<Vec<String>>,
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
    /// URL(s) providing access to this distribution (`dcat:accessURL`)
    #[serde(rename = "accessURL")]
    #[validate(custom(function = "is_urls"))]
    pub access_url: Vec<String>,
    /// Data service IRI(s) giving access to this distribution (`dcat:accessService`)
    #[serde(rename = "accessService")]
    pub access_service: Option<Vec<String>>,
    /// Direct download URL(s) (`dcat:downloadURL`)
    #[serde(rename = "downloadURL")]
    #[validate(custom(function = "is_urls"))]
    pub download_url: Option<Vec<String>>,
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
    #[serde(rename = "conformsTo")]
    pub conforms_to: Option<Vec<String>>,
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
    #[serde(rename = "hadRole")]
    pub had_role: String,
}
impl From<datacite::Record> for Dataset {
    fn from(record: datacite::Record) -> Self {
        let fields = record.extract_fields();
        Dataset::build_from_fields(&fields).unwrap_or_else(|err| panic!("failed to convert DataCite Record to DCAT Dataset: {err}"))
    }
}
impl From<&datacite::Record> for Dataset {
    fn from(record: &datacite::Record) -> Self {
        Dataset::from(record.clone())
    }
}
