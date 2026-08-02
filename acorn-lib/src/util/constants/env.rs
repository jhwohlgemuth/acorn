/// Environment variable name for cache TTL override.
pub const CACHE_TTL: &str = "ACORN_CACHE_TTL";
/// Environment variable used to select a Chrome or Chromium executable for PDF export.
pub const CHROME_PATH: &str = "CHROME_PATH";
/// Environment variable used to select a database backend at runtime.
pub const DATABASE_BACKEND: &str = "ACORN_DATABASE_BACKEND";
/// Environment variable used to override database path at runtime.
pub const DATABASE_PATH: &str = "ACORN_DATABASE_PATH";
/// Environment variable used to set the minimum download count for model fallbacks.
pub const MINIMUM_DOWNLOAD_COUNT: &str = "ACORN_MINIMUM_DOWNLOAD_COUNT";
/// GitLab CI server hostname environment variable
pub const CI_SERVER_HOST: &str = "CI_SERVER_HOST";
/// Environment variables forwarded to detached GitLab bot containers
pub const GITLAB_CONTAINER_ENV: [&str; 3] = ["GITLAB_TOKEN", "GITLAB_WEBHOOK_TOKEN", "GITLAB_WEBHOOK_SIGNING_TOKEN"];
/// Environment variable used to disable local database use at runtime.
pub const NO_LOCAL_DATABASE: &str = "ACORN_NO_LOCAL_DATABASE";
/// Environment variable used to select readability metric for readability checks at runtime.
pub const READABILITY_METRIC: &str = "ACORN_READABILITY_METRIC";
/// Environment variable used to set the Hugging Face fallback search limit.
pub const SEARCH_LIMIT: &str = "ACORN_SEARCH_LIMIT";
/// Environment variable used to set minimum shell-lint severity.
pub const SHELL_LINT_MIN_SEVERITY: &str = "ACORN_SHELL_LINT_MIN_SEVERITY";
/// Environment variable used to select the terminal user-interface theme.
pub const TUI_THEME: &str = "ACORN_TUI_THEME";
/// Environment variable names used to resolve GitLab API tokens.
pub const GITLAB_TOKEN_VARIABLE_NAMES: [&str; 3] = ["CI_JOB_TOKEN", "GITLAB_TOKEN", "PRIVATE_TOKEN"];
/// Environment variable names used to resolve Hugging Face API tokens.
pub const HUGGINGFACE_TOKEN_VARIABLE_NAMES: [&str; 3] = ["HF_TOKEN", "HF_API_TOKEN", "HUGGINGFACE_HUB_TOKEN"];
/// Hugging Face Hub cache locations in resolution order.
pub const HUGGINGFACE_CACHE_LOCATIONS: [&str; 5] = [
    "$HF_HUB_CACHE",
    "$HUGGINGFACE_HUB_CACHE",
    "$HF_HOME/hub",
    "$XDG_CACHE_HOME/huggingface/hub",
    "~/.cache/huggingface/hub",
];
/// Environment variable names used to resolve RAiD API tokens.
pub const RAID_TOKEN_VARIABLE_NAMES: [&str; 1] = ["RAID_API_TOKEN"];
