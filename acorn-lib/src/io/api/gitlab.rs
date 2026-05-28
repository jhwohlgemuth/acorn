//! Module for interacting with GitLab API
//!
// TODO: Add custom field list and query pair type
// TODO: Finish modeling necessary API endpoints
use crate::io::api::{
    Configuration, DatabasePersistence, EmptyField, Endpoint, Fallback, RemoteResource, ResponseContent, TreeEntryType, ValueValidator,
    INCLUDED_ENDPOINTS,
};
use crate::io::config::{RunnerDetails, RunnerStatus, RunnerType};
use crate::io::database::schema::{ProgrammingLanguageRow, Table};
use crate::io::database::{Database, Operations};
use crate::io::{first_env_var, with_progress, ApiResult, ProgressType};
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
use tracing::debug;

/// Trait for adding creation and registration functionality (e.g., runners, issues, merge requests, etc.)
pub trait Create {
    /// Create a new instance of the struct with default values
    fn create(_options: &Options) -> ApiResult<Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
    /// Register a new instance of the struct with specified values
    fn register(self) -> ApiResult<Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
/// Type for GitLab API descendent groups response
pub type GroupsResponse = Vec<GroupDetails>;
/// Type for GitLab API all runners response
pub type RunnersResponse = Vec<RunnerMetadata>;
/// Type for GitLab API response for a tree
pub type TreeResponse = Vec<TreeEntry>;
/// Type for GitLab programming language entries
pub type ProgrammingLanguageEntries = Vec<ProgrammingLanguageMetadata>;
/// Type for GitLab project programming language usage entries
pub type ProgrammingLanguageUseEntries = Vec<ProgrammingLanguageUseMetadata>;
/// Access level of the runner
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    /// Not protected
    NotProtected,
    /// Ref protected
    RefProtected,
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
/// GitLab API error response
///
/// Captures error responses from the GitLab API, which can be a message string,
/// a message object with field-level errors, or an error/error_description pair.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message string or object with field-level errors
    message: Option<serde_json::Value>,
    /// Simple error string (OAuth-style)
    error: Option<String>,
    /// Detailed error description (OAuth-style)
    error_description: Option<String>,
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
    pub author: UserMetadata,
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
    /// Request body payload
    pub body: Option<String>,
    /// GitLab domain (defaults to gitlab.com)
    #[builder(default = String::from("gitlab.com"))]
    pub domain: String,
    /// Project or group identifier
    pub identifier: Option<String>,
    /// Internal resource identifier (e.g., merge request IID)
    pub internal_identifier: Option<String>,
    /// GitLab runner metadata necessary for creation
    #[builder(default = RunnerMetadata::default())]
    pub runner_metadata: RunnerMetadata,
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
/// Programming language usage entry for a project
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProgrammingLanguageUseMetadata {
    /// Display name of the language
    pub name: String,
    /// Relative share of repository content for this language
    pub percentage: f64,
}
/// Parsed response for GitLab project language usage
#[derive(Clone, Debug, Default, Serialize)]
pub struct ProgrammingLanguageUseResponse {
    /// Flattened language usage entries
    pub languages: ProgrammingLanguageUseEntries,
}
/// GitLab API response for creating a runner
/// ### Example JSON response
/// ```json
/// {
///     "id": 9171,
///     "token": "<access-token>",
///     "token_expires_at": null
/// }
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunnerCreationResponse {
    /// Numeric ID of the runner
    #[serde(default)]
    #[serde(rename = "id")]
    pub identifier: u64,
    /// Runner access token
    pub token: Option<String>,
    /// Runner access token expiration timestamp
    pub token_expires_at: Option<String>,
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
#[derive(Clone, Debug, Builder, Serialize, Deserialize)]
#[builder(start_fn = init, on(String, into), on(&str, into))]
pub struct RunnerMetadata {
    /// Numeric ID of the runner
    #[serde(rename = "id")]
    pub identifier: Option<u64>,
    /// Whether the runner is active
    #[builder(default)]
    #[serde(default)]
    pub active: bool,
    /// Whether the runner is online
    ///
    /// Apparently, GitLab's API may return `null` for this field when the runner has never been contacted, so we use an `Option<bool>` to capture that possibility.
    #[serde(default)]
    pub online: Option<bool>,
    /// Whether the runner is paused
    #[builder(default)]
    #[serde(default)]
    pub paused: bool,
    /// Whether the runner runs untagged jobs
    #[builder(default)]
    #[serde(default)]
    pub run_untagged: bool,
    /// Whether the runner is shared
    #[builder(default)]
    #[serde(default, rename = "is_shared")]
    pub shared: bool,
    /// CPU architecture reported by the runner
    pub architecture: Option<String>,
    /// Runner description
    pub description: Option<String>,
    /// Runner IP address
    pub ip_address: Option<String>,
    /// Type of runner (for example, `project_type`)
    #[builder(with = |value: &str| RunnerType::from(value))]
    #[builder(default = RunnerType::Project)]
    pub runner_type: RunnerType,
    /// Created by user
    pub created_by: Option<UserMetadata>,
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
    #[builder(with = |values: &[&str]| values.iter().map(|s| s.to_string()).collect::<Vec<String>>())]
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
pub struct UserMetadata {
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
impl Default for RunnerMetadata {
    fn default() -> Self {
        Self::init().build()
    }
}
impl From<RunnerDetails> for RunnerMetadata {
    fn from(value: RunnerDetails) -> Self {
        let RunnerDetails {
            name,
            runner_type,
            description,
            tags,
            ..
        } = value;
        Self {
            access_level: None,
            active: false,
            architecture: None,
            contacted_at: None,
            created_at: None,
            created_by: None,
            description,
            groups: None,
            identifier: None,
            ip_address: None,
            job_execution_status: None,
            paused: false,
            maintenance_note: None,
            maximum_timeout: None,
            name: Some(name),
            online: Some(false),
            platform: None,
            projects: None,
            revision: None,
            shared: matches!(runner_type, RunnerType::Instance),
            runner_type,
            run_untagged: false,
            status: None,
            tags,
            version: None,
        }
    }
}
impl RunnerMetadata {
    /// Whether the runner is active and online
    pub fn is_available(&self) -> bool {
        let Self { active, online, paused, .. } = self;
        *active && online.unwrap_or(false) && !*paused
    }
    /// Return a copy of runner metadata with project or group identifier set
    pub fn with_identifier(self, value: u64) -> Self {
        Self {
            identifier: Some(value),
            ..self
        }
    }
}
impl Configuration for Options {
    /// Build options from common GitLab CI environment variables.
    /// - `CI_JOB_TOKEN` -> `token` (defaults to empty string when unset)
    /// - `CI_PROJECT_ID` -> `identifier`
    /// - `CI_MERGE_REQUEST_IID` -> `internal_identifier`
    /// - `CI_SERVER_HOST` -> `domain` (defaults to gitlab.com when unset)
    ///
    /// See <https://docs.gitlab.com/ci/variables/predefined_variables> for more information on available GitLab CI environment variables
    fn from_env() -> Self {
        if let Err(why) = dotenvy::from_filename(".env") {
            debug!("=> {} Load .env — {why}", Label::skip());
        }
        Self {
            token: first_env_var(&["CI_JOB_TOKEN", "GITLAB_TOKEN"]).unwrap_or_default(),
            identifier: var("CI_PROJECT_ID").ok(),
            internal_identifier: var("CI_MERGE_REQUEST_IID").ok(),
            domain: var("CI_SERVER_HOST").unwrap_or_else(|_| "gitlab.com".to_string()),
            body: None,
            runner_metadata: RunnerMetadata::default(),
        }
    }
    /// Return a copy of options with request body payload set
    fn with_body(self, value: impl Into<String>) -> Self {
        Self {
            body: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with GitLab domain set
    fn with_domain(self, value: impl Into<String>) -> Self {
        Self {
            domain: value.into(),
            ..self
        }
    }
}
impl Options {
    /// Return a copy of options with project or group identifier set
    pub fn with_identifier(self, value: impl Into<String>) -> Self {
        Self {
            identifier: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with runner metadata set
    pub fn with_runner(self, metadata: RunnerMetadata) -> Self {
        Self {
            runner_metadata: metadata,
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
impl ProgrammingLanguageUseResponse {
    /// Parse a raw language-to-percentage map into normalized entries
    pub fn parse(data: HashMap<String, f64>) -> Self {
        let mut languages = data
            .into_iter()
            .map(|(name, percentage)| ProgrammingLanguageUseMetadata { name, percentage })
            .collect::<ProgrammingLanguageUseEntries>();
        languages.sort_by(|a, b| a.name.cmp(&b.name));
        Self { languages }
    }
    /// Get language data entry tuples, (name, percentage), sorted by percentage in descending order
    pub fn entries(&self) -> Vec<(String, f64)> {
        let mut entries = self
            .languages
            .iter()
            .map(|ProgrammingLanguageUseMetadata { name, percentage }| (name.clone(), *percentage))
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        entries
    }
    /// Get language names, sorted by percentage in descending order
    pub fn names(&self) -> Vec<String> {
        self.entries().into_iter().map(|(name, _)| name).collect()
    }
}
impl<'de> serde::Deserialize<'de> for ProgrammingLanguagesResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        HashMap::<String, ProgrammingLanguageDetails>::deserialize(deserializer).map(Self::parse)
    }
}
impl<'de> serde::Deserialize<'de> for ProgrammingLanguageUseResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        HashMap::<String, f64>::deserialize(deserializer).map(Self::parse)
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
/// Create a new runner using project or group identifier
///
/// See <https://docs.gitlab.com/api/users/#create-a-runner-linked-to-a-user> for more information on the GitLab API
pub async fn create_runner(options: &Options) -> ApiResult<RunnerCreationResponse> {
    #[derive(Deserialize)]
    struct StrictRunnerCreationResponse {
        #[serde(rename = "id")]
        identifier: u64,
        token: Option<String>,
        token_expires_at: Option<String>,
    }
    let template = "gitlab::api";
    let action = "runner::create";
    let runner_metadata = &options.runner_metadata;
    let runner_type = &runner_metadata.runner_type;
    let description = runner_metadata.description.as_deref().unwrap_or_default();
    let tags = runner_metadata.tags.as_deref().unwrap_or_default();
    let run_untagged = runner_metadata.run_untagged;
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain.as_str())) {
        | Ok(endpoint) => {
            let mut params = vec![
                param!(Header, "PRIVATE-TOKEN", &options.token),
                param!(Body, "description", &description),
                param!(Body, &format!("{runner_type}_id"), &options.identifier.as_deref().unwrap_or_default()),
                param!(Body, "runner_type", &format!("{runner_type}_type")),
                param!(Body, "run_untagged", &run_untagged.to_string()),
            ];
            if !tags.is_empty() {
                params.push(param!(Body, "tag_list", &tags.join(",")));
            }
            let response = endpoint.invoke(action, Some(params)).await;
            match response {
                | Ok(ResponseContent::Json(content)) => match serde_json::from_str::<StrictRunnerCreationResponse>(&content) {
                    | Ok(parsed) => Ok(RunnerCreationResponse {
                        identifier: parsed.identifier,
                        token: parsed.token,
                        token_expires_at: parsed.token_expires_at,
                    }),
                    | Err(_) => {
                        let rendered = serde_json::from_str::<serde_json::Value>(&content)
                            .ok()
                            .and_then(|value| serde_json::to_string_pretty(&value).ok())
                            .unwrap_or(content);
                        Err(eyre!("{rendered}"))
                    }
                },
                | Ok(other) => endpoint.handle::<RunnerCreationResponse>(Ok(other)),
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
/// Get descendant groups of a group by identifier
pub async fn groups(options: &Options) -> ApiResult<GroupsResponse> {
    let template = "gitlab::api";
    let action = "groups";
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain.as_str())) {
        | Ok(endpoint) => {
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &options.token),
                param!(TemplateValue, "identifier", &options.identifier.as_deref().unwrap_or_default()),
                param!(FieldList, "page", "1"),
                param!(KeyValuePair, "per_page", "100"),
            ];
            let response = endpoint.invoke_with::<PaginationField, EmptyField>(action, Some(params)).await;
            endpoint.handle_or::<GroupsResponse, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Get programming languages used by a GitLab project
///
/// See <https://docs.gitlab.com/api/projects/?utm_source=perplexity#retrieve-programming-language-usage-information> for more information on GitLab API.
pub async fn language_use(options: &Options) -> ApiResult<ProgrammingLanguageUseResponse> {
    let template = "gitlab::api";
    let action = "languages";
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain.as_str())) {
        | Ok(endpoint) => {
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &options.token),
                param!(TemplateValue, "identifier", &options.identifier.as_deref().unwrap_or_default()),
            ];
            let response = endpoint.invoke_with::<PaginationField, EmptyField>(action, Some(params)).await;
            endpoint.handle::<ProgrammingLanguageUseResponse>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Download programming language metadata from GitLab linguist source file
pub async fn languages() -> ApiResult<ProgrammingLanguagesResponse> {
    let names = ["gitlab::org", "github::org"];
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
pub async fn merge_request_note(options: &Options) -> ApiResult<MergeRequestNoteResponse> {
    let template = "gitlab::api";
    let action = "merge-request-note";
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain.as_str())) {
        | Ok(endpoint) => {
            let body = options.body.as_deref().unwrap_or_default();
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &options.token),
                param!(Body, "body", &body),
                param!(TemplateValue, "identifier", &options.identifier.as_deref().unwrap_or_default()),
                param!(
                    TemplateValue,
                    "internal_identifier",
                    &options.internal_identifier.as_deref().unwrap_or_default()
                ),
            ];
            let response = endpoint.invoke_with::<PaginationField, EmptyField>(action, Some(params)).await;
            endpoint.handle::<MergeRequestNoteResponse>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Get runner details by identifier
pub async fn runner(options: &Options) -> ApiResult<RunnerMetadata> {
    let template = "gitlab::api";
    let action = "runner";
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain.as_str())) {
        | Ok(endpoint) => {
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &options.token),
                param!(TemplateValue, "identifier", &options.identifier.as_deref().unwrap_or_default()),
            ];
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle::<RunnerMetadata>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Get runners visible to user associated with given token
pub async fn runners(options: &Options) -> ApiResult<RunnersResponse> {
    let template = "gitlab::api";
    let action = "runners";
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain.as_str())) {
        | Ok(endpoint) => {
            let params = vec![param!(Header, "PRIVATE-TOKEN", &options.token)];
            let response = endpoint.invoke_with::<PaginationField, EmptyField>(action, Some(params)).await;
            endpoint.handle_or::<RunnersResponse, Fallback<ErrorResponse>>(response)
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
    let template = "gitlab::api";
    match Endpoint::from_template(template).map(|e| e.with_domain(host.into())) {
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
                .handle_or::<TreeResponse, Fallback<ErrorResponse>>(response)
                .map(|entries| entries.into_iter().filter(|e| e.is_blob()).map(|e| e.path()).collect())
        }
        | Err(why) => Err(why),
    }
}
