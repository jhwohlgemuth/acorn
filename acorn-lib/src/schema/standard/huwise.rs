//! HuWise dataset schema models
//!
#[cfg(feature = "std")]
use crate::io::{read_file, write_file, ApiResult, InputOutput};
use crate::prelude::*;
use crate::schema::standard::crosswalk::{self, mapping::datacite_to_huwise, CrosswalkError, FieldValue, Fields, SchemaBuilder, SchemaExtractor};
use crate::schema::standard::datacite;
#[cfg(feature = "std")]
use crate::util::MimeType;
use crate::util::ToProse;
#[cfg(feature = "std")]
use crate::PathBuf;
#[cfg(feature = "std")]
use ammonia::clean;
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use validator::Validate;

#[cfg(not(feature = "std"))]
fn clean(value: &str) -> String {
    value.to_string()
}

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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct CustomTemplate {
    /// Projects field (custom template)
    pub projects: Option<String>,
    /// Source-of-data field
    #[serde(rename = "source-of-data")]
    pub source_of_data: Option<String>,
}
/// Top-level dataset container from HuWise exports
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    #[validate(nested)]
    pub metas: Meta,
    /// Feature records associated with the dataset
    pub features: Vec<Value>,
}
/// Metadata sections attached to a dataset
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Meta {
    /// DCAT metadata
    #[validate(nested)]
    pub dcat: Option<Dcat>,
    /// Default metadata
    #[serde(rename = "default")]
    #[validate(nested)]
    pub r#default: Option<DefaultMeta>,
    /// Dublin Core metadata
    #[serde(rename = "dublin-core")]
    #[validate(nested)]
    pub dublin_core: Option<DublinCore>,
    /// DCAT-AP metadata
    #[validate(nested)]
    pub dcat_ap: Option<DcatAp>,
    /// Custom template metadata
    #[serde(rename = "custom-template")]
    #[validate(nested)]
    pub custom_template: Option<CustomTemplate>,
    /// DataCite metadata
    #[validate(nested)]
    pub datacite: Option<Datacite>,
}
/// DCAT metadata block
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    #[validate(email)]
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    #[validate(url)]
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    pub kind: Option<DublinCoreType>,
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    #[validate(email)]
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
impl TryFrom<datacite::Record> for Dataset {
    type Error = CrosswalkError;

    fn try_from(record: datacite::Record) -> Result<Self, Self::Error> {
        let mapping = datacite_to_huwise();
        crosswalk::convert(&record, &mapping).map(|(dataset, _)| dataset)
    }
}
impl TryFrom<&datacite::Record> for Dataset {
    type Error = CrosswalkError;

