//! Cryptographically secure secret generation.
use crate::cli::Void;
use crate::io::{is_stdout_piped, write_stdout};
use acorn::util::generate_secret;
use arboard::{Clipboard, Error as ClipboardError};
use color_eyre::eyre::eyre;

pub(super) fn deliver<Copy, Write>(secret: &str, paste: bool, stdout_piped: bool, copy: Copy, write: Write) -> Void
where
    Copy: FnOnce(&str) -> Result<(), ClipboardError>,
    Write: FnOnce(&str),
{
    let copied = match paste {
        | true => copy(secret).map_err(|why| eyre!("Failed to copy the generated secret to the clipboard — {why}")),
        | false => Ok(()),
    };
    copied.map(|()| {
        if stdout_piped {
            write(secret);
        }
    })
}
/// Generate one 256-bit secret and deliver its lowercase hexadecimal representation.
pub fn run(paste: bool) -> Void {
    generate_secret()
        .map_err(|why| eyre!("Failed to generate a cryptographically secure secret — {why}"))
        .and_then(|secret| {
            deliver(
                &secret,
                paste,
                is_stdout_piped(),
                |value| Clipboard::new().and_then(|mut clipboard| clipboard.set_text(value)),
                |value| write_stdout(format!("{value}\n")),
            )
        })
}
