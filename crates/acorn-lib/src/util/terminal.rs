//! Terminal output helpers
use crate::prelude::*;
#[cfg(feature = "std")]
use crate::skip;
#[cfg(feature = "std")]
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
#[cfg(feature = "std")]
use comfy_table::presets::UTF8_FULL;
#[cfg(feature = "std")]
use comfy_table::{ContentArrangement, Table};
#[cfg(feature = "unicode")]
use console::Emoji;
#[cfg(feature = "std")]
use owo_colors::{OwoColorize, Style, Styled};
#[cfg(feature = "std")]
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};

/// Struct for using and sharing colorized logging labels
pub struct Label {}
impl Label {
    /// Prefix for use when logging a warning, caution, etc..
    #[cfg(not(feature = "unicode"))]
    pub const CAUTION: &str = "!!! ";
    /// Prefix for use when logging a warning, caution, etc.
    #[cfg(feature = "unicode")]
    pub const CAUTION: Emoji<'_, '_> = Emoji("⚠️ ", "!!! ");
    /// Prefix for use when logging a success, pass, etc.
    #[cfg(not(feature = "unicode"))]
    pub const CHECKMARK: &str = "✓ ";
    /// Prefix for use when logging a success, pass, etc.
    #[cfg(feature = "unicode")]
    pub const CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "☑ ");
    /// Template string to customize the progress bar
    pub const PROGRESS_BAR_TEMPLATE: &str = "  {spinner:.green}{pos:>5} of{len:^5}[{bar:40.green}] {msg}";
    /// Template string for progress spinner (indeterminate)
    pub const PROGRESS_SPINNER_TEMPLATE: &str = "  {spinner:.green}  {msg}";
    /// Template string for progress counter (simple X of Y)
    pub const PROGRESS_COUNTER_TEMPLATE: &str = "{pos:>5} of{len:^5} {msg}";
    /// Dry-run label text
    pub const DRY_RUN: &str = " DRY_RUN ■ ";
    /// Styled dry-run label
    #[cfg(feature = "std")]
    pub fn dry_run() -> Styled<&'static &'static str> {
        Self::DRY_RUN.style(Style::new().black().on_yellow())
    }
    /// Plain dry-run label
    #[cfg(not(feature = "std"))]
    pub fn dry_run() -> String {
        String::from(Self::DRY_RUN)
    }
    /// Invalid label
    pub fn invalid() -> String {
        Self::fmt_invalid(" ✗ INVALID")
    }
    /// Format an invalid label
    pub fn fmt_invalid(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            value.style(Style::new().red().on_default_color()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            value.to_string()
        }
    }
    /// Valid label
    pub fn valid() -> String {
        Self::fmt_valid(" ✓ VALID  ")
    }
    /// Format a valid label
    pub fn fmt_valid(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            value.style(Style::new().green().on_default_color()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            value.to_string()
        }
    }
    /// Failure label
    pub fn fail() -> String {
        Self::fmt_fail("FAIL")
    }
    /// Format a failure label
    pub fn fmt_fail(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            format!(" ✗ {value} ").style(Style::new().white().on_red()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            format!(" ✗ {value} ")
        }
    }
    /// Found label
    pub fn found() -> String {
        Self::fmt_found("FOUND")
    }
    /// Format a found label
    pub fn fmt_found(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            value.to_string().style(Style::new().green().on_default_color()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            value.to_string()
        }
    }
    /// Not-found label
    pub fn not_found() -> String {
        Self::fmt_not_found("NOT_FOUND")
    }
    /// Format a not-found label
    pub fn fmt_not_found(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            value.style(Style::new().red().on_default_color()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            value.to_string()
        }
    }
    /// Output label
    pub fn output() -> String {
        Self::fmt_output("OUTPUT")
    }
    /// Format an output label
    pub fn fmt_output(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            value.style(Style::new().cyan().dimmed().on_default_color()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            value.to_string()
        }
    }
    /// Pass label
    pub fn pass() -> String {
        Self::fmt_pass("SUCCESS")
    }
    /// Format a pass label
    pub fn fmt_pass(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            format!("{}{}", Self::CHECKMARK, value)
                .style(Style::new().green().bold().on_default_color())
                .to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            format!("{}{value}", Self::CHECKMARK)
        }
    }
    /// Read label
    #[cfg(feature = "std")]
    pub fn read() -> Styled<&'static &'static str> {
        "READ".style(Style::new().green().on_default_color())
    }
    /// Plain read label
    #[cfg(not(feature = "std"))]
    pub fn read() -> String {
        String::from("READ")
    }
    /// Rejected label
    pub fn rejected() -> String {
        Self::fmt_rejected("REJECTED")
    }
    /// Format a rejected label
    pub fn fmt_rejected(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            format!("🛑 {value} ").style(Style::new().red().on_default_color()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            format!("🛑 {value} ")
        }
    }
    /// Run label
    pub fn run() -> String {
        Self::fmt_run("RUN")
    }
    /// Format a run label
    pub fn fmt_run(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            format!(" {value} ▶ ").style(Style::new().black().on_yellow()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            format!(" {value} ▶ ")
        }
    }
    /// Skip label
    pub fn skip() -> String {
        Self::fmt_skip("SKIP")
    }
    /// Format a skip label
    pub fn fmt_skip(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            format!("{}{}", Self::CAUTION, value)
                .style(Style::new().yellow().on_default_color())
                .to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            format!("{}{value}", Self::CAUTION)
        }
    }
    /// Using label
    pub fn using() -> String {
        Self::fmt_using("USING")
    }
    /// Format a using label.
    pub fn fmt_using(value: &str) -> String {
        #[cfg(feature = "std")]
        {
            value.style(Style::new().cyan()).to_string()
        }
        #[cfg(not(feature = "std"))]
        {
            value.to_string()
        }
    }
}
/// Print line-oriented changes with ANSI colors
#[cfg(feature = "std")]
pub fn print_changes(old: &str, new: &str) {
    print_changes_with_color(old, new, true);
}
/// Print line-oriented changes with optional ANSI colors
#[cfg(feature = "std")]
pub fn print_changes_with_color(old: &str, new: &str, color: bool) {
    let changes = text_diff_changes_with_color(old, new, color);
    if changes.iter().all(|(tag, _)| *tag == Equal) {
        skip!("No format changes");
    } else {
        changes.into_iter().for_each(|(_, change)| print!("{change}"));
    }
}
/// Print rows as a host-formatted table
#[cfg(feature = "std")]
pub fn print_values_as_table<T>(headers: Vec<&str>, rows: Vec<Vec<String>>, title: Option<T>)
where
    T: ToString,
{
    println!("{}", values_as_table(headers, rows, title));
}
/// Render rows as a host-formatted table without writing to stdout
#[cfg(feature = "std")]
pub fn values_as_table<T>(headers: Vec<&str>, rows: Vec<Vec<String>>, title: Option<T>) -> String
where
    T: ToString,
{
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);
    rows.into_iter().for_each(|row| {
        table.add_row(row);
    });
    title.map_or_else(|| table.to_string(), |value| format!("{} \n{table}", value.to_string()))
}
/// Return line-oriented diff changes with ANSI colors
#[cfg(feature = "std")]
pub fn text_diff_changes(old: &str, new: &str) -> Vec<(ChangeTag, String)> {
    text_diff_changes_with_color(old, new, true)
}
/// Return line-oriented diff changes with optional ANSI colors
#[cfg(feature = "std")]
pub fn text_diff_changes_with_color(old: &str, new: &str, color: bool) -> Vec<(ChangeTag, String)> {
    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .map(|line| {
            let tag = line.tag();
            let text = match (color, tag) {
                | (true, Delete) => format!("- {line}").red().to_string(),
                | (true, Insert) => format!("+ {line}").green().to_string(),
                | (true, Equal) => format!("  {line}").dimmed().to_string(),
                | (false, Delete) => format!("- {line}"),
                | (false, Insert) => format!("+ {line}"),
                | (false, Equal) => format!("  {line}"),
            };
            (tag, text)
        })
        .collect()
}
