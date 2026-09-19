//! Cryptographically secure secret generation.
use crate::cli::Void;
use acorn::util::generate_secret;
use arboard::{Clipboard, Error as ClipboardError};
use color_eyre::eyre::eyre;

pub(super) fn deliver<Copy>(secret: String, clipboard: bool, copy: Copy) -> Void
where
    Copy: FnOnce(String) -> Result<(), ClipboardError>,
{
    match clipboard {
        | false => {
            println!("{secret}");
            Ok(())
        }
        | true => copy(secret)
            .map_err(|why| eyre!("Failed to copy the generated secret to the clipboard — {why}"))
            .map(|()| eprintln!("Secret copied to clipboard")),
    }
}
/// Generate one 256-bit secret and deliver its lowercase hexadecimal representation.
pub fn run(clipboard: bool) -> Void {
    generate_secret()
        .map_err(|why| eyre!("Failed to generate a cryptographically secure secret — {why}"))
        .and_then(|secret| {
            deliver(secret, clipboard, |value| {
                Clipboard::new().and_then(|mut clipboard| clipboard.set_text(value))
            })
        })
}
