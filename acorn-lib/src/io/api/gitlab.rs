//! Module for interacting with GitLab API
//!
// TODO: Add custom field list and query pair type
// TODO: Finish modeling necessary API endpoints
use crate::io::api::{DatabasePersistence, EmptyField, Endpoint, RemoteResource, ResponseContent, TreeEntryType, ValueValidator, INCLUDED_ENDPOINTS};
use crate::io::config::{RunnerStatus, RunnerType};
use crate::io::database::schema::{ProgrammingLanguageRow, Table};
use crate::io::database::{Database, Operations};
use crate::io::{with_progress, ApiResult, ProgressType};
use crate::param;
use crate::prelude::var;
use crate::prelude::HashMap;
use crate::util::{Label, Searchable};
use async_trait::async_trait;
use bon::Builder;
use color_eyre::eyre::{self, eyre};
use core::fmt;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Type for GitLab API descendent groups response
pub type GroupsResponse = Vec<GroupDetails>;
/// Type for GitLab API all runners response
pub type RunnersResponse = Vec<RunnerDetails>;
/// Type for GitLab API response for a tree
pub type TreeResponse = Vec<TreeEntry>;
/// Type for GitLab programming language entries
pub type ProgrammingLanguageEntries = Vec<ProgrammingLanguageMetadata>;
/// Access level of the runner
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    /// Not protected
    NotProtected,
    /// Ref protected
    RefProtected,
}
/// Language metadata details from GitLab languages YAML file
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProgrammingLanguageDetails {
    /// Canonical language identifier
    pub language_id: Option<u64>,
    /// Language category (for example, `programming`, `data`, `markup`, or `prose`)
    #[serde(rename = "type")]
    pub language_type: Option<String>,
    /// Display color (hex string)
    pub color: Option<String>,
    /// Optional parent language group
    pub group: Option<String>,
}
/// Normalized programming language metadata with explicit language name
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProgrammingLanguageMetadata {
    /// Display name of the language
    pub name: String,
    /// Canonical language identifier
    pub language_id: Option<u64>,
    /// Language category (for example, `programming`, `data`, `markup`, or `prose`)
    pub language_type: Option<String>,
    /// Display color (hex string)
    pub color: Option<String>,
    /// Optional parent language group
    pub group: Option<String>,
}
/// Parsed response for GitLab language metadata
#[derive(Clone, Debug, Default, Serialize)]
pub struct ProgrammingLanguagesResponse {
    /// Flattened language metadata entries
    pub languages: ProgrammingLanguageEntries,
}
/// Group visibility level
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupVisibility {
    /// Public visibility
    #[default]
    Public,
    /// Internal visibility
    Internal,
    /// Private visibility
    Private,
}
/// GitLab emoji shortcodes
///
/// See <https://www.webfx.com/tools/emoji-cheat-sheet/> for full list of supported emoji shortcodes
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
pub enum Emoji {
    /// :seedling: 🌱
    #[display(":seedling:")]
    Seedling,
}
/// Pagination list parameters
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaginationField {
    /// Column by which to order by
    OrderBy,
    /// Page number to retrieve (default: 1)
    Page,
    /// Enable keyset pagination
    Pagination,
    /// Number of items to list per page (default: 20, max: 100)
    PerPage,
    /// Sort order
    Sort,
}
/// Valid values for pagination order_by field
///
/// Projects can be ordered by
/// - `created_at` (default)
/// - `id`
/// - `last_activity_at`
/// - `name`
/// - `path`
/// - `similarity`
/// - `star_count`
/// - `updated_at`
///
/// Groups can be ordered by
/// - `name` (default)
/// - `id`
/// - `path`
/// - `similarity`
///
/// Issues can be ordered by
/// - `created_at` (default)
/// - `due_date`
/// - `label_priority`
/// - `milestone_due`
/// - `popularity`
/// - `priority`
/// - `relative_position`
/// - `title`
/// - `updated_at`
/// - `weight`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderByValue {
    /// Sort by creation timestamp
    CreatedAt,
    /// Sort by full hierarchical name
    FullName,
    /// Sort by unique identifier
    #[serde(rename = "id")]
    Identifier,
    /// Sort by label priority
    LabelPriority,
    /// Sort by last activity timestamp
    LastActivityAt,
    /// Sort by milestone due date
    MilestoneDue,
    /// Sort by human-readable name
    Name,
    /// Sort by URL-encoded path
    Path,
    /// Sort by popularity
    Popularity,
    /// Sort by due date
    DueDate,
    /// Sort by priority
    Priority,
    /// Sort by manual relative position
    RelativePosition,
    /// Sort by search similarity score
    Similarity,
    /// Sort by title
    Title,
    /// Sort by last update timestamp
    UpdatedAt,
    /// Sort by weight/priority
    Weight,
}
/// Valid values for pagination sort field
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortValue {
    /// Descending order
    #[default]
    #[serde(rename = "desc")]
    Descending,
    /// Ascending order
    #[serde(rename = "asc")]
    Ascending,
}
/// Runner group details
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupDetails {
    /// Numeric ID of the group
    #[serde(rename = "id")]
    pub identifier: u64,
    /// URL of the group page
    #[serde(rename = "web_url")]
    pub url: String,
    /// Group name
    pub name: String,
    /// Group path
    pub path: Option<String>,
    /// Group description
    pub description: Option<String>,
    /// Whether emails are disabled
    #[serde(default)]
    pub emails_disabled: bool,
    /// Whether emails are enabled
    #[serde(default)]
    pub emails_enabled: bool,
    /// Whether diff previews appear in emails
    #[serde(default)]
    pub show_diff_preview_in_email: bool,
    /// Group visibility level
    pub visibility: Option<GroupVisibility>,
    /// Whether sharing with other groups is locked
    #[serde(default)]
    pub share_with_group_lock: bool,
    /// Whether two-factor authentication is required
    #[serde(default)]
    pub require_two_factor_authentication: bool,
    /// Whether LFS is enabled
    #[serde(default)]
    pub lfs_enabled: bool,
    /// Whether the group is archived
    #[serde(default)]
    pub archived: bool,
    /// Duo features enabled flag
    #[serde(default)]
    pub duo_features_enabled: bool,
    /// Duo features lock flag
    #[serde(default)]
    pub lock_duo_features_enabled: bool,
    /// Auto Duo code review enabled flag
    #[serde(default)]
    pub auto_duo_code_review_enabled: bool,
    /// Whether math rendering limits are enabled
    #[serde(default)]
    pub math_rendering_limits_enabled: bool,
    /// Whether math rendering limits are locked
    #[serde(default)]
    pub lock_math_rendering_limits_enabled: bool,
    /// Whether access requests are enabled
    #[serde(default)]
    pub request_access_enabled: bool,
    /// Grace period for two-factor authentication
    pub two_factor_grace_period: Option<u64>,
    /// Project creation level
    pub project_creation_level: Option<String>,
    /// Auto DevOps enabled flag
    pub auto_devops_enabled: Option<bool>,
    /// Subgroup creation level
    pub subgroup_creation_level: Option<String>,
    /// Whether mentions are disabled
    pub mentions_disabled: Option<bool>,
    /// Default branch name
    pub default_branch: Option<String>,
    /// Default branch protection mode
    pub default_branch_protection: Option<u64>,
    /// Default branch protection policy details
    pub default_branch_protection_defaults: Option<RunnerGroupBranchProtectionDefaults>,
    /// Group avatar URL
    #[serde(rename = "avatar_url")]
    pub avatar_url: Option<String>,
    /// Group full display name
    pub full_name: Option<String>,
    /// Group full path
    pub full_path: Option<String>,
    /// Group creation timestamp in ISO-8601 format
    pub created_at: Option<String>,
    /// Parent group identifier
    pub parent_id: Option<u64>,
    /// Organization identifier
    pub organization_id: Option<u64>,
    /// Shared runners setting
    pub shared_runners_setting: Option<String>,
    /// Maximum artifacts size limit
    pub max_artifacts_size: Option<u64>,
    /// Group deletion schedule date
    pub marked_for_deletion_on: Option<String>,
    /// LDAP common name
    #[serde(rename = "ldap_cn")]
    pub ldap_common_name: Option<String>,
    /// LDAP access value
    pub ldap_access: Option<String>,
    /// File template project identifier
    pub file_template_project_id: Option<u64>,
    /// Wiki access level
    pub wiki_access_level: Option<String>,
    /// Duo core features enabled flag
    pub duo_core_features_enabled: Option<bool>,
}
/// GitLab API response for creating a merge request comment
/// ### Example JSON response
/// ```json
/// {
///     "id": 1774626,
///     "type": null,
///     "body": "comment text",
///     "author": {
///         "id": 4862,
///         "username": "o9w",
///         "public_email": "wohlgemuthjh@ornl.gov",
///         "name": "Wohlgemuth, Jason",
///         "state": "active",
///         "locked": false,
///         "avatar_url": "https://code.ornl.gov/uploads/-/system/user/avatar/4862/avatar.png",
///         "web_url": "https://code.ornl.gov/o9w"
///     },
///     "created_at": "2026-04-11T22:47:15.052Z",
///     "updated_at": "2026-04-11T22:47:15.052Z",
///     "system": false,
///     "noteable_id": 116322,
///     "noteable_type": "MergeRequest",
///     "project_id": 16689,
///     "resolvable": false,
///     "confidential": false,
///     "internal": false,
///     "imported": false,
///     "imported_from": "none",
///     "noteable_iid": 11,
///     "commands_changes": {}
/// }
///
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MergeRequestNoteResponse {
    /// Numeric ID of the note
    #[serde(rename = "id")]
    pub identifier: u64,
    /// Optional note type
    #[serde(rename = "type")]
    pub note_type: Option<String>,
    /// Comment body text
    pub body: String,
    /// Author details
    pub author: UserDetails,
    /// Creation timestamp in ISO-8601 format
    pub created_at: String,
    /// Last update timestamp in ISO-8601 format
    pub updated_at: String,
    /// Whether this is a system note
    pub system: bool,
    /// Numeric identifier of associated noteable object
    pub noteable_id: u64,
    /// Internal identifier (IID) of associated noteable object
    pub noteable_iid: u64,
    /// Type of associated noteable object
    pub noteable_type: String,
    /// Numeric project identifier
    pub project_id: u64,
    /// Whether the note is resolvable
    pub resolvable: bool,
    /// Whether the note is confidential
    pub confidential: bool,
    /// Whether the note is internal
    pub internal: bool,
    /// Whether the note was imported
    pub imported: bool,
    /// Source from which note was imported
    pub imported_from: String,
    /// Parsed quick action command changes
    pub commands_changes: serde_json::Value,
}
/// Options for GitLab API requests
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = with_token, on(String, into))]
pub struct Options {
    /// Authentication token
    #[builder(start_fn)]
    pub token: String,
    /// Project or group identifier
    pub identifier: Option<String>,
    /// Internal resource identifier (e.g., merge request IID)
    pub internal_identifier: Option<String>,
    /// Request body payload
    pub body: Option<String>,
    /// GitLab domain (defaults to gitlab.com)
    #[builder(default = String::from("gitlab.com"))]
    pub domain: String,
}
/// GitLab API response for runner details
/// ### Example JSON response
/// ```json
/// {
///     "active": true,
///     "paused": false,
///     "architecture": null,
///     "description": "test-1-20150125",
///     "id": 6,
///     "ip_address": "",
///     "is_shared": false,
///     "runner_type": "project_type",
///     "contacted_at": "2016-01-25T16:39:48.066Z",
///     "maintenance_note": null,
///     "name": null,
///     "online": true,
///     "status": "online",
///     "platform": null,
///     "projects": [
///         {
///             "id": 1,
///             "name": "GitLab Community Edition",
///             "name_with_namespace": "GitLab.org / GitLab Community Edition",
///             "path": "gitlab-foss",
///             "path_with_namespace": "gitlab-org/gitlab-foss"
///         }
///     ],
///     "revision": null,
///     "tag_list": [
///         "ruby",
///         "mysql"
///     ],
///     "version": null,
///     "access_level": "ref_protected",
///     "maximum_timeout": 3600
/// }
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunnerDetails {
    /// Numeric ID of the runner
    #[serde(rename = "id")]
    pub identifier: u64,
    /// Whether the runner is active
    #[serde(default)]
    pub active: bool,
    /// Whether the runner is online
    #[serde(default)]
    pub online: bool,
    /// Whether the runner is paused
    #[serde(default)]
    pub paused: bool,
    /// Whether the runner is shared
    #[serde(default)]
    #[serde(rename = "is_shared")]
    pub shared: bool,
    /// CPU architecture reported by the runner
    pub architecture: Option<String>,
    /// Runner description
    pub description: Option<String>,
    /// Runner IP address
    pub ip_address: Option<String>,
    /// Type of runner (for example, `project_type`)
    pub runner_type: Option<RunnerType>,
    /// Created by user
    pub created_by: Option<UserDetails>,
    /// Created timestamp in ISO-8601 format
    pub created_at: Option<String>,
    /// Last contact timestamp in ISO-8601 format
    pub contacted_at: Option<String>,
    /// Optional maintenance note
    pub maintenance_note: Option<String>,
    /// Optional display name
    pub name: Option<String>,
    /// Current runner status
    pub status: Option<RunnerStatus>,
    /// Current job execution status
    pub job_execution_status: Option<String>,
    /// Optional platform string
    pub platform: Option<String>,
    /// Projects associated with this runner
    pub projects: Option<Vec<RunnerScope>>,
    /// Groups associated with this runner
    pub groups: Option<Vec<RunnerScope>>,
    /// Optional Git revision for the runner version
    pub revision: Option<String>,
    /// Runner tags
    #[serde(rename = "tag_list")]
    pub tags: Option<Vec<String>>,
    /// Optional runner version
    pub version: Option<String>,
    /// Access level for this runner
    pub access_level: Option<AccessLevel>,
    /// Maximum timeout in seconds
    pub maximum_timeout: Option<u64>,
}
/// Access level entry for branch protection settings
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunnerGroupAccessLevel {
    /// Numeric access level
    pub access_level: u64,
}
/// Runner group branch protection defaults
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunnerGroupBranchProtectionDefaults {
    /// Access levels allowed to push
    pub allowed_to_push: Vec<RunnerGroupAccessLevel>,
    /// Whether force push is allowed
    pub allow_force_push: bool,
    /// Access levels allowed to merge
    pub allowed_to_merge: Vec<RunnerGroupAccessLevel>,
}
/// Runner scope details for project or group entries
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunnerScope {
    /// Numeric ID of the scope entry
    #[serde(rename = "id")]
    pub identifier: u64,
    /// Scope name
    pub name: String,
    /// Project path
    pub path: Option<String>,
    /// Project name including namespace
    pub name_with_namespace: Option<String>,
    /// Project path including namespace
    pub path_with_namespace: Option<String>,
    /// URL of the group page
    #[serde(rename = "web_url")]
    pub url: Option<String>,
}
/// Struct for GitLab tree entry
///
/// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreeEntry {
    /// Integer ID of GitLab project
    ///
    /// See <https://docs.gitlab.com/api/projects/#get-a-single-project> for more information
    pub id: String,
    /// Name of tree entry
    pub name: String,
    /// Type of tree entry
    #[serde(rename = "type")]
    pub entry_type: TreeEntryType,
    /// Path of tree entry
    ///
    /// The path inside the repository Used to get content of subdirectories
    pub path: String,
    /// Mode of tree entry
    pub mode: String,
}
/// User details
/// ### Example JSON response
/// ```json
/// {
///     "avatar_url": String("https://code.ornl.gov/uploads/-/system/user/avatar/4862/avatar.png"),
///     "id": Number(4862),
///     "locked": Bool(false),
///     "name": String("Wohlgemuth, Jason"),
///     "public_email": String("wohlgemuthjh@ornl.gov"),
///     "state": String("active"),
///     "username": String("o9w"),
///     "web_url": String("https://code.ornl.gov/o9w"),
/// }
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDetails {
    /// URL of user avatar image
    pub avatar_url: String,
    /// Numeric ID of user
    #[serde(rename = "id")]
    pub identifier: u64,
    /// Whether the user is locked
    pub locked: bool,
    /// User's full name
    pub name: String,
    /// User's public email address
    #[serde(rename = "public_email")]
    pub email: Option<String>,
    /// User state (for example, "active")
    pub state: String,
    /// Username/handle of the user
    pub username: String,
    /// URL of the user's profile page
    #[serde(rename = "web_url")]
    pub url: String,
}
impl Options {
    /// Build options from common GitLab CI environment variables.
    /// - `CI_JOB_TOKEN` -> `token` (defaults to empty string when unset)
    /// - `CI_PROJECT_ID` -> `identifier`
    /// - `CI_MERGE_REQUEST_IID` -> `internal_identifier`
    /// - `CI_SERVER_HOST` -> `domain` (defaults to gitlab.com when unset)
    ///
    /// See <https://docs.gitlab.com/ci/variables/predefined_variables> for more information on available GitLab CI environment variables
    pub fn from_env() -> Self {
        let _ = dotenvy::from_filename(".env");
        Self {
            token: var("CI_JOB_TOKEN").unwrap_or_default(),
            identifier: var("CI_PROJECT_ID").ok(),
            internal_identifier: var("CI_MERGE_REQUEST_IID").ok(),
            domain: var("CI_SERVER_HOST").unwrap_or_else(|_| "gitlab.com".to_string()),
            body: None,
        }
    }
    /// Return a copy of options with request body payload set
    pub fn with_body(self, body: impl Into<String>) -> Self {
        Self {
            body: Some(body.into()),
            ..self
        }
    }
    /// Return a copy of options with project or group identifier set
    pub fn with_identifier(self, identifier: impl Into<String>) -> Self {
        Self {
            identifier: Some(identifier.into()),
            ..self
        }
    }
    /// Return a copy of options with GitLab domain set
    pub fn with_domain(self, domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            ..self
        }
    }
}
impl From<ProgrammingLanguageMetadata> for ProgrammingLanguageRow {
    fn from(value: ProgrammingLanguageMetadata) -> Self {
        let ProgrammingLanguageMetadata {
            name,
            language_id,
            language_type,
            color,
            group,
        } = value;
        ProgrammingLanguageRow::init()
            .name(name)
            .maybe_language_id(language_id.and_then(|value| i64::try_from(value).ok()))
            .maybe_language_type(language_type)
            .maybe_color(color)
            .maybe_group_name(group)
            .build()
    }
}
impl ProgrammingLanguagesResponse {
    /// Parse a raw language map, retaining only `programming` type entries
    pub fn parse(data: HashMap<String, ProgrammingLanguageDetails>) -> Self {
        let languages = data
            .into_iter()
            .filter_map(|(name, details)| {
                details
                    .language_type
                    .as_ref()
                    .map(|kind| kind.eq_ignore_ascii_case("programming"))
                    .filter(|is_programming| *is_programming)
                    .map(|_| ProgrammingLanguageMetadata {
                        name,
                        language_id: details.language_id,
                        language_type: details.language_type,
                        color: details.color,
                        group: details.group,
                    })
            })
            .collect();
        Self { languages }
    }
}
impl<'de> serde::Deserialize<'de> for ProgrammingLanguagesResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        HashMap::<String, ProgrammingLanguageDetails>::deserialize(deserializer).map(Self::parse)
    }
}
#[async_trait]
impl DatabasePersistence for ProgrammingLanguagesResponse {
    /// Persist GitLab programming language metadata to local database
    async fn persist(self, database: Database<Table>) -> ApiResult<usize> {
        let Self { languages } = self;
        let message: fn(&ProgrammingLanguageMetadata) -> String = |item| format!("Saving \"{}\" language metadata", item.name);
        let operation = |item| async { database.insert(ProgrammingLanguageRow::from(item)) };
        let finish = |count| format!("{}Saved metadata for {count} programming languages", Label::CHECKMARK);
        with_progress(languages, message, operation, finish, None, ProgressType::Bar)
            .await
            .map(|counts| counts.into_iter().sum())
            .map_err(eyre::Report::msg)
    }
}
impl Default for Options {
    fn default() -> Self {
        Self::from_env()
    }
}
impl fmt::Display for PaginationField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            | PaginationField::OrderBy => "order_by",
            | PaginationField::Page => "page",
            | PaginationField::Pagination => "pagination",
            | PaginationField::PerPage => "per_page",
            | PaginationField::Sort => "sort",
        };
        write!(f, "{}", s)
    }
}
impl TryFrom<&str> for PaginationField {
    type Error = String;

