//! Small terminal presentation primitives.
#[cfg(feature = "terminal")]
use owo_colors::{OwoColorize, Style};

/// Shared logging labels with optional terminal styling.
pub struct Label;
#[cfg(feature = "terminal")]
impl Label {
    /// Prefix for warnings and cautions.
    pub const CAUTION: &str = "⚠️ ";
    /// Prefix for successful operations.
    pub const CHECKMARK: &str = "✅ ";
    /// Prefix for re-running a watched command.
    pub const RERUN: &str = "🔄 ";
    /// Progress bar template.
    pub const PROGRESS_BAR_TEMPLATE: &str = "  {spinner:.green}{pos:>5} of{len:^5}[{bar:40.green}] {msg}";
    /// Indeterminate progress spinner template.
    pub const PROGRESS_SPINNER_TEMPLATE: &str = "  {spinner:.green}  {msg}";
    /// Simple progress counter template.
    pub const PROGRESS_COUNTER_TEMPLATE: &str = "{pos:>5} of{len:^5} {msg}";
    /// Dry-run label text.
    pub const DRY_RUN: &str = " DRY_RUN ■ ";
    /// Changed label.
    pub fn changed() -> String {
        Self::fmt_changed("Change Detected")
    }
    /// Format a changed label.
    pub fn fmt_changed(value: &str) -> String {
        format!("{}{value}", Self::RERUN)
            .style(Style::new().yellow().bold().on_default_color())
            .to_string()
    }
    /// Styled dry-run label.
    pub fn dry_run() -> String {
        Self::DRY_RUN.style(Style::new().black().on_yellow()).to_string()
    }
    /// Invalid label.
    pub fn invalid() -> String {
        Self::fmt_invalid(" ✗ INVALID")
    }
    /// Format an invalid label.
    pub fn fmt_invalid(value: &str) -> String {
        value.style(Style::new().red().on_default_color()).to_string()
    }
    /// Valid label.
    pub fn valid() -> String {
        Self::fmt_valid(" ✓ VALID  ")
    }
    /// Format a valid label.
    pub fn fmt_valid(value: &str) -> String {
        value.style(Style::new().green().on_default_color()).to_string()
    }
    /// Failure label.
    pub fn fail() -> String {
        Self::fmt_fail("FAIL")
    }
    /// Format a failure label.
    pub fn fmt_fail(value: &str) -> String {
        format!(" ✗ {value} ").style(Style::new().white().on_red()).to_string()
    }
    /// Found label.
    pub fn found() -> String {
        Self::fmt_found("FOUND")
    }
    /// Format a found label.
    pub fn fmt_found(value: &str) -> String {
        value.style(Style::new().green().on_default_color()).to_string()
    }
    /// Not-found label.
    pub fn not_found() -> String {
        Self::fmt_not_found("NOT_FOUND")
    }
    /// Format a not-found label.
    pub fn fmt_not_found(value: &str) -> String {
        value.style(Style::new().red().on_default_color()).to_string()
    }
    /// Output label.
    pub fn output() -> String {
        Self::fmt_output("OUTPUT")
    }
    /// Format an output label.
    pub fn fmt_output(value: &str) -> String {
        value.style(Style::new().cyan().dimmed().on_default_color()).to_string()
    }
    /// Pass label.
    pub fn pass() -> String {
        Self::fmt_pass("SUCCESS")
    }
    /// Format a pass label.
    pub fn fmt_pass(value: &str) -> String {
        format!("{}{value}", Self::CHECKMARK)
            .style(Style::new().green().bold().on_default_color())
            .to_string()
    }
    /// Read label.
    pub fn read() -> String {
        "READ".style(Style::new().green().on_default_color()).to_string()
    }
    /// Rejected label.
    pub fn rejected() -> String {
        Self::fmt_rejected("REJECTED")
    }
    /// Format a rejected label.
    pub fn fmt_rejected(value: &str) -> String {
        format!("🛑 {value} ").style(Style::new().red().on_default_color()).to_string()
    }
    /// Run label.
    pub fn run() -> String {
        Self::fmt_run("RUN")
    }
    /// Format a run label.
    pub fn fmt_run(value: &str) -> String {
        format!(" {value} ▶ ").style(Style::new().black().on_yellow()).to_string()
    }
    /// Skip label.
    pub fn skip() -> String {
        Self::fmt_skip("SKIP")
    }
    /// Format a skip label.
    pub fn fmt_skip(value: &str) -> String {
        format!("{}{value}", Self::CAUTION)
            .style(Style::new().yellow().on_default_color())
            .to_string()
    }
    /// Using label.
    pub fn using() -> String {
        Self::fmt_using("USING")
    }
    /// Warning label.
    pub fn warning() -> String {
        Self::fmt_skip("WARNING")
    }
    /// Format a using label.
    pub fn fmt_using(value: &str) -> String {
        value.style(Style::new().cyan()).to_string()
    }
}
#[cfg(not(feature = "terminal"))]
impl Label {
    /// Prefix for warnings and cautions.
    pub const CAUTION: &str = "⚠️ ";
    /// Prefix for successful operations.
    pub const CHECKMARK: &str = "✅ ";
    /// Prefix for re-running a watched command.
    pub const RERUN: &str = "🔄 ";
    /// Progress bar template.
    pub const PROGRESS_BAR_TEMPLATE: &str = "  {spinner}{pos:>5} of{len:^5}[{bar:40}] {msg}";
    /// Indeterminate progress spinner template.
    pub const PROGRESS_SPINNER_TEMPLATE: &str = "  {spinner}  {msg}";
    /// Simple progress counter template.
    pub const PROGRESS_COUNTER_TEMPLATE: &str = "{pos:>5} of{len:^5} {msg}";
    /// Dry-run label text.
    pub const DRY_RUN: &str = " DRY_RUN ■ ";
    /// Changed label.
    pub fn changed() -> String {
        Self::fmt_changed("Change Detected")
    }
    /// Format a changed label.
    pub fn fmt_changed(value: &str) -> String {
        format!("{}{value}", Self::RERUN)
    }
    /// Plain dry-run label.
    pub fn dry_run() -> String {
        Self::DRY_RUN.to_string()
    }
    /// Invalid label.
    pub fn invalid() -> String {
        Self::fmt_invalid(" ✗ INVALID")
    }
    /// Format an invalid label.
    pub fn fmt_invalid(value: &str) -> String {
        value.to_string()
    }
    /// Valid label.
    pub fn valid() -> String {
        Self::fmt_valid(" ✓ VALID  ")
    }
    /// Format a valid label.
    pub fn fmt_valid(value: &str) -> String {
        value.to_string()
    }
    /// Failure label.
    pub fn fail() -> String {
        Self::fmt_fail("FAIL")
    }
    /// Format a failure label.
    pub fn fmt_fail(value: &str) -> String {
        format!(" ✗ {value} ")
    }
    /// Found label.
    pub fn found() -> String {
        Self::fmt_found("FOUND")
    }
    /// Format a found label.
    pub fn fmt_found(value: &str) -> String {
        value.to_string()
    }
    /// Not-found label.
    pub fn not_found() -> String {
        Self::fmt_not_found("NOT_FOUND")
    }
    /// Format a not-found label.
    pub fn fmt_not_found(value: &str) -> String {
        value.to_string()
    }
    /// Output label.
    pub fn output() -> String {
        Self::fmt_output("OUTPUT")
    }
    /// Format an output label.
    pub fn fmt_output(value: &str) -> String {
        value.to_string()
    }
    /// Pass label.
    pub fn pass() -> String {
        Self::fmt_pass("SUCCESS")
    }
    /// Format a pass label.
    pub fn fmt_pass(value: &str) -> String {
        format!("{}{value}", Self::CHECKMARK)
    }
    /// Read label.
    pub fn read() -> String {
        "READ".to_string()
    }
    /// Rejected label.
    pub fn rejected() -> String {
        Self::fmt_rejected("REJECTED")
    }
    /// Format a rejected label.
    pub fn fmt_rejected(value: &str) -> String {
        format!("🛑 {value} ")
    }
    /// Run label.
    pub fn run() -> String {
        Self::fmt_run("RUN")
    }
    /// Format a run label.
    pub fn fmt_run(value: &str) -> String {
        format!(" {value} ▶ ")
    }
    /// Skip label.
    pub fn skip() -> String {
        Self::fmt_skip("SKIP")
    }
    /// Format a skip label.
    pub fn fmt_skip(value: &str) -> String {
        format!("{}{value}", Self::CAUTION)
    }
    /// Using label.
    pub fn using() -> String {
        Self::fmt_using("USING")
    }
    /// Warning label.
    pub fn warning() -> String {
        Self::fmt_skip("WARNING")
    }
    /// Format a using label.
    pub fn fmt_using(value: &str) -> String {
        value.to_string()
    }
}
