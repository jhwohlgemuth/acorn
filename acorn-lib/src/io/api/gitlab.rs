//! Module for interacting with GitLab API
//!
// TODO: Add custom field list and query pair type
// TODO: Finish modeling necessary API endpoints
use crate::io::api::{RemoteResource, Searchable, TreeEntryType, INCLUDED_ENDPOINTS};
use crate::io::config::{RunnerStatus, RunnerType};
use crate::param;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Type for GitLab API descendent groups response
pub type GroupsResponse = Vec<GroupDetails>;
/// Type for GitLab API all runners response
pub type RunnersResponse = Vec<RunnerDetails>;
/// Type for GitLab API response for a tree
pub type TreeResponse = Vec<TreeEntry>;
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupVisibility {
    /// Public visibility
    Public,
    /// Internal visibility
    Internal,
    /// Private visibility
    Private,
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
    pub ldap_cn: Option<String>,
    /// LDAP access value
    pub ldap_access: Option<String>,
    /// File template project identifier
    pub file_template_project_id: Option<u64>,
    /// Wiki access level
    pub wiki_access_level: Option<String>,
    /// Duo core features enabled flag
    pub duo_core_features_enabled: Option<bool>,
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
/// Get descendant groups of a group by identifier
pub async fn groups(identifier: impl Into<String>, token: impl Into<String>) -> Result<GroupsResponse, String> {
    let name = "GitLab";
    let action = "groups";
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &token.into()),
                param!(TemplateValue, "identifier", &identifier.into()),
                // param!(FieldList, "page", "1"),
                param!(FieldList, "per_page", "100"),
            ];
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle::<GroupsResponse>(response)
        }
        | None => Err(format!("{name} API endpoint not found")),
    }
}
/// Get runner details by identifier
pub async fn runner(identifier: impl Into<String>, token: impl Into<String>) -> Result<RunnerDetails, String> {
    let name = "GitLab";
    let action = "runners";
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let params = vec![
                param!(Header, "PRIVATE-TOKEN", &token.into()),
                param!(TemplateValue, "identifier", &identifier.into()),
            ];
            let response = endpoint.invoke(action, Some(params)).await;
            dbg!(&response);
            endpoint.handle::<RunnerDetails>(response)
        }
        | None => Err(format!("{name} API endpoint not found")),
    }
}
/// Get runners visible to user associated with given token
pub async fn runners(token: impl Into<String>) -> Result<RunnersResponse, String> {
    let name = "GitLab";
    let action = "runners";
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let params = vec![param!(Header, "PRIVATE-TOKEN", &token.into())];
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle::<RunnersResponse>(response)
        }
        | None => Err(format!("{name} API endpoint not found")),
    }
}
