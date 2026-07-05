//! OpenCode configuration data model
//!
//! See <https://opencode.ai/docs/config> for more information on OpenCode configuration.
use super::{Provider, ProviderDetails};
use alloc::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Permission rules keyed by tool or command pattern
pub type PermissionObjectConfig = StringMap<PermissionRuleConfig>;
/// String-keyed map used throughout OpenCode config sections
pub type StringMap<T> = BTreeMap<String, T>;
/// Agent availability mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Available only as a subagent
    Subagent,
    /// Available only as a primary agent
    Primary,
    /// Available as both a primary agent and subagent
    All,
}
/// Automatic update behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AutoUpdateConfig {
    /// Enable or disable automatic updates
    Bool(bool),
    /// Show update notifications without installing updates
    Notify(Notify),
}
/// Value that can be a simple boolean or named configuration map
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoolOrMap<T> {
    /// Enables or disables built-in behavior
    Bool(bool),
    /// Enables built-ins with named overrides or custom entries
    Map(StringMap<T>),
}
/// Layout setting (deprecated; no longer configurable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutConfig {
    /// Automatic layout selection
    Auto,
    /// Stretch layout
    Stretch,
}
/// Runtime log level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    /// Debug-level logging
    DEBUG,
    /// Informational logging
    INFO,
    /// Warning-level logging
    WARN,
    /// Error-level logging
    ERROR,
}
/// Language server configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LspConfig {
    /// Disables a named language server
    Disabled {
        /// Whether the language server is disabled
        disabled: bool,
    },
    /// Configures a language server process
    Server(LspServerConfig),
}
/// Model Context Protocol server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpConfig {
    /// Local MCP server launched by OpenCode
    #[serde(rename = "local")]
    Local(McpLocalConfig),
    /// Remote MCP server reached over HTTP
    #[serde(rename = "remote")]
    Remote(McpRemoteConfig),
    /// Minimal entry that only toggles an inherited server
    #[serde(untagged)]
    EnabledOnly {
        /// Whether the MCP server is enabled on startup
        enabled: bool,
    },
}
/// OAuth setting for a remote MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpOAuthOrFalse {
    /// OAuth client configuration
    Config(McpOAuthConfig),
    /// Disables OAuth auto-detection when set to false
    Disabled(bool),
}
/// Update notification mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Notify {
    /// Notify when an update is available
    Notify,
}
/// Permission action for a tool or command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    /// Ask for approval before running
    Ask,
    /// Allow without approval
    Allow,
    /// Deny the operation
    Deny,
}
/// Global permission configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermissionConfig {
    /// Single action applied broadly
    Action(PermissionAction),
    /// Tool-specific permission rules
    Object(PermissionObjectConfig),
}
/// Permission rule for a tool or nested command pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermissionRuleConfig {
    /// Single action for this rule
    Action(PermissionAction),
    /// Nested command-pattern actions for this rule
    Object(StringMap<PermissionAction>),
}
/// Plugin reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginConfig {
    /// Package or plugin name
    Name(String),
    /// Package or plugin name with options
    WithOptions(String, Value),
}
/// Experimental policy action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    /// Controls whether a provider may be used
    #[serde(rename = "provider.use")]
    ProviderUse,
}
/// Experimental policy effect
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyEffect {
    /// Allows the action
    Allow,
    /// Denies the action
    Deny,
}
/// Named reference source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReferenceConfig {
    /// Reference expressed as a string
    String(String),
    /// Reference to a git repository
    Git(ReferenceGit),
    /// Reference to a local path
    Local(ReferenceLocal),
}
/// Conversation sharing behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShareConfig {
    /// Share only when explicitly requested
    Manual,
    /// Automatically share new conversations
    Auto,
    /// Disable sharing
    Disabled,
}
/// Specialized agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    /// Model to use for this agent
    pub model: Option<String>,
    /// Default variant for the configured model
    pub variant: Option<String>,
    /// Sampling temperature for this agent
    pub temperature: Option<f64>,
    /// Top-p sampling value for this agent
    pub top_p: Option<f64>,
    /// System prompt or instructions for this agent
    pub prompt: Option<String>,
    /// Per-tool enablement map
    #[deprecated(note = "Use `tools` instead")]
    pub tools: Option<StringMap<bool>>,
    /// Whether this agent is disabled
    pub disable: Option<bool>,
    /// Description of when to use this agent
    pub description: Option<String>,
    /// Whether this agent is primary, subagent, or both
    pub mode: Option<AgentMode>,
    /// Whether to hide this subagent from autocomplete
    pub hidden: Option<bool>,
    /// Provider-specific or agent-specific options
    pub options: Option<Value>,
    /// Hex color or theme color for this agent
    pub color: Option<String>,
    /// Maximum agentic iterations before forcing a text-only response
    pub steps: Option<u64>,
    /// Maximum step count
    #[deprecated(note = "Use `steps` instead")]
    #[serde(rename = "maxSteps")]
    pub max_steps: Option<u64>,
    /// Permission rules that apply to this agent
    pub permission: Option<PermissionConfig>,
}
/// Attachment processing configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AttachmentConfig {
    /// Image attachment limits and resizing behavior
    pub image: Option<ImageAttachmentConfig>,
}
/// Server and runtime OpenCode configuration
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Named agent configurations
    pub agent: Option<StringMap<AgentConfig>>,
    /// Attachment processing configuration
    pub attachment: Option<AttachmentConfig>,
    /// Automatic sharing flag
    #[deprecated(note = "Use `share` instead")]
    pub autoshare: Option<bool>,
    /// Automatic update behavior
    pub autoupdate: Option<AutoUpdateConfig>,
    /// Custom command configuration
    pub command: Option<StringMap<CommandConfig>>,
    /// Context compaction behavior
    pub compaction: Option<CompactionConfig>,
    /// Default primary agent used when none is specified
    pub default_agent: Option<String>,
    /// Provider IDs that should not be loaded
    pub disabled_providers: Option<Vec<String>>,
    /// Provider IDs that are allowed when an allowlist is desired
    pub enabled_providers: Option<Vec<String>>,
    /// Enterprise configuration
    pub enterprise: Option<EnterpriseConfig>,
    /// Experimental configuration
    pub experimental: Option<ExperimentalConfig>,
    /// Formatter enablement or formatter overrides
    pub formatter: Option<BoolOrMap<FormatterConfig>>,
    /// Instruction files or glob patterns to include
    pub instructions: Option<Vec<String>>,
    /// Layout option
    #[deprecated(note = "Remove this field; layout is no longer configurable")]
    pub layout: Option<LayoutConfig>,
    /// Runtime log level
    #[serde(rename = "logLevel")]
    pub log_level: Option<LogLevel>,
    /// LSP enablement or LSP server overrides
    pub lsp: Option<BoolOrMap<LspConfig>>,
    /// MCP server configurations
    pub mcp: Option<StringMap<McpConfig>>,
    /// Agent configuration map
    #[deprecated(note = "Use `agent` instead")]
    pub mode: Option<StringMap<AgentConfig>>,
    /// Main model in `provider/model` format
    pub model: Option<String>,
    /// Permission rules for tools and operations
    pub permission: Option<PermissionConfig>,
    /// Plugins loaded from packages or configured with options
    pub plugin: Option<Vec<PluginConfig>>,
    /// Custom provider configuration and model overrides
    pub provider: Option<StringMap<ProviderDetails>>,
    /// Named references field
    #[deprecated(note = "Use `references` instead")]
    pub reference: Option<StringMap<ReferenceConfig>>,
    /// Named git or local directory references
    pub references: Option<StringMap<ReferenceConfig>>,
    /// JSON schema reference for config validation
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    /// Server settings for `opencode serve` and `opencode web`
    pub server: Option<ServerConfig>,
    /// Conversation sharing behavior
    pub share: Option<ShareConfig>,
    /// Default shell used for terminal and agent tool calls
    pub shell: Option<String>,
    /// Additional skill paths or URLs
    pub skills: Option<SkillsConfig>,
    /// Small model for lightweight tasks such as title generation
    pub small_model: Option<String>,
    /// Whether filesystem snapshots are recorded for undo and revert
    pub snapshot: Option<bool>,
    /// Tool output truncation thresholds
    pub tool_output: Option<ToolOutputConfig>,
    /// Tool enablement map
    pub tools: Option<StringMap<bool>>,
    /// Username displayed in conversations
    pub username: Option<String>,
    /// File watcher settings
    pub watcher: Option<WatcherConfig>,
}
/// Custom command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandConfig {
    /// Prompt template for the command
    pub template: String,
    /// Human-readable command description
    pub description: Option<String>,
    /// Agent to run the command with
    pub agent: Option<String>,
    /// Model to run the command with
    pub model: Option<String>,
    /// Model variant to run the command with
    pub variant: Option<String>,
    /// Whether the command should run as a subtask
    pub subtask: Option<bool>,
}
/// Context compaction behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CompactionConfig {
    /// Whether to compact automatically when context is full
    pub auto: Option<bool>,
    /// Whether to prune old tool outputs to save tokens
    pub prune: Option<bool>,
    /// Number of recent user turns to keep verbatim
    pub tail_turns: Option<u64>,
    /// Maximum recent-turn tokens to preserve verbatim
    pub preserve_recent_tokens: Option<u64>,
    /// Token buffer reserved for compaction
    pub reserved: Option<u64>,
}
/// Enterprise configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EnterpriseConfig {
    /// Enterprise service URL
    pub url: Option<String>,
}
/// Experimental settings that may change without notice
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ExperimentalConfig {
    /// Whether paste summaries are disabled
    pub disable_paste_summary: Option<bool>,
    /// Whether the batch tool is enabled
    pub batch_tool: Option<bool>,
    /// Whether OpenTelemetry spans are enabled for AI SDK calls
    #[serde(rename = "openTelemetry")]
    pub open_telemetry: Option<bool>,
    /// Tools that should only be available to primary agents
    pub primary_tools: Option<Vec<String>>,
    /// Whether the agent loop continues after a tool call is denied
    pub continue_loop_on_deny: Option<bool>,
    /// Timeout in milliseconds for MCP requests
    pub mcp_timeout: Option<u64>,
    /// Policy statements for supported resources such as providers
    pub policies: Option<Vec<ExperimentalPolicy>>,
}
/// Experimental policy statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentalPolicy {
    /// Action controlled by the policy
    pub action: PolicyAction,
    /// Whether the action is allowed or denied
    pub effect: PolicyEffect,
    /// Resource affected by the policy
    pub resource: Provider,
}
/// Formatter command configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FormatterConfig {
    /// Whether this formatter is disabled
    pub disabled: Option<bool>,
    /// Command and arguments used to run the formatter
    pub command: Option<Vec<String>>,
    /// Environment variables for the formatter process
    pub environment: Option<StringMap<String>>,
    /// File extensions handled by this formatter
    pub extensions: Option<Vec<String>>,
}
/// Image attachment limits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ImageAttachmentConfig {
    /// Whether oversized images are resized before provider requests
    pub auto_resize: Option<bool>,
    /// Maximum image width before resizing or rejection
    pub max_width: Option<u64>,
    /// Maximum image height before resizing or rejection
    pub max_height: Option<u64>,
    /// Maximum base64 payload size before resizing or rejection
    pub max_base64_bytes: Option<u64>,
}
/// Language server process configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspServerConfig {
    /// Command and arguments used to run the language server
    pub command: Vec<String>,
    /// File extensions handled by the language server
    pub extensions: Option<Vec<String>>,
    /// Whether this language server is disabled
    pub disabled: Option<bool>,
    /// Environment variables for the language server process
    pub env: Option<StringMap<String>>,
    /// Initialization options sent to the language server
    pub initialization: Option<Value>,
}
/// Local MCP server process configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpLocalConfig {
    /// Command and arguments used to run the MCP server
    pub command: Vec<String>,
    /// Working directory for the MCP server process
    pub cwd: Option<String>,
    /// Environment variables for the MCP server process
    pub environment: Option<StringMap<String>>,
    /// Whether the MCP server is enabled on startup
    pub enabled: Option<bool>,
    /// Request timeout in milliseconds
    pub timeout: Option<u64>,
}
/// OAuth configuration for a remote MCP server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct McpOAuthConfig {
    /// OAuth client ID
    #[serde(rename = "clientId")]
    pub client_id: Option<String>,
    /// OAuth client secret
    #[serde(rename = "clientSecret")]
    pub client_secret: Option<String>,
    /// OAuth scopes requested during authorization
    pub scope: Option<String>,
    /// Local OAuth callback server port
    #[serde(rename = "callbackPort")]
    pub callback_port: Option<u16>,
    /// OAuth redirect URI
    #[serde(rename = "redirectUri")]
    pub redirect_uri: Option<String>,
}
/// Remote MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpRemoteConfig {
    /// URL of the remote MCP server
    pub url: String,
    /// Whether the MCP server is enabled on startup
    pub enabled: Option<bool>,
    /// Headers sent with remote MCP requests
    pub headers: Option<StringMap<String>>,
    /// OAuth configuration or disabled flag
    pub oauth: Option<McpOAuthOrFalse>,
    /// Request timeout in milliseconds
    pub timeout: Option<u64>,
}
/// Git repository reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceGit {
    /// Repository URL or identifier
    pub repository: String,
    /// Branch to use from the repository
    pub branch: Option<String>,
    /// Description of this reference
    pub description: Option<String>,
    /// Whether this reference is hidden from selection surfaces
    pub hidden: Option<bool>,
}
/// Local directory reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceLocal {
    /// Local path to the reference
    pub path: String,
    /// Description of this reference
    pub description: Option<String>,
    /// Whether this reference is hidden from selection surfaces
    pub hidden: Option<bool>,
}
/// HTTP server settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    /// Port to listen on
    pub port: Option<u64>,
    /// Hostname to listen on
    pub hostname: Option<String>,
    /// Whether mDNS service discovery is enabled
    pub mdns: Option<bool>,
    /// Custom mDNS domain name
    #[serde(rename = "mdnsDomain")]
    pub mdns_domain: Option<String>,
    /// Additional CORS origins allowed for browser clients
    pub cors: Option<Vec<String>>,
}
/// Additional skill sources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SkillsConfig {
    /// Additional paths to skill folders
    pub paths: Option<Vec<String>>,
    /// URLs used to fetch skills
    pub urls: Option<Vec<String>>,
}
/// Tool output truncation thresholds
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ToolOutputConfig {
    /// Maximum preview lines before full output is saved to disk
    pub max_lines: Option<u64>,
    /// Maximum preview bytes before full output is saved to disk
    pub max_bytes: Option<u64>,
}
/// File watcher configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WatcherConfig {
    /// Glob patterns ignored by the file watcher
    pub ignore: Option<Vec<String>>,
}
