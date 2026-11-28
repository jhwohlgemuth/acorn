//! Oakley's speech bubble command.

use crate::cli::Void;
use std::io::{self, Write};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const OAKLEY: &str = "      ██▄
       ▄▄█▄▄▄
   ▄▄▄▀▀▄▄▄▄▀▀▄▄▄
  ▀▀████▀▀▀▀████▀▀
 ███▄▄▄▄████▄▄▄▄███
  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
  ████▀▀████▀▀████
  ▀███  ████  ███▀
    ▀██████████▀
       ▀▀██▀▀";

fn display_width(text: &str) -> usize {
    text.graphemes(true).map(UnicodeWidthStr::width).sum()
}
fn render(text: &str, wrap: usize) -> String {
    let lines = wrap_lines(text, wrap);
    let horizontal = lines.iter().map(|line| display_width(line)).max().unwrap_or(0).saturating_add(2);
    let content = horizontal.saturating_sub(2);
    let margin = 20usize.saturating_sub(horizontal).max(7);
    let indent = " ".repeat(margin);
    let stem = 20usize.saturating_sub(margin);
    let mut output = format!("{indent}╭{}╮\n", "─".repeat(horizontal));
    for line in lines {
        output.push_str(&format!(
            "{indent}│ {line}{} │\n",
            " ".repeat(content.saturating_sub(display_width(&line)))
        ));
    }
    output.push_str(&format!(
        "{indent}╰{}┬{}╯\n",
        "─".repeat(stem.saturating_sub(1)),
        "─".repeat(horizontal.saturating_sub(stem))
    ));
    output.push_str("        ╭───────────╯\n");
    output.push_str(OAKLEY);
    output.push_str("\n\n");
    output
}
/// Print Oakley with a wrapped speech bubble.
pub(crate) fn run(text: &str, wrap: usize) -> Void {
    write_to(&mut io::stdout().lock(), text, wrap).map_err(Into::into)
}
fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        let mut used = 0usize;
        for word in paragraph.split_whitespace() {
            let word_width = display_width(word);
            if used > 0 && used.saturating_add(1).saturating_add(word_width) > width {
                lines.push(core::mem::take(&mut line));
                used = 0;
            }
            if used > 0 {
                line.push(' ');
                used = used.saturating_add(1);
            }
            for grapheme in word.graphemes(true) {
                let grapheme_width = UnicodeWidthStr::width(grapheme);
                if used > 0 && used.saturating_add(grapheme_width) > width {
                    lines.push(core::mem::take(&mut line));
                    used = 0;
                }
                line.push_str(grapheme);
                used = used.saturating_add(grapheme_width);
            }
        }
        lines.push(line);
    }
    lines
}
fn write_to(output: &mut impl Write, text: &str, wrap: usize) -> io::Result<()> {
    output.write_all(render(text, wrap).as_bytes()).and_then(|()| output.flush())
}

#[cfg(test)]
mod tests;
