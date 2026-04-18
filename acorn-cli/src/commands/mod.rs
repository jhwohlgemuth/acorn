use acorn::util::Label;
use std::process::exit;

pub mod check;
pub mod doctor;
pub mod download;
pub mod export;
pub mod format;
pub mod gather;
pub mod link;

pub(crate) fn preflight(offline: bool, quiet: bool, supported: bool, command: &str) {
    if offline && !quiet {
        println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
    }
    if offline && !supported {
        eprintln!("=> {} Offline mode is not implemented for '{command}' command", Label::fail());
        exit(exitcode::UNAVAILABLE);
    }
}
