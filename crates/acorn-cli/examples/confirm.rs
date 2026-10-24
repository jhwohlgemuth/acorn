//! Try the CLI's interactive Yes and No buttons in a terminal.
#[cfg(not(test))]
use color_eyre::eyre::Result;
#[cfg(not(test))]
use interactive::{PromptOutcome, Viewport};

// Share the command renderer; its own PTY tests belong to the CLI test binary.
#[cfg(not(test))]
#[allow(dead_code)]
#[path = "../src/io/interactive/mod.rs"]
mod interactive;
#[cfg(not(test))]
fn main() -> Result<()> {
    Viewport::open()
        .and_then(|mut viewport| viewport.confirm("Apply the sample change?", false))
        .map(|outcome| match outcome {
            | PromptOutcome::Cancelled => println!("Cancelled"),
            | PromptOutcome::Selected(true) => println!("Yes selected"),
            | PromptOutcome::Selected(false) => println!("No selected"),
        })
}
