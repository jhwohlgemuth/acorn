/// Environment variable name for cache TTL override.
pub const CACHE_TTL: &str = "ACORN_CACHE_TTL";
/// Environment variable used to select a database backend at runtime.
pub const DATABASE_BACKEND: &str = "ACORN_DATABASE_BACKEND";
/// Environment variable used to override database path at runtime.
pub const DATABASE_PATH: &str = "ACORN_DATABASE_PATH";
/// Environment variable used to disable local database use at runtime.
pub const NO_LOCAL_DATABASE: &str = "ACORN_NO_LOCAL_DATABASE";
/// Environment variable used to select readability metric for readability checks at runtime.
pub const READABILITY_METRIC: &str = "ACORN_READABILITY_METRIC";
/// Environment variable used to set minimum shell-lint severity.
pub const SHELL_LINT_MIN_SEVERITY: &str = "ACORN_SHELL_LINT_MIN_SEVERITY";
/// Environment variable names used to resolve GitLab API tokens.
pub const GITLAB_TOKEN_VARIABLE_NAMES: [&str; 3] = ["CI_JOB_TOKEN", "GITLAB_TOKEN", "PRIVATE_TOKEN"];
/// Environment variable names used to resolve RAiD API tokens.
pub const RAID_TOKEN_VARIABLE_NAMES: [&str; 1] = ["RAID_API_TOKEN"];
