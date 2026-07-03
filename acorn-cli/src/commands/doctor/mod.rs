use crate::cli::arguments::Diagnostic;
use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::doctor::{MemoryInformation, NetworkInformation, SystemInformation, SystemSoftwareInformation, TableFormatPrint};
use color_eyre::eyre::{Report, Result};
use tracing::warn;

fn should_run(checks: &[Diagnostic], diagnostic: Diagnostic) -> bool {
    [diagnostic, Diagnostic::All].iter().any(|&x| checks.contains(&x))
}
#[allow(clippy::too_many_arguments)]
pub fn run(fix: &bool, interactive: &bool, report: &bool, check: &[Diagnostic], offline: bool) -> Result<(), Report> {
    let options = CommandOptions::init().offline(offline).build();
    preflight!(&options);
    if *interactive {
        #[cfg(feature = "tui")]
        {
            acorn_tui::run_tui(acorn_tui::Screen::Doctor)
        }
        #[cfg(not(feature = "tui"))]
        {
            Err(color_eyre::eyre::eyre!("TUI feature not enabled. Build with: cargo build --features tui"))
        }
    } else if *fix {
        Err(color_eyre::eyre::eyre!("Autofix is not implemented yet"))
    } else if *report {
        let sys = SystemInformation::init();
        let mem = MemoryInformation::init();
        let net = NetworkInformation::init();
        let sw = SystemSoftwareInformation::init();
        let data = build_report_json(&sys, &mem, &net, &sw);
        let json = serde_json::to_string_pretty(&data).unwrap_or_default();
        println!("{json}");
        Ok(())
    } else {
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
}
#[allow(unused_variables)]
fn build_report_json(
    sys: &SystemInformation,
    mem: &MemoryInformation,
    net: &NetworkInformation,
    sw: &SystemSoftwareInformation,
) -> serde_json::Value {
    use serde_json::json;
    let network_interfaces: Vec<serde_json::Value> = net
        .networks
        .iter()
        .filter(|n| !n.ip_address.is_empty())
        .map(|n| {
            json!({
                "ip_addresses": n.ip_address,
                "mac_address": n.mac_address,
                "mtu": n.mtu,
            })
        })
        .collect();
    json!({
        "report_type": "doctor",
        "system": {
            "name": sys.name,
            "kernel_version": sys.kernel_version,
            "os_version": sys.os_version,
            "host_name": sys.host_name,
            "cpu_architecture": sys.cpu_arch,
            "cpu_count": sys.cpu_count,
        },
        "memory": {
            "total": mem.total,
            "available": mem.available,
            "used": mem.used,
            "swap": mem.swap,
        },
        "network": network_interfaces,
        "software": {
            "acorn": { "version": sw.acorn.version, "path": sw.acorn.path },
            "git": { "version": sw.git.version, "path": sw.git.path },
            "node": { "version": sw.node.version, "path": sw.node.path },
            "npm": { "version": sw.npm.version, "path": sw.npm.path },
            "npx": { "version": sw.npx.version, "path": sw.npx.path },
            "pandoc": { "version": sw.pandoc.version, "path": sw.pandoc.path },
            "vale": { "version": sw.vale.version, "path": sw.vale.path },
        }
    })
}

#[cfg(test)]
mod tests;
