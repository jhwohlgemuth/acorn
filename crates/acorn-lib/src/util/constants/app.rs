use jiff::SignedDuration;

/// Maximum number of archive bytes inspected while inferring its format.
pub const ARCHIVE_INFERENCE_BYTES: u64 = 8192;
/// Maximum number of entries accepted from an archive.
pub const MAX_ARCHIVE_ENTRIES: usize = 100_000;
/// Maximum expanded size accepted from an archive.
pub const MAX_ARCHIVE_EXPANDED_BYTES: u64 = 100 * 1024 * 1024 * 1024;
/// ASCII art logo for ACORN
pub const LOGO: &str = r"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣄⣀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣀⠀⢴⣶⠀⢶⣦⠀⢄⣀⠀⠠⢾⣿⠿⠿⠿⠿⢦⠀
⠀⠀⠀⠀⠀⠀⠺⠿⠇⢸⣿⣇⠘⣿⣆⠘⣿⡆⠠⣄⡀⠀⠀⠀⠀⠀⠀    
⠀⠀⠀⠀⢀⣴⣶⣶⣤⣄⡉⠛⠀⢹⣿⡄⢹⣿⡀⢻⣧⠀⡀⠀⠀⠀⠀    
⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡈⠓⠀⣿⣧⠈⢿⡆⠸⡄⠀⠀⠀   █████████     █████████     ███████    ███████████   ██████   █████
⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣈⠙⢆⠘⣿⡀⢻⠀⠀   ███▒▒▒▒▒███   ███▒▒▒▒▒███  ███▒▒▒▒▒███ ▒▒███▒▒▒▒▒███ ▒▒██████ ▒▒███
⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠹⣧⠈⠀⠀  ▒███    ▒███  ███     ▒▒▒  ███     ▒▒███ ▒███    ▒███  ▒███▒███ ▒███
⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠈⠃⠀⠀⠀ ▒███████████ ▒███         ▒███      ▒███ ▒██████████   ▒███▒▒███▒███
⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀ ▒███▒▒▒▒▒███ ▒███         ▒███      ▒███ ▒███▒▒▒▒▒███  ▒███ ▒▒██████⠀
⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀ ▒███    ▒███ ▒▒███     ███▒▒███     ███  ▒███    ▒███  ▒███  ▒▒█████
⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀ █████   █████ ▒▒█████████  ▒▒▒███████▒   █████   █████ █████  ▒▒█████⠀
⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀▒▒▒▒▒   ▒▒▒▒▒   ▒▒▒▒▒▒▒▒▒     ▒▒▒▒▒▒▒    ▒▒▒▒▒   ▒▒▒▒▒ ▒▒▒▒▒    ▒▒▒▒▒
⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠉⠀⠀⠀⠀⠀⠀⠀    
   ⣿⣿⠿⠿⠿⠿⠿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀~ Accessible Content Optimization for Research Needs ~
