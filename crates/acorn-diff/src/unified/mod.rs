use alloc::string::{String, ToString};
use similar::TextDiff;

/// Creates a unified line diff with three lines of context.
#[must_use]
pub fn unified_diff(before: &str, after: &str, before_label: &str, after_label: &str) -> String {
    TextDiff::from_lines(before, after)
        .unified_diff()
        .context_radius(3)
        .header(before_label, after_label)
        .to_string()
}

#[cfg(test)]
mod tests;
