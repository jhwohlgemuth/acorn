use super::{display_width, render, wrap_lines, write_to};
use std::io::{self, Write};

struct FailingWriter {
    fail_on_flush: bool,
}
impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail_on_flush {
            Ok(bytes.len())
        } else {
            Err(io::Error::other("output full"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_on_flush {
            Err(io::Error::other("flush failed"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_render_sizes_bubble_to_short_text() {
    let output = render("Hi", 40);
    assert!(output.starts_with("                ╭────╮\n                │ Hi │\n                ╰───┬╯\n"));
}
#[test]
fn test_render_uses_oakley_and_speech_tail() {
    let output = render("Hello, Oakley!", 40);
    assert!(output.starts_with("       ╭────────────────╮\n       │ Hello, Oakley! │\n"));
    assert!(output.contains("        ╭───────────╯\n      ██▄\n"));
    assert!(output.ends_with("       ▀▀██▀▀\n\n"));
}
#[test]
fn test_wrap_breaks_long_words_at_width() {
    assert_eq!(wrap_lines("abcdefghijk", 4), ["abcd", "efgh", "ijk"]);
}
#[test]
fn test_wrap_preserves_paragraphs_and_unicode_width() {
    assert_eq!(wrap_lines("one two three\n\nOak 🌰", 7), ["one two", "three", "", "Oak 🌰"]);
    assert_eq!(wrap_lines("Oak 🌰", 4), ["Oak", "🌰"]);
    assert_eq!(display_width("👩‍🔬"), 2);
    assert_eq!(wrap_lines("👩‍🔬", 2), ["👩‍🔬"]);
    assert_eq!(wrap_lines("a 👩‍🔬 b", 2), ["a", "👩‍🔬", "b"]);
}
#[test]
fn test_write_propagates_output_failure() {
    assert_eq!(
        write_to(&mut FailingWriter { fail_on_flush: false }, "Hello", 40)
            .expect_err("failed write")
            .to_string(),
        "output full"
    );
    assert_eq!(
        write_to(&mut FailingWriter { fail_on_flush: true }, "Hello", 40)
            .expect_err("failed flush")
            .to_string(),
        "flush failed"
    );
}