";
/// Compact ASCII art logo for terminal interfaces with limited space.
pub const COMPACT_LOGO: [&str; 6] = [
    " █████╗  ██████╗ ██████╗ ██████╗ ███╗   ██╗",
    "██╔══██╗██╔════╝██╔═══██╗██╔══██╗████╗  ██║",
    "███████║██║     ██║   ██║██████╔╝██╔██╗ ██║",
    "██╔══██║██║     ██║   ██║██╔══██╗██║╚██╗██║",
    "██║  ██║╚██████╗╚██████╔╝██║  ██║██║ ╚████║",
    "╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝",
];
/// Maximum number of converted documents retained in the process-local cache.
pub const DOCUMENT_CACHE_LIMIT: usize = 64;
/// Maximum supported document size in bytes.
pub const MAX_DOCUMENT_BYTES: usize = 100 * 1024 * 1024;
/// Maximum supported PDF page count.
pub const MAX_PDF_PAGES: u32 = 1_000;
/// Base URL for deploying ORNL data.
pub const BASE_URL: &str = "https://research.ornl.gov";
/// ORNL disclaimer.
pub const DISCLAIMER: &str = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
/// Application name.
pub const APPLICATION: &str = "acorn";
/// Supported research activity data filenames and extensions.
pub const SUPPORTED_RAD_FILETYPES: [&str; 6] = ["index.json", "jsonc", "yaml", "yml", "md", "markdown"];
/// Delay used to coalesce rapid filesystem notifications.
pub const WATCH_DEBOUNCE_INTERVAL: SignedDuration = SignedDuration::from_millis(300);
/// Maximum number of online synchronization attempts.
pub const ONLINE_SYNC_ATTEMPTS: usize = 3;
/// Initial exponential backoff delay for retryable HTTP requests, in milliseconds.
pub const HTTP_RETRY_INITIAL_BACKOFF_MILLISECONDS: i64 = 250;
/// Maximum backoff delay for retryable HTTP requests, in seconds.
pub const HTTP_RETRY_MAX_BACKOFF_SECONDS: i64 = 60;
/// Header used to authenticate GitLab API requests with a private token.
pub const GITLAB_PRIVATE_TOKEN_HEADER: &str = "PRIVATE-TOKEN";
/// Header used to authenticate GitLab API requests with a CI job token.
pub const GITLAB_JOB_TOKEN_HEADER: &str = "JOB-TOKEN";
/// GitLab token environment variables paired with their authentication headers in resolution order.
pub const GITLAB_CREDENTIAL_VARIABLE_PAIRS: [(&str, &str); 3] = [
    ("GITLAB_TOKEN", GITLAB_PRIVATE_TOKEN_HEADER),
    ("PRIVATE_TOKEN", GITLAB_PRIVATE_TOKEN_HEADER),
    ("CI_JOB_TOKEN", GITLAB_JOB_TOKEN_HEADER),
];
/// Maximum file size accepted by merge request analysis
pub const MAX_ANALYSIS_FILE_BYTES: u64 = 1_048_576;
/// Maximum number of durable GitLab operation attempts
pub const MAX_GITLAB_OPERATION_ATTEMPTS: u32 = 5;
/// Maximum durable GitLab operation retry delay
pub const MAX_GITLAB_OPERATION_BACKOFF_SECONDS: i64 = 300;
/// Hidden marker used to find the bot-authored merge request analysis note
pub const MERGE_REQUEST_REPORT_MARKER: &str = "<!-- acorn:merge-request-analysis -->";
/// Hidden marker used to find the bot-authored work-item citation intake note
pub const WORK_ITEM_REPORT_MARKER: &str = "<!-- acorn:work-item-intake -->";
/// Organization name.
pub const ORGANIZATION: &str = "ornl";
/// Organization qualifier.
pub const QUALIFIER: &str = "org";
/// Default OpenAlex API domain.
pub const DEFAULT_OPENALEX_DOMAIN: &str = "api.openalex.org";
/// OpenAlex work fields used for research-output enrichment.
pub const OPENALEX_WORK_FIELDS: &str = "id,doi,title,type,publication_date,authorships,grants,open_access,primary_location,best_oa_location,keywords";
/// Default Hugging Face repository revision used when none is specified.
pub const DEFAULT_HUGGINGFACE_MODEL_REVISION: &str = "main";
/// Explicit URI prefix used to route model references through OCI.
pub const OCI_PREFIX: &str = "oci://";
/// OCI image manifest media type accepted for model artifacts.
pub const OCI_IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
/// OCI image index media type accepted for model artifact indirection.
pub const OCI_IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// Docker manifest-list media type accepted for model artifact indirection.
pub const DOCKER_MANIFEST_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
/// OCI image configuration media type used to identify runnable images.
pub const OCI_IMAGE_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
/// Docker image configuration media type used to identify runnable images.
pub const DOCKER_IMAGE_CONFIG: &str = "application/vnd.docker.container.image.v1+json";
/// Uncompressed OCI image layer media type accepted for model files.
pub const OCI_LAYER: &str = "application/vnd.oci.image.layer.v1.tar";
/// Compressed OCI image layer media type accepted for model files.
pub const OCI_LAYER_GZIP: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
/// ACORN-specific OCI layer media type for model files.
pub const ACORN_MODEL_LAYER: &str = "application/vnd.acorn.model.file.v1";
/// KitOps ModelKit configuration media type.
pub const KITOPS_MODELKIT_CONFIG: &str = "application/vnd.kitops.modelkit.config.v1+json";
/// Gzip-compressed KitOps ModelKit model layer media type.
pub const KITOPS_MODELKIT_MODEL_GZIP: &str = "application/vnd.kitops.modelkit.model.v1.tar+gzip";
/// CNCF ModelPack artifact type.
pub const MODELPACK_MANIFEST: &str = "application/vnd.cncf.model.manifest.v1+json";
/// CNCF ModelPack configuration media type.
pub const MODELPACK_CONFIG: &str = "application/vnd.cncf.model.config.v1+json";
/// Prefix shared by CNCF ModelPack model-weight layer media types.
pub const MODELPACK_WEIGHT_PREFIX: &str = "application/vnd.cncf.model.weight.v1.";
/// CNCF ModelPack raw model-weight layer media type.
pub const MODELPACK_WEIGHT_RAW: &str = "application/vnd.cncf.model.weight.v1.raw";
/// CNCF ModelPack uncompressed TAR model-weight layer media type.
pub const MODELPACK_WEIGHT_TAR: &str = "application/vnd.cncf.model.weight.v1.tar";
/// CNCF ModelPack gzip-compressed TAR model-weight layer media type.
pub const MODELPACK_WEIGHT_TAR_GZIP: &str = "application/vnd.cncf.model.weight.v1.tar+gzip";
/// CNCF ModelPack zstd-compressed TAR model-weight layer media type.
pub const MODELPACK_WEIGHT_TAR_ZSTD: &str = "application/vnd.cncf.model.weight.v1.tar+zstd";
/// Minimum supported ORAS CLI version.
pub const MINIMUM_ORAS_VERSION: (u64, u64, u64) = (1, 3, 0);
/// Default llama-swap configuration path beneath the user's home directory.
pub const DEFAULT_LLAMA_SWAP_CONFIG_PATH: &str = ".config/llama-swap/config.yaml";
/// Default OpenCode configuration path beneath the user's home directory.
pub const DEFAULT_OPENCODE_CONFIG_PATH: &str = ".config/opencode/opencode.jsonc";
/// Default VS Code language-model path relative to the platform configuration directory.
///
/// This resolves beneath `~/Library/Application Support` on macOS, `~/.config` on Linux,
/// and `%APPDATA%` on Windows.
pub const DEFAULT_VSCODE_CONFIG_PATH: &str = "Code/User/chatLanguageModels.json";
/// Default Goose configuration path beneath the user's home directory on macOS and Linux.
pub const DEFAULT_GOOSE_CONFIG_PATH: &str = ".config/goose/config.yaml";
/// Default Goose configuration path beneath the Windows roaming application-data directory.
pub const DEFAULT_GOOSE_WINDOWS_CONFIG_PATH: &str = "Block/goose/config/config.yaml";
/// Default directory under the user's home directory for downloaded models.
pub const DEFAULT_MODELS_DIRECTORY: &str = ".models";
/// Character cutoff used when shortening paths for display.
pub const DISPLAY_PATH_CHARACTER_COUNT: usize = 20;
/// Default model search result limit.
pub const DEFAULT_HUGGINGFACE_SEARCH_LIMIT: usize = 20;
/// Default minimum download count for Hugging Face fallback models.
pub const DEFAULT_HUGGINGFACE_MINIMUM_DOWNLOAD_COUNT: u64 = 100;
/// Default Hugging Face model search term.
pub const DEFAULT_HUGGINGFACE_SEARCH_TERM: &str = "gguf";
/// Default cache TTL in seconds (1 month = 30 days)
pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;
/// File size threshold in bytes for switching to large-file read strategy (100 MB).
pub const LARGE_FILE_THRESHOLD_BYTES: u64 = 100 * 1024 * 1024;
/// Default ACORN configuration filenames searched in the current working directory.
pub const DEFAULT_CONFIG_FILENAMES: [&str; 5] = [".acorn.json", ".acorn.jsonc", ".acorn.yaml", ".acorn.yml", ".acorn"];
/// Pinned Chrome for Testing version used for PDF export.
pub const CHROME_VERSION: (u32, u32, u32, u32) = (151, 0, 7922, 47);
/// Files ignored by default when processing configured buckets.
pub const IGNORE: [&str; 5] = [".gitignore", ".gitlab-ci.yml", ".gitkeep", ".DS_Store", "README.md"];
/// Default base name used for configured GitLab runner containers.
pub const DEFAULT_RUNNER_NAME: &str = "acorn-runner";
/// Default Docker daemon socket path.
pub const DOCKER_SOCKET: &str = "/var/run/docker.sock";
/// Default ACORN HTTP User-Agent header value when std is available.
#[cfg(feature = "std")]
pub const ACORN_USER_AGENT: &str = concat!("ACORN/", env!("CARGO_PKG_VERSION"), " (https://acorn.ornl.gov; mailto:research@ornl.gov)");
/// Default ACORN HTTP User-Agent header value when std is unavailable.
#[cfg(not(feature = "std"))]
pub const ACORN_USER_AGENT: &str = "ACORN (https://acorn.ornl.gov; mailto:research@ornl.gov)";
/// ORNL Green brand color.
pub const ORNL_COLOR_GREEN: [u8; 3] = [0x00, 0x66, 0x2C];
/// Hale Navy brand color.
pub const ORNL_COLOR_HALE_NAVY: [u8; 3] = [0x00, 0x45, 0x4D];
/// Dark Matter brand color.
pub const ORNL_COLOR_DARK_MATTER: [u8; 3] = [0x37, 0x3A, 0x36];
/// Graphite brand color.
pub const ORNL_COLOR_GRAPHITE: [u8; 3] = [0xDB, 0xDC, 0xDB];
/// Polar brand color.
pub const ORNL_COLOR_POLAR: [u8; 3] = [0xFF, 0xFF, 0xFF];
/// Energy brand color.
pub const ORNL_COLOR_ENERGY: [u8; 3] = [0x7D, 0xBA, 0x00];
/// Mist brand color.
pub const ORNL_COLOR_MIST: [u8; 3] = [0x8B, 0xFE, 0xBF];
/// Biome brand color.
pub const ORNL_COLOR_BIOME: [u8; 3] = [0x00, 0xB3, 0x8F];
/// Aqua brand color.
pub const ORNL_COLOR_AQUA: [u8; 3] = [0x00, 0xBD, 0xB5];
/// Infinity brand color.
pub const ORNL_COLOR_INFINITY: [u8; 3] = [0x00, 0x6B, 0xA6];
/// Hydro brand color.
pub const ORNL_COLOR_HYDRO: [u8; 3] = [0x00, 0x57, 0x76];
/// Forge brand color.
pub const ORNL_COLOR_FORGE: [u8; 3] = [0xFF, 0x9E, 0x1B];
/// Spark brand color.
pub const ORNL_COLOR_SPARK: [u8; 3] = [0xFE, 0x50, 0x00];
/// Plasma brand color.
pub const ORNL_COLOR_PLASMA: [u8; 3] = [0xB5, 0x00, 0x94];
/// Pulsar brand color.
pub const ORNL_COLOR_PULSAR: [u8; 3] = [0x4E, 0x00, 0x8E];
/// RGB ORNL brand primary color with full opacity.
pub const COLOR_PRIMARY: [u8; 4] = [ORNL_COLOR_GREEN[0], ORNL_COLOR_GREEN[1], ORNL_COLOR_GREEN[2], 255];
/// RGB color for transparency.
pub const COLOR_TRANSPARENT: [u8; 4] = [255, 255, 255, 0];
/// Automated Readability Index (ARI) maximum allowed value.
///
/// This value maps to 12th grade (high school senior) reading level.
pub const MAX_ALLOWED_ARI: f64 = 18.0;
/// Coleman-Liau Index (CLI) maximum allowed value.
///
/// This value maps to 12th grade (high school senior) reading level.
pub const MAX_ALLOWED_CLI: f64 = 12.0;
/// Flesch-Kincaid Grade Level (FKGL) maximum allowed value.
///
/// This value maps to 12th grade (high school senior) reading level.
pub const MAX_ALLOWED_FKGL: f64 = 12.0;
/// Flesch Reading Ease Score (FRES) maximum allowed value.
///
/// This value maps to 12th grade (high school senior) reading level.
pub const MAX_ALLOWED_FRES: f64 = 50.0;
/// Gunning Fog Index (GFI) maximum allowed value.
///
/// This value maps to 12th grade (high school senior) reading level.
pub const MAX_ALLOWED_GFI: f64 = 12.0;
/// Lix Index (Lix) maximum allowed value.
///
/// This value is somewhere in between "very easy" (20) and "very difficult" (60), skewed toward "very difficult".
pub const MAX_ALLOWED_LIX: f64 = 50.0;
/// Simple Measure of Gobbledygook (SMOG) maximum allowed value.
///
/// This value maps to upper end of high school (12th grade) reading level.
pub const MAX_ALLOWED_SMOG: f64 = 13.0;
/// Embedded skill asset filename.
pub const SKILL_FILENAME: &str = "SKILL.md";
/// Embedded skill asset relative path inside the `skills/` embed root.
pub const SKILL_PATH: &str = "acorn/SKILL.md";
/// Prompt text copied to clipboard by `acorn skill` for agent use.
pub const SKILL_CLIPBOARD_PROMPT: &str = "Load the ACORN skill and use it for tasks during this session. Run `acorn skill` to get the skill path.";
