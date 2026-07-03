//! Database schema model
//!
use super::backend::{self, BackendRow, Connection, Params};
use super::{Database, Row, SelectQuery, TableSchemaProvider};
use crate::io::api::{self, DatabasePersistence};
use crate::io::database::macros::{build_query, define_required_fn};
use crate::io::{create_progress_bar, finish_progress_bar, ApiResult, ProgressType};
use crate::prelude::PathBuf;
use crate::util::{print_values_as_table, Label};
use acorn_macros::DatabaseRow;
use async_trait::async_trait;
use bon::Builder;
use chrono::{DateTime, Utc};
use color_eyre::eyre::eyre;
use core::fmt;
use serde::{Deserialize, Serialize};

define_required_fn!(1, required1, A, a);
define_required_fn!(3, required3, A, a, B, b, C, c);
define_required_fn!(4, required4, A, a, B, b, C, c, D, d);
define_required_fn!(5, required5, A, a, B, b, C, c, D, d, E, e);
/// Database tables
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[repr(i32)]
pub enum Table {
    /// CLI activity log
    Activity = 0,
    /// Research activity index entries from buckets
    Catalog = 1,
    /// SPDX license data
    Licenses = 2,
    /// Cached link check results
    LinkCache = 3,
    /// Validation/check results history
    ValidationHistory = 4,
    /// Downloaded research activity snapshots
    ResearchActivityCache = 5,
    /// Programming language metadata cached from GitLab
    ProgrammingLanguages = 6,
    /// Models.dev model metadata
    Models = 7,
    /// Models.dev provider metadata
    Providers = 8,
}
/// Activity log entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = activity, order_by = "executed_at DESC")]
pub struct ActivityRow {
    /// Unique identifier for the activity entry
    pub id: Option<i64>,
    /// Command that was executed
    pub command: Option<String>,
    /// When the command was executed
    pub executed_at: Option<DateTime<Utc>>,
    /// Path provided by user (if any)
    pub user_path: Option<String>,
    /// Whether the command succeeded
    pub success: Option<bool>,
}
/// Catalog/index entry from bucket
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = catalog, order_by = "title")]
pub struct CatalogRow {
    /// Unique identifier for the catalog entry
    pub id: Option<i64>,
    /// Name of the bucket
    pub bucket_name: Option<String>,
    /// URL of the bucket repository
    pub bucket_url: Option<String>,
    /// Research activity identifier
    pub identifier: Option<String>,
    /// Research activity title
    pub title: Option<String>,
    /// When the entry was last updated
    pub updated_at: Option<DateTime<Utc>>,
}
/// SPDX license entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = licenses)]
pub struct LicenseRow {
    /// Unique identifier for the license entry
    pub id: Option<i64>,
    /// Unique license identifier (ex., "MIT")
    pub identifier: Option<String>,
    /// License name (ex., "MIT License")
    pub name: Option<String>,
    /// Specifies whether the entire license is deprecated
    pub is_deprecated: Option<bool>,
    /// Specifies whether the License is listed as free by the Free Software Foundation (FSF)
    pub is_free_software: Option<bool>,
    /// Specifies whether the license is approved by the Open Source Initiative (OSI)
    pub is_open_source: Option<bool>,
}
/// Programming language metadata entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = programming_languages, order_by = "name")]
pub struct ProgrammingLanguageRow {
    /// Unique identifier for the row
    pub id: Option<i64>,
    /// Stable language identifier from GitLab linguist metadata
    pub language_id: Option<i64>,
    /// Language display name
    pub name: Option<String>,
    /// Language category (for example, programming, markup, data, prose)
    pub language_type: Option<String>,
    /// Optional color associated with language
    pub color: Option<String>,
    /// Optional parent group name
    pub group_name: Option<String>,
}
/// Models.dev model metadata entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = models)]
pub struct ModelRow {
    /// Unique identifier for the row
    pub id: Option<i64>,
    /// Unique model identifier from models.dev
    pub model_id: Option<String>,
    /// Model display name
    pub name: Option<String>,
    /// Model family (e.g., llama, gemma)
    pub family: Option<String>,
    /// Model variant (e.g., coder, instruct)
    pub variant: Option<String>,
    /// Model version string
    pub version: Option<String>,
    /// Whether the model supports file attachment
    pub attachment: Option<bool>,
    /// Whether the model weights are openly available
    pub open_weights: Option<bool>,
    /// Whether the model supports extended reasoning
    pub reasoning: Option<bool>,
    /// Whether the model supports structured output
    pub structured_output: Option<bool>,
    /// Whether the model supports temperature configuration
    pub temperature: Option<bool>,
    /// Whether the model supports tool calling
    pub tool_call: Option<bool>,
    /// Number of parameters in billions
    pub parameters: Option<i64>,
    /// Release date of the model
    pub release_date: Option<String>,
    /// Knowledge cutoff date
    pub knowledge: Option<String>,
    /// Date the model was last updated
    pub last_updated: Option<String>,
    /// Maximum context window size in tokens
    pub limit_context: Option<i64>,
    /// Maximum output token count
    pub limit_output: Option<i64>,
    /// Maximum input token count
    pub limit_input: Option<i64>,
    /// Input modalities supported (JSON array)
    pub modality_input: Option<String>,
    /// Output modalities supported (JSON array)
    pub modality_output: Option<String>,
    /// Cost per million input tokens
    pub cost_input: Option<f64>,
    /// Cost per million output tokens
    pub cost_output: Option<f64>,
    /// Cost per million cached input tokens
    pub cost_cache_read: Option<f64>,
    /// Cost per million cached write tokens
    pub cost_cache_write: Option<f64>,
    /// Extended reasoning cost per million tokens
    pub cost_reasoning: Option<f64>,
    /// Cost per million input audio tokens
    pub cost_input_audio: Option<f64>,
    /// Cost per million output audio tokens
    pub cost_output_audio: Option<f64>,
    /// Pricing for context windows exceeding 200K tokens (JSON)
    pub cost_over_200k: Option<String>,
    /// Pricing tiers (JSON array)
    pub cost_tiers: Option<String>,
    /// Benchmark evaluation results (JSON array)
    pub benchmarks: Option<String>,
    /// Download sources for model weights (JSON array)
    pub weights: Option<String>,
}
/// Models.dev provider metadata entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = providers)]
pub struct ProviderRow {
    /// Unique identifier for the row
    pub id: Option<i64>,
    /// Unique provider identifier from models.dev
    pub provider_id: Option<String>,
    /// Provider display name
    pub name: Option<String>,
    /// Provider description
    pub description: Option<String>,
    /// API endpoint base URL
    pub endpoint: Option<String>,
    /// Documentation URL
    pub documentation: Option<String>,
    /// Supported authentication methods (JSON array)
    pub authentication: Option<String>,
    /// Environment variables required (JSON array)
    pub env: Option<String>,
    /// npm package name for the provider's SDK
    pub npm: Option<String>,
    /// Provider website URL
    pub url: Option<String>,
    /// Date the provider was established
    pub established_date: Option<String>,
    /// Date the provider details were last updated
    pub last_updated: Option<String>,
}
/// Link check cache entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = link_cache)]
pub struct LinkCacheRow {
    /// Unique identifier for the cache entry
    pub id: Option<i64>,
    /// The URL that was checked
    pub url: Option<String>,
    /// HTTP status code returned
    pub status_code: Option<i32>,
    /// Whether the URL was reachable
    pub is_reachable: Option<bool>,
    /// When the URL was checked
    pub checked_at: Option<DateTime<Utc>>,
    /// When the cache entry expires
    pub expires_at: Option<DateTime<Utc>>,
}
/// Research activity cache entry
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = research_activity_cache)]
pub struct ResearchActivityCacheRow {
    /// Unique identifier for the cache entry
    pub id: Option<i64>,
    /// Research activity identifier
    pub identifier: Option<String>,
    /// Source bucket name
    pub source_bucket: Option<String>,
    /// Research activity title
    pub title: Option<String>,
    /// When the activity was downloaded
    pub downloaded_at: Option<DateTime<Utc>>,
    /// Local file path where the activity is stored
    pub file_path: Option<String>,
}
/// Validation/check result history
#[derive(Builder, Clone, DatabaseRow, Debug, Default, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[row(table = validation_history, order_by = "checked_at DESC")]
pub struct ValidationRow {
    /// Unique identifier for the validation entry
    pub id: Option<i64>,
    /// Path to the file that was validated
    pub path: Option<String>,
    /// Type of check performed (schema, link, prose, readability)
    pub check_type: Option<String>,
    /// Whether the validation passed
    pub success: Option<bool>,
    /// Validation message or error details
    pub message: Option<String>,
    /// When the validation was performed
    pub checked_at: Option<DateTime<Utc>>,
}
impl From<&str> for Table {
    fn from(value: &str) -> Self {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            | "activity" => Self::Activity,
            | "catalog" => Self::Catalog,
            | "licenses" | "license" => Self::Licenses,
            | "link_cache" | "linkcache" => Self::LinkCache,
            | "validation_history" | "validationhistory" => Self::ValidationHistory,
            | "research_activity_cache" | "researchactivitycache" => Self::ResearchActivityCache,
            | "programming_languages" | "programminglanguages" | "languages" | "language" => Self::ProgrammingLanguages,
            | "models" | "model" => Self::Models,
            | "providers" | "provider" => Self::Providers,
            | _ => Self::Activity,
        }
    }
}
impl From<&BackendRow<'_>> for ModelRow {
    fn from(row: &BackendRow<'_>) -> Self {
        ModelRow {
            id: row.get(0).ok(),
            model_id: row.get(1).ok(),
            name: row.get(2).ok(),
            family: row.get(3).ok(),
            variant: row.get(4).ok(),
            version: row.get(5).ok(),
            attachment: row.get::<_, i32>(6).ok().map(|value| value != 0),
            open_weights: row.get::<_, i32>(7).ok().map(|value| value != 0),
            reasoning: row.get::<_, i32>(8).ok().map(|value| value != 0),
            structured_output: row.get::<_, i32>(9).ok().map(|value| value != 0),
            temperature: row.get::<_, i32>(10).ok().map(|value| value != 0),
            tool_call: row.get::<_, i32>(11).ok().map(|value| value != 0),
            parameters: row.get(12).ok(),
            release_date: row.get(13).ok(),
            knowledge: row.get(14).ok(),
            last_updated: row.get(15).ok(),
            limit_context: row.get(16).ok(),
            limit_output: row.get(17).ok(),
            limit_input: row.get(18).ok(),
            modality_input: row.get(19).ok(),
            modality_output: row.get(20).ok(),
            cost_input: row.get(21).ok(),
            cost_output: row.get(22).ok(),
            cost_cache_read: row.get(23).ok(),
            cost_cache_write: row.get(24).ok(),
            cost_reasoning: row.get(25).ok(),
            cost_input_audio: row.get(26).ok(),
            cost_output_audio: row.get(27).ok(),
            cost_over_200k: row.get(28).ok(),
            cost_tiers: row.get(29).ok(),
            benchmarks: row.get(30).ok(),
            weights: row.get(31).ok(),
        }
    }
}
impl From<BackendRow<'_>> for ModelRow {
    fn from(row: BackendRow<'_>) -> Self {
        ModelRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for ProviderRow {
    fn from(row: &BackendRow<'_>) -> Self {
        ProviderRow {
            id: row.get(0).ok(),
            provider_id: row.get(1).ok(),
            name: row.get(2).ok(),
            description: row.get(3).ok(),
            endpoint: row.get(4).ok(),
            documentation: row.get(5).ok(),
            authentication: row.get(6).ok(),
            env: row.get(7).ok(),
            npm: row.get(8).ok(),
            url: row.get(9).ok(),
            established_date: row.get(10).ok(),
            last_updated: row.get(11).ok(),
        }
    }
}
impl From<BackendRow<'_>> for ProviderRow {
    fn from(row: BackendRow<'_>) -> Self {
        ProviderRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for ActivityRow {
    fn from(row: &BackendRow<'_>) -> Self {
        ActivityRow {
            id: row.get(0).ok(),
            command: row.get(1).ok(),
            executed_at: parse_rfc3339_column(row, 2),
            user_path: row.get(3).ok(),
            success: row.get::<_, i32>(4).ok().map(|value| value != 0),
        }
    }
}
impl From<BackendRow<'_>> for ActivityRow {
    fn from(row: BackendRow<'_>) -> Self {
        ActivityRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for LicenseRow {
    fn from(row: &BackendRow<'_>) -> Self {
        LicenseRow {
            id: row.get(0).ok(),
            identifier: row.get(1).ok(),
            name: row.get(2).ok(),
            is_deprecated: row.get::<_, i32>(3).ok().map(|value| value != 0),
            is_free_software: row.get::<_, i32>(4).ok().map(|value| value != 0),
            is_open_source: row.get::<_, i32>(5).ok().map(|value| value != 0),
        }
    }
}
impl From<BackendRow<'_>> for LicenseRow {
    fn from(row: BackendRow<'_>) -> Self {
        LicenseRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for ProgrammingLanguageRow {
    fn from(row: &BackendRow<'_>) -> Self {
        ProgrammingLanguageRow {
            id: row.get(0).ok(),
            language_id: row.get(1).ok(),
            name: row.get(2).ok(),
            language_type: row.get(3).ok(),
            color: row.get(4).ok(),
            group_name: row.get(5).ok(),
        }
    }
}
impl From<BackendRow<'_>> for ProgrammingLanguageRow {
    fn from(row: BackendRow<'_>) -> Self {
        ProgrammingLanguageRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for CatalogRow {
    fn from(row: &BackendRow<'_>) -> Self {
        CatalogRow {
            id: row.get(0).ok(),
            bucket_name: row.get(1).ok(),
            bucket_url: row.get(2).ok(),
            identifier: row.get(3).ok(),
            title: row.get(4).ok(),
            updated_at: parse_rfc3339_column(row, 5),
        }
    }
}
impl From<BackendRow<'_>> for CatalogRow {
    fn from(row: BackendRow<'_>) -> Self {
        CatalogRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for LinkCacheRow {
    fn from(row: &BackendRow<'_>) -> Self {
        LinkCacheRow {
            id: row.get(0).ok(),
            url: row.get(1).ok(),
            status_code: row.get(2).ok(),
            is_reachable: row.get::<_, i32>(3).ok().map(|value| value != 0),
            checked_at: parse_rfc3339_column(row, 4),
            expires_at: parse_rfc3339_column(row, 5),
        }
    }
}
impl From<BackendRow<'_>> for LinkCacheRow {
    fn from(row: BackendRow<'_>) -> Self {
        LinkCacheRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for ResearchActivityCacheRow {
    fn from(row: &BackendRow<'_>) -> Self {
        ResearchActivityCacheRow {
            id: row.get(0).ok(),
            identifier: row.get(1).ok(),
            source_bucket: row.get(2).ok(),
            title: row.get(3).ok(),
            downloaded_at: parse_rfc3339_column(row, 4),
            file_path: row.get(5).ok(),
        }
    }
}
impl From<BackendRow<'_>> for ResearchActivityCacheRow {
    fn from(row: BackendRow<'_>) -> Self {
        ResearchActivityCacheRow::from(&row)
    }
}
impl From<&BackendRow<'_>> for ValidationRow {
    fn from(row: &BackendRow<'_>) -> Self {
        ValidationRow {
            id: row.get(0).ok(),
            path: row.get(1).ok(),
            check_type: row.get(2).ok(),
            success: row.get::<_, i32>(3).ok().map(|value| value != 0),
            message: row.get(4).ok(),
            checked_at: parse_rfc3339_column(row, 5),
        }
    }
}
impl From<BackendRow<'_>> for ValidationRow {
    fn from(row: BackendRow<'_>) -> Self {
        ValidationRow::from(&row)
    }
}
impl From<ActivityRow> for Vec<String> {
    fn from(row: ActivityRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.command.unwrap_or_default(),
            row.executed_at.map_or_else(String::new, |v| v.to_rfc3339()),
            row.user_path.unwrap_or_default(),
            row.success.map_or_else(String::new, |v| v.to_string()),
        ]
    }
}
impl From<CatalogRow> for Vec<String> {
    fn from(row: CatalogRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.bucket_name.unwrap_or_default(),
            row.bucket_url.unwrap_or_default(),
            row.identifier.unwrap_or_default(),
            row.title.unwrap_or_default(),
            row.updated_at.map_or_else(String::new, |v| v.to_rfc3339()),
        ]
    }
}
impl From<LicenseRow> for Vec<String> {
    fn from(row: LicenseRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.identifier.unwrap_or_default(),
            row.name.unwrap_or_default(),
            row.is_deprecated.map_or_else(String::new, |v| v.to_string()),
            row.is_free_software.map_or_else(String::new, |v| v.to_string()),
            row.is_open_source.map_or_else(String::new, |v| v.to_string()),
        ]
    }
}
impl From<ProgrammingLanguageRow> for Vec<String> {
    fn from(row: ProgrammingLanguageRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.language_id.map_or_else(String::new, |v| v.to_string()),
            row.name.unwrap_or_default(),
            row.language_type.unwrap_or_default(),
            row.color.unwrap_or_default(),
            row.group_name.unwrap_or_default(),
        ]
    }
}
impl From<LinkCacheRow> for Vec<String> {
    fn from(row: LinkCacheRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.url.unwrap_or_default(),
            row.status_code.map_or_else(String::new, |v| v.to_string()),
            row.is_reachable.map_or_else(String::new, |v| v.to_string()),
            row.checked_at.map_or_else(String::new, |v| v.to_rfc3339()),
            row.expires_at.map_or_else(String::new, |v| v.to_rfc3339()),
        ]
    }
}
impl From<ResearchActivityCacheRow> for Vec<String> {
    fn from(row: ResearchActivityCacheRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.identifier.unwrap_or_default(),
            row.source_bucket.unwrap_or_default(),
            row.title.unwrap_or_default(),
            row.downloaded_at.map_or_else(String::new, |v| v.to_rfc3339()),
            row.file_path.unwrap_or_default(),
        ]
    }
}
impl From<ValidationRow> for Vec<String> {
    fn from(row: ValidationRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.path.unwrap_or_default(),
            row.check_type.unwrap_or_default(),
            row.success.map_or_else(String::new, |v| v.to_string()),
            row.message.unwrap_or_default(),
            row.checked_at.map_or_else(String::new, |v| v.to_rfc3339()),
        ]
    }
}
impl From<ModelRow> for Vec<String> {
    fn from(row: ModelRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.model_id.unwrap_or_default(),
            row.name.unwrap_or_default(),
            row.family.unwrap_or_default(),
            row.variant.unwrap_or_default(),
            row.version.unwrap_or_default(),
            row.attachment.map_or_else(String::new, |v| v.to_string()),
            row.open_weights.map_or_else(String::new, |v| v.to_string()),
            row.reasoning.map_or_else(String::new, |v| v.to_string()),
            row.structured_output.map_or_else(String::new, |v| v.to_string()),
            row.temperature.map_or_else(String::new, |v| v.to_string()),
            row.tool_call.map_or_else(String::new, |v| v.to_string()),
            row.parameters.map_or_else(String::new, |v| v.to_string()),
            row.release_date.unwrap_or_default(),
            row.knowledge.unwrap_or_default(),
            row.last_updated.unwrap_or_default(),
            row.limit_context.map_or_else(String::new, |v| v.to_string()),
            row.limit_output.map_or_else(String::new, |v| v.to_string()),
            row.limit_input.map_or_else(String::new, |v| v.to_string()),
            row.modality_input.unwrap_or_default(),
            row.modality_output.unwrap_or_default(),
            row.cost_input.map_or_else(String::new, |v| v.to_string()),
            row.cost_output.map_or_else(String::new, |v| v.to_string()),
            row.cost_cache_read.map_or_else(String::new, |v| v.to_string()),
            row.cost_cache_write.map_or_else(String::new, |v| v.to_string()),
            row.cost_reasoning.map_or_else(String::new, |v| v.to_string()),
            row.cost_input_audio.map_or_else(String::new, |v| v.to_string()),
            row.cost_output_audio.map_or_else(String::new, |v| v.to_string()),
            row.cost_over_200k.unwrap_or_default(),
            row.cost_tiers.unwrap_or_default(),
            row.benchmarks.unwrap_or_default(),
            row.weights.unwrap_or_default(),
        ]
    }
}
impl From<ProviderRow> for Vec<String> {
    fn from(row: ProviderRow) -> Self {
        vec![
            row.id.map_or_else(String::new, |v| v.to_string()),
            row.provider_id.unwrap_or_default(),
            row.name.unwrap_or_default(),
            row.description.unwrap_or_default(),
            row.endpoint.unwrap_or_default(),
            row.documentation.unwrap_or_default(),
            row.authentication.unwrap_or_default(),
            row.env.unwrap_or_default(),
            row.npm.unwrap_or_default(),
            row.url.unwrap_or_default(),
            row.established_date.unwrap_or_default(),
            row.last_updated.unwrap_or_default(),
        ]
    }
}
impl fmt::Display for ActivityRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ActivityRow = id: {}, command: {}, success: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.command.as_ref()),
            self.success.unwrap_or(false),
        )
        .and_then(|_| write_optional_field(f, "path", self.user_path.as_ref()))
        .and_then(|_| write_optional_timestamp(f, "executed_at", self.executed_at.as_ref()))
    }
}
impl fmt::Display for LicenseRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LicenseRow = id: {}, identifier: {}, name: {}, deprecated: {}, free: {}, osi: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.identifier.as_ref()),
            display_or_none(self.name.as_ref()),
            self.is_deprecated.unwrap_or(false),
            self.is_free_software.unwrap_or(false),
            self.is_open_source.unwrap_or(false),
        )
    }
}
impl fmt::Display for CatalogRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CatalogRow = id: {}, bucket: {}, identifier: {}, title: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.bucket_name.as_ref()),
            display_or_none(self.identifier.as_ref()),
            display_or_none(self.title.as_ref())
        )
        .and_then(|_| write_optional_timestamp(f, "updated_at", self.updated_at.as_ref()))
    }
}
impl fmt::Display for ProgrammingLanguageRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ProgrammingLanguageRow = id: {}, language_id: {}, name: {}, type: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.language_id.as_ref()),
            display_or_none(self.name.as_ref()),
            display_or_none(self.language_type.as_ref())
        )
        .and_then(|_| write_optional_field(f, "color", self.color.as_ref()))
        .and_then(|_| write_optional_field(f, "group_name", self.group_name.as_ref()))
    }
}
impl fmt::Display for LinkCacheRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LinkCacheRow = id: {}, url: {}, status: {}, reachable: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.url.as_ref()),
            self.status_code.unwrap_or(0),
            self.is_reachable.unwrap_or(false)
        )
        .and_then(|_| write_optional_timestamp(f, "checked_at", self.checked_at.as_ref()))
        .and_then(|_| write_optional_timestamp(f, "expires_at", self.expires_at.as_ref()))
    }
}
impl fmt::Display for ResearchActivityCacheRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ResearchActivityCacheRow = id: {}, identifier: {}, bucket: {}, title: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.identifier.as_ref()),
            display_or_none(self.source_bucket.as_ref()),
            display_or_none(self.title.as_ref())
        )
        .and_then(|_| write_optional_field(f, "file_path", self.file_path.as_ref()))
        .and_then(|_| write_optional_timestamp(f, "downloaded_at", self.downloaded_at.as_ref()))
    }
}
impl fmt::Display for ValidationRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ValidationRow = id: {}, path: {}, type: {}, success: {}, message: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.path.as_ref()),
            display_or_none(self.check_type.as_ref()),
            self.success.unwrap_or(false),
            display_or_none(self.message.as_ref())
        )
        .and_then(|_| write_optional_timestamp(f, "checked_at", self.checked_at.as_ref()))
    }
}
impl fmt::Display for ModelRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ModelRow = id: {}, model_id: {}, name: {}, family: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.model_id.as_ref()),
            display_or_none(self.name.as_ref()),
            display_or_none(self.family.as_ref())
        )
    }
}
impl fmt::Display for ProviderRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ProviderRow = id: {}, provider_id: {}, name: {}",
            display_or_none(self.id.as_ref()),
            display_or_none(self.provider_id.as_ref()),
            display_or_none(self.name.as_ref())
        )
        .and_then(|_| write_optional_field(f, "description", self.description.as_ref()))
    }
}
#[async_trait]
impl TableSchemaProvider for Table {
    fn all() -> &'static [Self] {
        &[
            Table::Activity,
            Table::Catalog,
            Table::Licenses,
            Table::LinkCache,
            Table::ValidationHistory,
            Table::ResearchActivityCache,
            Table::ProgrammingLanguages,
            Table::Models,
            Table::Providers,
        ]
    }
    /// Returns CREATE TABLE SQL statement
    fn create_statement(&self) -> String {
        let columns = match self {
            | Table::Activity => {
                r#"
                    command TEXT NOT NULL,
                    executed_at TEXT NOT NULL,
                    user_path TEXT,
                    success INTEGER NOT NULL DEFAULT 1
                "#
            }
            | Table::Catalog => {
                r#"
                    bucket_name TEXT NOT NULL,
                    bucket_url TEXT NOT NULL,
                    identifier TEXT NOT NULL,
                    title TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    UNIQUE(bucket_name, identifier)
                "#
            }
            | Table::Licenses => {
                r#"
                    identifier TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    is_deprecated INTEGER NOT NULL,
                    is_free_software INTEGER NOT NULL,
                    is_open_source INTEGER NOT NULL
                "#
            }
            | Table::LinkCache => {
                r#"
                    url TEXT NOT NULL UNIQUE,
                    status_code INTEGER NOT NULL,
                    is_reachable INTEGER NOT NULL,
                    checked_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL
                "#
            }
            | Table::ValidationHistory => {
                r#"
                    path TEXT NOT NULL,
                    check_type TEXT NOT NULL,
                    success INTEGER NOT NULL,
                    message TEXT NOT NULL,
                    checked_at TEXT NOT NULL
                "#
            }
            | Table::ResearchActivityCache => {
                r#"
                    identifier TEXT NOT NULL UNIQUE,
                    source_bucket TEXT NOT NULL,
                    title TEXT NOT NULL,
                    downloaded_at TEXT NOT NULL,
                    file_path TEXT NOT NULL
                "#
            }
            | Table::ProgrammingLanguages => {
                r#"
                    language_id INTEGER,
                    name TEXT NOT NULL UNIQUE,
                    language_type TEXT,
                    color TEXT,
                    group_name TEXT
                "#
            }
            | Table::Models => {
                r#"
                    model_id TEXT NOT NULL UNIQUE,
                    name TEXT,
                    family TEXT,
                    variant TEXT,
                    version TEXT,
                    attachment INTEGER,
                    open_weights INTEGER,
                    reasoning INTEGER,
                    structured_output INTEGER,
                    temperature INTEGER,
                    tool_call INTEGER,
                    parameters INTEGER,
                    release_date TEXT,
                    knowledge TEXT,
                    last_updated TEXT,
                    limit_context INTEGER,
                    limit_output INTEGER,
                    limit_input INTEGER,
                    modality_input TEXT,
                    modality_output TEXT,
                    cost_input REAL,
                    cost_output REAL,
                    cost_cache_read REAL,
                    cost_cache_write REAL,
                    cost_reasoning REAL,
                    cost_input_audio REAL,
                    cost_output_audio REAL,
                    cost_over_200k TEXT,
                    cost_tiers TEXT,
                    benchmarks TEXT,
                    weights TEXT
                "#
            }
            | Table::Providers => {
                r#"
                    provider_id TEXT NOT NULL UNIQUE,
                    name TEXT,
                    description TEXT,
                    endpoint TEXT,
                    documentation TEXT,
                    authentication TEXT,
                    env TEXT,
                    npm TEXT,
                    url TEXT,
                    established_date TEXT,
                    last_updated TEXT
                "#
            }
        };

        format!(
            r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {},
                    {}
                )
                "#,
            self.name(),
            id_column_definition(),
            columns.trim()
        )
    }
    /// Returns the table name as a string
    fn name(&self) -> &'static str {
        match self {
            | Table::Activity => "activity",
            | Table::Catalog => "catalog",
            | Table::Licenses => "licenses",
            | Table::LinkCache => "link_cache",
            | Table::ValidationHistory => "validation_history",
            | Table::ResearchActivityCache => "research_activity_cache",
            | Table::ProgrammingLanguages => "programming_languages",
            | Table::Models => "models",
            | Table::Providers => "providers",
        }
    }
    async fn populate(&self) -> ApiResult<usize> {
        match &self {
            | Table::Licenses => {
                let progress = create_progress_bar(0, ProgressType::Spinner);
                progress.set_message("Downloading SPDX license data...");
                let response = api::spdx::download().await;
                let message = format!("{}SPDX metadata download complete", Label::CHECKMARK);
                finish_progress_bar(&progress, message);
                match response {
                    | Ok(data) => data.persist(Database::<Self>::from_path(None)).await,
                    | Err(why) => Err(eyre!("Failed to download SPDX license data — {why}")),
                }
            }
            | Table::ProgrammingLanguages => {
                let progress = create_progress_bar(0, ProgressType::Spinner);
                progress.set_message("Downloading GitLab programming language data...");
                let response = api::gitlab::languages().await;
                let message = format!("{}GitLab programming language metadata download complete", Label::CHECKMARK);
                finish_progress_bar(&progress, message);
                match response {
                    | Ok(data) => data.persist(Database::<Self>::from_path(None)).await,
                    | Err(why) => Err(eyre!("Failed to download GitLab programming language data — {why}")),
                }
            }
            | Table::Models => {
                let progress = create_progress_bar(0, ProgressType::Spinner);
                progress.set_message("Downloading models.dev catalog (for models)...");
                let response = api::models_dev::download_cached().await;
                let message = format!("{}Models.dev catalog download complete", Label::CHECKMARK);
                finish_progress_bar(&progress, message);
                match response {
                    | Ok(catalog) => catalog.models().persist(Database::<Self>::from_path(None)).await,
                    | Err(why) => Err(eyre!("Failed to download models.dev catalog — {why}")),
                }
            }
            | Table::Providers => {
                let progress = create_progress_bar(0, ProgressType::Spinner);
                progress.set_message("Downloading models.dev catalog (for providers)...");
                let response = api::models_dev::download_cached().await;
                let message = format!("{}Models.dev catalog download complete", Label::CHECKMARK);
                finish_progress_bar(&progress, message);
                match response {
                    | Ok(catalog) => catalog.providers().persist(Database::<Self>::from_path(None)).await,
                    | Err(why) => Err(eyre!("Failed to download models.dev catalog — {why}")),
                }
            }
            | table => Err(eyre!("populate() not implemented for {} table", table.name())),
        }
    }
    /// Print all table rows
    /// ### Example
    /// ```ignore
    /// use acorn::io::database::{TableSchemaProvider, schema::Table};
    ///
    /// Table::Activity.print(database_path.clone());
    /// ```
    fn print(self, path: Option<PathBuf>) {
        fn print_rows<R>(table: Table, path: Option<&PathBuf>)
        where
            R: Into<Vec<String>> + Row + Default + fmt::Display,
            for<'row> R: From<&'row BackendRow<'row>>,
        {
            let row = R::default();
            let fields = <R as Row>::fields();
            let query = format!("SELECT {} FROM {}", fields.join(", "), table.name());
            let (query, params) = row.build_select_query(&query);
            let rows = table
                .rows::<R, _>(&query, params, path)
                .map(|rows| rows.into_iter().map(Into::into).collect::<Vec<Vec<String>>>())
                .unwrap_or_default();
            print_values_as_table::<String>(fields.to_vec(), rows, Some(table.name().to_string()));
        }
        match self {
            | Table::Activity => print_rows::<ActivityRow>(self, path.as_ref()),
            | Table::Catalog => print_rows::<CatalogRow>(self, path.as_ref()),
            | Table::Licenses => print_rows::<LicenseRow>(self, path.as_ref()),
            | Table::ProgrammingLanguages => print_rows::<ProgrammingLanguageRow>(self, path.as_ref()),
            | Table::LinkCache => print_rows::<LinkCacheRow>(self, path.as_ref()),
            | Table::ValidationHistory => print_rows::<ValidationRow>(self, path.as_ref()),
            | Table::ResearchActivityCache => print_rows::<ResearchActivityCacheRow>(self, path.as_ref()),
            | Table::Models => print_rows::<ModelRow>(self, path.as_ref()),
            | Table::Providers => print_rows::<ProviderRow>(self, path.as_ref()),
        }
    }
    /// Get table rows
    fn rows<R, P>(&self, query: &str, params: P, path: Option<&PathBuf>) -> ApiResult<Vec<R>>
    where
        P: Params,
        for<'row> R: Row + From<&'row BackendRow<'row>>,
    {
        Database::<Table>::from_path(path.cloned()).with_connection(|conn| {
            conn.prepare(query)
                .map_err(|why| eyre!("=> {} Failed to prepare {} query: {why}", Label::fail(), self.name()))
                .and_then(|mut stmt| {
                    stmt.query_map(params, |row| Ok(R::from(row)))
                        .map_err(|why| eyre!("=> {} Failed to query {}: {why}", Label::fail(), self.name()))
                        .and_then(|rows| {
                            rows.collect::<core::result::Result<Vec<_>, _>>()
                                .map_err(|why| eyre!("=> {} Failed to read {} rows: {why}", Label::fail(), self.name()))
                        })
                })
        })
    }
}
impl Row for ActivityRow {
    fn table(&self) -> Table {
        Table::Activity
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            command,
            executed_at,
            user_path,
            success,
            ..
        } = self.clone();
        let executed_at = executed_at.unwrap_or_else(Utc::now).to_rfc3339();
        let success = i32::from(success.unwrap_or(true));
        let required = required1(command);
        match required {
            | Ok((command,)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, command, executed_at, user_path, success];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for CatalogRow {
    fn table(&self) -> Table {
        Table::Catalog
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            bucket_name,
            bucket_url,
            identifier,
            title,
            updated_at,
            ..
        } = self.clone();
        let updated_at = updated_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(bucket_name, bucket_url, identifier, title);
        match required {
            | Ok((bucket_name, bucket_url, identifier, title)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, bucket_name, bucket_url, identifier, title, updated_at];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for LicenseRow {
    fn table(&self) -> Table {
        Table::Licenses
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            identifier,
            name,
            is_deprecated,
            is_free_software,
            is_open_source,
            ..
        } = self.clone();
        let required = required5(identifier, name, is_deprecated, is_free_software, is_open_source);
        match required {
            | Ok((identifier, name, is_deprecated, is_free_software, is_open_source)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![
                        id,
                        identifier,
                        name,
                        i32::from(is_deprecated),
                        i32::from(is_free_software),
                        i32::from(is_open_source)
                    ];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for LinkCacheRow {
    fn table(&self) -> Table {
        Table::LinkCache
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            url,
            status_code,
            is_reachable,
            checked_at,
            expires_at,
            ..
        } = self.clone();
        let checked_at = checked_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(url, status_code, is_reachable, expires_at);
        match required {
            | Ok((url, status_code, is_reachable, expires_at)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, url, status_code, i32::from(is_reachable), checked_at, expires_at.to_rfc3339()];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for ProgrammingLanguageRow {
    fn table(&self) -> Table {
        Table::ProgrammingLanguages
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            language_id,
            name,
            language_type,
            color,
            group_name,
            ..
        } = self.clone();
        let required = required1(name);
        match required {
            | Ok((name,)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, language_id, name, language_type, color, group_name];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for ResearchActivityCacheRow {
    fn table(&self) -> Table {
        Table::ResearchActivityCache
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            identifier,
            source_bucket,
            title,
            downloaded_at,
            file_path,
            ..
        } = self.clone();
        let downloaded_at = downloaded_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required4(identifier, source_bucket, title, file_path);
        match required {
            | Ok((identifier, source_bucket, title, file_path)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, identifier, source_bucket, title, downloaded_at, file_path];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for ValidationRow {
    fn table(&self) -> Table {
        Table::ValidationHistory
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            path,
            check_type,
            success,
            message,
            checked_at,
            ..
        } = self.clone();
        let success = i32::from(success.unwrap_or(true));
        let checked_at = checked_at.unwrap_or_else(Utc::now).to_rfc3339();
        let required = required3(path, check_type, message);
        match required {
            | Ok((path, check_type, message)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![id, path, check_type, success, message, checked_at];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for ModelRow {
    fn table(&self) -> Table {
        Table::Models
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            model_id,
            name,
            family,
            variant,
            version,
            attachment,
            open_weights,
            reasoning,
            structured_output,
            temperature,
            tool_call,
            parameters,
            release_date,
            knowledge,
            last_updated,
            limit_context,
            limit_output,
            limit_input,
            modality_input,
            modality_output,
            cost_input,
            cost_output,
            cost_cache_read,
            cost_cache_write,
            cost_reasoning,
            cost_input_audio,
            cost_output_audio,
            cost_over_200k,
            cost_tiers,
            benchmarks,
            weights,
            ..
        } = self.clone();
        let required = required1(model_id);
        match required {
            | Ok((model_id,)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![
                        id,
                        model_id,
                        name,
                        family,
                        variant,
                        version,
                        attachment.map(i32::from),
                        open_weights.map(i32::from),
                        reasoning.map(i32::from),
                        structured_output.map(i32::from),
                        temperature.map(i32::from),
                        tool_call.map(i32::from),
                        parameters,
                        release_date,
                        knowledge,
                        last_updated,
                        limit_context,
                        limit_output,
                        limit_input,
                        modality_input,
                        modality_output,
                        cost_input,
                        cost_output,
                        cost_cache_read,
                        cost_cache_write,
                        cost_reasoning,
                        cost_input_audio,
                        cost_output_audio,
                        cost_over_200k,
                        cost_tiers,
                        benchmarks,
                        weights,
                    ];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl Row for ProviderRow {
    fn table(&self) -> Table {
        Table::Providers
    }
    fn insert(self, conn: &Connection) -> ApiResult<usize> {
        let Self {
            provider_id,
            name,
            description,
            endpoint,
            documentation,
            authentication,
            env,
            npm,
            url,
            established_date,
            last_updated,
            ..
        } = self.clone();
        let required = required1(provider_id);
        match required {
            | Ok((provider_id,)) => match self.next_row_id(conn) {
                | Ok(id) => {
                    let params = backend::params![
                        id,
                        provider_id,
                        name,
                        description,
                        endpoint,
                        documentation,
                        authentication,
                        env,
                        npm,
                        url,
                        established_date,
                        last_updated,
                    ];
                    execute_insert(&self, conn, params)
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    fn build_select_query(&self, base: &str) -> SelectQuery {
        Self::build_select_query(self, base)
    }
}
impl LinkCacheRow {
    /// Check if the cache entry is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Utc::now() > e).unwrap_or(true)
    }
}
fn display_or_none<T>(value: Option<&T>) -> String
where
    T: fmt::Display,
{
    value.map(ToString::to_string).unwrap_or_else(|| "None".to_string())
}
fn execute_insert<R, P>(row: &R, conn: &Connection, params: P) -> ApiResult<usize>
where
    R: Row,
    P: Params,
{
    let table_name = row.table().name();
    let table_columns = <R as Row>::fields();
    let fields = table_columns.join(", ");
    let placeholders = table_columns.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query = format!("INSERT INTO {table_name} ({fields}) VALUES ({placeholders})");
    conn.execute(&query, params)
        .map_err(|why| eyre!("=> {} Failed to insert row: {why}", Label::fail()))
}
fn parse_rfc3339_column(row: &BackendRow<'_>, index: usize) -> Option<DateTime<Utc>> {
    row.get::<_, String>(index)
        .ok()
        .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
        .map(|value| value.with_timezone(&Utc))
}
fn write_optional_field<T>(f: &mut fmt::Formatter<'_>, label: &str, value: Option<&T>) -> fmt::Result
where
    T: fmt::Display,
{
    match value {
        | Some(value) => write!(f, ", {label}: {value}"),
        | None => Ok(()),
    }
}
fn write_optional_timestamp(f: &mut fmt::Formatter<'_>, label: &str, value: Option<&DateTime<Utc>>) -> fmt::Result {
    match value {
        | Some(value) => write!(f, ", {label}: {}", value.format("%Y-%m-%d %H:%M:%S")),
        | None => Ok(()),
    }
}

#[cfg(feature = "duckdb")]
fn id_column_definition() -> &'static str {
    "id BIGINT PRIMARY KEY"
}

#[cfg(not(feature = "duckdb"))]
fn id_column_definition() -> &'static str {
    "id INTEGER PRIMARY KEY AUTOINCREMENT"
}
