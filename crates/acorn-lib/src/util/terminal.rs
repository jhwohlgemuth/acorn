//! Terminal output helpers
#[cfg(any(feature = "analysis", feature = "std"))]
use crate::skip;
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_core::prelude::Vec;
#[cfg(any(feature = "analysis", feature = "std"))]
use acorn_core::prelude::{format, String, ToString};
#[cfg(any(feature = "analysis", feature = "std"))]
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
#[cfg(any(feature = "analysis", feature = "std"))]
use comfy_table::presets::UTF8_FULL;
#[cfg(any(feature = "analysis", feature = "std"))]
use comfy_table::{ContentArrangement, Table};
#[cfg(any(feature = "analysis", feature = "std"))]
use owo_colors::OwoColorize;
#[cfg(any(feature = "analysis", feature = "std"))]
use similar::{
    ChangeTag::{self, Delete, Equal, Insert},
    TextDiff,
};

/// Print line-oriented changes with ANSI colors
#[cfg(any(feature = "analysis", feature = "std"))]
pub fn print_changes(old: &str, new: &str) {
    print_changes_with_color(old, new, true);
}
/// Print line-oriented changes with optional ANSI colors
#[cfg(any(feature = "analysis", feature = "std"))]
pub fn print_changes_with_color(old: &str, new: &str, color: bool) {
    let changes = text_diff_changes_with_color(old, new, color);
    if changes.iter().all(|(tag, _)| *tag == Equal) {
        skip!("No format changes");
    } else {
        changes.into_iter().for_each(|(_, change)| print!("{change}"));
    }
}
/// Print rows as a host-formatted table
#[cfg(any(feature = "analysis", feature = "std"))]
pub fn print_values_as_table<T>(headers: Vec<&str>, rows: Vec<Vec<String>>, title: Option<T>)
where
    T: ToString,
{
    println!("{}", values_as_table(headers, rows, title));
}
/// Render rows as a host-formatted table without writing to stdout
#[cfg(any(feature = "analysis", feature = "std"))]
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
#[cfg(any(feature = "analysis", feature = "std"))]
pub fn text_diff_changes(old: &str, new: &str) -> Vec<(ChangeTag, String)> {
    text_diff_changes_with_color(old, new, true)
}
/// Return line-oriented diff changes with optional ANSI colors
#[cfg(any(feature = "analysis", feature = "std"))]
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
