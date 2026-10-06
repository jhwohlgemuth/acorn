//! Terminal output helpers
#[cfg(feature = "analysis")]
use crate::skip;
#[cfg(feature = "analysis")]
use acorn_core::prelude::Vec;
#[cfg(feature = "analysis")]
use acorn_core::prelude::{format, String, ToString};
#[cfg(feature = "analysis")]
use acorn_diff::unified_diff;
#[cfg(feature = "analysis")]
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
#[cfg(feature = "analysis")]
use comfy_table::presets::UTF8_FULL;
#[cfg(feature = "analysis")]
use comfy_table::{ContentArrangement, Table};
#[cfg(feature = "analysis")]
use owo_colors::OwoColorize;
#[cfg(feature = "analysis")]
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};

/// Print line-oriented changes with ANSI colors
#[cfg(feature = "analysis")]
pub fn print_changes(old: &str, new: &str) {
    print_changes_with_color(old, new, true);
}
/// Print line-oriented changes with optional ANSI colors
#[cfg(feature = "analysis")]
pub fn print_changes_with_color(old: &str, new: &str, color: bool) {
    let diff = unified_diff(old, new, "before", "after");
    if diff.is_empty() {
        skip!("No format changes");
    } else {
        diff.split_inclusive('\n').for_each(|line| match (color, line) {
            | (false, line) => print!("{line}"),
            | (true, line) if line.starts_with("--- ") || line.starts_with('-') => print!("{}", line.red()),
            | (true, line) if line.starts_with("+++ ") || line.starts_with('+') => print!("{}", line.green()),
            | (true, line) if line.starts_with("@@") => print!("{}", line.cyan()),
            | (true, line) => print!("{}", line.dimmed()),
        });
    }
}
/// Print rows as a host-formatted table
#[cfg(feature = "analysis")]
pub fn print_values_as_table<T>(headers: Vec<&str>, rows: Vec<Vec<String>>, title: Option<T>)
where
    T: ToString,
{
    println!("{}", values_as_table(headers, rows, title));
}
/// Return line-oriented diff changes with ANSI colors
#[cfg(feature = "analysis")]
pub fn text_diff_changes(old: &str, new: &str) -> Vec<(ChangeTag, String)> {
    text_diff_changes_with_color(old, new, true)
}
/// Return line-oriented diff changes with optional ANSI colors
#[cfg(feature = "analysis")]
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
/// Render rows as a host-formatted table without writing to stdout
#[cfg(feature = "analysis")]
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
