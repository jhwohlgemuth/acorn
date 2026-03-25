//! HuWise dataset schema models
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Collection of HuWise datasets
pub type Catalog = Vec<Dataset>;
/// Dublin Core type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum DublinCoreType {
    /// Dataset entry
    #[serde(rename = "Dataset")]
    Dataset,
}
/// Custom template metadata block
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CustomTemplate {
    /// Projects field (custom template)
    pub projects: Option<String>,
    /// Source-of-data field
    #[serde(rename = "source-of-data")]
    pub source_of_data: Option<String>,
}
/// Top-level dataset container from HuWise exports
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Dataset {
    /// Unique dataset identifier
    pub dataset_id: String,
    /// Whether the dataset has attachments
    pub has_attachments: bool,
    /// Number of attachments
    pub attachments_count: u64,
    /// Whether the dataset has records
    pub has_records: bool,
    /// Field definitions or schema details
    pub fields: Value,
    /// Structured metadata blocks
    pub metas: Meta,
    /// Feature records associated with the dataset
    pub features: Vec<Value>,
}
/// Metadata sections attached to a dataset
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Meta {
    /// DCAT metadata
    pub dcat: Option<Dcat>,
    /// Default metadata
    #[serde(rename = "default")]
    pub r#default: Option<DefaultMeta>,
    /// Dublin Core metadata
    #[serde(rename = "dublin-core")]
    pub dublin_core: Option<DublinCore>,
    /// DCAT-AP metadata
    pub dcat_ap: Option<DcatAp>,
    /// Custom template metadata
    #[serde(rename = "custom-template")]
    pub custom_template: Option<CustomTemplate>,
    /// DataCite metadata
    pub datacite: Option<Datacite>,
}
/// DCAT metadata block
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Dcat {
    /// Creation timestamp
    pub created: Option<String>,
    /// Issued timestamp
    pub issued: Option<String>,
    /// Creator name
    pub creator: Option<String>,
    /// Contributor name
    pub contributor: Option<String>,
    /// Contact name
    pub contact_name: Option<String>,
    /// Contact email
    pub contact_email: Option<String>,
    /// Accrual periodicity
    #[serde(rename = "accrualperiodicity")]
    pub accrual_periodicity: Option<String>,
    /// Spatial coverage
    pub spatial: Option<String>,
    /// Temporal coverage
    pub temporal: Option<String>,
    /// Granularity description
    pub granularity: Option<String>,
    /// Data quality notes
    #[serde(rename = "dataquality")]
    pub data_quality: Option<String>,
    /// Publisher type
    pub publisher_type: Option<String>,
    /// Conformance reference
    pub conforms_to: Option<String>,
    /// Temporal coverage start
    pub temporal_coverage_start: Option<String>,
    /// Temporal coverage end
    pub temporal_coverage_end: Option<String>,
    /// Access rights
    #[serde(rename = "accessRights")]
    pub access_rights: Option<String>,
    /// Relation reference
    pub relation: Option<String>,
}
/// Default metadata block
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DefaultMeta {
    /// Dataset title
    pub title: Option<String>,
    /// Dataset description (often HTML)
    pub description: Option<String>,
    /// Theme labels
    pub theme: Option<Vec<String>>,
    /// Keyword labels
    pub keyword: Option<Vec<String>>,
    /// License identifier
    pub license: Option<String>,
    /// License URL
    pub license_url: Option<String>,
    /// Language code
    pub language: Option<String>,
    /// Metadata language codes
    pub metadata_languages: Option<Vec<String>>,
    /// Timezone string
    pub timezone: Option<String>,
    /// Last modified timestamp
    pub modified: Option<String>,
    /// Update metadata on metadata changes
    pub modified_updates_on_metadata_change: Option<bool>,
    /// Update metadata on data changes
    pub modified_updates_on_data_change: Option<bool>,
    /// Data processed timestamp
    pub data_processed: Option<String>,
    /// Metadata processed timestamp
    pub metadata_processed: Option<String>,
    /// Geographic reference
    pub geographic_reference: Option<String>,
    /// Whether the geographic reference is automatic
    pub geographic_reference_auto: Option<bool>,
    /// Territorial reference
    pub territory: Option<String>,
    /// Geometry type labels
    pub geometry_types: Option<Vec<String>>,
    /// Bounding box definition
    pub bbox: Option<String>,
    /// Publisher name
    pub publisher: Option<String>,
    /// Reference links
    pub references: Option<String>,
    /// Record count
    pub records_count: Option<u64>,
    /// Attribution text
    pub attributions: Option<String>,
    /// Source domain identifier
    pub source_domain: Option<String>,
    /// Source domain title
    pub source_domain_title: Option<String>,
    /// Source domain address
    pub source_domain_address: Option<String>,
    /// Source dataset identifier
    pub source_dataset: Option<String>,
    /// Shared catalog identifier
    pub shared_catalog: Option<String>,
    /// Whether the dataset is federated
    pub federated: Option<bool>,
    /// Parent domain identifier
    pub parent_domain: Option<String>,
    /// Update frequency
    pub update_frequency: Option<String>,
}
/// Dublin Core metadata block
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DublinCore {
    /// Title of the dataset
    pub title: Option<String>,
    /// Alternative title
    pub alternative: Option<String>,
    /// Subject keywords
    pub subject: Option<Vec<String>>,
    /// Description text (often HTML)
    pub description: Option<String>,
    /// Abstract summary
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    /// Table of content or outline
    #[serde(rename = "tableOfContent")]
    pub table_of_content: Option<String>,
    /// Dublin Core type
    #[serde(rename = "type")]
    pub r#type: Option<DublinCoreType>,
    /// Language code
    pub language: Option<String>,
    /// Coverage notes
    pub coverage: Option<String>,
    /// Spatial coverage
    pub spatial: Option<String>,
    /// Temporal start
    #[serde(rename = "temporal_start")]
    pub temporal_start: Option<String>,
    /// Temporal end
    #[serde(rename = "temporal_end")]
    pub temporal_end: Option<String>,
    /// Relation reference
    pub relation: Option<String>,
    /// Source reference
    pub source: Option<String>,
    /// Replaces reference
    pub replaces: Option<String>,
    /// References value
    pub references: Option<String>,
    /// Requirements value
    pub requires: Option<String>,
    /// Conforms-to reference
    #[serde(rename = "conformsTo")]
    pub conforms_to: Option<String>,
    /// Has-format reference
    #[serde(rename = "hasFormat")]
    pub has_format: Option<String>,
    /// Has-part reference
    #[serde(rename = "hasPart")]
    pub has_part: Option<String>,
    /// Has-version reference
    #[serde(rename = "hasVersion")]
    pub has_version: Option<String>,
    /// Is-format-of reference
    #[serde(rename = "isFormatOf")]
    pub is_format_of: Option<String>,
    /// Is-part-of reference
    #[serde(rename = "isPartOf")]
    pub is_part_of: Option<String>,
    /// Is-version-of reference
    #[serde(rename = "isVersionOf")]
    pub is_version_of: Option<String>,
    /// Is-referenced-by reference
    #[serde(rename = "isReferencedBy")]
    pub is_referenced_by: Option<String>,
    /// Is-replaced-by reference
    #[serde(rename = "isReplacedBy")]
    pub is_replaced_by: Option<String>,
    /// Is-required-by reference
    #[serde(rename = "isRequiredBy")]
    pub is_required_by: Option<String>,
    /// Contributor names
    pub contributor: Option<Vec<String>>,
    /// Creator name
    pub creator: Option<String>,
    /// Publisher name
    pub publisher: Option<String>,
    /// Rights statement
    pub rights: Option<String>,
    /// Access rights
    #[serde(rename = "accessRights")]
    pub access_rights: Option<String>,
    /// License identifier
    pub license: Option<String>,
    /// Date start
    #[serde(rename = "date_start")]
    pub date_start: Option<String>,
    /// Date end
    #[serde(rename = "date_end")]
    pub date_end: Option<String>,
    /// Availability start date
    #[serde(rename = "available_start")]
    pub available_start: Option<String>,
    /// Availability end date
    #[serde(rename = "available_end")]
    pub available_end: Option<String>,
    /// Created date
    pub created: Option<String>,
    /// Accepted date
    #[serde(rename = "dateAccepted")]
    pub date_accepted: Option<String>,
    /// Copyrighted date
    #[serde(rename = "dateCopyrighted")]
    pub date_copyrighted: Option<String>,
    /// Submitted date
    #[serde(rename = "dateSubmitted")]
    pub date_submitted: Option<String>,
    /// Issued date
    pub issued: Option<String>,
    /// Modified date
    pub modified: Option<String>,
    /// Valid start date
    #[serde(rename = "valid_start")]
    pub valid_start: Option<String>,
    /// Valid end date
    #[serde(rename = "valid_end")]
    pub valid_end: Option<String>,
    /// Format identifier
    pub format: Option<String>,
    /// Extent string
    pub extent: Option<String>,
    /// Medium string
    pub medium: Option<String>,
    /// Identifier value
    pub identifier: Option<String>,
    /// Bibliographic citation
    #[serde(rename = "bibliographicCitation")]
    pub bibliographic_citation: Option<String>,
    /// Rights holder
    #[serde(rename = "rightsHolder")]
    pub rights_holder: Option<String>,
    /// Accrual method
    #[serde(rename = "accrualMethod")]
    pub accrual_method: Option<String>,
    /// Accrual periodicity
    #[serde(rename = "accrualPeriodicity")]
    pub accrual_periodicity: Option<String>,
    /// Accrual policy
    #[serde(rename = "accrualPolicy")]
    pub accrual_policy: Option<String>,
    /// Audience
    pub audience: Option<String>,
    /// Education level
    #[serde(rename = "educationLevel")]
    pub education_level: Option<String>,
    /// Instructional method
    #[serde(rename = "instructionalMethod")]
    pub instructional_method: Option<String>,
    /// Mediator
    pub mediator: Option<String>,
    /// Provenance description
    pub provenance: Option<String>,
}
/// DCAT-AP metadata block
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DcatAp {
    /// Title
    pub title: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Theme label
    pub theme: Option<String>,
    /// Keyword labels
    pub keyword: Option<Vec<String>>,
    /// Contact name
    pub contact_name: Option<String>,
    /// Contact email
    pub contact_email: Option<String>,
    /// Publisher name
    pub publisher_name: Option<String>,
    /// Publisher type
    pub publisher_type: Option<String>,
    /// Spatial bounding box
    pub spatial_bbox: Option<String>,
    /// Spatial centroid
    pub spatial_centroid: Option<String>,
    /// Temporal start date
    #[serde(rename = "temporal_startDate")]
    pub temporal_start_date: Option<String>,
    /// Temporal end date
    #[serde(rename = "temporal_endDate")]
    pub temporal_end_date: Option<String>,
    /// Accrual periodicity
    #[serde(rename = "accrualPeriodicity")]
    pub accrual_periodicity: Option<String>,
}
/// DataCite metadata block
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Datacite {
    /// Identifier value
    pub identifier: Option<String>,
    /// Title value
    pub title: Option<String>,
    /// Alternative title
    pub alternative_title: Option<String>,
    /// Publisher name
    pub publisher: Option<String>,
    /// Creator value
    pub creator: Option<Value>,
    /// Publication year
    pub publication_year: Option<String>,
    /// Subject keywords
    pub subject: Option<Vec<String>>,
    /// Contributor value
    pub contributor: Option<Value>,
    /// Date value
    pub date: Option<Value>,
    /// Language value
    pub language: Option<String>,
    /// Resource type
    pub resource_type: Option<Value>,
    /// Alternate identifier
    pub alternate_identifier: Option<String>,
    /// Related identifier
    #[serde(rename = "relatedidentifier")]
    pub related_identifier: Option<Value>,
    /// Size string
    pub size: Option<String>,
    /// Format string
    pub format: Option<String>,
    /// Version string
    pub version: Option<String>,
    /// Rights statement
    pub rights: Option<Value>,
    /// Description text
    pub description: Option<String>,
    /// Geolocation value
    pub geolocation: Option<String>,
}