    fn try_from(record: &datacite::Record) -> Result<Self, Self::Error> {
        Dataset::try_from(record.clone())
    }
}
impl SchemaBuilder for Dataset {
    fn build_from_fields(fields: &Fields) -> Result<Self, CrosswalkError> {
        let dataset_id = fields.get_string("identifier")?;
        let datacite_meta = Some(build_datacite_block(fields));
        let meta = Meta {
            datacite: datacite_meta,
            r#default: None,
            dublin_core: None,
            dcat: None,
            dcat_ap: None,
            custom_template: None,
        };
        Ok(Dataset {
            dataset_id,
            has_attachments: false,
            attachments_count: 0,
            has_records: false,
            fields: Value::Array(vec![]),
            metas: meta,
            features: vec![],
        })
    }
}
fn build_datacite_block(fields: &Fields) -> Datacite {
    let title = fields.get_string_opt("title");
    let identifier = fields.get_string_opt("identifier");
    let creator = fields
        .get_string_vec_opt("creators")
        .map(|names| Value::Array(names.into_iter().map(Value::String).collect()));
    let publication_year = fields.get_number_opt("publication-year").map(|y| (y as i32).to_string());
    let description = fields.get_string_opt("description");
    let subject = fields.get_string_vec_opt("subjects");
    let language = fields.get_string_opt("language");
    let publisher = fields.get_string_opt("publisher");
    let resource_type = fields.get_string_opt("resource-type").map(Value::String);
    let rights = fields.get_string_opt("license").map(Value::String);
    let version = fields.get_string_opt("version");
    Datacite {
        identifier,
        title,
        alternative_title: None,
        publisher,
        creator,
        publication_year,
        date: None,
        subject,
        contributor: None,
        description,
        language,
        resource_type,
        alternate_identifier: None,
        related_identifier: None,
        size: None,
        format: None,
        version,
        rights,
        geolocation: None,
    }
}
impl SchemaExtractor for Dataset {
    fn extract_fields(&self) -> Fields {
        fn extract_json_string_array(value: &Value) -> Option<Vec<String>> {
            match value {
                | Value::Array(arr) => {
                    let strings: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    if !strings.is_empty() {
                        Some(strings)
                    } else {
                        None
                    }
                }
                | Value::String(s) => Some(vec![s.clone()]),
                | _ => None,
            }
        }
        let mut fields = Fields::new();
        fields.insert("identifier", FieldValue::String(self.dataset_id.clone()));
        if let Some(meta) = &self.metas.datacite {
            if let Some(value) = &meta.title {
                fields.insert("title", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.identifier {
                fields.insert("doi", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.creator {
                if let Some(creators) = extract_json_string_array(value) {
                    if !creators.is_empty() {
                        fields.insert("creators", FieldValue::StringVec(creators));
                    }
                }
            }
            if let Some(value) = &meta.publication_year {
                if let Ok(year) = value.parse::<f64>() {
                    fields.insert("publication-year", FieldValue::Number(year));
                }
            }
            if let Some(value) = &meta.description {
                fields.insert("description", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.subject {
                if !value.is_empty() {
                    fields.insert("subjects", FieldValue::StringVec(value.clone()));
                }
            }
            if let Some(value) = &meta.language {
                fields.insert("language", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.publisher {
                fields.insert("publisher", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.resource_type {
                if let Some(s) = value.as_str() {
                    fields.insert("resource-type", FieldValue::String(s.to_string()));
                }
            }
            if let Some(value) = &meta.rights {
                if let Some(s) = value.as_str() {
                    fields.insert("license", FieldValue::String(s.to_string()));
                }
            }
            if let Some(value) = &meta.version {
                fields.insert("version", FieldValue::String(value.clone()));
            }
        } else if let Some(meta) = &self.metas.r#default {
            if let Some(title) = &meta.title {
                fields.insert("title", FieldValue::String(title.clone()));
            }
            if let Some(value) = &meta.description {
                let sanitized = clean(value);
                fields.insert("description", FieldValue::String(sanitized));
            }
            if let Some(value) = &meta.keyword {
                if !value.is_empty() {
                    fields.insert("subjects", FieldValue::StringVec(value.clone()));
                }
            }
            if let Some(value) = &meta.language {
                fields.insert("language", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.publisher {
                fields.insert("publisher", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.license {
                fields.insert("license", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.modified {
                fields.insert("updated", FieldValue::Date(value.clone()));
            }
        } else if let Some(meta) = &self.metas.dublin_core {
            if let Some(value) = &meta.title {
                fields.insert("title", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.abstract_text {
                let sanitized = clean(value);
                fields.insert("description", FieldValue::String(sanitized));
            }
            if let Some(value) = &meta.creator {
                fields.insert("creators", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.subject {
                if !value.is_empty() {
                    fields.insert("subjects", FieldValue::StringVec(value.clone()));
                }
            }
            if let Some(value) = &meta.language {
                fields.insert("language", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.publisher {
                fields.insert("publisher", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.issued {
                fields.insert("publication-year", FieldValue::Date(value.clone()));
            }
            if let Some(value) = &meta.license {
                fields.insert("license", FieldValue::String(value.clone()));
            }
        } else if let Some(meta) = &self.metas.dcat {
            if let Some(value) = &meta.creator {
                fields.insert("creators", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.issued {
                fields.insert("publication-year", FieldValue::Date(value.clone()));
            }
            if let Some(value) = &meta.spatial {
                fields.insert("spatial", FieldValue::String(value.clone()));
            }
        } else if let Some(meta) = &self.metas.dcat_ap {
            if let Some(value) = &meta.title {
                fields.insert("title", FieldValue::String(value.clone()));
            }
            if let Some(value) = &meta.description {
                let sanitized = clean(value);
                fields.insert("description", FieldValue::String(sanitized));
            }
            if let Some(value) = &meta.keyword {
                if !value.is_empty() {
                    fields.insert("subjects", FieldValue::StringVec(value.clone()));
                }
            }
            if let Some(value) = &meta.publisher_name {
                fields.insert("publisher", FieldValue::String(value.clone()));
            }
        }
        fields
    }
}
impl ToProse for Dataset {
    fn to_prose(&self) -> String {
        self.metas
            .datacite
            .iter()
            .flat_map(|meta| meta.title.iter().cloned())
            .chain(self.metas.datacite.iter().flat_map(|meta| meta.description.iter().cloned()))
            .chain(self.metas.datacite.iter().flat_map(|meta| meta.subject.iter().flatten().cloned()))
            .chain(self.metas.r#default.iter().flat_map(|meta| meta.title.iter().cloned()))
            .chain(
                self.metas
                    .r#default
                    .iter()
                    .flat_map(|meta| meta.description.iter().map(|value| clean(value))),
            )
            .chain(self.metas.r#default.iter().flat_map(|meta| meta.keyword.iter().flatten().cloned()))
            .chain(self.metas.dublin_core.iter().flat_map(|meta| meta.title.iter().cloned()))
            .chain(
                self.metas
                    .dublin_core
                    .iter()
                    .flat_map(|meta| meta.description.iter().map(|value| clean(value))),
            )
            .chain(
                self.metas
                    .dublin_core
                    .iter()
                    .flat_map(|meta| meta.abstract_text.iter().map(|value| clean(value))),
            )
            .chain(self.metas.dublin_core.iter().flat_map(|meta| meta.subject.iter().flatten().cloned()))
            .chain(self.metas.dcat_ap.iter().flat_map(|meta| meta.title.iter().cloned()))
            .chain(
                self.metas
                    .dcat_ap
                    .iter()
                    .flat_map(|meta| meta.description.iter().map(|value| clean(value))),
            )
            .chain(self.metas.dcat_ap.iter().flat_map(|meta| meta.keyword.iter().flatten().cloned()))
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}
#[cfg(feature = "std")]
impl InputOutput for Dataset {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Dataset> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Json => Dataset::read_json(source),
            | MimeType::Yaml => Dataset::read_yaml(source),
            | _ => Err(eyre!("Unsupported HuWise data file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Dataset> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum JsonInput {
            One(Box<Dataset>),
            Many(Vec<Dataset>),
        }

        read_file(path).and_then(|content| {
            serde_json::from_str::<JsonInput>(&content)
                .map_err(|why| eyre!("Failed to parse JSON HuWise dataset — {why}"))
                .and_then(|value| match value {
                    | JsonInput::One(dataset) => Ok(*dataset),
                    | JsonInput::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one HuWise dataset but found none")),
                        | len => Err(eyre!("Expected one HuWise dataset but found {len}")),
                    },
                })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Dataset> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum YamlInput {
            One(Box<Dataset>),
            Many(Vec<Dataset>),
        }

        read_file(path).and_then(|content| {
            serde_norway::from_str::<YamlInput>(&content)
                .map_err(|why| eyre!("Failed to parse YAML HuWise dataset — {why}"))
                .and_then(|value| match value {
                    | YamlInput::One(dataset) => Ok(*dataset),
                    | YamlInput::Many(datasets) => match datasets.len() {
                        | 1 => datasets
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one HuWise dataset but found none")),
                        | len => Err(eyre!("Expected one HuWise dataset but found {len}")),
                    },
                })
        })
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported HuWise data file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON HuWise dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML HuWise dataset — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