    fn try_from(value: &str) -> eyre::Result<Self, Self::Error> {
        match value {
            | "order_by" => Ok(PaginationField::OrderBy),
            | "page" => Ok(PaginationField::Page),
            | "pagination" => Ok(PaginationField::Pagination),
            | "per_page" => Ok(PaginationField::PerPage),
            | "sort" => Ok(PaginationField::Sort),
            | _ => Err(format!("Invalid GitLab pagination field: {value}")),
        }
    }
}
impl TryFrom<&str> for OrderByValue {
    type Error = String;

    fn try_from(value: &str) -> eyre::Result<Self, Self::Error> {
        match value {
            | "created_at" => Ok(OrderByValue::CreatedAt),
            | "due_date" => Ok(OrderByValue::DueDate),
            | "full_name" => Ok(OrderByValue::FullName),
            | "id" => Ok(OrderByValue::Identifier),
            | "label_priority" => Ok(OrderByValue::LabelPriority),
            | "last_activity_at" => Ok(OrderByValue::LastActivityAt),
            | "milestone_due" => Ok(OrderByValue::MilestoneDue),
            | "name" => Ok(OrderByValue::Name),
            | "path" => Ok(OrderByValue::Path),
            | "popularity" => Ok(OrderByValue::Popularity),
            | "priority" => Ok(OrderByValue::Priority),
            | "relative_position" => Ok(OrderByValue::RelativePosition),
            | "similarity" => Ok(OrderByValue::Similarity),
            | "title" => Ok(OrderByValue::Title),
            | "updated_at" => Ok(OrderByValue::UpdatedAt),
            | "weight" => Ok(OrderByValue::Weight),
            | _ => Err(format!("Invalid GitLab order_by value: {value}")),
        }
    }
}
impl TryFrom<&str> for SortValue {
    type Error = String;

