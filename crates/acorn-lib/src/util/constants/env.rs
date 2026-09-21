/// Environment variable name for cache TTL override.
pub const CACHE_TTL: &str = "ACORN_CACHE_TTL";
/// Environment variable used to select a Chrome or Chromium executable for PDF export.
pub const CHROME_PATH: &str = "CHROME_PATH";
/// GitLab CI server hostname environment variable
pub const CI_SERVER_HOST: &str = "CI_SERVER_HOST";
/// Environment variable used to select the citation format for resolved raw gather output.
pub const CITATION_FORMAT: &str = "ACORN_CITATION_FORMAT";
/// Environment variable containing a CiteAs API token.
pub const CITEAS_API_TOKEN: &str = "CITEAS_API_TOKEN";
/// Environment variable overriding the CiteAs API host.
pub const CITEAS_SERVER_HOST: &str = "CITEAS_SERVER_HOST";
/// Environment variable used to select a database backend at runtime.
pub const DATABASE_BACKEND: &str = "ACORN_DATABASE_BACKEND";
/// Environment variable used to override database path at runtime.
pub const DATABASE_PATH: &str = "ACORN_DATABASE_PATH";
/// Environment variables forwarded to detached GitLab bot containers
pub const GITLAB_CONTAINER_ENV: [&str; 3] = ["GITLAB_TOKEN", "GITLAB_WEBHOOK_TOKEN", "GITLAB_WEBHOOK_SIGNING_TOKEN"];
/// Environment variable names used to resolve GitLab API tokens.
pub const GITLAB_TOKEN_VARIABLE_NAMES: [&str; 3] = ["GITLAB_TOKEN", "PRIVATE_TOKEN", "CI_JOB_TOKEN"];
/// Environment variable containing the bearer token for the gRPC endpoint.
pub const GRPC_TOKEN: &str = "ACORN_GRPC_TOKEN";
/// Environment variable used to set the maximum HTTP retry count for outbound requests.
pub const HTTP_MAX_RETRIES: &str = "ACORN_HTTP_MAX_RETRIES";
/// Environment variable used to set the maximum time a streaming download may idle without new bytes, in seconds.
pub const HTTP_STALL_TIMEOUT: &str = "ACORN_HTTP_STALL_TIMEOUT";
/// Hugging Face Hub cache locations in resolution order.
pub const HUGGINGFACE_CACHE_LOCATIONS: [&str; 5] = [
    "$HF_HUB_CACHE",
    "$HUGGINGFACE_HUB_CACHE",
    "$HF_HOME/hub",
    "$XDG_CACHE_HOME/huggingface/hub",
    "~/.cache/huggingface/hub",
];
/// Environment variable names used to resolve Hugging Face API tokens.
pub const HUGGINGFACE_TOKEN_VARIABLE_NAMES: [&str; 3] = ["HF_TOKEN", "HF_API_TOKEN", "HUGGINGFACE_HUB_TOKEN"];
/// Optional environment variable reserved by the shared API options contract for ISNI.
pub const ISNI_API_TOKEN: &str = "ISNI_API_TOKEN";
/// Environment variable overriding the ISNI SRU host.
pub const ISNI_SERVER_HOST: &str = "ISNI_SERVER_HOST";
/// Environment variable used to set the minimum download count for model fallbacks.
pub const MINIMUM_DOWNLOAD_COUNT: &str = "ACORN_MINIMUM_DOWNLOAD_COUNT";
/// Environment variable used to set the model download whitelist.
pub const MODEL_WHITELIST: &str = "ACORN_MODEL_WHITELIST";
/// Environment variable used to disable local database use at runtime.
pub const NO_LOCAL_DATABASE: &str = "ACORN_NO_LOCAL_DATABASE";
/// Environment variable containing an OpenAI API key.
pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
/// Environment variable overriding the OpenAI API host.
pub const OPENAI_SERVER_HOST: &str = "OPENAI_SERVER_HOST";
/// Environment variable overriding the OpenAlex API host.
pub const OPENALEX_API_HOST: &str = "OPENALEX_API_HOST";
/// Environment variable containing an OpenAlex API key.
pub const OPENALEX_API_KEY: &str = "OPENALEX_API_KEY";
/// Environment variable containing an ORCID API token.
pub const ORCID_API_TOKEN: &str = "ORCID_API_TOKEN";
/// Environment variable overriding the ORCID API host.
pub const ORCID_SERVER_HOST: &str = "ORCID_SERVER_HOST";
/// Environment variable containing a RAiD API token.
pub const RAID_API_TOKEN: &str = "RAID_API_TOKEN";
/// Environment variable overriding the RAiD API host.
pub const RAID_SERVER_HOST: &str = "RAID_SERVER_HOST";
/// Environment variable names used to resolve RAiD API tokens.
pub const RAID_TOKEN_VARIABLE_NAMES: [&str; 1] = [RAID_API_TOKEN];
/// Environment variable used to select readability metric for readability checks at runtime.
pub const READABILITY_METRIC: &str = "ACORN_READABILITY_METRIC";
/// Environment variable containing a ROR API token.
pub const ROR_API_TOKEN: &str = "ROR_API_TOKEN";
/// Environment variable overriding the ROR API host.
pub const ROR_SERVER_HOST: &str = "ROR_SERVER_HOST";
/// Environment variable containing the bearer token for the JSON-RPC HTTP endpoint.
pub const RPC_TOKEN: &str = "ACORN_RPC_TOKEN";
/// Environment variable used to set the Hugging Face fallback search limit.
pub const SEARCH_LIMIT: &str = "ACORN_SEARCH_LIMIT";
/// Environment variable used to select the terminal user-interface theme.
pub const TUI_THEME: &str = "ACORN_TUI_THEME";
