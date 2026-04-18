use crate::cli::arguments::Diagnostic;
use crate::commands::preflight;
use acorn::doctor::{MemoryInformation, NetworkInformation, SystemInformation, SystemSoftwareInformation, TableFormatPrint};
use color_eyre::eyre::{Report, Result};
use tracing::warn;

fn should_run(checks: &[Diagnostic], diagnostic: Diagnostic) -> bool {
    [diagnostic, Diagnostic::All].iter().any(|&x| checks.contains(&x))
}
#[allow(clippy::too_many_arguments)]
pub async fn run(fix: &bool, interactive: &bool, check: &[Diagnostic], offline: &bool) -> Result<(), Report> {
    preflight(*offline, false, false, "doctor");
    if *fix {
        unimplemented!("Autofix is not implemented yet");
    }
    if *interactive {
        unimplemented!("Interactive mode is not implemented yet");
    }
    if should_run(check, Diagnostic::System) {
        SystemInformation::init().print();
    }
    if should_run(check, Diagnostic::Memory) {
        MemoryInformation::init().print();
    }
    if should_run(check, Diagnostic::Network) {
        NetworkInformation::init().print();
    }
    if should_run(check, Diagnostic::Gpu) {
        warn!("GPU diagnostics is not implemented yet");
    }
    if should_run(check, Diagnostic::Software) {
        SystemSoftwareInformation::init().print();
    }
    Ok(())
}

#[cfg(test)]
mod tests;