    fn try_from(value: &str) -> eyre::Result<Self, Self::Error> {
        match value {
            | "asc" => Ok(SortValue::Ascending),
            | "desc" => Ok(SortValue::Descending),
            | _ => Err(format!("Invalid GitLab sort order: {value}")),
        }
    }
}
impl TreeEntry {
    /// Get path of tree entry
    pub fn path(self) -> String {
        self.path
    }
    /// Whether tree entry is a blob
    pub fn is_blob(&self) -> bool {
        let Self { entry_type, .. } = self;
        entry_type.eq(&TreeEntryType::Blob)
    }
}
impl ValueValidator for PaginationField {
    /// Validate pagination field values according to GitLab API documentation
    fn is_valid(&self, value: &str) -> bool {
        match self {
            | PaginationField::OrderBy => OrderByValue::try_from(value).is_ok(),
            | PaginationField::Page => value.parse::<u64>().is_ok(),
            | PaginationField::PerPage => value.parse::<u64>().is_ok(),
            | PaginationField::Sort => SortValue::try_from(value).is_ok(),
            | _ => true,
        }
    }
}
/// Get descendant groups of a group by identifier
pub async fn groups(options: Options) -> ApiResult<GroupsResponse> {
    let action = "groups";
    let Options {
        token, identifier, domain, ..
    } = options;
    match Endpoint::from_template("gitlab").map(|e| e.with_domain(&domain)) {
        | Ok(endpoint) => {
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &token),
                param!(TemplateValue, "identifier", &identifier.unwrap_or_default()),
                param!(FieldList, "page", "1"),
                param!(KeyValuePair, "per_page", "100"),
            ];
            let response = endpoint.invoke_with::<PaginationField, EmptyField>(action, Some(params)).await;
            endpoint.handle::<GroupsResponse>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Download programming language metadata from GitLab linguist source file
pub async fn languages() -> ApiResult<ProgrammingLanguagesResponse> {
    let names = ["languages::gitlab", "languages::github"];
    let action = "languages";
    async fn fetch(name: &str, action: &str) -> ApiResult<ProgrammingLanguagesResponse> {
        match INCLUDED_ENDPOINTS.find_by_name(name) {
            | Some(endpoint) => {
                let response = endpoint.invoke(action, None).await.map(|content| match content {
                    | ResponseContent::Raw(content) => ResponseContent::Yaml(content),
                    | other => other,
                });
                endpoint.handle::<ProgrammingLanguagesResponse>(response)
            }
            | None => Err(eyre!("{name} API endpoint not found")),
        }
    }
    let mut errors = Vec::new();
    for name in names {
        match fetch(name, action).await {
            | Ok(response) => return Ok(response),
            | Err(why) => errors.push(format!("{name}={why}")),
        }
    }
    Err(eyre!("Failed to download and parse language metadata — {}", errors.join("; ")))
}
/// Add a note (comment) to a merge request using project identifier and merge request IID
///
/// When used in CI environment, the project identifier can be obtained from the `CI_PROJECT_ID` environment variable and the merge request IID can be obtained from the `CI_MERGE_REQUEST_IID` environment variable.
/// The GitLab token must be provided in the `CI_JOB_TOKEN` environment variable.
///
/// See <https://docs.gitlab.com/api/notes/#create-a-merge-request-note> for more information on this API endpoint and required parameters
pub async fn merge_request_note(options: Options) -> ApiResult<MergeRequestNoteResponse> {
    let action = "merge-request-note";
    let Options {
        token,
        identifier,
        internal_identifier,
        body,
        domain,
    } = options;
    match Endpoint::from_template("gitlab").map(|e| e.with_domain(&domain)) {
        | Ok(endpoint) => {
            let body = body.unwrap_or_default();
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &token),
                param!(Body, "body", &body),
                param!(TemplateValue, "identifier", &identifier.unwrap_or_default()),
                param!(TemplateValue, "internal_identifier", &internal_identifier.unwrap_or_default()),
            ];
            let response = endpoint.invoke_with::<PaginationField, EmptyField>(action, Some(params)).await;
            endpoint.handle::<MergeRequestNoteResponse>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Get runner details by identifier
pub async fn runner(options: Options) -> ApiResult<RunnerDetails> {
    let action = "runners";
    let Options {
        token, identifier, domain, ..
    } = options;
    match Endpoint::from_template("gitlab").map(|e| e.with_domain(&domain)) {
        | Ok(endpoint) => {
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &token),
                param!(TemplateValue, "identifier", &identifier.unwrap_or_default()),
            ];
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle::<RunnerDetails>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Get runners visible to user associated with given token
pub async fn runners(options: Options) -> ApiResult<RunnersResponse> {
    let action = "runners";
    let Options { token, domain, .. } = options;
    match Endpoint::from_template("gitlab").map(|e| e.with_domain(&domain)) {
        | Ok(endpoint) => {
            let params = vec![param!(Header, "PRIVATE-TOKEN", &token)];
            let response = endpoint.invoke_with::<PaginationField, EmptyField>(action, Some(params)).await;
            endpoint.handle::<RunnersResponse>(response)
        }
        | Err(why) => Err(why),
    }
}

/// Fetch one page of repository tree blob paths for a GitLab project
pub(crate) async fn tree_paths(
    host: impl Into<String>,
    identifier: impl Into<String>,
    directory: impl Into<String>,
    page: u32,
) -> ApiResult<Vec<String>> {
    match Endpoint::from_template("gitlab").map(|e| e.with_domain(host.into())) {
        | Ok(endpoint) => {
            let params = vec![
                param!(TemplateValue, "identifier", &identifier.into()),
                param!(KeyValuePair, "per_page", "100"),
                param!(KeyValuePair, "page", &page.to_string()),
                param!(KeyValuePair, "recursive", "true"),
                param!(KeyValuePair, "path", &directory.into()),
            ];
            let response = endpoint.invoke("tree", Some(params)).await;
            endpoint
                .handle::<TreeResponse>(response)
                .map(|entries| entries.into_iter().filter(|e| e.is_blob()).map(|e| e.path()).collect())
        }
        | Err(why) => Err(why),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::api::IntoBody;
    use core::iter::FromIterator;

    #[test]
    fn test_add_comment_payload_uses_body_field() {
        let payload = vec![param!(Body, "body", "This is a test comment!!!")].into_body();
        assert_eq!(payload, serde_json::json!({ "body": "This is a test comment!!!" }));
    }
    #[test]
    fn test_programming_languages_response_parse_filters_programming_only() {
        let data = HashMap::from_iter([
            (
                "Python".to_string(),
                ProgrammingLanguageDetails {
                    language_id: Some(303),
                    language_type: Some("programming".to_string()),
                    color: Some("#3572A5".to_string()),
                    group: None,
                },
            ),
            (
                "YAML".to_string(),
                ProgrammingLanguageDetails {
                    language_id: Some(407),
                    language_type: Some("data".to_string()),
                    color: Some("#cb171e".to_string()),
                    group: None,
                },
            ),
        ]);
        let response = ProgrammingLanguagesResponse::parse(data);
        assert_eq!(response.languages.len(), 1);
        assert_eq!(response.languages[0].name, "Python");
        assert_eq!(response.languages[0].language_id, Some(303));
    }
}
