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
/// Base URL for deploying ORNL data.
pub const BASE_URL: &str = "https://research.ornl.gov";
/// ORNL disclaimer.
pub const DISCLAIMER: &str = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
/// Application name.
pub const APPLICATION: &str = "acorn";
/// Organization name.
pub const ORGANIZATION: &str = "ornl";
/// Organization qualifier.
pub const QUALIFIER: &str = "org";
/// Default cache TTL in seconds (1 month = 30 days)
pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;
/// File size threshold in bytes for switching to large-file read strategy (100 MB).
pub const LARGE_FILE_THRESHOLD_BYTES: u64 = 100 * 1024 * 1024;
/// Default ACORN configuration filenames searched in the current working directory.
pub const DEFAULT_CONFIG_FILENAMES: [&str; 3] = [".acorn.json", ".acorn.yaml", ".acorn.yml"];
/// Default affiliation.
pub const DEFAULT_AFFILIATION: &str = "Oak Ridge National Laboratory";
/// Default base name used for configured GitLab runner containers.
pub const DEFAULT_RUNNER_NAME: &str = "acorn-runner";
/// RGB ORNL brand primary color.
///
/// See <https://www.olcf.ornl.gov/about-olcf/media-assets/>.
pub const COLOR_PRIMARY: [u8; 4] = [0, 121, 52, 255];
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
