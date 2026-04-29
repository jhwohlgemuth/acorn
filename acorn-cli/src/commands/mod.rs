use crate::cli::CommandOptions;
use acorn::util::Label;
use std::process::exit;

pub mod check;
pub mod create;
pub mod doctor;
pub mod download;
pub mod export;
pub mod format;
pub mod gather;
pub mod link;

macro_rules! preflight {
    ($options:expr) => {
        $crate::commands::preflight_inner(module_path!().rsplit("::").next().unwrap_or(module_path!()), $options)
    };
}
pub(crate) use preflight;

fn preflight_inner(command: &str, options: &CommandOptions) {
    let CommandOptions {
        offline,
        quiet,
        supported,
        path,
        reference,
        ..
    } = options;
    if let Some(value) = path {
        if !value.exists() {
            eprintln!("=> {} Path not found: {}", Label::fail(), value.display());
            exit(exitcode::NOINPUT);
        }
    }
    if let Some(value) = reference {
        if !value.exists() {
            eprintln!("=> {} Reference not found: {}", Label::fail(), value.display());
            exit(exitcode::NOINPUT);
        }
    }
    if *offline {
        if !*quiet {
            println!("=> {} ACORN is running in offline mode", Label::fmt_skip("OFFLINE"));
        }
        if !*supported {
            eprintln!("=> {} Offline mode is not implemented for '{command}' command", Label::fail());
            exit(exitcode::UNAVAILABLE);
        }
    }
}
