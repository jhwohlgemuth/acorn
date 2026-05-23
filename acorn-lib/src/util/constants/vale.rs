/// URL for Vale releases.
pub const VALE_RELEASES_URL: &str = "https://github.com/errata-ai/vale/releases";
/// Version of Vale to use with ACORN.
pub const VALE_VERSION: &str = "3.9.4";
/// Default Vale configuration path.
pub const DEFAULT_VALE_ROOT: &str = "./.vale/";
/// URL for custom ORNL Science Vale package.
pub const DEFAULT_VALE_PACKAGE_URL: &str = "https://code.ornl.gov/research-enablement/vale-package/-/archive/v0.0.1/vale-package-v0.0.1.zip";
/// Custom Vale package name.
pub const CUSTOM_VALE_PACKAGE_NAME: &str = "Science";
/// Enabled Vale packages.
pub const ENABLED_VALE_PACKAGES: [&str; 4] = ["Google", "proselint", "write-good", "Joblint"];
/// Disabled Vale rules.
pub const DISABLED_VALE_RULES: [&str; 15] = [
    "Vale.Terms",
    "Google.EmDash",
    "Google.Contractions",
    "Google.GenderBias",
    "Google.Headings",
    "Google.Latin",
    "Google.Parens",
    "Google.Quotes",
    "Google.We",
    "Joblint.Competitive",
    "proselint.GenderBias",
    "write-good.E-Prime",
    "write-good.Passive",
    "write-good.TooWordy",
    "write-good.Weasel",
];
