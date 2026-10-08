//! Deterministic host diagnostic discovery

use super::{FindingStatus, Record, Records};
use crate::io::FromCommand;
use acorn_core::util::SemanticVersion;
use acorn_schema::software::{Software, SoftwareSnapshot, SOFTWARE};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use bon::Builder;
use std::{
    env::{current_exe, var_os},
    path::PathBuf,
};
use sysinfo::{Networks, System};
use which::which;
/// Complete secret-free host snapshot used to produce diagnostic records.
#[derive(Builder, Clone, Debug, Default, Eq, PartialEq)]
#[builder(start_fn = init, on(String, into))]
pub struct HostSnapshot {
    /// CPU architecture.
    #[builder(default)]
    pub architecture: String,
    /// Logical CPU count.
    #[builder(default)]
    pub cpu_count: usize,
    /// Optional GPU summary when a supported probe exists.
    pub gpu: Option<String>,
    /// Operating-system kernel version.
    #[builder(default)]
    pub kernel_version: String,
    /// Total host memory in bytes.
    #[builder(default)]
    pub memory_total: u64,
    /// Available host memory in bytes.
    #[builder(default)]
    pub memory_available: u64,
    /// Number of network interfaces reported by the host.
    #[builder(default)]
    pub network_interfaces: usize,
    /// Operating-system name.
    #[builder(default)]
    pub os_name: String,
    /// Operating-system version.
    #[builder(default)]
    pub os_version: String,
    /// Deterministically ordered software observations.
    #[builder(default)]
    pub software: Vec<SoftwareSnapshot>,
    /// Total host swap in bytes.
    #[builder(default)]
    pub swap_total: u64,
}
impl HostSnapshot {
    /// Convert this injected host snapshot into deterministic discovery findings.
    pub fn diagnose(&self) -> Records {
        let system = Record::finding()
            .category("system")
            .subject("operating-system")
            .status(FindingStatus::from(!self.os_name.is_empty()))
            .observed(format!("{} {}", self.os_name, self.os_version).trim().to_string())
            .expected("detectable operating system")
            .details(BTreeMap::from([
                ("architecture".to_string(), self.architecture.clone()),
                ("cpu_count".to_string(), self.cpu_count.to_string()),
                ("kernel_version".to_string(), self.kernel_version.clone()),
            ]))
            .call();
        let memory = Record::finding()
            .category("memory")
            .subject("physical-memory")
            .status(FindingStatus::from(self.memory_total > 0))
            .observed(self.memory_total.to_string())
            .expected("non-zero total bytes")
            .details(BTreeMap::from([
                ("available_bytes".to_string(), self.memory_available.to_string()),
                ("swap_bytes".to_string(), self.swap_total.to_string()),
            ]))
            .call();
        let network = Record::finding()
            .category("network")
            .subject("interfaces")
            .status(FindingStatus::from(self.network_interfaces > 0))
            .observed(self.network_interfaces.to_string())
            .expected("at least one interface")
            .details(BTreeMap::new())
            .call();
        let gpu = Record::finding()
            .category("gpu")
            .subject("accelerator")
            .status(self.gpu.as_ref().map_or(FindingStatus::Unavailable, |_| FindingStatus::Healthy))
            .observed(self.gpu.clone().unwrap_or_else(|| "not detected".to_string()))
            .expected("optional accelerator information")
            .details(BTreeMap::new())
            .call();
        let software = self.software.iter().map(|software| {
            let installed = software.path.is_some();
            Record::finding()
                .category("software")
                .subject(software.name.clone())
                .status(FindingStatus::from(installed))
                .observed(if installed { "installed" } else { "missing" })
                .expected("installed executable")
                .details(BTreeMap::from([
                    ("path".to_string(), software.path.clone().unwrap_or_default()),
                    (
                        "version".to_string(),
                        software.version.as_ref().map(ToString::to_string).unwrap_or_default(),
                    ),
                ]))
                .call()
        });
        Records::default().with_findings([system, memory, network, gpu].into_iter().chain(software))
    }
}
impl Records {
    /// Capture a normalized, secret-free snapshot of the current host.
    pub fn capture_host() -> HostSnapshot {
        let mut system = System::new_all();
        system.refresh_all();
        HostSnapshot::init()
            .architecture(System::cpu_arch())
            .cpu_count(system.cpus().len())
            .kernel_version(System::kernel_version().unwrap_or_default())
            .memory_available(system.available_memory())
            .memory_total(system.total_memory())
            .network_interfaces(Networks::new_with_refreshed_list().len())
            .os_name(System::name().unwrap_or_default())
            .os_version(System::os_version().unwrap_or_default())
            .software(SOFTWARE.into_iter().map(software_snapshot).collect())
            .swap_total(system.total_swap())
            .build()
    }
    /// Capture and diagnose the current host.
    pub fn diagnose_host() -> Self {
        Self::capture_host().diagnose()
    }
}
fn software_path(software: Software) -> Option<PathBuf> {
    match software {
        | Software::Acorn => current_exe().ok(),
        | Software::Chromium => var_os("CHROME_PATH").map(PathBuf::from).filter(|path| path.is_file()).or_else(|| {
            ["chromium", "chromium-browser", "google-chrome"]
                .into_iter()
                .find_map(|command| which(command).ok())
        }),
        | _ => which(software.command()).ok(),
    }
}
fn software_snapshot(software: Software) -> SoftwareSnapshot {
    let path = software_path(software);
    let version = match software {
        | Software::Acorn => Some(SemanticVersion::from(env!("CARGO_PKG_VERSION"))),
        | _ => path
            .as_ref()
            .and_then(|path| SemanticVersion::from_command(path.to_string_lossy().as_ref())),
    };
    SoftwareSnapshot::new(software, path.map(|path| path.to_string_lossy().into_owned()), version)
}

#[cfg(test)]
mod tests;
