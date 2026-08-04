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
/// Base URL for deploying ORNL data.
pub const BASE_URL: &str = "https://research.ornl.gov";
/// ORNL disclaimer.
pub const DISCLAIMER: &str = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
/// Application name.
pub const APPLICATION: &str = "acorn";
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
/// Default Hugging Face domain
pub const DEFAULT_HUGGINGFACE_DOMAIN: &str = "huggingface.co";
/// Default Hugging Face repository revision used when none is specified.
pub const DEFAULT_HUGGINGFACE_MODEL_REVISION: &str = "main";
/// Default llama-swap configuration path beneath the user's home directory.
pub const DEFAULT_LLAMA_SWAP_CONFIG_PATH: &str = ".config/llama-swap/config.yaml";
/// Default directory under the user's home directory for downloaded models.
pub const DEFAULT_MODELS_DIRECTORY: &str = ".models";
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
/// Default affiliation.
pub const DEFAULT_AFFILIATION: &str = "Oak Ridge National Laboratory";
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
