use acorn_lib::doctor::{MemoryInformation, NetworkInformation, SystemInformation, SystemSoftwareInformation, TableFormatPrint};
use acorn_lib::util::cli::Diagnostic;
use color_eyre::eyre::{Report, Result};
use tracing::warn;

fn should_run(checks: &[Diagnostic], diagnostic: Diagnostic) -> bool {
    [diagnostic, Diagnostic::All].iter().any(|&x| checks.contains(&x))
}
#[allow(clippy::too_many_arguments)]
pub fn run(fix: &bool, interactive: &bool, check: &[Diagnostic]) -> Result<(), Report> {
    if *fix {
        unimplemented!("Autofix is not implemented yet");
    }
    if *interactive {
        unimplemented!("Interactive mode is not implemented yet");
    }
    if should_run(check, Diagnostic::Software) {
        SystemSoftwareInformation::init().print();
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
    // TODO: Add support for GPU information
    if should_run(check, Diagnostic::Gpu) {
        warn!("GPU diagnostics is not implemented yet");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
